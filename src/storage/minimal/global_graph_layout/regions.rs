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

//! Global graph region boundary constants and GID conversion functions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub const UNICODE_REGION_START: u128 = 0;
pub const UNICODE_REGION_END: u128 = 1_114_111;
pub const SHORT_DC_REGION_START: u128 = 1_114_112;
pub const SHORT_DC_REGION_END: u128 = 2_228_223;
pub const FORMAT_REGION_START: u128 = 2_228_224;
pub const FORMAT_REGION_END: u128 = 3_342_335;
pub const SYSTEM_REGION_START: u128 = 8_589_934_592;
pub const SYSTEM_REGION_END: u128 = 12_884_901_887;

/// Converts a short Document Character (Dc) ID to its Global Graph ID.
#[must_use]
pub fn dc_to_gid(dc_id: u64) -> u128 {
    SHORT_DC_REGION_START.saturating_add(u128::from(dc_id))
}

/// Converts a short Format ID to its Global Graph ID.
#[must_use]
pub fn format_to_gid(fmt_id: u64) -> u128 {
    FORMAT_REGION_START.saturating_add(u128::from(fmt_id))
}

/// Formats a Global Graph ID into its short prefix representation (`dc:N`, `fmt:N`, `uni:N`, `gid:N`).
#[must_use]
pub fn gid_to_short(gid: u128) -> String {
    if gid <= UNICODE_REGION_END {
        format!("uni:{gid}")
    } else if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&gid) {
        format!("dc:{offset}", offset = gid.saturating_sub(SHORT_DC_REGION_START))
    } else if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&gid) {
        format!(
            "fmt:{offset}",
            offset = gid.saturating_sub(FORMAT_REGION_START)
        )
    } else {
        format!("gid:{gid}")
    }
}

