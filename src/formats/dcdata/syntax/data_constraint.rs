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

//! Data constraint parsing shared by the Dc syntax parser and validator.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Parses an optional format and decoded byte count for a data construct.
pub fn parse_data_constraint(
    subtype: &str,
) -> anyhow::Result<(Option<&str>, Option<usize>)> {
    let (format, length) = if subtype.starts_with("bytes=") {
        (None, Some(subtype))
    } else if let Some((format, length)) = subtype.split_once(';') {
        (Some(format.trim()), Some(length.trim()))
    } else {
        (Some(subtype.trim()), None)
    };
    anyhow::ensure!(
        format.is_none_or(|format| !format.is_empty()),
        "Missing data format"
    );
    let bytes = if let Some(length) = length {
        let digits = length.strip_prefix("bytes=").ok_or_else(|| {
            anyhow::anyhow!("Expected bytes=N data constraint")
        })?;
        anyhow::ensure!(
            !digits.is_empty()
                && digits.bytes().all(|byte| byte.is_ascii_digit()),
            "Data byte count must be a nonnegative integer"
        );
        Some(digits.parse::<usize>()?)
    } else {
        None
    };
    Ok((format, bytes))
}

/// A single clause in a `&`-separated `[number:...]` subtype constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberConstraintClause<'a> {
    /// A format shorthand (`f<N>`), format identifier (`Natural0`), or named construct.
    Target(&'a str),
    /// A lower bound (`>=` when `exclusive == false`, `>` when `exclusive == true`).
    LowerBound {
        exclusive: bool,
        bound: crate::graph_ledger::NumericBoundValue,
    },
    /// An upper bound (`<=` when `exclusive == false`, `<` when `exclusive == true`).
    UpperBound {
        exclusive: bool,
        bound: crate::graph_ledger::NumericBoundValue,
    },
}

/// Parses a `[number:...]` subtype constraint into its `&`-separated clauses.
pub fn parse_number_constraint(
    subtype: &str,
) -> anyhow::Result<Vec<NumberConstraintClause<'_>>> {
    let trimmed = subtype.trim();
    anyhow::ensure!(
        !trimmed.is_empty(),
        "Empty [number:] subtype constraint"
    );
    let mut clauses = Vec::new();
    for raw_part in trimmed.split('&') {
        let part = raw_part.trim();
        anyhow::ensure!(
            !part.is_empty(),
            "Empty clause in [number:{subtype}] subtype constraint"
        );
        if let Some(rest) = part.strip_prefix(">=") {
            clauses.push(NumberConstraintClause::LowerBound {
                exclusive: false,
                bound: crate::graph_ledger::NumericBoundValue::parse_dsl_bound(
                    rest,
                )?,
            });
        } else if let Some(rest) = part.strip_prefix('>') {
            clauses.push(NumberConstraintClause::LowerBound {
                exclusive: true,
                bound: crate::graph_ledger::NumericBoundValue::parse_dsl_bound(
                    rest,
                )?,
            });
        } else if let Some(rest) = part.strip_prefix("<=") {
            clauses.push(NumberConstraintClause::UpperBound {
                exclusive: false,
                bound: crate::graph_ledger::NumericBoundValue::parse_dsl_bound(
                    rest,
                )?,
            });
        } else if let Some(rest) = part.strip_prefix('<') {
            clauses.push(NumberConstraintClause::UpperBound {
                exclusive: true,
                bound: crate::graph_ledger::NumericBoundValue::parse_dsl_bound(
                    rest,
                )?,
            });
        } else {
            clauses.push(NumberConstraintClause::Target(part));
        }
    }
    Ok(clauses)
}
