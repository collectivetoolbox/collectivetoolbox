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

//! Canonical parser and data model for Document Character (Dc) shorthands.
//!
//! Implements shorthand syntax according to `README.shorthand.md`:
//! - Bare integer: short Dc (`\d+`)
//! - `u` prefix: Unicode character, hexadecimal (`u[0-9a-f]+`)
//! - `f` prefix: Format Dc (`f\d+`)
//! - `l` prefix: Long Dc (`l\d+`)
//! - `L` prefix: Local graph ID (`L\d+`)
//!
//! Shorthand tokens are strictly case-sensitive. Valid shorthands match
//! `[flL]?\d+` or `u[0-9a-f]+`; non-matching strings are rejected.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, bail, ensure};

use crate::global_graph_layout::{
    UNICODE_REGION_END, dc_to_gid, format_to_gid,
};

/// Maximum valid short Dc ID.
pub const MAX_SHORT_DC: u32 = 1_114_111;

/// Maximum valid short Format ID.
pub const MAX_FORMAT_ID: usize = 1_114_111;

/// Representation of a parsed Document Character (Dc) shorthand identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DcShorthand {
    /// Bare integer short Dc ID (`\d+`), mapping to `SHORT_DC_REGION_START + id`.
    Short(u32),
    /// Unicode codepoint (`u[0-9a-f]+`), mapping to `0..=0x10FFFF`.
    Unicode(u32),
    /// Format ID (`f\d+`), mapping to `FORMAT_REGION_START + id`.
    Format(usize),
    /// Long Dc ID (`l\d+`), mapping to exact 128-bit Dc ID.
    Long(u128),
    /// Local graph ID (`L\d+`), equivalent to short Dc 296 followed by a Dc
    /// number representation of the integer following the `L`.
    Local(u128),
}

impl DcShorthand {
    /// Parses a string slice into a `DcShorthand`.
    ///
    /// The string must match `[flL]?\d+` or `u[0-9a-f]+` per `README.shorthand.md`.
    /// Tokens are strictly case-sensitive; uppercase variants like `F`, `U`,
    /// or uppercase hex digits in `u` are rejected.
    ///
    /// # Errors
    /// Returns an error if the input string is empty, contains invalid prefixes
    /// or characters, or exceeds numerical bounds.
    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        ensure!(!trimmed.is_empty(), "Empty Dc shorthand");

        if let Some(hex_part) = trimmed.strip_prefix('u') {
            ensure!(
                !hex_part.is_empty(),
                "Unicode shorthand 'u' requires at least one hex digit"
            );
            ensure!(
                hex_part
                    .chars()
                    .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
                "Invalid Unicode shorthand '{trimmed}': prefix 'u' must be followed by lowercase hex digits [0-9a-f]+"
            );
            let cp = u32::from_str_radix(hex_part, 16)
                .map_err(|e| anyhow::anyhow!("Invalid hex in Unicode shorthand '{trimmed}': {e}"))?;
            ensure!(
                u128::from(cp) <= UNICODE_REGION_END,
                "Unicode codepoint 0x{cp:X} exceeds maximum 0x10FFFF"
            );
            return Ok(Self::Unicode(cp));
        }

        if let Some(rest) = trimmed.strip_prefix('f') {
            ensure!(
                !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
                "Invalid format shorthand '{trimmed}': prefix 'f' must be followed by decimal digits [0-9]+"
            );
            let fmt_id = rest
                .parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Invalid format ID in shorthand '{trimmed}': {e}"))?;
            ensure!(
                fmt_id <= MAX_FORMAT_ID,
                "Format ID {fmt_id} exceeds maximum {MAX_FORMAT_ID}"
            );
            return Ok(Self::Format(fmt_id));
        }

        if let Some(rest) = trimmed.strip_prefix('l') {
            ensure!(
                !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
                "Invalid long Dc shorthand '{trimmed}': prefix 'l' must be followed by decimal digits [0-9]+"
            );
            let val = rest
                .parse::<u128>()
                .map_err(|e| anyhow::anyhow!("Invalid long Dc in shorthand '{trimmed}': {e}"))?;
            return Ok(Self::Long(val));
        }

