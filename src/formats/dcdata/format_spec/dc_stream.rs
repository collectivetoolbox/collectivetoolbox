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

//! Document Character (Dc) token stream encoder and decoder.
//!
//! Implements canonical balanced-group prefix encoding for format specifications:
//! `298 operator left right 299`
//! as specified in `docs/file-info.md`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, bail, ensure};
use ctb_storage_minimal::shorthand::DcShorthand;
use serde::{Deserialize, Serialize};

use super::ast::{
    FormatExpr, FormatOp, MAX_FORMAT_EXPR_DEPTH, MAX_FORMAT_EXPR_NODES,
};

/// Token in a Dc format specification stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DcToken {
    /// Document Character shorthand (e.g. 298, 299, 300, f0, l2228766).
    Dc(DcShorthand),
    /// Registered named type identifier.
    NamedType(String),
}

impl std::fmt::Display for DcToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dc(shorthand) => write!(f, "{shorthand}"),
            Self::NamedType(name) => write!(f, "{name}"),
        }
    }
}

impl Serialize for DcToken {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DcToken {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Ok(shorthand) = DcShorthand::parse(&s) {
            Ok(Self::Dc(shorthand))
        } else {
            Ok(Self::NamedType(s))
        }
    }
}

/// Encodes a `FormatExpr` into a canonical slice of `DcToken`s.
#[must_use]
pub fn encode_dc_stream(expr: &FormatExpr) -> Vec<DcToken> {
    let mut tokens = Vec::new();
    encode_expr_recursive(expr, &mut tokens);
    tokens
}

/// Encodes a `FormatExpr` into a space-separated symbolic token stream string.
#[must_use]
pub fn encode_dc_stream_string(expr: &FormatExpr) -> String {
    let tokens = encode_dc_stream(expr);
    let parts: Vec<String> = tokens.iter().map(ToString::to_string).collect();
    parts.join(" ")
}

fn encode_expr_recursive(expr: &FormatExpr, tokens: &mut Vec<DcToken>) {
    match expr {
        FormatExpr::Dc(shorthand) => {
            tokens.push(DcToken::Dc(*shorthand));
        }
        FormatExpr::NamedType(name) => {
            tokens.push(DcToken::NamedType(name.clone()));
        }
        FormatExpr::Convert(left, right) => {
            encode_binary_op(FormatOp::Convert, left, right, tokens);
        }
        FormatExpr::Transmute(left, right) => {
            encode_binary_op(FormatOp::Transmute, left, right, tokens);
        }
        FormatExpr::Transform(left, right) => {
            encode_binary_op(FormatOp::Transform, left, right, tokens);
        }
        FormatExpr::Union(children) => {
            encode_nary_op(FormatOp::Union, children, tokens);
        }
        FormatExpr::Intersection(children) => {
            encode_nary_op(FormatOp::Intersection, children, tokens);
        }
    }
}

fn encode_binary_op(
    op: FormatOp,
    left: &FormatExpr,
    right: &FormatExpr,
    tokens: &mut Vec<DcToken>,
) {
    tokens.push(DcToken::Dc(DcShorthand::Short(298)));
    tokens.push(DcToken::Dc(DcShorthand::Short(op.dc_id())));
    encode_expr_recursive(left, tokens);
    encode_expr_recursive(right, tokens);
    tokens.push(DcToken::Dc(DcShorthand::Short(299)));
}

