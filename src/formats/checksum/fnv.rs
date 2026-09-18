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

//! Fowler-Noll-Vo (FNV-1a) hash wrappers.
//! Documentation: RFC9923 - https://www.rfc-editor.org/rfc/rfc9923.txt

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub use ctb_build_support::fnv::{
    fnv1a32, fnv1a32_update, fnv1a64, fnv1a64_update, FNV1A_32_INIT,
    FNV1A_32_PRIME, FNV1A_64_INIT, FNV1A_64_PRIME,
};

use ctb_utilities::string::to_hex;

/// Computes the 64-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a64_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a64(data.as_ref()).to_be_bytes())
}

/// Computes the 32-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a32_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a32(data.as_ref()).to_be_bytes())
}

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

    #[crate::ctb_test]
    fn test_fnv1a_reexport() {
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
        assert_eq!(fnv1a32(b"foobar"), 0xbf9c_f968);
        assert_eq!(fnv1a64_hex(b"foobar"), "85944171f73967e8");
    }
}
