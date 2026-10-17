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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Parses an optional format and decoded byte count for a data construct.
pub fn parse_data_constraint(subtype: &str) -> anyhow::Result<(Option<&str>, Option<usize>)> {
    let (format, length) = if subtype.starts_with("bytes=") {
        (None, Some(subtype))
    } else if let Some((format, length)) = subtype.split_once(';') {
        (Some(format.trim()), Some(length.trim()))
    } else {
        (Some(subtype.trim()), None)
    };
    anyhow::ensure!(format.is_none_or(|format| !format.is_empty()), "Missing data format");
    let bytes = if let Some(length) = length {
        let digits = length.strip_prefix("bytes=").ok_or_else(|| anyhow::anyhow!("Expected bytes=N data constraint"))?;
        anyhow::ensure!(!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
            "Data byte count must be a nonnegative integer");
        Some(digits.parse::<usize>()?)
    } else {
        None
    };
    Ok((format, bytes))
}