fn encode_nary_op(op: FormatOp, children: &[FormatExpr], tokens: &mut Vec<DcToken>) {
    if children.is_empty() {
        return;
    }
    if children.len() == 1 {
        if let Some(first) = children.first() {
            encode_expr_recursive(first, tokens);
        }
        return;
    }

    // Fold left-associatively: (A op B) op C
    let mut iter = children.iter();
    if let Some(first) = iter.next() {
        let mut accumulated_tree = first.clone();
        for next in iter {
            accumulated_tree = match op {
                FormatOp::Union => {
                    FormatExpr::Union(vec![accumulated_tree, next.clone()])
                }
                FormatOp::Intersection => {
                    FormatExpr::Intersection(vec![accumulated_tree, next.clone()])
                }
                _ => unreachable!("Only used for Union and Intersection"),
            };
        }

        // Now encode the binary folded tree
        match accumulated_tree {
            FormatExpr::Union(sub) => {
                if sub.len() == 2 {
                    if let (Some(l), Some(r)) = (sub.first(), sub.get(1)) {
                        encode_binary_op(op, l, r, tokens);
                    }
                }
            }
            FormatExpr::Intersection(sub) => {
                if sub.len() == 2 {
                    if let (Some(l), Some(r)) = (sub.first(), sub.get(1)) {
                        encode_binary_op(op, l, r, tokens);
                    }
                }
            }
            other => encode_expr_recursive(&other, tokens),
        }
    }
}

/// Decodes a string of space-separated Dc tokens into a `FormatExpr`.
///
/// # Errors
/// Returns an error if the token stream has unmatched groups, unknown operators,
/// or exceeds depth/size limits.
pub fn decode_dc_stream_string(input: &str) -> Result<FormatExpr> {
    let tokens = parse_tokens_from_str(input)?;
    decode_dc_stream(&tokens)
}

/// Parses a space-separated string of tokens into `Vec<DcToken>`.
fn parse_tokens_from_str(input: &str) -> Result<Vec<DcToken>> {
    let mut tokens = Vec::new();
    for part in input.split_whitespace() {
        if let Ok(shorthand) = DcShorthand::parse(part) {
            tokens.push(DcToken::Dc(shorthand));
        } else {
            tokens.push(DcToken::NamedType(part.to_string()));
        }
    }
    Ok(tokens)
}

/// Decodes a slice of `DcToken`s into a `FormatExpr`.
///
/// # Errors
/// Returns an error on structural or syntax mismatch.
pub fn decode_dc_stream(tokens: &[DcToken]) -> Result<FormatExpr> {
    ensure!(!tokens.is_empty(), "Empty Dc token stream");
    let mut pos = 0usize;
    let expr = decode_recursive(tokens, &mut pos, 0)?;

    ensure!(
        pos == tokens.len(),
        "Trailing unused tokens in Dc stream starting at token index {pos}"
    );

    let depth = expr.depth();
    ensure!(
        depth <= MAX_FORMAT_EXPR_DEPTH,
        "Decoded format expression depth {depth} exceeds maximum allowed depth {MAX_FORMAT_EXPR_DEPTH}"
    );

    let nodes = expr.node_count();
    ensure!(
        nodes <= MAX_FORMAT_EXPR_NODES,
        "Decoded format expression node count {nodes} exceeds maximum allowed nodes {MAX_FORMAT_EXPR_NODES}"
    );

    Ok(expr)
}

