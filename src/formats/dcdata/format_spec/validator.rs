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

//! Semantic validator for format specification expressions.
//!
//! Enforces operand constraints:
//! - Named types must be explicitly registered in `README-named-types.csv`.
//! - Transformation operands in `A : T` must reference valid transformation entities.
//! - Rejects unverified nicknames or aliases in persisted expressions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, bail, ensure};
use ctb_storage_minimal::shorthand::DcShorthand;

use super::ast::{
    FormatExpr, MAX_FORMAT_EXPR_DEPTH, MAX_FORMAT_EXPR_NODES,
};

/// Set of registered stable named types defined in `README-named-types.csv`.
pub const REGISTERED_NAMED_TYPES: &[&str] = &[
    "number",
    "string",
    "date",
    "data",
    "checksum",
    "unicode_any",
    "identifier",
    "type",
    "format",
    "list",
    "key_value",
    "duration",
    "datemagnitude",
    "asciilatinletter",
    "asciilatinletterdigit",
    "asciiprintable",
    "filename",
    "filepath",
    "ipaddr",
    "statement",
    "value",
];

/// Known transformation format IDs (category `transformation`).
pub const KNOWN_TRANSFORMATION_FORMAT_IDS: &[usize] = &[
    19,  // SemanticToText
    20,  // CodeToText
    323, // AlturaMacToWin
    324, // AlturaWinToMac
];

/// Validates semantic constraints of a `FormatExpr`.
///
/// # Errors
/// Returns an error if an unregistered named type is encountered, if a transformation
/// target is invalid, or if tree complexity bounds are exceeded.
pub fn validate_format_expr(expr: &FormatExpr) -> Result<()> {
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

    validate_node_recursive(expr)
}

fn validate_node_recursive(expr: &FormatExpr) -> Result<()> {
    match expr {
        FormatExpr::Dc(shorthand) => {
            validate_shorthand_operand(shorthand)?;
        }
        FormatExpr::NamedType(name) => {
            ensure!(
                REGISTERED_NAMED_TYPES.contains(&name.as_str()),
                "Unregistered named type '{name}' in format specification; only stable types from README-named-types.csv are permitted"
            );
        }
        FormatExpr::Convert(left, right) | FormatExpr::Transmute(left, right) => {
            validate_node_recursive(left)?;
            validate_node_recursive(right)?;
        }
        FormatExpr::Transform(left, target) => {
            validate_node_recursive(left)?;
            validate_transformation_target(target)?;
        }
        FormatExpr::Union(children) | FormatExpr::Intersection(children) => {
            ensure!(
                children.len() >= 2,
                "Conjunction or alternation must have at least 2 children, found {}",
                children.len()
            );
            for child in children {
                validate_node_recursive(child)?;
            }
        }
    }
    Ok(())
}

fn validate_shorthand_operand(shorthand: &DcShorthand) -> Result<()> {
    match shorthand {
        DcShorthand::Format(fmt_id) => {
            ensure!(
                *fmt_id <= ctb_storage_minimal::shorthand::MAX_FORMAT_ID,
                "Format ID f{fmt_id} exceeds maximum known format bounds"
            );
        }
        DcShorthand::Short(short_dc) => {
            ensure!(
                *short_dc <= ctb_storage_minimal::shorthand::MAX_SHORT_DC,
                "Short Dc ID {short_dc} exceeds maximum known bounds"
            );
        }
        DcShorthand::Unicode(cp) => {
            ensure!(
                *cp <= 0x0010_FFFF,
                "Unicode codepoint u{cp:x} exceeds maximum 0x10FFFF"
            );
        }
        DcShorthand::Long(_) | DcShorthand::Local(_) => {}
    }
    Ok(())
}

fn validate_transformation_target(target: &FormatExpr) -> Result<()> {
    match target {
        FormatExpr::Dc(DcShorthand::Format(fmt_id)) => {
            ensure!(
                KNOWN_TRANSFORMATION_FORMAT_IDS.contains(fmt_id),
                "Format f{fmt_id} is not a registered transformation format"
            );
            Ok(())
        }
        FormatExpr::NamedType(name) => {
            ensure!(
                name == "type" || name == "format",
                "Named type '{name}' is not valid as a transformation target"
            );
            Ok(())
        }
        other => {
            bail!(
                "Invalid transformation target '{other:?}': expected format reference (e.g. f323) or registered transformation"
            );
        }
    }
}
