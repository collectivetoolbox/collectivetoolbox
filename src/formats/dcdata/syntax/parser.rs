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

/// Parses a target token strictly according to canonical shorthand formatting rules:
/// - Format ID: `f<digits>` (e.g. `f80`)
/// - Unicode codepoint: `u<lowercase-hex>` with 1..=6 hex digits (e.g. `u0020`, `u12ab`)
/// - Short Dc ID: `<digits>` (e.g. `246`, `0`)
pub fn parse_target_token(token: &str) -> Option<CharTarget> {
    match ctb_storage_minimal::shorthand::DcShorthand::parse(token).ok()? {
        ctb_storage_minimal::shorthand::DcShorthand::Short(id) => Some(CharTarget::Dc(id)),
        ctb_storage_minimal::shorthand::DcShorthand::Format(id) => {
            if let Some(fid) = ctb_utilities::format_id::FormatId::from_shorthand(token) {
                Some(CharTarget::Format(fid))
            } else if id == ctb_storage_minimal::shorthand::MAX_FORMAT_ID {
                Some(CharTarget::FormatRegionEnd)
            } else {
                None
            }
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

    // Rule macro expansion: [260:]
    if let Some(rule_tok) = trimmed.strip_suffix(':') {
        let target = parse_target_token(rule_tok).with_context(|| {
            format!("Invalid rule reference target in '[{trimmed}]': '{rule_tok}'")
        })?;
        return Ok(SyntaxElement::exact(SyntaxTerm::RuleRef { target }));
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

        let mut name_words = Vec::new();
        let mut capture_var = None;
        let mut elem_quantifier = Quantifier::ExactOne;

        for tok in name_part_raw.split_whitespace() {
            if let Some(var) = tok.strip_prefix('$') {
                ensure!(
                    capture_var.is_none(),
                    "Multiple capture variables in named construct '[{trimmed}]'"
                );
                let (clean_var, q) = if let Some(stripped) = var.strip_suffix('?') {
                    (stripped, Quantifier::Optional)
                } else if let Some(stripped) = var.strip_suffix('*') {
                    (stripped, Quantifier::ZeroOrMore)
                } else if let Some(stripped) = var.strip_suffix('+') {
                    (stripped, Quantifier::OneOrMore)
                } else {
                    (var, Quantifier::ExactOne)
                };
                elem_quantifier = q;
                capture_var = Some(clean_var.trim().to_string());
            } else {
                ensure!(
                    capture_var.is_none(),
                    "Unexpected token after capture variable in named construct name '[{trimmed}]': '{tok}'"
                );
                name_words.push(tok);
            }
        }
        let name = name_words.join(" ");

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
        if name == "data" {
            super::parse_data_constraint(subtype_str)?;
        }

        return Ok(SyntaxElement::with_quantifier(
            SyntaxTerm::NamedConstruct {
                name: name.to_string(),
                subtype: Some(subtype_str.to_string()),
                capture_var,
            },
            elem_quantifier,
        ));
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

    let mut name_words: Vec<&str> = Vec::new();
    let mut capture_var = None;
    let mut elem_quantifier = Quantifier::ExactOne;

    for tok in &tokens {
        if let Some(var) = tok.strip_prefix('$') {
            ensure!(
                capture_var.is_none(),
                "Multiple capture variables in named construct '[{trimmed}]'"
            );
            let (clean_var, q) = if let Some(stripped) = var.strip_suffix('?') {
                (stripped, Quantifier::Optional)
            } else if let Some(stripped) = var.strip_suffix('*') {
                (stripped, Quantifier::ZeroOrMore)
            } else if let Some(stripped) = var.strip_suffix('+') {
                (stripped, Quantifier::OneOrMore)
            } else {
                (var, Quantifier::ExactOne)
            };
            elem_quantifier = q;
            capture_var = Some(clean_var.trim().to_string());
        } else {
            ensure!(
                capture_var.is_none(),
                "Unexpected token after capture variable in named construct '[{trimmed}]': '{tok}'"
            );
            name_words.push(tok.as_str());
        }
    }
    let name = name_words.join(" ");

    ensure!(
        !name.is_empty(),
        "Named construct in brackets must have a name: '[{trimmed}]'"
    );

    Ok(SyntaxElement::with_quantifier(
        SyntaxTerm::NamedConstruct {
            name,
            subtype: None,
            capture_var,
        },
        elem_quantifier,
    ))
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
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx)?;
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

            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx)?;
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

            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx)?;
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
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx)?;
            idx = next_idx;
            elements.push(SyntaxElement::with_quantifier(
                SyntaxTerm::RuleRef { target },
                quantifier,
            ));
            continue;
        }

        if let Some(target) = parse_target_token(token_trim) {
            let (quantifier, next_idx) = parse_trailing_quantifier(&chars, idx)?;
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
fn parse_trailing_quantifier(chars: &[char], idx: usize) -> Result<(Quantifier, usize)> {
    Ok(if let Some(&q_char) = chars.get(idx) {
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
                ensure!(end < chars.len(), "Unclosed quantifier at offset {idx}");
                let sub = chars.get(start..end)
                    .context("Invalid quantifier bounds")?;
                let content: String = sub.iter().collect();
                let trimmed = content.trim();
                let quantifier = if let Some((min_s, max_s)) =
                    trimmed.split_once("..").or_else(|| trimmed.split_once(','))
                {
                    let min = if min_s.trim().is_empty() {
                        0
                    } else {
                        min_s.trim().parse::<usize>().with_context(|| {
                            format!("Invalid minimum in quantifier '{{{content}}}'")
                        })?
                    };
                    let max = if max_s.trim().is_empty() {
                        None
                    } else {
                        Some(max_s.trim().parse::<usize>().with_context(|| {
                            format!("Invalid maximum in quantifier '{{{content}}}'")
                        })?)
                    };
                    ensure!(
                        max.is_none_or(|max| min <= max),
                        "Reversed bounds in quantifier '{{{content}}}'"
                    );
                    Quantifier::Range { min, max }
                } else {
                    let count = trimmed.parse::<usize>().with_context(|| {
                        format!("Invalid count in quantifier '{{{content}}}'")
                    })?;
                    Quantifier::Range { min: count, max: Some(count) }
                };
                (quantifier, end.saturating_add(1))
            }
            _ => (Quantifier::ExactOne, idx),
        }
    } else {
        (Quantifier::ExactOne, idx)
    })
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

