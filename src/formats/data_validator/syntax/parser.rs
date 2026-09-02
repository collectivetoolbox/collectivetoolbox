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

//! Parser and lexer implementation for the Document Character (Dc) syntax DSL.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
use anyhow::{Context, Result, bail, ensure};

/// Parses a target token strictly according to canonical formatting rules:
/// - Format ID: `f<digits>` (e.g. `f80`)
/// - Unicode codepoint: `u<lowercase-hex>` with 1..=6 hex digits (e.g. `u0020`, `u12ab`)
/// - Short Dc ID: `<digits>` (e.g. `246`, `0`)
pub fn parse_target_token(token: &str) -> Option<CharTarget> {
    let s = token.trim();
    if s.is_empty() {
        return None;
    }

    // Format target: f<digits>
    if let Some(rest) = s.strip_prefix('f') {
        if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(fmt_id) = rest.parse::<usize>() {
                return Some(CharTarget::Format(fmt_id));
            }
        }
        return None;
    }

    // Unicode target: u<lowercase-hex> (strictly 1..=6 lowercase hex characters)
    if let Some(rest) = s.strip_prefix('u') {
        if (1..=6).contains(&rest.len())
            && rest.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        {
            if let Ok(cp) = u32::from_str_radix(rest, 16) {
                if cp <= 0x0010_FFFF {
                    return Some(CharTarget::Unicode(cp));
                }
            }
        }
        return None;
    }

    // Short Dc ID: decimal digits only
    if s.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(dc_id) = s.parse::<u32>() {
            return Some(CharTarget::Dc(dc_id));
        }
    }

    None
}

/// Parses an action invocation string like `lang.assign($ident, $val)`.
fn parse_syntax_action(raw: &str) -> Result<SyntaxAction> {
    let trimmed = raw.trim();
    let Some(open_idx) = trimmed.find('(') else {
        bail!("Action invocation is missing argument list opening '(': '{trimmed}'");
    };
    let Some(close_idx) = trimmed.rfind(')') else {
        bail!("Action invocation is missing argument list closing ')': '{trimmed}'");
    };
    ensure!(
        open_idx < close_idx,
        "Mismatched parenthesis in action: '{trimmed}'"
    );

    let method = match trimmed.get(..open_idx) {
        Some(m) => m.trim().to_string(),
        None => bail!("Failed to extract action method name"),
    };
    ensure!(!method.is_empty(), "Action method name cannot be empty");

    let args_str = match trimmed.get(open_idx.saturating_add(1)..close_idx) {
        Some(a) => a.trim(),
        None => "",
    };

    let mut args = Vec::new();
    if !args_str.is_empty() {
        for arg in args_str.split(',') {
            let arg_trim = arg.trim();
            if arg_trim.is_empty() {
                continue;
            }
            if let Some(var_name) = arg_trim.strip_prefix('$') {
                args.push(ActionArg::Variable(var_name.trim().to_string()));
            } else {
                let clean_literal = arg_trim.trim_matches('"').trim_matches('\'');
                args.push(ActionArg::Literal(clean_literal.to_string()));
            }
        }
    }

    Ok(SyntaxAction { method, args })
}

/// Disambiguates and parses content within square brackets `[...]`.
fn parse_bracket_content(content: &str) -> Result<SyntaxElement> {
    let trimmed = content.trim();
    ensure!(!trimmed.is_empty(), "Empty bracket construct '[]'");

    // Negated character set: [^248 255]
    if let Some(rest) = trimmed.strip_prefix('^') {
        let mut members = Vec::new();
        for tok in rest.split_whitespace() {
            let target = parse_target_token(tok)
                .with_context(|| format!("Invalid character token in negated set: '{tok}'"))?;
            members.push(target);
        }
        ensure!(!members.is_empty(), "Negated character set '[^]' cannot be empty");
        return Ok(SyntaxElement::exact(SyntaxTerm::CharSet {
            negated: true,
            members,
        }));
    }

    // Optional rule macro expansion: [260:]
    if let Some(rule_tok) = trimmed.strip_suffix(':') {
        let target = parse_target_token(rule_tok).with_context(|| {
            format!("Invalid rule reference target in '[{trimmed}]': '{rule_tok}'")
        })?;
        return Ok(SyntaxElement::with_quantifier(
            SyntaxTerm::RuleRef { target },
            Quantifier::Optional,
        ));
    }

    // Range: [260-265]
    if let Some((start_tok, end_tok)) = trimmed.split_once('-') {
        if let (Some(start), Some(end)) = (
            parse_target_token(start_tok.trim()),
            parse_target_token(end_tok.trim()),
        ) {
            return Ok(SyntaxElement::exact(SyntaxTerm::CharRange {
                start,
                end,
            }));
        }
    }

    // Check if all whitespace-separated tokens are valid CharTargets (Character Set: [246 247])
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if !tokens.is_empty() {
        let mut targets = Vec::new();
        let mut all_valid = true;
        for tok in &tokens {
            if let Some(target) = parse_target_token(tok) {
                targets.push(target);
            } else {
                all_valid = false;
                break;
            }
        }
        if all_valid {
            return Ok(SyntaxElement::exact(SyntaxTerm::CharSet {
                negated: false,
                members: targets,
            }));
        }
    }

    // Named construct: [identifier $ident] or [type:transformation] or [statement]
    let mut name_part = "";
    let mut capture_var = None;

    for tok in tokens {
        if let Some(var) = tok.strip_prefix('$') {
            capture_var = Some(var.trim().to_string());
        } else if name_part.is_empty() {
            name_part = tok;
        } else {
            bail!("Unexpected extra token in named construct '[{trimmed}]': '{tok}'");
        }
    }

    ensure!(
        !name_part.is_empty(),
        "Named construct in brackets must have a name: '[{trimmed}]'"
    );

    let (name, subtype) = if let Some((base, sub)) = name_part.split_once(':') {
        (base.trim().to_string(), Some(sub.trim().to_string()))
    } else {
        (name_part.trim().to_string(), None)
    };

    Ok(SyntaxElement::exact(SyntaxTerm::NamedConstruct {
        name,
        subtype,
        capture_var,
    }))
}

