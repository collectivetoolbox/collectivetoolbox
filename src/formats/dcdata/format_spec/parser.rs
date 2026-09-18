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

//! Text parser for format specification DSL expressions.
//!
//! Enforces strict grouping and disambiguation rules as defined in
//! `docs/file-info.md`:
//! - Operational operators (`>`, `!`, `:`) bind left-to-right.
//! - Conjunction (`&`) and alternation (`|`) cannot be mixed with each other
//!   without explicit parentheses.
//! - Conjunction and alternation cannot be mixed with operational operators
//!   without explicit parentheses.
//! - Supports `@chain(...)` directive wrapper in source data.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail, ensure};
use ctb_storage_minimal::shorthand::DcShorthand;

use super::ast::{
    FormatExpr, FormatOp, MAX_FORMAT_EXPR_DEPTH, MAX_FORMAT_EXPR_NODES,
};

/// Lexical token in the format specification DSL.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    Gt,
    Bang,
    Colon,
    Amp,
    Pipe,
    LParen,
    RParen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

/// Tokenizes a format specification string into lexical tokens.
fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        match ch {
            '>' => tokens.push(Token {
                kind: TokenKind::Gt,
                start: idx,
                end: idx.saturating_add(1),
            }),
            '!' => tokens.push(Token {
                kind: TokenKind::Bang,
                start: idx,
                end: idx.saturating_add(1),
            }),
            ':' => tokens.push(Token {
                kind: TokenKind::Colon,
                start: idx,
                end: idx.saturating_add(1),
            }),
            '&' => tokens.push(Token {
                kind: TokenKind::Amp,
                start: idx,
                end: idx.saturating_add(1),
            }),
            '|' => tokens.push(Token {
                kind: TokenKind::Pipe,
                start: idx,
                end: idx.saturating_add(1),
            }),
            '(' => tokens.push(Token {
                kind: TokenKind::LParen,
                start: idx,
                end: idx.saturating_add(1),
            }),
            ')' => tokens.push(Token {
                kind: TokenKind::RParen,
                start: idx,
                end: idx.saturating_add(1),
            }),
            _ if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' => {
                let start = idx;
                let mut end = idx.saturating_add(ch.len_utf8());
                let mut ident = String::new();
                ident.push(ch);

                while let Some(&(next_idx, next_ch)) = chars.peek() {
                    if next_ch.is_ascii_alphanumeric()
                        || next_ch == '_'
                        || next_ch == '-'
                    {
                        ident.push(next_ch);
                        end = next_idx.saturating_add(next_ch.len_utf8());
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::Ident(ident),
                    start,
                    end,
                });
            }
            _ => {
                bail!(
                    "Unexpected character '{ch}' at offset {idx} in format specification"
                );
            }
        }
    }

    Ok(tokens)
}

/// Parses a format specification expression from a string slice.
///
/// Accepts both bare expressions (e.g. `f15 > f542`) and expressions wrapped in
/// `@chain(...)` directives (e.g. `@chain(((f15 > f542) ! f0) > f0)`).
///
/// # Errors
/// Returns an error on syntax errors, ambiguous operator mixtures, exceeding
/// maximum depth/complexity limits, or invalid token structures.
pub fn parse_format_expr(input: &str) -> Result<FormatExpr> {
    let trimmed = input.trim();
    ensure!(!trimmed.is_empty(), "Empty format specification expression");

    let expr_str = if let Some(stripped) = trimmed.strip_prefix("@chain(") {
        ensure!(
            stripped.ends_with(')'),
            "Malformed '@chain(...)': missing closing parenthesis"
        );
        let inner = &stripped[..stripped.len().saturating_sub(1)];
        inner.trim()
    } else {
        trimmed
    };

    ensure!(
        !expr_str.is_empty(),
        "Empty format specification inside '@chain(...)'"
    );

    let tokens = tokenize(expr_str)?;
    ensure!(!tokens.is_empty(), "No tokens found in format specification");

    let expr = parse_scope(&tokens, 0)?;

    let depth = expr.depth();
    ensure!(
        depth <= MAX_FORMAT_EXPR_DEPTH,
        "Format expression depth {depth} exceeds maximum allowed depth {MAX_FORMAT_EXPR_DEPTH}"
    );

    let nodes = expr.node_count();
    ensure!(
        nodes <= MAX_FORMAT_EXPR_NODES,
        "Format expression node count {nodes} exceeds maximum allowed nodes {MAX_FORMAT_EXPR_NODES}"
    );

    Ok(expr)
}

