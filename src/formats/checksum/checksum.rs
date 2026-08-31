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

//! Provides hashing algorithms including xxHash and SHA-256.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod xxhash;

use ctb_utilities::string::{to_hex, to_hex_0x};
use sha2::{Digest, Sha256 as Sha256Digest};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// xxHash 32-bit algorithm, non-cryptographic
    XxHash32,
    /// xxHash 64-bit algorithm, non-cryptographic
    XxHash64,
    /// xxHash3 64-bit algorithm, non-cryptographic
    XxHash3_64,
    /// xxHash3 128-bit algorithm, non-cryptographic
    XxHash3_128,
    /// SHA-256 cryptographic hash algorithm
    Sha256,
}

#[expect(
    non_upper_case_globals,
    reason = "Alias constant matches enum variant naming"
)]
pub const Sha256: HashAlgorithm = HashAlgorithm::Sha256;

impl TryFrom<&str> for HashAlgorithm {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_ascii_lowercase().as_str() {
            "xxh32" | "xxhash32" => Ok(HashAlgorithm::XxHash32),
            "xxh64" | "xxhash64" => Ok(HashAlgorithm::XxHash64),
            "xxh3" | "xxhash3_64" | "xxhash3-64" => {
                Ok(HashAlgorithm::XxHash3_64)
            }
            "xxh128" | "xxhash128" | "xxhash3_128" | "xxhash3-128" => {
                Ok(HashAlgorithm::XxHash3_128)
            }
            "sha256" | "sha-256" | "sha2_256" | "sha2-256" => {
                Ok(HashAlgorithm::Sha256)
            }
            _ => anyhow::bail!("Unknown hash algorithm: {s}"),
        }
    }
}

impl TryFrom<String> for HashAlgorithm {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl std::str::FromStr for HashAlgorithm {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

/// Computes the SHA-256 hash of `data`.
pub fn sha256(data: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Sha256Digest::new();
    hasher.update(data.as_ref());
    let mut out = [0_u8; 32];
    out.copy_from_slice(&hasher.finalize());
    out
}

/// Computes the SHA-256 hash of `data` as a lowercase hex string.
pub fn sha256_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&sha256(data))
}

/// Computes the hash of the given data using the specified algorithm.
pub fn hash(data: &[u8], algo: HashAlgorithm) -> Vec<u8> {
    match algo {
        HashAlgorithm::XxHash32 => xxhash::xxhash32(data).to_vec(),
        HashAlgorithm::XxHash64 => xxhash::xxhash64(data).to_vec(),
        HashAlgorithm::XxHash3_64 => xxhash::xxhash3_64(data).to_vec(),
        HashAlgorithm::XxHash3_128 => xxhash::xxhash3_128(data).to_vec(),
        HashAlgorithm::Sha256 => sha256(data).to_vec(),
    }
}

/// Computes the hash of the given data using the specified algorithm and
/// returns the result formatted as a hex string, optionally prefixed with `0x`.
pub fn hash_hex(data: &[u8], algo: HashAlgorithm, prefix_0x: bool) -> String {
    let bytes = hash(data, algo);
    if prefix_0x {
        to_hex_0x(&bytes)
    } else {
        to_hex(&bytes)
    }
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
    fn test_hash_hex() {
        let data = b"hello world";
        assert_eq!(hash_hex(data, HashAlgorithm::XxHash32, false), "cebb6622");
        assert_eq!(hash_hex(data, HashAlgorithm::XxHash32, true), "0xcebb6622");

        assert_eq!(
            hash_hex(data, HashAlgorithm::XxHash64, false),
            "45ab6734b21e6968"
        );
        assert_eq!(
            hash_hex(data, HashAlgorithm::XxHash64, true),
            "0x45ab6734b21e6968"
        );

        assert_eq!(
            hash_hex(data, HashAlgorithm::Sha256, false),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
        assert_eq!(
            sha256_hex(data),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
