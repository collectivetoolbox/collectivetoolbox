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

//! Text formatter and canonical printer for format specification expressions.
//!
//! Emits unambiguous format expression strings with minimal, necessary parentheses
//! adhering to the associativity and grouping rules of `docs/file-info.md`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;

use super::ast::{FormatExpr, FormatOp};

impl fmt::Display for FormatExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_expr(self))
    }
}

/// Formats a `FormatExpr` into its canonical text representation.
#[must_use]
pub fn format_expr(expr: &FormatExpr) -> String {
    format_expr_inner(expr, None, false)
}

/// Formats a `FormatExpr` wrapped in a `@chain(...)` directive.
#[must_use]
pub fn format_chain_directive(expr: &FormatExpr) -> String {
    format!("@chain({})", format_expr(expr))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParentContext {
    Operational(FormatOp),
    Union,
    Intersection,
}

fn format_expr_inner(
    expr: &FormatExpr,
    parent_ctx: Option<ParentContext>,
    is_right_child_of_op: bool,
) -> String {
    match expr {
        FormatExpr::Dc(shorthand) => shorthand.to_string(),
        FormatExpr::NamedType(name) => name.clone(),
        FormatExpr::Convert(left, right) => {
            let left_str = format_expr_inner(
                left,
                Some(ParentContext::Operational(FormatOp::Convert)),
                false,
            );
            let right_str = format_expr_inner(
                right,
                Some(ParentContext::Operational(FormatOp::Convert)),
                true,
            );
            let res = format!("{left_str} > {right_str}");
            wrap_operational(res, FormatOp::Convert, parent_ctx, is_right_child_of_op)
        }
        FormatExpr::Transmute(left, right) => {
            let left_str = format_expr_inner(
                left,
                Some(ParentContext::Operational(FormatOp::Transmute)),
                false,
            );
            let right_str = format_expr_inner(
                right,
                Some(ParentContext::Operational(FormatOp::Transmute)),
                true,
            );
            let res = format!("{left_str} ! {right_str}");
            wrap_operational(
                res,
                FormatOp::Transmute,
                parent_ctx,
                is_right_child_of_op,
            )
        }
        FormatExpr::Transform(left, right) => {
            let left_str = format_expr_inner(
                left,
                Some(ParentContext::Operational(FormatOp::Transform)),
                false,
            );
            let right_str = format_expr_inner(
                right,
                Some(ParentContext::Operational(FormatOp::Transform)),
                true,
            );
            let res = format!("{left_str} : {right_str}");
            wrap_operational(
                res,
                FormatOp::Transform,
                parent_ctx,
                is_right_child_of_op,
            )
        }
        FormatExpr::Union(children) => {
            let parts: Vec<String> = children
                .iter()
                .map(|child| format_expr_inner(child, Some(ParentContext::Union), false))
                .collect();
            let res = parts.join(" & ");
            match parent_ctx {
                Some(
                    ParentContext::Operational(_)
                    | ParentContext::Intersection,
                ) => format!("({res})"),
                _ => res,
            }
        }
        FormatExpr::Intersection(children) => {
            let parts: Vec<String> = children
                .iter()
                .map(|child| {
                    format_expr_inner(child, Some(ParentContext::Intersection), false)
                })
                .collect();
            let res = parts.join(" | ");
            match parent_ctx {
                Some(
                    ParentContext::Operational(_) | ParentContext::Union,
                ) => format!("({res})"),
                _ => res,
            }
        }
    }
}

fn wrap_operational(
    s: String,
    current_op: FormatOp,
    parent_ctx: Option<ParentContext>,
    is_right_child_of_op: bool,
) -> String {
    if is_right_child_of_op {
        return format!("({s})");
    }
    match parent_ctx {
        Some(ParentContext::Operational(parent_op)) if parent_op != current_op => {
            format!("({s})")
        }
        Some(ParentContext::Union | ParentContext::Intersection) => {
            format!("({s})")
        }
        _ => s,
    }
}