        if let Some(rest) = trimmed.strip_prefix('L') {
            ensure!(
                !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
                "Invalid local graph ID shorthand '{trimmed}': prefix 'L' must be followed by decimal digits [0-9]+"
            );
            let val = rest
                .parse::<u128>()
                .map_err(|e| anyhow::anyhow!("Invalid local graph ID in shorthand '{trimmed}': {e}"))?;
            return Ok(Self::Local(val));
        }

        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            let val = trimmed
                .parse::<u32>()
                .map_err(|e| anyhow::anyhow!("Invalid short Dc shorthand '{trimmed}': {e}"))?;
            ensure!(
                val <= MAX_SHORT_DC,
                "Short Dc {val} exceeds maximum {MAX_SHORT_DC}"
            );
            return Ok(Self::Short(val));
        }

        bail!(
            "Unrecognized or invalid Dc shorthand '{trimmed}'. Expected bare integer, 'u<hex>', 'f<digits>', 'l<digits>', or 'L<digits>'"
        );
    }

    /// Maps the shorthand representation into a 128-bit Global Graph ID.
    ///
    /// # Errors
    /// Returns an error if this is a `Local` shorthand (which expands to Dc 296
    /// followed by a Dc number instead of a single global ID).
    pub fn to_global_id(&self) -> Result<u128> {
        match self {
            Self::Short(val) => Ok(dc_to_gid(u64::from(*val))),
            Self::Unicode(cp) => Ok(u128::from(*cp)),
            Self::Format(fmt_id) => {
                let id_u64 = u64::try_from(*fmt_id)
                    .map_err(|e| anyhow::anyhow!("Format ID overflow: {e}"))?;
                Ok(format_to_gid(id_u64))
            }
            Self::Long(val) => Ok(*val),
            Self::Local(val) => bail!(
                "Local graph ID shorthand 'L{val}' cannot be converted to a single global Dc ID (equivalent to short Dc 296 followed by a Dc number)"
            ),
        }
    }

    /// Formats the shorthand as its canonical string representation.
    #[must_use]
    pub fn to_shorthand_string(&self) -> String {
        match self {
            Self::Short(id) => format!("{id}"),
            Self::Unicode(cp) => format!("u{cp:x}"),
            Self::Format(id) => format!("f{id}"),
            Self::Long(id) => format!("l{id}"),
            Self::Local(id) => format!("L{id}"),
        }
    }
}

impl std::fmt::Display for DcShorthand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_shorthand_string())
    }
}

impl std::str::FromStr for DcShorthand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Checks whether a string conforms to the shorthand format regex
/// `[flL]?\d+` or `u[0-9a-f]+` and valid range boundaries.
#[must_use]
pub fn is_valid_shorthand(s: &str) -> bool {
    DcShorthand::parse(s).is_ok()
}

/// Parses a format ID shorthand string, accepting either `f<digits>` or bare `<digits>`.
///
/// Case-sensitive: non-matching strings are rejected per `README.shorthand.md`.
///
/// # Errors
/// Returns an error if the string is empty, contains non-digits, or exceeds `MAX_FORMAT_ID`.
pub fn parse_format_shorthand(s: &str) -> Result<usize> {
    let trimmed = s.trim();
    if let Some(rest) = trimmed.strip_prefix('f') {
        ensure!(
            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()),
            "Invalid format ID '{trimmed}': prefix 'f' must be followed by decimal digits"
        );
        let id = rest
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid format ID integer '{trimmed}': {e}"))?;
        ensure!(
            id <= MAX_FORMAT_ID,
            "Format ID {id} exceeds maximum {MAX_FORMAT_ID}"
        );
        Ok(id)
    } else if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let id = trimmed
            .parse::<usize>()
            .map_err(|e| anyhow::anyhow!("Invalid format ID integer '{trimmed}': {e}"))?;
        ensure!(
            id <= MAX_FORMAT_ID,
            "Format ID {id} exceeds maximum {MAX_FORMAT_ID}"
        );
        Ok(id)
    } else {
        bail!("Invalid format shorthand '{trimmed}': must be 'f<digits>' or non-negative integer");
    }
}