/// Returns `true` if `entry` (after optional `"case:"` prefix) is a `~...~`-delimited
/// regex extension pattern.
#[must_use]
pub fn is_extension_regex(entry: &str) -> bool {
    let trimmed = entry.trim();
    let trimmed = trimmed
        .strip_prefix("case:")
        .map_or(trimmed, str::trim);
    trimmed.starts_with('~') && trimmed.ends_with('~') && trimmed.len() >= 2
}

/// Parses a `~...~`-delimited regular expression from the format `Extensions` column
/// into a [`DcSyntaxRule`] using the common `.Semantic:Pattern` primitives.
///
/// Unanchored start (no leading `^`) prepends `600*` (`Wildcard pattern` repeated zero
/// or more times); unanchored end (no trailing unescaped `$`) appends `600*`.
pub fn parse_extension_regex(entry: &str) -> Result<DcSyntaxRule> {
    let raw_trimmed = entry.trim();
    let body_trimmed = raw_trimmed
        .strip_prefix("case:")
        .map_or(raw_trimmed, str::trim);

    let Some(inner) = body_trimmed
        .strip_prefix('~')
        .and_then(|s| s.strip_suffix('~'))
    else {
        bail!("Extension regex must be enclosed in '~...~': '{raw_trimmed}'");
    };
    ensure!(
        !inner.is_empty(),
        "Regex extension pattern '~...~' cannot be empty"
    );

    let chars: Vec<char> = inner.chars().collect();
    let anchored_start = chars.first() == Some(&'^');
    let start_idx = usize::from(anchored_start);

    // Check whether the final character is an unescaped '$' anchor
    let mut end_idx = chars.len();
    let mut anchored_end = false;
    if end_idx > start_idx && chars.get(end_idx.saturating_sub(1)) == Some(&'$')
    {
        let mut backslashes = 0usize;
        let mut scan = end_idx.saturating_sub(1);
        while scan > start_idx
            && chars.get(scan.saturating_sub(1)) == Some(&'\\')
        {
            backslashes = backslashes.saturating_add(1);
            scan = scan.saturating_sub(1);
        }
        if backslashes.is_multiple_of(2) {
            anchored_end = true;
            end_idx = end_idx.saturating_sub(1);
        }
    }

    let core_slice = chars
        .get(start_idx..end_idx)
        .context("Invalid slice bounds for regex extension body")?;
    let core_pattern = parse_regex_alternation(core_slice)?;

    let wildcard_short = crate::dc::DC_WILDCARD_PATTERN
        .to_short()
        .context("DC_WILDCARD_PATTERN must be a valid short Dc")?;
    let wildcard_star = SyntaxElement::with_quantifier(
        SyntaxTerm::CharRef(CharTarget::Dc(wildcard_short)),
        Quantifier::ZeroOrMore,
    );

    let pattern = match core_pattern {
        SyntaxPattern::Sequence(mut elems) => {
            if !anchored_start {
                elems.insert(0, wildcard_star.clone());
            }
            if !anchored_end {
                elems.push(wildcard_star);
            }
            SyntaxPattern::Sequence(elems)
        }
        SyntaxPattern::Alternation(branches) => {
            if anchored_start && anchored_end {
                SyntaxPattern::Alternation(branches)
            } else {
                let mut elems = Vec::new();
                if !anchored_start {
                    elems.push(wildcard_star.clone());
                }
                elems.push(SyntaxElement::exact(SyntaxTerm::Group(
                    SyntaxPattern::Alternation(branches),
                )));
                if !anchored_end {
                    elems.push(wildcard_star);
                }
                SyntaxPattern::Sequence(elems)
            }
        }
    };

    Ok(DcSyntaxRule {
        pattern,
        action: None,
        raw: raw_trimmed.to_string(),
    })
}

