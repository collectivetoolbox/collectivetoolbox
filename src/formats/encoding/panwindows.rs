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

//! Panorama Windows (CP1252 ANSI) character encoding and decoding utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::LazyLock;
use crate::mapping::SingleByteMapping;

/// Pre-initialized `SingleByteMapping` for Panorama Windows encoding.
pub(crate) static PANWINDOWS_MAPPING: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    let mut decode_table = ['\0'; 256];
    let mut encode_table = HashMap::new();
    for i in 0u8..=255 {
        let byte_slice = [i];
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(&byte_slice);
        if let Some(ch) = cow.chars().next() {
            if let Some(slot) = decode_table.get_mut(usize::from(i)) {
                *slot = ch;
            }
            encode_table.entry(ch).or_insert(i);
        }
    }
    SingleByteMapping::from_raw(decode_table, encode_table)
});

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
    use ctb_formats_utilities::encoding::CharEncoding;

    #[crate::ctb_test]
    fn test_panwindows_encoding() {
        let enc = CharEncoding::pan_windows();
        let original = "Hello, World! ñ ü á";
        let encoded = crate::encode(enc, original).unwrap();
        let decoded = crate::decode(enc, &encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