/// Tokenizes and parses a pattern string into `SyntaxPattern`.
fn parse_pattern_expression(raw: &str) -> Result<SyntaxPattern> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(SyntaxPattern::Sequence(Vec::new()));
    }

    // Split top-level alternations ('|') not inside parentheses or brackets
    let mut alt_parts = Vec::new();
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut current_segment = String::new();

    for ch in trimmed.chars() {
        match ch {
            '(' => {
                depth_paren = depth_paren.saturating_add(1);
                current_segment.push(ch);
            }
            ')' => {
                depth_paren = depth_paren.saturating_sub(1);
                current_segment.push(ch);
            }
            '[' => {
                depth_bracket = depth_bracket.saturating_add(1);
                current_segment.push(ch);
            }
            ']' => {
                depth_bracket = depth_bracket.saturating_sub(1);
                current_segment.push(ch);
            }
            '|' if depth_paren == 0 && depth_bracket == 0 => {
                alt_parts.push(current_segment.trim().to_string());
                current_segment.clear();
            }
            _ => {
                current_segment.push(ch);
            }
        }
    }
    if !current_segment.trim().is_empty() {
        alt_parts.push(current_segment.trim().to_string());
    }

    if alt_parts.len() > 1 {
        let mut branches = Vec::new();
        for part in alt_parts {
            branches.push(parse_pattern_expression(&part)?);
        }
        return Ok(SyntaxPattern::Alternation(branches));
    }

    // Parse sequence of elements
    let mut elements = Vec::new();
    let chars: Vec<char> = trimmed.chars().collect();
    let mut idx = 0usize;

    while idx < chars.len() {
        let Some(&ch) = chars.get(idx) else {
            break;
        };

        if ch.is_whitespace() {
            idx = idx.saturating_add(1);
            continue;
        }

        if ch == '~' {
            idx = idx.saturating_add(1);
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx);
            idx = next_idx;
            elements.push(SyntaxElement::with_quantifier(
                SyntaxTerm::SelfChar,
                quantifier,
            ));
            continue;
        }

        if ch == '(' {
            let start = idx.saturating_add(1);
            let mut depth = 1usize;
            let mut end = start;
            while end < chars.len() && depth > 0 {
                let Some(&c) = chars.get(end) else {
                    break;
                };
                if c == '(' {
                    depth = depth.saturating_add(1);
                } else if c == ')' {
                    depth = depth.saturating_sub(1);
                }
                if depth > 0 {
                    end = end.saturating_add(1);
                }
            }
            ensure!(
                depth == 0,
                "Unclosed parenthesis in pattern: '{trimmed}'"
            );

            // Reason for fallback: slice bounds checked by depth parser, empty substring defaults to empty string
            let inner: String = chars
                .get(start..end)
                .map(|s| s.iter().collect())
                .unwrap_or_default();
            let sub_pattern = parse_pattern_expression(&inner)?;
            idx = end.saturating_add(1);

            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx);
            idx = next_idx;

            elements.push(SyntaxElement::with_quantifier(
                SyntaxTerm::Group(sub_pattern),
                quantifier,
            ));
            continue;
        }

        if ch == '[' {
            let start = idx.saturating_add(1);
            let mut depth = 1usize;
            let mut end = start;
            while end < chars.len() && depth > 0 {
                let Some(&c) = chars.get(end) else {
                    break;
                };
                if c == '[' {
                    depth = depth.saturating_add(1);
                } else if c == ']' {
                    depth = depth.saturating_sub(1);
                }
                if depth > 0 {
                    end = end.saturating_add(1);
                }
            }
            ensure!(depth == 0, "Unclosed bracket in pattern: '{trimmed}'");

            // Reason for fallback: slice bounds checked by depth parser, empty substring defaults to empty string
            let inner: String = chars
                .get(start..end)
                .map(|s| s.iter().collect())
                .unwrap_or_default();
            let mut element = parse_bracket_content(&inner)?;
            idx = end.saturating_add(1);

            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx);
            idx = next_idx;
            if quantifier != Quantifier::ExactOne {
                element.quantifier = quantifier;
            }

            elements.push(element);
            continue;
        }

        // Bare target token or bare rule reference (e.g. "248", "258:", "f80", "u0020")
        let start = idx;
        while idx < chars.len() {
            let Some(&c) = chars.get(idx) else {
                break;
            };
            if c.is_whitespace() || c == '(' || c == ')' || c == '[' || c == ']' || c == '|' {
                break;
            }
            idx = idx.saturating_add(1);
        }

        // Reason for fallback: bounds checked by token scan, empty slice defaults to empty string
        let token_str: String = chars
            .get(start..idx)
            .map(|s| s.iter().collect())
            .unwrap_or_default();
        let token_trim = token_str.trim();

        if let Some(rule_tok) = token_trim.strip_suffix(':') {
            let target = parse_target_token(rule_tok).with_context(|| {
                format!("Invalid bare rule reference in pattern: '{token_trim}'")
            })?;
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx);
            idx = next_idx;
            elements.push(SyntaxElement::with_quantifier(
                SyntaxTerm::RuleRef { target },
                quantifier,
            ));
            continue;
        }

        if let Some(target) = parse_target_token(token_trim) {
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx);
            idx = next_idx;
            elements.push(SyntaxElement::with_quantifier(
                SyntaxTerm::CharRef(target),
                quantifier,
            ));
            continue;
        }

        bail!("Unrecognized token in syntax pattern: '{token_trim}' in '{trimmed}'");
    }

    Ok(SyntaxPattern::Sequence(elements))
}

