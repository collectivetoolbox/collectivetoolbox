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

//! Thin wrapper around the xxHash fast non-cryptographic hash algorithms.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use twox_hash::{XxHash3_64, XxHash3_128, XxHash32, XxHash64};

/// Thin wrapper for xxhash32. Returns the 4-byte hash.
pub fn xxhash32(data: &[u8]) -> [u8; 4] {
    XxHash32::oneshot(0, data).to_be_bytes()
}

/// Thin wrapper for xxhash64. Returns the 8-byte hash.
pub fn xxhash64(data: &[u8]) -> [u8; 8] {
    XxHash64::oneshot(0, data).to_be_bytes()
}

/// Thin wrapper for `xxhash3_64`. Returns the 8-byte hash.
pub fn xxhash3_64(data: &[u8]) -> [u8; 8] {
    XxHash3_64::oneshot(data).to_be_bytes()
}

/// Thin wrapper for `xxhash3_128`. Returns the 16-byte hash.
pub fn xxhash3_128(data: &[u8]) -> [u8; 16] {
    XxHash3_128::oneshot(data).to_be_bytes()
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
    fn test_xxhash32() {
        assert_eq!(xxhash32(&[]), [0x02, 0xcc, 0x5d, 0x05]);
        assert_eq!(xxhash32(b"hello world"), [0xce, 0xbb, 0x66, 0x22]);
    }

    #[crate::ctb_test]
    fn test_xxhash64() {
        assert_eq!(
            xxhash64(&[]),
            [0xef, 0x46, 0xdb, 0x37, 0x51, 0xd8, 0xe9, 0x99]
        );
        assert_eq!(
            xxhash64(b"hello world"),
            [0x45, 0xab, 0x67, 0x34, 0xb2, 0x1e, 0x69, 0x68]
        );
    }

    #[crate::ctb_test]
    fn test_xxhash3_64() {
        assert_eq!(
            xxhash3_64(&[]),
            [0x2d, 0x06, 0x80, 0x05, 0x38, 0xd3, 0x94, 0xc2]
        );
        assert_eq!(
            xxhash3_64(b"hello world"),
            [0xd4, 0x47, 0xb1, 0xea, 0x40, 0xe6, 0x98, 0x8b]
        );
    }

    #[crate::ctb_test]
    fn test_xxhash3_128() {
        assert_eq!(
            xxhash3_128(&[]),
            [
                0x99, 0xaa, 0x06, 0xd3, 0x01, 0x47, 0x98, 0xd8, 0x60, 0x01,
                0xc3, 0x24, 0x46, 0x8d, 0x49, 0x7f
            ]
        );
        assert_eq!(
            xxhash3_128(b"hello world"),
            [
                0xdf, 0x8d, 0x09, 0xe9, 0x3f, 0x87, 0x49, 0x00, 0xa9, 0x9b,
                0x87, 0x75, 0xcc, 0x15, 0xb6, 0xc7
            ]
        );
    }
}