/// Parses a sequence of tokens within a single grouping scope.
fn parse_scope(tokens: &[Token], current_depth: usize) -> Result<FormatExpr> {
    ensure!(
        current_depth <= MAX_FORMAT_EXPR_DEPTH,
        "Format expression nesting exceeds maximum depth limit of {MAX_FORMAT_EXPR_DEPTH}"
    );
    ensure!(!tokens.is_empty(), "Empty expression within parentheses");

    // Partition the token slice at the current scope depth into operands and operators
    let mut items = Vec::new();
    let mut operators = Vec::new();

    let mut idx = 0usize;
    while idx < tokens.len() {
        let token = &tokens[idx];
        match &token.kind {
            TokenKind::LParen => {
                let start_idx = idx;
                let mut paren_depth = 1usize;
                idx = idx.saturating_add(1);

                while idx < tokens.len() && paren_depth > 0 {
                    match &tokens[idx].kind {
                        TokenKind::LParen => {
                            paren_depth = paren_depth.saturating_add(1);
                        }
                        TokenKind::RParen => {
                            paren_depth = paren_depth.saturating_sub(1);
                        }
                        _ => {}
                    }
                    idx = idx.saturating_add(1);
                }

                ensure!(
                    paren_depth == 0,
                    "Unclosed opening parenthesis '(' in format specification"
                );

                let inner_tokens = &tokens[start_idx.saturating_add(1)..idx.saturating_sub(1)];
                let sub_expr = parse_scope(inner_tokens, current_depth.saturating_add(1))?;
                items.push(sub_expr);
            }
            TokenKind::RParen => {
                bail!("Unexpected closing parenthesis ')' at offset {}", token.start);
            }
            TokenKind::Ident(ident) => {
                let operand = parse_operand(ident)?;
                items.push(operand);
                idx = idx.saturating_add(1);
            }
            TokenKind::Gt | TokenKind::Bang | TokenKind::Colon | TokenKind::Amp | TokenKind::Pipe => {
                let op = match &token.kind {
                    TokenKind::Gt => FormatOp::Convert,
                    TokenKind::Bang => FormatOp::Transmute,
                    TokenKind::Colon => FormatOp::Transform,
                    TokenKind::Amp => FormatOp::Union,
                    TokenKind::Pipe => FormatOp::Intersection,
                    _ => unreachable!("Handled by outer match"),
                };
                operators.push((op, token.start));
                idx = idx.saturating_add(1);
            }
        }
    }

    ensure!(
        items.len() == operators.len().saturating_add(1),
        "Mismatched operators and operands in format specification: found {} operands and {} operators",
        items.len(),
        operators.len()
    );

    if operators.is_empty() {
        let first = items
            .pop()
            .context("Internal parser error: missing operand")?;
        return Ok(first);
    }

    // Check operator consistency in this scope
    let has_union = operators.iter().any(|(op, _)| *op == FormatOp::Union);
    let has_intersection = operators.iter().any(|(op, _)| *op == FormatOp::Intersection);
    let has_operational = operators
        .iter()
        .any(|(op, _)| matches!(op, FormatOp::Convert | FormatOp::Transmute | FormatOp::Transform));

    if (has_union || has_intersection) && has_operational {
        bail!(
            "Ambiguous operator mixing: explicit parentheses are required when combining conjunction ('&') or alternation ('|') with operations ('>', '!', ':')"
        );
    }

    if has_union && has_intersection {
        bail!(
            "Ambiguous operator mixing: neither '&' nor '|' takes precedence over the other; explicit parentheses are required"
        );
    }

    if has_union {
        return Ok(FormatExpr::Union(items));
    }

    if has_intersection {
        return Ok(FormatExpr::Intersection(items));
    }

    // Operational operators: evaluate left-to-right
    let mut item_iter = items.into_iter();
    let mut current_expr = item_iter
        .next()
        .context("Internal parser error: empty operand list")?;

    for (op, _) in operators {
        let next_item = item_iter
            .next()
            .context("Internal parser error: missing right operand")?;
        match op {
            FormatOp::Convert => {
                current_expr =
                    FormatExpr::Convert(Box::new(current_expr), Box::new(next_item));
            }
            FormatOp::Transmute => {
                current_expr =
                    FormatExpr::Transmute(Box::new(current_expr), Box::new(next_item));
            }
            FormatOp::Transform => {
                current_expr =
                    FormatExpr::Transform(Box::new(current_expr), Box::new(next_item));
            }
            FormatOp::Union | FormatOp::Intersection => {
                unreachable!("Filtered out by consistency checks above");
            }
        }
    }

    Ok(current_expr)
}

/// Parses a leaf token into a `FormatExpr::Dc` or `FormatExpr::NamedType`.
fn parse_operand(ident: &str) -> Result<FormatExpr> {
    if let Ok(shorthand) = DcShorthand::parse(ident) {
        return Ok(FormatExpr::Dc(shorthand));
    }

    // Registered named types: lower_snake_case or lowercase
    if is_valid_named_type_syntax(ident) {
        return Ok(FormatExpr::NamedType(ident.to_string()));
    }

    bail!(
        "Invalid format specification operand '{ident}': expected numeric Dc shorthand (e.g. f542, l2228766, 0) or registered named type"
    )
}

/// Validates whether an identifier string adheres to the lexical syntax of named types.
fn is_valid_named_type_syntax(ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    ident
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
