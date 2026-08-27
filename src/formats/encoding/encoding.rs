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

//! Character encoding and decoding definitions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};

pub mod cp437;
pub mod line_endings;
pub mod mapping;
pub mod neo;
pub mod standard;
pub mod unicode;

pub use mapping::{
    CharEncoding, LineEndingFormat, LineEndingKind, LineEndingOption, LowArea,
    NeoRegion, SingleByteMapping, TerminationMode, asc, chr, chr_char, decode,
    decode_with_options, encode, encode_with_options, mapping, transcode,
};

static ENCODING_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_encoding_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ENCODING_DATA_DIR, key)
}

#[cfg(test)]
#[expect(
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

    #[crate::ctb_test]
    fn test_placeholder() {
        // unless there's a placeholder, the use super::*; will be deleted by
        // formatting
        assert!(true);
    }
}
