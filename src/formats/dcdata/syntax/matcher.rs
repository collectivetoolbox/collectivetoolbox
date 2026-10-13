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
use crate::dc::{
    DC_END_FORMAT_REPRESENTATION, DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
    DC_TYPE_INTERSECTION, DC_TYPE_UNION,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Maximum recursion depth allowed during grammar rule expansion.
pub const MAX_SYNTAX_EXPANSION_DEPTH: usize = 32;

pub(super) fn pattern_starts_with_self(pattern: &SyntaxPattern) -> bool {
    match pattern {
        SyntaxPattern::Sequence(elements) => elements.first().is_some_and(|element| {
            match &element.term {
                SyntaxTerm::SelfChar => true,
                SyntaxTerm::Group(pattern) => pattern_starts_with_self(pattern),
                _ => false,
            }
        }),
        SyntaxPattern::Alternation(branches) =>
            !branches.is_empty() && branches.iter().all(pattern_starts_with_self),
    }
}

fn pattern_has_payload_constraint(pattern: &SyntaxPattern) -> bool {
    match pattern {
        SyntaxPattern::Sequence(elements) => elements.iter().any(|element| match &element.term {
            SyntaxTerm::NamedConstruct { name, subtype: Some(_), .. } =>
                name == "data" || name == "string",
            SyntaxTerm::Group(pattern) => pattern_has_payload_constraint(pattern),
            _ => false,
        }),
        SyntaxPattern::Alternation(branches) => branches.iter().any(pattern_has_payload_constraint),
    }
}

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
    /// True if greedy trailing Dc 512 consumption should be suppressed because the enclosing sequence explicitly expects 512.
    pub suppress_trailing_dc_512: bool,
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
            suppress_trailing_dc_512: false,
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

fn encapsulated_byte_length(stream: &[u128]) -> Result<usize> {
    let payload_end = stream.len().checked_sub(1).context("Missing binary terminator")?;
    let mut payload_start = payload_end;
    while let Some(index) = payload_start.checked_sub(1) {
        let token = *stream.get(index).context("Missing binary token")?;
        if !crate::dc_number_minimal::is_dc_base64_encapsulation_char(DcChar::from_u128(token)) {
            break;
        }
        payload_start = index;
    }
    let payload = stream.get(payload_start..payload_end).context("Invalid binary bounds")?;
    ensure!(payload.len().is_multiple_of(4), "Invalid Base64 payload length");
    let digits = payload.iter().map(|token|
        crate::dc_number_minimal::dc_base64_char_to_digit(DcChar::from_u128(*token))
    ).collect::<Result<Vec<_>>>()?;
    let padding = digits.iter().rev().take_while(|digit| **digit == 64).count();
    ensure!(padding <= 2, "Invalid Base64 padding");
    let unpadded = digits.len().checked_sub(padding).context("Invalid Base64 padding")?;
    ensure!(digits.get(..unpadded).context("Invalid Base64 bounds")?.iter().all(|digit| *digit < 64),
        "Embedded Base64 padding");
    if padding > 0 {
        let last = digits.get(unpadded.checked_sub(1).context("Missing Base64 digit")?)
            .context("Missing Base64 digit")?;
        let mask = if padding == 2 { 15 } else { 3 };
        ensure!(last & mask == 0, "Noncanonical Base64 trailing bits");
    }
    digits.len().checked_div(4).and_then(|groups| groups.checked_mul(3))
        .and_then(|bytes| bytes.checked_sub(padding)).context("Binary byte count overflow")
}

/// Locates the end of a balanced container sequence taking escaping into account.
pub(super) fn find_balanced_closing(stream: &[u128], opener: u128, closer: u128) -> Option<usize> {
    let dc_escape = DC_ESCAPE.to_u128();
    let mut depth = 0usize;
    let mut skip_next = false;
    let mut idx = 0usize;
    while let Some(&tok) = stream.get(idx) {
        idx = idx.saturating_add(1);
        if skip_next {
            skip_next = false;
            continue;
        }
        if tok == dc_escape {
            skip_next = true;
            continue;
        }
        if depth > 0 && tok != opener {
            let remaining = stream.get(idx.saturating_sub(1)..)?;
            let mut diagnostics = Vec::new();
            let framed_len = if tok == DC_LITERAL_BEGIN {
                scan_literal_frame(remaining, MatchMode::Strict, &mut diagnostics, 0)
                    .map(|(_, consumed)| consumed)
            } else if tok == DC_IDENTIFIER_BEGIN {
                scan_identifier_frame(remaining, MatchMode::Strict, &mut diagnostics, 0)
                    .map(|(_, consumed)| consumed)
            } else {
                None
            };
            if let Some(consumed) = framed_len {
                idx = idx.saturating_sub(1).saturating_add(consumed);
                continue;
            }
            if tok == DC_LITERAL_BEGIN || tok == DC_IDENTIFIER_BEGIN {
                return None;
            }
        }
        if tok == opener {
            depth = depth.saturating_add(1);
        } else if tok == closer {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
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

    // Equivalent and partially equivalent representations can wrap any value.
    let dc_equiv = DC_EQUIVALENT_REPRESENTATIONS.to_u128();
    let dc_partial_equiv = DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128();
    if (first == dc_equiv || first == dc_partial_equiv)
        && !matches!(term, SyntaxTerm::Group(pattern) if pattern_has_payload_constraint(pattern))
        && !matches!(
            term,
            SyntaxTerm::SelfChar
                | SyntaxTerm::CharRef(CharTarget::Dc(397 | 561))
                | SyntaxTerm::RuleRef {
                    target: CharTarget::Dc(397 | 561),
                }
        )
    {
        if let Some(ref resolver) = context.resolver.clone() {
            let wrapper = if first == dc_equiv { 397 } else { 561 };
            if let Some(rule) = resolver.resolve_rule(&CharTarget::Dc(wrapper)) {
                let old_self = context.self_dc;
                context.self_dc = Some(first);
                let alt_outcome = match_pattern(stream, &rule.pattern, context);
                context.self_dc = old_self;
                if alt_outcome.is_matched() {
                    if matches!(term, SyntaxTerm::NamedConstruct { name, subtype: Some(_), .. }
                        if name == "data" || name == "string")
                    {
                        if context.depth >= context.max_depth {
                            context.has_errors = true;
                            context.diagnostics.push(SyntaxDiagnostic {
                                message: "Constrained representation expansion depth limit reached".to_owned(),
                                token_offset: 0, is_error: true,
                            });
                            return MatchOutcome::Mismatch;
                        }
                        let Some(mut values) = stream.get(2..alt_outcome.consumed_tokens().saturating_sub(1)) else {
                            return MatchOutcome::Mismatch;
                        };
                        let mut constrained_context = context.clone();
                        constrained_context.depth = constrained_context.depth.saturating_add(1);
                        for header in ["number", "type"] {
                            let header_term = SyntaxTerm::NamedConstruct {
                                name: header.to_owned(), subtype: None, capture_var: None,
                            };
                            let header_outcome = match_term_single(values, &header_term, &mut constrained_context);
                            if header_outcome.is_matched() {
                                let Some(rest) = values.get(header_outcome.consumed_tokens()..) else {
                                    return MatchOutcome::Mismatch;
                                };
                                values = rest;
                            } else if header == "type" {
                                return MatchOutcome::Mismatch;
                            }
                        }
                        let value_term = SyntaxTerm::NamedConstruct {
                            name: "value".to_owned(), subtype: None, capture_var: None,
                        };
                        let mut matched = false;
                        while !values.is_empty() {
                            if values.first() == Some(&crate::dc::DC_LIST_ELEMENT_SEPARATOR.to_u128()) {
                                let Some(rest) = values.get(1..) else { return MatchOutcome::Mismatch; };
                                values = rest;
                                continue;
                            }
                            let mut candidate_context = constrained_context.clone();
                            let candidate = match_term_single(values, term, &mut candidate_context);
                            let consumed = if candidate.is_matched() && !candidate_context.has_errors {
                                matched = true;
                                candidate.consumed_tokens()
                            } else {
                                if let SyntaxTerm::NamedConstruct { name, .. } = term {
                                    let unconstrained = SyntaxTerm::NamedConstruct {
                                        name: name.clone(), subtype: None, capture_var: None,
                                    };
                                    let mut trial = constrained_context.clone();
                                    if match_term_single(values, &unconstrained, &mut trial).is_matched() {
                                        return MatchOutcome::Mismatch;
                                    }
                                }
                                match_term_single(values, &value_term, &mut constrained_context).consumed_tokens()
                            };
                            if consumed == 0 { return MatchOutcome::Mismatch; }
                            let Some(rest) = values.get(consumed..) else { return MatchOutcome::Mismatch; };
                            values = rest;
                        }
                        if !matched { return MatchOutcome::Mismatch; }
                    }
                    return alt_outcome;
                }
            }
        }
    }

    // Optional value wrappers: Dc 315 (present) and Dc 316 (absent)
    let dc_opt_present = DC_OPTIONAL_PRESENT.to_u128();
    let dc_opt_absent = DC_OPTIONAL_ABSENT.to_u128();
    if first == dc_opt_present
        && !matches!(
            term,
            SyntaxTerm::SelfChar
                | SyntaxTerm::CharRef(CharTarget::Dc(315))
                | SyntaxTerm::RuleRef {
                    target: CharTarget::Dc(315),
                }
        )
    {
        if let Some(sub_stream) = stream.get(1..) {
            let inner_outcome = match_term_single(sub_stream, term, context);
            if inner_outcome.is_matched() {
                return MatchOutcome::Matched {
                    consumed: inner_outcome.consumed_tokens().saturating_add(1),
                };
            }
        }
    }
    if first == dc_opt_absent
        && !matches!(
            term,
            SyntaxTerm::SelfChar
                | SyntaxTerm::CharRef(CharTarget::Dc(316))
                | SyntaxTerm::RuleRef {
                    target: CharTarget::Dc(316),
                }
        )
    {
        return MatchOutcome::Matched { consumed: 1 };
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
            if *negated {
                if in_set {
                    return MatchOutcome::Mismatch;
                }
                let dc_escape = DC_ESCAPE.to_u128();
                if first == dc_escape && stream.len() >= 2 {
                    return MatchOutcome::Matched { consumed: 2 };
                }
                if let Some(closer_tok) = context.resolver.as_ref().and_then(|r| {
                    CharTarget::from_u128(first).and_then(|target| r.closer_for_opener(&target))
                }) {
                    if let Some(balanced_len) = find_balanced_closing(stream, first, closer_tok) {
                        return MatchOutcome::Matched { consumed: balanced_len };
                    }
                }
                MatchOutcome::Matched { consumed: 1 }
            } else if in_set {
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
            if name == "data" {
                if let Some(subtype) = subtype {
                    let (format, bytes) = match super::parse_data_constraint(subtype) {
                        Ok(constraint) => constraint,
                        Err(error) => {
                            context.has_errors = true;
                            context.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Invalid data constraint: {error}"),
                                token_offset: 0, is_error: true,
                            });
                            return MatchOutcome::Mismatch;
                        }
                    };
                    let bare_data = SyntaxTerm::NamedConstruct {
                        name: name.clone(), subtype: None, capture_var: None,
                    };
                    let mut data_context = context.clone();
                    if data_context.resolver.is_none() {
                        data_context.resolver = Some(super::resolver::default_syntax_resolver().clone());
                    }
                    let outcome = match_term_single(stream, &bare_data, &mut data_context);
                    context.has_errors |= data_context.has_errors;
                    context.diagnostics = data_context.diagnostics;
                    context.warnings = data_context.warnings;
                    if !outcome.is_matched() { return MatchOutcome::Mismatch; }
                    if let Some(format) = format {
                        use ctb_utilities::format_id::FormatId;
                        let id = FormatId::from_shorthand(format)
                            .or_else(|| FormatId::from_ident(format)).and_then(|format| format.dc_id());
                        if id.is_none() || stream.get(1).copied() != id {
                            return MatchOutcome::Mismatch;
                        }
                    }
                    let consumed = outcome.consumed_tokens();
                    if let Some(bytes) = bytes {
                        let Some(data) = stream.get(..consumed) else { return MatchOutcome::Mismatch; };
                        match encapsulated_byte_length(data) {
                            Ok(length) if length == bytes => {}
                            Ok(_) => return MatchOutcome::Mismatch,
                            Err(error) => {
                                context.has_errors = true;
                                context.diagnostics.push(SyntaxDiagnostic {
                                    message: format!("Invalid constrained binary payload: {error}"),
                                    token_offset: 0, is_error: true,
                                });
                                return MatchOutcome::Mismatch;
                            }
                        }
                    }
                    if let Some(var) = capture_var {
                        if let Some(data) = stream.get(..consumed) {
                            context.captured_vars.entry(var.clone()).or_default().extend_from_slice(data);
                        }
                    }
                    return outcome;
                }
            }
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

            // Format or type category check: [format:<category>] or [formats:<category>] or [type:<category>]
            if (name == "format" || name == "formats" || name == "type") && subtype.is_some() {
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
                let dc_intersect = DC_TYPE_INTERSECTION.to_u128();
                let dc_union = DC_TYPE_UNION.to_u128();
                let matches = crate::dc_char::is_type_token(first)
                    // Reason for fallback: when syntax resolver is omitted, cannot verify membership in EL Types or Formats scripts
                    || context.resolver.as_ref().map_or(false, |r| {
                        r.matches_script("EL Types", first) || r.matches_script("Formats", first)
                    });
                if matches && first != dc_intersect && first != dc_union {
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
                    // Reason for fallback: when syntax resolver is omitted, cannot verify membership in Formats script
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
                        if let Some(subtype_str) = subtype {
                            if context.depth >= context.max_depth {
                                context.has_errors = true;
                                context.diagnostics.push(SyntaxDiagnostic {
                                    message: "String constraint expansion depth limit reached".to_owned(),
                                    token_offset: 0,
                                    is_error: true,
                                });
                                return MatchOutcome::Mismatch;
                            }
                            let sub_rule = match crate::syntax::parser::parse_dc_syntax(subtype_str) {
                                Ok(rule) => rule,
                                Err(error) => {
                                    context.has_errors = true;
                                    context.diagnostics.push(SyntaxDiagnostic {
                                        message: format!("Invalid string constraint: {error}"),
                                        token_offset: 0,
                                        is_error: true,
                                    });
                                    return MatchOutcome::Mismatch;
                                }
                            };
                            let mut sub_context = MatchContext::new_with_token(None).with_mode(context.mode);
                            sub_context.depth = context.depth.saturating_add(1);
                            sub_context.max_depth = context.max_depth;
                            sub_context.call_stack = context.call_stack.clone();
                            sub_context.resolver = Some(if let Some(resolver) = &context.resolver {
                                Arc::clone(resolver)
                            } else {
                                super::resolver::default_syntax_resolver().clone()
                            });
                            let sub_outcome = match_pattern(&lit.decoded_payload, &sub_rule.pattern, &mut sub_context);
                            let has_errors = sub_context.has_errors;
                            context.has_errors |= has_errors;
                            context.diagnostics.extend(sub_context.diagnostics);
                            context.warnings.extend(sub_context.warnings);
                            if has_errors || !sub_outcome.is_matched()
                                || sub_outcome.consumed_tokens() != lit.decoded_payload.len()
                            {
                                return MatchOutcome::Mismatch;
                            }
                        }
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
                if context.resolver.is_none() && subtype.is_none() {
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
                            (usize::from(!pattern_starts_with_self(&rule.pattern)), true)
                        }
                        _ => (0usize, false),
                    };

                    // Reason for fallback: slice beyond stream bounds defaults to empty slice
                    let sub_stream = stream.get(offset_after_fmt..).unwrap_or(&[]);
                    let outcome = match_pattern(sub_stream, &rule.pattern, context);

                    context.depth = context.depth.saturating_sub(1);
                    context.call_stack.pop();
                    context.self_dc = old_self;

                    if outcome.is_matched() {
                        let consumed_payload = outcome.consumed_tokens();
                        let mut total = offset_after_fmt.saturating_add(consumed_payload);
                        let dc_end_repr = DC_END_FORMAT_REPRESENTATION.to_u128();
                        if has_fmt_token
                            && !context.suppress_trailing_dc_512
                            && stream.get(total) == Some(&dc_end_repr)
                        {
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
pub(crate) fn match_element(
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
            MatchOutcome::Matched { consumed: 0 } => {
                count = count.saturating_add(1);
                break;
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
            let mut zero_match = None;
            for branch in branches {
                let mut trial_ctx = context.clone();
                let outcome = match_pattern(stream, branch, &mut trial_ctx);
                if outcome.is_matched() {
                    if outcome.consumed_tokens() > 0 {
                        *context = trial_ctx;
                        return outcome;
                    } else if zero_match.is_none() {
                        zero_match = Some((outcome, trial_ctx));
                    }
                }
            }
            if let Some((outcome, trial_ctx)) = zero_match {
                *context = trial_ctx;
                return outcome;
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

                let next_is_512 = elements
                    .get(idx.saturating_add(1))
                    .is_some_and(|e| e.term == SyntaxTerm::CharRef(CharTarget::Dc(512)));
                let old_suppress = context.suppress_trailing_dc_512;
                if next_is_512 {
                    context.suppress_trailing_dc_512 = true;
                }
                let outcome = match_element(remaining, elem, context);
                context.suppress_trailing_dc_512 = old_suppress;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::parser::parse_dc_syntax;

    fn literal(text: &str) -> Vec<u128> {
        let mut tokens = vec![
            DcChar::from_short(260).to_u128(),
            DcChar::from_short(262).to_u128(),
            DcChar::from_short(264).to_u128(),
            DcChar::from_short(263).to_u128(),
        ];
        tokens.extend(text.chars().map(u128::from));
        tokens.push(DcChar::from_short(261).to_u128());
        tokens
    }

    #[crate::ctb_test]
    fn constrained_strings_enforce_ascii_and_exact_length() -> Result<()> {
        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        for with_resolver in [false, true] {
            for (text, valid) in [
                ("TEXT", true), ("    ", true), ("~ !A", true),
                ("", false), ("ABC", false), ("ABCDE", false),
                ("abc\n", false), ("abc\u{7f}", false), ("abc\u{e9}", false),
            ] {
                let mut context = MatchContext::default().with_mode(MatchMode::Strict);
                if with_resolver {
                    context.resolver = Some(super::super::resolver::default_syntax_resolver().clone());
                }
                let tokens = literal(text);
                let outcome = match_syntax_rule_with_context(&tokens, &rule, &mut context);
                ensure!(outcome.is_matched() == valid, "Wrong string constraint result for {text:?}");
                if valid {
                    ensure!(outcome.consumed_tokens() == tokens.len(), "String not fully consumed");
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn malformed_or_depth_limited_constraints_fail_explicitly() -> Result<()> {
        let tokens = literal("TEXT");
        let invalid = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some("]".to_owned()),
            capture_var: None,
        };
        let mut context = MatchContext::default().with_mode(MatchMode::Strict);
        ensure!(!match_term_single(&tokens, &invalid, &mut context).is_matched(),
            "Malformed string constraint was ignored");
        ensure!(context.has_errors && !context.diagnostics.is_empty(), "Missing constraint error");

        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        let mut context = MatchContext::default().with_mode(MatchMode::Strict);
        context.max_depth = 0;
        ensure!(!match_syntax_rule_with_context(&tokens, &rule, &mut context).is_matched(),
            "String constraint exceeded depth limit");
        ensure!(context.has_errors, "Missing depth error");
        let mut context = MatchContext::default().with_mode(MatchMode::Strict);
        ensure!(!match_syntax_rule_with_context(&[u128::from('A')], &rule, &mut context).is_matched(),
            "Unframed string bypassed constraint");
        Ok(())
    }
}
