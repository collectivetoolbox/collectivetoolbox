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
    CharTarget, DcSyntaxRule, Quantifier, SyntaxElement, SyntaxPattern,
    SyntaxTerm,
};
use std::collections::HashMap;

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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MatchContext {
    /// The Short ID of the defining character (for resolving `~`).
    pub self_dc: Option<u32>,
    /// Captured character token sequences keyed by variable name (e.g. `ident`, `val`).
    pub captured_vars: HashMap<String, Vec<u32>>,
    /// Diagnostic warnings collected during resilient recovery.
    pub warnings: Vec<String>,
}

impl MatchContext {
    /// Creates a new match context for a specific defining Dc.
    #[must_use]
    pub fn new(self_dc: Option<u32>) -> Self {
        Self {
            self_dc,
            captured_vars: HashMap::new(),
            warnings: Vec::new(),
        }
    }
}

/// Helper checking if a single character token matches a `CharTarget`.
fn target_matches_token(target: &CharTarget, token: u32) -> bool {
    match target {
        CharTarget::Dc(id) => *id == token,
        CharTarget::Unicode(cp) => *cp == token,
        CharTarget::Format(_) => false,
    }
}

/// Evaluates a single `SyntaxTerm` against a character stream.
fn match_term_single(
    stream: &[u32],
    term: &SyntaxTerm,
    context: &mut MatchContext,
) -> MatchOutcome {
    let Some(&first) = stream.first() else {
        return MatchOutcome::Mismatch;
    };

    match term {
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
            let matches = match (start, end) {
                (CharTarget::Dc(s), CharTarget::Dc(e)) => (*s..=*e).contains(&first),
                (CharTarget::Unicode(s), CharTarget::Unicode(e)) => (*s..=*e).contains(&first),
                _ => false,
            };
            if matches {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
        SyntaxTerm::Group(group_pat) => match_pattern(stream, group_pat, context),
        SyntaxTerm::NamedConstruct { capture_var, .. } => {
            if let Some(var) = capture_var {
                context
                    .captured_vars
                    .entry(var.clone())
                    .or_default()
                    .push(first);
            }
            MatchOutcome::Matched { consumed: 1 }
        }
        SyntaxTerm::RuleRef { target } => {
            if target_matches_token(target, first) {
                MatchOutcome::Matched { consumed: 1 }
            } else {
                MatchOutcome::Mismatch
            }
        }
    }
}

/// Matches a quantified `SyntaxElement` against a stream.
fn match_element(
    stream: &[u32],
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
                if !element.quantifier.allows_multiple() {
                    break;
                }
            }
            MatchOutcome::MatchedWithRecovery { consumed, warning } => {
                total_consumed = total_consumed.saturating_add(consumed);
                count = count.saturating_add(1);
                context.warnings.push(warning);
                if !element.quantifier.allows_multiple() {
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
    }
}

/// Evaluates a `SyntaxPattern` against a character token stream.
pub fn match_pattern(
    stream: &[u32],
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

                // Tag-soup resilient recovery: If remaining is empty but sequence expects closing delimiter
                if remaining.is_empty() {
                    if elem.quantifier.allows_zero() {
                        continue;
                    }
                    let warning = format!(
                        "Unclosed syntax structure at end-of-stream (element index {idx} in sequence)"
                    );
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

/// Evaluates a full `DcSyntaxRule` against a character token stream.
#[must_use]
pub fn match_syntax_rule(
    stream: &[u32],
    rule: &DcSyntaxRule,
    self_dc: Option<u32>,
) -> (MatchOutcome, MatchContext) {
    let mut context = MatchContext::new(self_dc);
    let outcome = match_pattern(stream, &rule.pattern, &mut context);
    (outcome, context)
}
