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
    ActionArg, CharTarget, DcSyntaxRule, MatchMode, ParsedDocument,
    ParsedElement, Quantifier, SyntaxAction, SyntaxDiagnostic, SyntaxElement,
    SyntaxPattern, SyntaxTerm,
};
use super::framing::{
    DC_IDENTIFIER_BEGIN, DC_LITERAL_BEGIN, scan_identifier_frame,
    scan_literal_frame,
};
use anyhow::{Context, Result, bail, ensure};

/// Parses a target token strictly according to canonical shorthand formatting rules:
/// - Format ID: `f<digits>` (e.g. `f80`)
/// - Unicode codepoint: `u<lowercase-hex>` with 1..=6 hex digits (e.g. `u0020`, `u12ab`)
/// - Short Dc ID: `<digits>` (e.g. `246`, `0`)
pub fn parse_target_token(token: &str) -> Option<CharTarget> {
    match ctb_storage_minimal::shorthand::DcShorthand::parse(token).ok()? {
        ctb_storage_minimal::shorthand::DcShorthand::Short(id) => Some(CharTarget::Dc(id)),
        ctb_storage_minimal::shorthand::DcShorthand::Format(fmt_id) => {
            Some(CharTarget::Format(fmt_id))
        }
        ctb_storage_minimal::shorthand::DcShorthand::Unicode(cp) => {
            Some(CharTarget::Unicode(cp))
        }
        ctb_storage_minimal::shorthand::DcShorthand::Long(_)
        | ctb_storage_minimal::shorthand::DcShorthand::Local(_) => None,
    }
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

    // Negated character set: [^248 255] or [^(314 | 312)]
    if let Some(rest) = trimmed.strip_prefix('^') {
        let rest_trimmed = rest.trim();
        let tokens_str = if rest_trimmed.starts_with('(') && rest_trimmed.ends_with(')') {
            match rest_trimmed.get(1..rest_trimmed.len().saturating_sub(1)) {
                Some(s) => s.trim(),
                None => "",
            }
        } else if rest_trimmed.contains('|') {
            bail!(
                "Alternation '|' in negated set '[{trimmed}]' must be enclosed in parentheses, e.g. '[^({rest_trimmed})]'"
            );
        } else {
            rest_trimmed
        };

        let mut members = Vec::new();
        for tok in tokens_str.split(|c: char| c.is_whitespace() || c == '|') {
            let tok = tok.trim();
            if tok.is_empty() {
                continue;
            }
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

    // Range: [260-265] or [u41..u5a]
    if let Some((start_tok, end_tok)) = trimmed.split_once("..").or_else(|| trimmed.split_once('-')) {
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

    // Check if tokens are valid CharTargets (Character Set: [246 247] or [(246 | 247)])
    let positive_str = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        match trimmed.get(1..trimmed.len().saturating_sub(1)) {
            Some(s) => s.trim(),
            None => "",
        }
    } else {
        trimmed
    };
    let tokens: Vec<&str> = positive_str
        .split(|c: char| c.is_whitespace() || c == '|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
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

    // Check if there is a colon ':' separating name from subtype.
    // The colon must be at top level (depth_paren == 0 && depth_bracket == 0).
    let mut top_colon_idx = None;
    let mut depth_p = 0usize;
    let mut depth_b = 0usize;
    for (i, c) in trimmed.char_indices() {
        match c {
            '(' => depth_p = depth_p.saturating_add(1),
            ')' => depth_p = depth_p.saturating_sub(1),
            '[' => depth_b = depth_b.saturating_add(1),
            ']' => depth_b = depth_b.saturating_sub(1),
            ':' if depth_p == 0 && depth_b == 0 => {
                top_colon_idx = Some(i);
                break;
            }
            _ => {}
        }
    }

    if let Some(colon_idx) = top_colon_idx {
        let name_part_raw = match trimmed.get(..colon_idx) {
            Some(s) => s,
            None => bail!("Invalid name slice bounds in named construct '[{trimmed}]'"),
        };
        let subtype_part_raw = match trimmed.get(colon_idx.saturating_add(1)..) {
            Some(s) => s,
            None => bail!("Invalid subtype slice bounds in named construct '[{trimmed}]'"),
        };

        let mut name = "";
        let mut capture_var = None;

        for tok in name_part_raw.split_whitespace() {
            if let Some(var) = tok.strip_prefix('$') {
                ensure!(
                    capture_var.is_none(),
                    "Multiple capture variables in named construct '[{trimmed}]'"
                );
                capture_var = Some(var.trim().to_string());
            } else if name.is_empty() {
                name = tok;
            } else {
                bail!("Unexpected extra token in named construct name '[{trimmed}]': '{tok}'");
            }
        }

        let mut subtype_str = subtype_part_raw.trim();
        let mut last_top_whitespace = None;
        let mut depth_p = 0usize;
        let mut depth_b = 0usize;
        for (i, c) in subtype_str.char_indices() {
            match c {
                '(' => depth_p = depth_p.saturating_add(1),
                ')' => depth_p = depth_p.saturating_sub(1),
                '[' => depth_b = depth_b.saturating_add(1),
                ']' => depth_b = depth_b.saturating_sub(1),
                _ if c.is_whitespace() && depth_p == 0 && depth_b == 0 => {
                    last_top_whitespace = Some(i);
                }
                _ => {}
            }
        }
        if let Some(ws_idx) = last_top_whitespace {
            if let Some(candidate_slice) = subtype_str.get(ws_idx..) {
                let candidate = candidate_slice.trim();
                if let Some(var) = candidate.strip_prefix('$') {
                    if !var.is_empty() {
                        ensure!(
                            capture_var.is_none(),
                            "Multiple capture variables in named construct '[{trimmed}]'"
                        );
                        capture_var = Some(var.trim().to_string());
                        subtype_str = match subtype_str.get(..ws_idx) {
                            Some(s) => s.trim(),
                            None => subtype_str,
                        };
                    }
                }
            }
        }

        ensure!(
            !name.is_empty(),
            "Named construct in brackets must have a name: '[{trimmed}]'"
        );
        ensure!(
            !subtype_str.is_empty(),
            "Named construct in brackets cannot have an empty subtype: '[{trimmed}]'"
        );

        return Ok(SyntaxElement::exact(SyntaxTerm::NamedConstruct {
            name: name.to_string(),
            subtype: Some(subtype_str.to_string()),
            capture_var,
        }));
    }

    // Named construct without colon: [identifier $ident] or [statement]
    let mut tokens = Vec::new();
    let mut current_tok = String::new();
    let mut depth_p = 0usize;
    let mut depth_b = 0usize;
    for c in trimmed.chars() {
        match c {
            '(' => {
                depth_p = depth_p.saturating_add(1);
                current_tok.push(c);
            }
            ')' => {
                depth_p = depth_p.saturating_sub(1);
                current_tok.push(c);
            }
            '[' => {
                depth_b = depth_b.saturating_add(1);
                current_tok.push(c);
            }
            ']' => {
                depth_b = depth_b.saturating_sub(1);
                current_tok.push(c);
            }
            _ if c.is_whitespace() && depth_p == 0 && depth_b == 0 => {
                if !current_tok.is_empty() {
                    tokens.push(current_tok.clone());
                    current_tok.clear();
                }
            }
            _ => current_tok.push(c),
        }
    }
    if !current_tok.is_empty() {
        tokens.push(current_tok);
    }

    let mut name = "";
    let mut capture_var = None;

    for tok in &tokens {
        if let Some(var) = tok.strip_prefix('$') {
            ensure!(
                capture_var.is_none(),
                "Multiple capture variables in named construct '[{trimmed}]'"
            );
            capture_var = Some(var.trim().to_string());
        } else if name.is_empty() {
            name = tok.as_str();
        } else {
            bail!("Unexpected extra token in named construct '[{trimmed}]': '{tok}'");
        }
    }

    ensure!(
        !name.is_empty(),
        "Named construct in brackets must have a name: '[{trimmed}]'"
    );

    Ok(SyntaxElement::exact(SyntaxTerm::NamedConstruct {
        name: name.to_string(),
        subtype: None,
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

            let Some(slice) = chars.get(start..end) else {
                bail!("Invalid parenthesis slice bounds in pattern: '{trimmed}'");
            };
            let inner: String = slice.iter().collect();
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

            let Some(slice) = chars.get(start..end) else {
                bail!("Invalid bracket slice bounds in pattern: '{trimmed}'");
            };
            let inner: String = slice.iter().collect();
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
            if c.is_whitespace() || c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}' || c == '|' {
                break;
            }
            idx = idx.saturating_add(1);
        }

        let Some(token_slice) = chars.get(start..idx) else {
            bail!("Invalid token slice bounds {start}..{idx} in pattern: '{trimmed}'");
        };
        let token_str: String = token_slice.iter().collect();
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

/// Helper to consume a trailing quantifier character (`+`, `*`, `?`, or `{...}`).
fn parse_trailing_quantifier(chars: &[char], idx: usize) -> (Quantifier, usize) {
    if let Some(&q_char) = chars.get(idx) {
        match q_char {
            '+' => (Quantifier::OneOrMore, idx.saturating_add(1)),
            '*' => (Quantifier::ZeroOrMore, idx.saturating_add(1)),
            '?' => (Quantifier::Optional, idx.saturating_add(1)),
            '{' => {
                let start = idx.saturating_add(1);
                let mut end = start;
                while end < chars.len() && chars.get(end) != Some(&'}') {
                    end = end.saturating_add(1);
                }
                if end < chars.len() {
                    if let Some(sub) = chars.get(start..end) {
                        let content: String = sub.iter().collect();
                        let trimmed = content.trim();
                        let parsed = if let Some((min_s, max_s)) =
                            trimmed.split_once("..").or_else(|| trimmed.split_once(','))
                        {
                            // Reason for fallback: omitted min in range defaults to 0
                            let min = min_s.trim().parse::<usize>().unwrap_or(0);
                            let max = if max_s.trim().is_empty() {
                                None
                            } else {
                                max_s.trim().parse::<usize>().ok()
                            };
                            Some(Quantifier::Range { min, max })
                        } else if let Ok(n) = trimmed.parse::<usize>() {
                            Some(Quantifier::Range {
                                min: n,
                                max: Some(n),
                            })
                        } else {
                            None
                        };
                        if let Some(q) = parsed {
                            return (q, end.saturating_add(1));
                        }
                    }
                }
                (Quantifier::ExactOne, idx)
            }
            _ => (Quantifier::ExactOne, idx),
        }
    } else {
        (Quantifier::ExactOne, idx)
    }
}

/// Parses a complete Dc syntax rule declaration string (e.g. `":~ [^248 255]+ 248"`).
pub fn parse_dc_syntax(raw: &str) -> Result<DcSyntaxRule> {
    // Reason for fallback: Dc syntax declarations optionally omit the leading colon prefix
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
                let Some(after_slice) = chars.get(i.saturating_add(1)..) else {
                    bail!("Missing action slice after colon in syntax rule: '{clean}'");
                };
                let after: String = after_slice.iter().collect();
                if after.contains('(') && after.contains(')') {
                    let Some(before_slice) = chars.get(..i) else {
                        bail!("Missing pattern slice before colon in syntax rule: '{clean}'");
                    };
                    let before: String = before_slice.iter().collect();
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

const DC_PARAM_BEGIN: u32 = 258;
const DC_PARAM_END: u32 = 259;
const DC_REF_NAMED: u32 = 276;
const DC_INVOCATION: u32 = 279;

/// Parses a character token stream into a structured, non-evaluating
/// document representation.
///
/// Under `MatchMode::Permissive`, mangled or broken structures are preserved
/// as `ParsedElement::RawTokens` and diagnostics are logged without aborting
/// parsing. Under `MatchMode::Strict`, framing violations cause immediate
/// return with `has_errors: true`.
#[must_use]
pub fn parse_document_tokens(stream: &[u32], mode: MatchMode) -> ParsedDocument {
    let mut elements = Vec::new();
    let mut diagnostics = Vec::new();
    let mut offset = 0usize;

    while offset < stream.len() {
        let remaining = match stream.get(offset..) {
            Some(s) => s,
            None => break,
        };
        let Some(&first) = remaining.first() else {
            break;
        };

        // Framed literal (Dc 260 ... Dc 261)
        if first == DC_LITERAL_BEGIN {
            if let Some((lit, consumed)) =
                scan_literal_frame(remaining, mode, &mut diagnostics, offset)
            {
                elements.push(ParsedElement::Literal(lit));
                offset = offset.saturating_add(consumed);
                continue;
            } else if mode == MatchMode::Strict {
                return ParsedDocument {
                    elements,
                    diagnostics,
                    has_errors: true,
                };
            }
        }

        // Framed identifier (Dc 270 ... Dc 271)
        if first == DC_IDENTIFIER_BEGIN {
            if let Some((ident, consumed)) =
                scan_identifier_frame(remaining, mode, &mut diagnostics, offset)
            {
                elements.push(ParsedElement::Identifier(ident));
                offset = offset.saturating_add(consumed);
                continue;
            } else if mode == MatchMode::Strict {
                return ParsedDocument {
                    elements,
                    diagnostics,
                    has_errors: true,
                };
            }
        }

        // Parameter frame (Dc 258 ... Dc 259)
        if first == DC_PARAM_BEGIN {
            let mut param_consumed = 1usize;
            let mut inner_tokens = Vec::new();
            let mut found_end = false;

            while let Some(&tok) = remaining.get(param_consumed) {
                param_consumed = param_consumed.saturating_add(1);
                if tok == DC_PARAM_END {
                    found_end = true;
                    break;
                }
                inner_tokens.push(tok);
            }

            if !found_end {
                diagnostics.push(SyntaxDiagnostic {
                    message: "Unclosed parameter: missing terminator Dc 259"
                        .to_string(),
                    token_offset: offset.saturating_add(param_consumed),
                    is_error: true,
                });
                if mode == MatchMode::Strict {
                    return ParsedDocument {
                        elements,
                        diagnostics,
                        has_errors: true,
                    };
                }
            }

            let inner_doc = parse_document_tokens(&inner_tokens, mode);
            for mut diag in inner_doc.diagnostics {
                diag.token_offset = diag
                    .token_offset
                    .saturating_add(offset)
                    .saturating_add(1);
                diagnostics.push(diag);
            }
            elements.push(ParsedElement::Parameter(inner_doc.elements));
            offset = offset.saturating_add(param_consumed);
            continue;
        }

        // Reference named object (Dc 276 [identifier])
        if first == DC_REF_NAMED {
            let after_ref = match remaining.get(1..) {
                Some(s) => s,
                None => &[],
            };
            if let Some((ident, consumed)) = scan_identifier_frame(
                after_ref,
                mode,
                &mut diagnostics,
                offset.saturating_add(1),
            ) {
                elements.push(ParsedElement::Reference(ident));
                offset = offset.saturating_add(consumed).saturating_add(1);
                continue;
            }
        }

        // Routine invocation or built-in routine marker (e.g. Dc 256 lang.say or Dc 279)
        if first == DC_INVOCATION || first == 256 {
            let mut inv_consumed = 1usize;
            let mut args = Vec::new();

            let mut target_elem = ParsedElement::RawTokens(vec![first]);
            if first == DC_INVOCATION {
                let after_inv = match remaining.get(inv_consumed..) {
                    Some(s) => s,
                    None => &[],
                };
                if let Some((ident, consumed)) = scan_identifier_frame(
                    after_inv,
                    mode,
                    &mut diagnostics,
                    offset.saturating_add(inv_consumed),
                ) {
                    target_elem = ParsedElement::Identifier(ident);
                    inv_consumed = inv_consumed.saturating_add(consumed);
                }
            }

            loop {
                let after_args = match remaining.get(inv_consumed..) {
                    Some(s) => s,
                    None => break,
                };
                if after_args.first() == Some(&DC_PARAM_BEGIN) {
                    let mut p_consumed = 1usize;
                    let mut p_tokens = Vec::new();
                    let mut p_found_end = false;
                    while let Some(&tok) = after_args.get(p_consumed) {
                        p_consumed = p_consumed.saturating_add(1);
                        if tok == DC_PARAM_END {
                            p_found_end = true;
                            break;
                        }
                        p_tokens.push(tok);
                    }
                    if !p_found_end {
                        diagnostics.push(SyntaxDiagnostic {
                            message:
                                "Unclosed parameter in invocation: missing Dc 259"
                                    .to_string(),
                            token_offset: offset
                                .saturating_add(inv_consumed)
                                .saturating_add(p_consumed),
                            is_error: true,
                        });
                    }
                    let p_doc = parse_document_tokens(&p_tokens, mode);
                    args.push(ParsedElement::Parameter(p_doc.elements));
                    inv_consumed = inv_consumed.saturating_add(p_consumed);
                } else {
                    break;
                }
            }

            elements.push(ParsedElement::Invocation {
                target: Box::new(target_elem),
                args,
            });
            offset = offset.saturating_add(inv_consumed);
            continue;
        }

        // Fallback for unparsed raw token: accumulate contiguous raw tokens
        let mut raw_chunk = vec![first];
        offset = offset.saturating_add(1);
        while let Some(&tok) = stream.get(offset) {
            if tok == DC_LITERAL_BEGIN
                || tok == DC_IDENTIFIER_BEGIN
                || tok == DC_PARAM_BEGIN
                || tok == DC_REF_NAMED
                || tok == DC_INVOCATION
                || tok == 256
            {
                break;
            }
            raw_chunk.push(tok);
            offset = offset.saturating_add(1);
        }
        elements.push(ParsedElement::RawTokens(raw_chunk));
    }

    let has_errors = diagnostics.iter().any(|d| d.is_error);
    ParsedDocument {
        elements,
        diagnostics,
        has_errors,
    }
}
