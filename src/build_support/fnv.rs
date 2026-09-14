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

//! Fowler-Noll-Vo (FNV-1a) non-cryptographic hash function implementation.
//!
//! Provides 32-bit and 64-bit FNV-1a hashing suitable for build-time caching,
//! hash tables, and checksum verification with zero external dependencies.

/// Initial offset basis for 64-bit FNV-1a.
pub const FNV1A_64_INIT: u64 = 0xcbf2_9ce4_8422_2325;

/// Prime multiplier for 64-bit FNV-1a.
pub const FNV1A_64_PRIME: u64 = 0x0100_0000_01b3;

/// Initial offset basis for 32-bit FNV-1a.
pub const FNV1A_32_INIT: u32 = 0x811c_9dc5;

/// Prime multiplier for 32-bit FNV-1a.
pub const FNV1A_32_PRIME: u32 = 0x0100_0193;

/// Updates an existing 64-bit FNV-1a hash with the given byte slice.
#[must_use]
pub fn fnv1a64_update(mut hash: u64, data: &[u8]) -> u64 {
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV1A_64_PRIME);
    }
    hash
}

/// Computes the 64-bit FNV-1a hash of the given byte slice.
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    fnv1a64_update(FNV1A_64_INIT, data)
}

/// Updates an existing 32-bit FNV-1a hash with the given byte slice.
#[must_use]
pub fn fnv1a32_update(mut hash: u32, data: &[u8]) -> u32 {
    for byte in data {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(FNV1A_32_PRIME);
    }
    hash
}

/// Computes the 32-bit FNV-1a hash of the given byte slice.
#[must_use]
pub fn fnv1a32(data: &[u8]) -> u32 {
    fnv1a32_update(FNV1A_32_INIT, data)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv1a64_empty() {
        assert_eq!(fnv1a64(b""), FNV1A_64_INIT);
    }

    #[test]
    fn test_fnv1a64_known_vectors() {
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn test_fnv1a32_empty() {
        assert_eq!(fnv1a32(b""), FNV1A_32_INIT);
    }

    #[test]
    fn test_fnv1a32_known_vectors() {
        assert_eq!(fnv1a32(b"a"), 0xe40c_292c);
        assert_eq!(fnv1a32(b"foobar"), 0xbf9a_e36b);
    }

    #[test]
    fn test_fnv1a64_streaming_equivalent() {
        let chunk1 = b"foo";
        let chunk2 = b"bar";
        let full = b"foobar";

        let hash1 = fnv1a64_update(FNV1A_64_INIT, chunk1);
        let hash2 = fnv1a64_update(hash1, chunk2);

        assert_eq!(hash2, fnv1a64(full));
    }
}
