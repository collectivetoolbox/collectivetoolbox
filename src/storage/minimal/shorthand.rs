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

#[cfg(test)]
use anyhow::Result;

mod core;
pub use core::{
    DcShorthand, MAX_FORMAT_ID, MAX_SHORT_DC, is_valid_shorthand,
    parse_format_shorthand, parse_unicode_shorthand,
};

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
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
