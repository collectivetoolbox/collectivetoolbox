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

//! Constants and definitions for Document Character (Dc) data.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub const SHORT_DC_REGION_START: u128 = 1_114_112;
pub const SHORT_DC_REGION_END: u128 = 2_228_223;

/// Short Dc ID for embedding long (global graph) Dc IDs (Dc 308).
pub const SHORT_DC_LONG_DC: u32 = 308;
/// Global Graph ID for Dc 308 (`1_114_420`).
pub const GID_LONG_DC: u128 = 1_114_420;

/// Short Dc ID for escape / ignore following Dc (Dc 255).
pub const SHORT_DC_ESCAPE: u32 = 255;
/// Global Graph ID for Dc 255 (`1_114_367`).
pub const GID_ESCAPE: u128 = 1_114_367;

pub use crate::validation::{
    split_dc_aliases_column, validate_all_dc_files,
    validate_all_dc_files_from_disk, validate_dc_category_file,
    validate_dc_files_data,
};