/// Helper to consume a trailing quantifier character (`+`, `*`, `?`).
fn parse_trailing_quantifier(chars: &[char], idx: usize) -> (Quantifier, usize) {
    if let Some(&q_char) = chars.get(idx) {
        match q_char {
            '+' => (Quantifier::OneOrMore, idx.saturating_add(1)),
            '*' => (Quantifier::ZeroOrMore, idx.saturating_add(1)),
            '?' => (Quantifier::Optional, idx.saturating_add(1)),
            _ => (Quantifier::ExactOne, idx),
        }
    } else {
        (Quantifier::ExactOne, idx)
    }
}

/// Parses a complete Dc syntax rule declaration string (e.g. `":~ [^248 255]+ 248"`).
pub fn parse_dc_syntax(raw: &str) -> Result<DcSyntaxRule> {
    // Reason for fallback: declaration without leading colon defaults to unchanged trimmed string
    let clean = raw.trim().strip_prefix(':').unwrap_or(raw.trim()).trim();
    ensure!(!clean.is_empty(), "Empty Dc syntax declaration");

    // Check for action separator ':' outside of brackets/parentheses
    let mut pattern_str = clean.to_string();
    let mut action_opt = None;

    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let chars: Vec<char> = clean.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '(' => depth_paren = depth_paren.saturating_add(1),
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '[' => depth_bracket = depth_bracket.saturating_add(1),
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ':' if depth_paren == 0 && depth_bracket == 0 => {
                // Ensure this ':' is followed by an action expression like ' lang.action('
                // Reason for fallback: bounds checked by iteration, missing slice defaults to empty string
                let after: String = chars
                    .get(i.saturating_add(1)..)
                    .map(|s| s.iter().collect())
                    .unwrap_or_default();
                if after.contains('(') && after.contains(')') {
                    // Reason for fallback: bounds checked by iteration, missing slice defaults to empty string
                    let before: String = chars
                        .get(..i)
                        .map(|s| s.iter().collect())
                        .unwrap_or_default();
                    pattern_str = before.trim().to_string();
                    action_opt = Some(parse_syntax_action(&after)?);
                    break;
                }
            }
            _ => {}
        }
    }

    let pattern = parse_pattern_expression(&pattern_str)?;

    Ok(DcSyntaxRule {
        pattern,
        action: action_opt,
        raw: raw.trim().to_string(),
    })
}
