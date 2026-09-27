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

//! Abstract Syntax Tree (AST) definitions for format specifications.
//!
//! Provides the data structures representing format composition, conversion,
//! transmutation, transformation, conjunction, and alternation as outlined in
//! `docs/file-info.md`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_storage_minimal::shorthand::DcShorthand;
use serde::{Deserialize, Serialize};

/// Maximum expression recursion depth allowed to prevent stack overflow.
pub const MAX_FORMAT_EXPR_DEPTH: usize = 32;

/// Maximum number of AST nodes permitted in a single format expression.
pub const MAX_FORMAT_EXPR_NODES: usize = 256;

/// Operator types for format composition expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatOp {
    /// Format conversion / encoding: `>` (Dc 302).
    Convert,
    /// Format transmutation / representation reinterpretation: `!` (Dc 303).
    Transmute,
    /// Transformation application: `:` (Dc 301).
    Transform,
    /// Type union / conjunction of simultaneous constraints: `&` (Dc 300).
    Union,
    /// Type intersection / alternative constraints: `|` (Dc 516).
    Intersection,
}

impl FormatOp {
    /// Returns the short Dc ID corresponding to this operator.
    #[must_use]
    pub const fn dc_id(self) -> u32 {
        match self {
            Self::Union => 300,
            Self::Transform => 301,
            Self::Convert => 302,
            Self::Transmute => 303,
            Self::Intersection => 516,
        }
    }

    /// Returns the operator corresponding to a short Dc ID, if valid.
    #[must_use]
    pub const fn from_dc_id(id: u32) -> Option<Self> {
        match id {
            300 => Some(Self::Union),
            301 => Some(Self::Transform),
            302 => Some(Self::Convert),
            303 => Some(Self::Transmute),
            516 => Some(Self::Intersection),
            _ => None,
        }
    }

    /// Returns the textual symbol for this operator.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Union => "&",
            Self::Transform => ":",
            Self::Convert => ">",
            Self::Transmute => "!",
            Self::Intersection => "|",
        }
    }
}

/// Typed value in a format parameter binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamValue {
    /// Integer scalar (e.g. 16 in radix=16).
    Integer(i64),
    /// Dc shorthand reference (e.g. f354, or variant format Dcs).
    Dc(DcShorthand),
    /// Symbolic identifier.
    Ident(String),
    /// Quoted string literal.
    String(String),
    /// Nested format specification expression.
    Expr(Box<FormatExpr>),
}

/// Key-value binding in a parametric format application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParamBinding {
    /// Parameter dimension name (e.g. "radix", "case", "padding").
    pub name: String,
    /// Parameter value.
    pub value: ParamValue,
}

/// Core expression node in a format specification expression tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatExpr {
    /// Numeric Document Character shorthand (e.g. `f542`, `l2228766`, `0`, `u0020`).
    Dc(DcShorthand),
    /// Explicitly registered stable named type (e.g. `string`, `identifier`, `number`).
    NamedType(String),
    /// Format conversion / encoding: `A > B` (Dc 302).
    Convert(Box<FormatExpr>, Box<FormatExpr>),
    /// Format transmutation / representation reinterpretation: `A ! B` (Dc 303).
    Transmute(Box<FormatExpr>, Box<FormatExpr>),
    /// Transformation application: `A : T` (Dc 301).
    Transform(Box<FormatExpr>, Box<FormatExpr>),
    /// Type union / conjunction of simultaneous constraints: `A & B` (Dc 300).
    Union(Vec<FormatExpr>),
    /// Type intersection / alternative constraints: `A | B` (Dc 516).
    Intersection(Vec<FormatExpr>),
    /// Parametric format application: `target(key=value, ...)` (Dc 535).
    Apply {
        /// Application target expression (must resolve to a format Dc shorthand).
        target: Box<FormatExpr>,
        /// Bound parameter values.
        params: Vec<ParamBinding>,
    },
}

impl FormatExpr {
    /// Calculates the maximum tree depth of this expression.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Dc(_) | Self::NamedType(_) => 1,
            Self::Convert(l, r) | Self::Transmute(l, r) | Self::Transform(l, r) => {
                let max_child = l.depth().max(r.depth());
                max_child.saturating_add(1)
            }
            Self::Union(children) | Self::Intersection(children) => {
                let mut max_child = 0usize;
                for child in children {
                    max_child = max_child.max(child.depth());
                }
                max_child.saturating_add(1)
            }
            Self::Apply { target, params } => {
                let mut max_child = target.depth();
                for param in params {
                    if let ParamValue::Expr(sub) = &param.value {
                        max_child = max_child.max(sub.depth());
                    }
                }
                max_child.saturating_add(1)
            }
        }
    }

    /// Calculates the total number of AST nodes in this expression.
    #[must_use]
    pub fn node_count(&self) -> usize {
        match self {
            Self::Dc(_) | Self::NamedType(_) => 1,
            Self::Convert(l, r) | Self::Transmute(l, r) | Self::Transform(l, r) => {
                let left_nodes = l.node_count();
                let right_nodes = r.node_count();
                left_nodes
                    .saturating_add(right_nodes)
                    .saturating_add(1)
            }
            Self::Union(children) | Self::Intersection(children) => {
                let mut total = 1usize;
                for child in children {
                    total = total.saturating_add(child.node_count());
                }
                total
            }
            Self::Apply { target, params } => {
                let mut total = target.node_count().saturating_add(1);
                for param in params {
                    total = total.saturating_add(1);
                    if let ParamValue::Expr(sub) = &param.value {
                        total = total.saturating_add(sub.node_count());
                    }
                }
                total
            }
        }
    }

    /// Returns true if this expression is a leaf node (Dc or NamedType).
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        matches!(self, Self::Dc(_) | Self::NamedType(_))
    }
}

impl Serialize for FormatExpr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&super::formatter::format_expr(self))
    }
}

impl<'de> Deserialize<'de> for FormatExpr {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        super::parser::parse_format_expr(&s).map_err(serde::de::Error::custom)
    }
}

