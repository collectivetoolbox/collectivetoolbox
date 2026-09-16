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

pub use ctb_storage_minimal::global_graph_layout::{
    SHORT_DC_REGION_END, SHORT_DC_REGION_START,
};

pub use crate::dc_char::DcChar;

/// Start encapsulation of binary or non-text data (Dc 203).
pub const DC_START_ENCAPSULATION_BINARY: DcChar = DcChar::from_short(203);
/// End encapsulation of binary or non-text data (Dc 204).
pub const DC_END_ENCAPSULATION_BINARY: DcChar = DcChar::from_short(204);

/// Start encapsulation of UTF-8 data (Dc 191).
pub const DC_START_ENCAPSULATION_UTF8: DcChar = DcChar::from_short(191);
/// End encapsulation of UTF-8 data (Dc 192).
pub const DC_END_ENCAPSULATION_UTF8: DcChar = DcChar::from_short(192);

/// Replacement for incoming character with value not mapped to a Dc (Dc 207).
pub const DC_REPLACEMENT_UNAVAIL_DC: DcChar = DcChar::from_short(207);
/// Replacement for incoming character with value unknown or unrepresentable in Unicode (Dc 206).
pub const DC_REPLACEMENT_UNAVAIL_UNICODE: DcChar = DcChar::from_short(206);

/// Document Character for escape / ignore following Dc (Dc 255).
pub const DC_ESCAPE: DcChar = DcChar::from_short(255);
/// Alias for `DC_ESCAPE`.
pub const DC_ESCAPE_NEXT: DcChar = DC_ESCAPE;

/// Document Character for embedding long (global graph) Dc IDs (Dc 308).
pub const DC_LONG_DC: DcChar = DcChar::from_short(308);

/// Begin number in Dc stream (Dc 6).
pub const DC_BEGIN_NUMBER: DcChar = DcChar::from_short(6);
/// End number in Dc stream (Dc 7).
pub const DC_END_NUMBER: DcChar = DcChar::from_short(7);
/// Positive number sign in Dc stream (Dc 10).
pub const DC_POSITIVE: DcChar = DcChar::from_short(10);
/// Negative number sign in Dc stream (Dc 11).
pub const DC_NEGATIVE: DcChar = DcChar::from_short(11);
/// Format 199 in Dc stream.
pub const DC_FORMAT_199: DcChar = DcChar::from_format(199);

/// First Base64 encapsulation digit (digit 0 = 'A' = Dc 127).
pub const DC_BASE64_START: DcChar = DcChar::from_short(127);
/// Last Base64 encapsulation digit (digit 63 = '/' = Dc 190).
pub const DC_BASE64_END: DcChar = DcChar::from_short(190);
/// Base64 encapsulation padding character ('=' = Dc 195).
pub const DC_BASE64_PADDING: DcChar = DcChar::from_short(195);

/// Converts a short Document Character (Dc) ID to its long (Global Graph) ID.
#[must_use]
pub fn short_to_long_dc(short_id: u32) -> u128 {
    SHORT_DC_REGION_START.saturating_add(u128::from(short_id))
}

/// Converts a long (Global Graph) Document Character ID to its short Dc ID, if within short range.
#[must_use]
pub fn long_to_short_dc(dc_id: u128) -> Option<u32> {
    if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dc_id) {
        u32::try_from(dc_id.saturating_sub(SHORT_DC_REGION_START)).ok()
    } else {
        None
    }
}

pub use crate::validation::{
    parse_dc_aliases_column, split_dc_aliases_column, validate_all_dc_files,
    validate_all_dc_files_from_disk, validate_dc_aliases_spacing,
    validate_dc_category_file, validate_dc_files_data,
};