fn parse_regex_alternation(chars: &[char]) -> Result<SyntaxPattern> {
    let mut branches: Vec<Vec<char>> = Vec::new();
    let mut current = Vec::new();
    let mut depth_paren = 0usize;
    let mut in_bracket = false;
    let mut idx = 0usize;

    while idx < chars.len() {
        let Some(&ch) = chars.get(idx) else {
            break;
        };
        if ch == '\\' {
            current.push(ch);
            idx = idx.saturating_add(1);
            let esc = *chars
                .get(idx)
                .context("Trailing backslash in regex extension pattern")?;
            current.push(esc);
            idx = idx.saturating_add(1);
            continue;
        }
        if in_bracket {
            if ch == ']' {
                in_bracket = false;
            }
            current.push(ch);
            idx = idx.saturating_add(1);
            continue;
        }
        match ch {
            '[' => {
                in_bracket = true;
                current.push(ch);
            }
            '(' => {
                depth_paren = depth_paren.saturating_add(1);
                current.push(ch);
            }
            ')' => {
                ensure!(
                    depth_paren > 0,
                    "Unmatched ')' in regex extension pattern"
                );
                depth_paren = depth_paren.saturating_sub(1);
                current.push(ch);
            }
            '|' if depth_paren == 0 => {
                branches.push(std::mem::take(&mut current));
            }
            _ => {
                current.push(ch);
            }
        }
        idx = idx.saturating_add(1);
    }

    ensure!(
        !in_bracket,
        "Unclosed '[' character class in regex extension pattern"
    );
    ensure!(
        depth_paren == 0,
        "Unclosed '(' group in regex extension pattern"
    );

    if branches.is_empty() {
        parse_regex_sequence(&current)
    } else {
        branches.push(current);
        let mut parsed_branches = Vec::with_capacity(branches.len());
        for branch in branches {
            parsed_branches.push(parse_regex_sequence(&branch)?);
        }
        Ok(SyntaxPattern::Alternation(parsed_branches))
    }
}

