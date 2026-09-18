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