fn decode_recursive(
    tokens: &[DcToken],
    pos: &mut usize,
    depth: usize,
) -> Result<FormatExpr> {
    ensure!(
        depth <= MAX_FORMAT_EXPR_DEPTH,
        "Dc token stream nesting exceeds maximum depth limit of {MAX_FORMAT_EXPR_DEPTH}"
    );
    ensure!(
        *pos < tokens.len(),
        "Unexpected end of Dc token stream: missing operand"
    );

    let token = &tokens[*pos];
    *pos = pos.saturating_add(1);

    match token {
        DcToken::Dc(DcShorthand::Short(298)) => {
            // Group start: next token must be operator
            ensure!(
                *pos < tokens.len(),
                "Unexpected end of stream after group start '298': expected operator"
            );
            let op_token = &tokens[*pos];
            *pos = pos.saturating_add(1);

            let op = match op_token {
                DcToken::Dc(DcShorthand::Short(id)) => {
                    FormatOp::from_dc_id(*id).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Invalid operator in Dc group: expected operator 300, 301, 302, 303, or 516, found '{id}'"
                        )
                    })?
                }
                other => {
                    bail!(
                        "Invalid operator in Dc group: expected operator 300, 301, 302, 303, or 516, found '{other}'"
                    );
                }
            };

            let left = decode_recursive(tokens, pos, depth.saturating_add(1))?;
            let right = decode_recursive(tokens, pos, depth.saturating_add(1))?;

            ensure!(
                *pos < tokens.len(),
                "Unclosed Dc associativity group: expected terminating '299'"
            );
            let closing = &tokens[*pos];
            *pos = pos.saturating_add(1);

            ensure!(
                *closing == DcToken::Dc(DcShorthand::Short(299)),
                "Expected terminating '299' for Dc group, found '{closing}'"
            );

            Ok(build_op_expr(op, left, right))
        }
        DcToken::Dc(DcShorthand::Short(op_id))
            if FormatOp::from_dc_id(*op_id).is_some() =>
        {
            let op = FormatOp::from_dc_id(*op_id).expect("Guaranteed by guard");
            let left = decode_recursive(tokens, pos, depth.saturating_add(1))?;
            let right = decode_recursive(tokens, pos, depth.saturating_add(1))?;

            // Flexibly consume optional closing 299 if present
            if *pos < tokens.len()
                && tokens[*pos] == DcToken::Dc(DcShorthand::Short(299))
            {
                *pos = pos.saturating_add(1);
            }

            Ok(build_op_expr(op, left, right))
        }
        DcToken::Dc(DcShorthand::Short(299)) => {
            bail!("Unexpected closing group '299' without matching opening '298'");
        }
        DcToken::Dc(shorthand) => Ok(FormatExpr::Dc(*shorthand)),
        DcToken::NamedType(name) => Ok(FormatExpr::NamedType(name.clone())),
    }
}

fn build_op_expr(op: FormatOp, left: FormatExpr, right: FormatExpr) -> FormatExpr {
    match op {
        FormatOp::Convert => FormatExpr::Convert(Box::new(left), Box::new(right)),
        FormatOp::Transmute => {
            FormatExpr::Transmute(Box::new(left), Box::new(right))
        }
        FormatOp::Transform => {
            FormatExpr::Transform(Box::new(left), Box::new(right))
        }
        FormatOp::Union => flatten_or_create_union(left, right),
        FormatOp::Intersection => flatten_or_create_intersection(left, right),
    }
}

fn flatten_or_create_union(left: FormatExpr, right: FormatExpr) -> FormatExpr {
    match (left, right) {
        (FormatExpr::Union(mut l_vec), FormatExpr::Union(r_vec)) => {
            l_vec.extend(r_vec);
            FormatExpr::Union(l_vec)
        }
        (FormatExpr::Union(mut l_vec), single) => {
            l_vec.push(single);
            FormatExpr::Union(l_vec)
        }
        (single, FormatExpr::Union(mut r_vec)) => {
            r_vec.insert(0, single);
            FormatExpr::Union(r_vec)
        }
        (l, r) => FormatExpr::Union(vec![l, r]),
    }
}

fn flatten_or_create_intersection(
    left: FormatExpr,
    right: FormatExpr,
) -> FormatExpr {
    match (left, right) {
        (FormatExpr::Intersection(mut l_vec), FormatExpr::Intersection(r_vec)) => {
            l_vec.extend(r_vec);
            FormatExpr::Intersection(l_vec)
        }
        (FormatExpr::Intersection(mut l_vec), single) => {
            l_vec.push(single);
            FormatExpr::Intersection(l_vec)
        }
        (single, FormatExpr::Intersection(mut r_vec)) => {
            r_vec.insert(0, single);
            FormatExpr::Intersection(r_vec)
        }
        (l, r) => FormatExpr::Intersection(vec![l, r]),
    }
}