/// Parses a Unicode shorthand string (`u[0-9a-f]+`).
///
/// Case-sensitive per `README.shorthand.md`.
///
/// # Errors
/// Returns an error if the prefix is not 'u', hex characters are invalid or
/// uppercase, or the codepoint exceeds `0x10FFFF`.
pub fn parse_unicode_shorthand(s: &str) -> Result<u32> {
    let trimmed = s.trim();
    let Some(hex_part) = trimmed.strip_prefix('u') else {
        bail!("Invalid Unicode shorthand '{trimmed}': must start with lowercase 'u'");
    };
    ensure!(
        !hex_part.is_empty() && hex_part.len() <= 6,
        "Unicode shorthand '{trimmed}' must have 1..=6 hex digits"
    );
    ensure!(
        hex_part
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "Unicode shorthand '{trimmed}' must contain only lowercase hex digits [0-9a-f]"
    );
    let cp = u32::from_str_radix(hex_part, 16)
        .map_err(|e| anyhow::anyhow!("Invalid hex codepoint in '{trimmed}': {e}"))?;
    ensure!(
        u128::from(cp) <= UNICODE_REGION_END,
        "Unicode codepoint 0x{cp:X} exceeds maximum 0x10FFFF"
    );
    Ok(cp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_graph_layout::{FORMAT_REGION_START, SHORT_DC_REGION_START};

    #[crate::ctb_test]
    fn test_valid_shorthands() -> Result<()> {
        assert_eq!(DcShorthand::parse("0")?, DcShorthand::Short(0));
        assert_eq!(DcShorthand::parse("246")?, DcShorthand::Short(246));
        assert_eq!(DcShorthand::parse("f80")?, DcShorthand::Format(80));
        assert_eq!(DcShorthand::parse("f0")?, DcShorthand::Format(0));
        assert_eq!(DcShorthand::parse("u12a")?, DcShorthand::Unicode(0x012A));
        assert_eq!(DcShorthand::parse("u0020")?, DcShorthand::Unicode(0x20));
        assert_eq!(DcShorthand::parse("l1000")?, DcShorthand::Long(1000));
        assert_eq!(DcShorthand::parse("L42")?, DcShorthand::Local(42));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_case_sensitivity() {
        assert!(DcShorthand::parse("F80").is_err());
        assert!(DcShorthand::parse("U12a").is_err());
        assert!(DcShorthand::parse("u12A").is_err());
        assert!(DcShorthand::parse("U0020").is_err());
        assert!(DcShorthand::parse("").is_err());
        assert!(DcShorthand::parse("x123").is_err());
    }

    #[crate::ctb_test]
    fn test_to_global_id() -> Result<()> {
        assert_eq!(DcShorthand::parse("0")?.to_global_id()?, SHORT_DC_REGION_START);
        assert_eq!(
            DcShorthand::parse("10")?.to_global_id()?,
            SHORT_DC_REGION_START.saturating_add(10)
        );
        assert_eq!(DcShorthand::parse("u12a")?.to_global_id()?, 0x012A);
        assert_eq!(
            DcShorthand::parse("f0")?.to_global_id()?,
            FORMAT_REGION_START
        );
        assert_eq!(
            DcShorthand::parse("f80")?.to_global_id()?,
            FORMAT_REGION_START.saturating_add(80)
        );
        assert_eq!(DcShorthand::parse("l2228304")?.to_global_id()?, 2_228_304);
        assert!(DcShorthand::parse("L42")?.to_global_id().is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_format_shorthand_helper() -> Result<()> {
        assert_eq!(parse_format_shorthand("f80")?, 80);
        assert_eq!(parse_format_shorthand("80")?, 80);
        assert_eq!(parse_format_shorthand("  f0  ")?, 0);
        assert!(parse_format_shorthand("F80").is_err());
        assert!(parse_format_shorthand("abc").is_err());
        assert!(parse_format_shorthand("").is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_unicode_shorthand_helper() -> Result<()> {
        assert_eq!(parse_unicode_shorthand("u12a")?, 0x012A);
        assert_eq!(parse_unicode_shorthand("u0020")?, 0x20);
        assert!(parse_unicode_shorthand("U0020").is_err());
        assert!(parse_unicode_shorthand("u12A").is_err());
        assert!(parse_unicode_shorthand("u110000").is_err());
        Ok(())
    }
}