fn parse_regex_sequence(chars: &[char]) -> Result<SyntaxPattern> {
    let mut elements = Vec::new();
    let mut idx = 0usize;
    let wildcard_short = crate::dc::DC_WILDCARD_PATTERN
        .to_short()
        .context("DC_WILDCARD_PATTERN must be a valid short Dc")?;

    while idx < chars.len() {
        let Some(&ch) = chars.get(idx) else {
            break;
        };
        let (term, next_idx) = match ch {
            '.' => (
                SyntaxTerm::CharRef(CharTarget::Dc(wildcard_short)),
                idx.saturating_add(1),
            ),
            '\\' => {
                let esc_idx = idx.saturating_add(1);
                let &esc = chars
                    .get(esc_idx)
                    .context("Trailing backslash in regex extension pattern")?;
                let cp = match esc {
                    't' => u32::from('\t'),
                    'n' => u32::from('\n'),
                    'r' => u32::from('\r'),
                    other => u32::from(other),
                };
                (
                    SyntaxTerm::CharRef(CharTarget::Unicode(cp)),
                    esc_idx.saturating_add(1),
                )
            }
            '(' => {
                let mut start = idx.saturating_add(1);
                let mut depth = 1usize;
                let mut scan = start;
                let mut in_bracket = false;
                while scan < chars.len() && depth > 0 {
                    let Some(&c) = chars.get(scan) else {
                        break;
                    };
                    if c == '\\' {
                        scan = scan.saturating_add(2);
                        continue;
                    }
                    if in_bracket {
                        if c == ']' {
                            in_bracket = false;
                        }
                        scan = scan.saturating_add(1);
                        continue;
                    }
                    if c == '[' {
                        in_bracket = true;
                    } else if c == '(' {
                        depth = depth.saturating_add(1);
                    } else if c == ')' {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                    scan = scan.saturating_add(1);
                }
                ensure!(
                    depth == 0,
                    "Unclosed '(' group in regex extension pattern"
                );
                // Strip optional non-capturing prefix `(?:`
                if chars.get(start) == Some(&'?')
                    && chars.get(start.saturating_add(1)) == Some(&':')
                {
                    start = start.saturating_add(2);
                }
                let inner_slice = chars
                    .get(start..scan)
                    .context("Invalid group slice in regex extension pattern")?;
                let inner_pattern = parse_regex_alternation(inner_slice)?;
                (SyntaxTerm::Group(inner_pattern), scan.saturating_add(1))
            }
            '[' => {
                let start = idx.saturating_add(1);
                let mut scan = start;
                while scan < chars.len() {
                    let Some(&c) = chars.get(scan) else {
                        break;
                    };
                    if c == '\\' {
                        scan = scan.saturating_add(2);
                        continue;
                    }
                    if c == ']' && scan > start {
                        break;
                    }
                    scan = scan.saturating_add(1);
                }
                ensure!(
                    scan < chars.len(),
                    "Unclosed '[' character class in regex extension pattern"
                );
                let class_slice = chars.get(start..scan).context(
                    "Invalid character class slice in regex extension pattern",
                )?;
                let class_term = parse_regex_char_class(class_slice)?;
                (class_term, scan.saturating_add(1))
            }
            '^' | '$' => {
                bail!(
                    "Unescaped anchor '{ch}' is only supported at pattern boundaries"
                );
            }
            '*' | '+' | '?' | '{' | '}' => {
                bail!("Unexpected quantifier token '{ch}' without preceding term");
            }
            other => (
                SyntaxTerm::CharRef(CharTarget::Unicode(u32::from(other))),
                idx.saturating_add(1),
            ),
        };

        let (quantifier, after_q) = parse_trailing_quantifier(chars, next_idx)?;
        idx = after_q;
        elements.push(SyntaxElement::with_quantifier(term, quantifier));
    }

    Ok(SyntaxPattern::Sequence(elements))
}

fn parse_regex_char_class(chars: &[char]) -> Result<SyntaxTerm> {
    let negated = chars.first() == Some(&'^');
    let mut idx = usize::from(negated);
    let mut items: Vec<(u32, Option<u32>)> = Vec::new();

    let read_class_char =
        |slice: &[char], pos: usize| -> Result<(u32, usize)> {
            let &c = slice
                .get(pos)
                .context("Unexpected end of regex character class")?;
            if c == '\\' {
                let esc_pos = pos.saturating_add(1);
                let &esc = slice
                    .get(esc_pos)
                    .context("Trailing backslash in regex character class")?;
                let cp = match esc {
                    't' => u32::from('\t'),
                    'n' => u32::from('\n'),
                    'r' => u32::from('\r'),
                    other => u32::from(other),
                };
                Ok((cp, esc_pos.saturating_add(1)))
            } else {
                Ok((u32::from(c), pos.saturating_add(1)))
            }
        };

    while idx < chars.len() {
        let (start_cp, next_idx) = read_class_char(chars, idx)?;
        if chars.get(next_idx) == Some(&'-')
            && next_idx.saturating_add(1) < chars.len()
        {
            let (end_cp, after_end) =
                read_class_char(chars, next_idx.saturating_add(1))?;
            ensure!(
                start_cp <= end_cp,
                "Reversed character range in regex character class"
            );
            items.push((start_cp, Some(end_cp)));
            idx = after_end;
        } else {
            items.push((start_cp, None));
            idx = next_idx;
        }
    }

    ensure!(!items.is_empty(), "Empty regex character class '[]'");

    if !negated && items.len() == 1 {
        if let Some(&(start_cp, Some(end_cp))) = items.first() {
            return Ok(SyntaxTerm::CharRange {
                start: CharTarget::Unicode(start_cp),
                end: CharTarget::Unicode(end_cp),
            });
        }
    }

    let mut members = Vec::new();
    for (start_cp, end_opt) in items {
        if let Some(end_cp) = end_opt {
            for cp in start_cp..=end_cp {
                members.push(CharTarget::Unicode(cp));
            }
        } else {
            members.push(CharTarget::Unicode(start_cp));
        }
    }

    Ok(SyntaxTerm::CharSet { negated, members })
}
