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

pub mod cli;
pub mod fnv;
pub mod xxhash;

use ctb_formats_utilities::format_info::{FormatInfo, format_help_table};
use ctb_utilities::FormatId;
use ctb_utilities::string::{to_hex, to_hex_0x};
use sha2::{Digest, Sha256 as Sha256Digest};
use std::sync::LazyLock;

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl HashAlgorithm {
    /// List of all supported hash algorithms.
    pub const ALL_ALGORITHMS: &'static [HashAlgorithm] = &[
        Self::XxHash32,
        Self::XxHash64,
        Self::XxHash3_64,
        Self::XxHash3_128,
        Self::Sha256,
    ];

    /// Maps this hash algorithm variant to its global `FormatId`.
    #[must_use]
    pub const fn to_format_id(&self) -> FormatId {
        match self {
            Self::XxHash32 => FormatId::XxHash32,
            Self::XxHash64 => FormatId::XxHash64,
            Self::XxHash3_64 => FormatId::XxHash3_64,
            Self::XxHash3_128 => FormatId::XxHash3_128,
            Self::Sha256 => FormatId::Sha256,
        }
    }

    /// Converts a global `FormatId` to a `HashAlgorithm` if recognized.
    #[must_use]
    pub const fn from_format_id(id: FormatId) -> Option<Self> {
        match id {
            FormatId::XxHash32 => Some(Self::XxHash32),
            FormatId::XxHash64 => Some(Self::XxHash64),
            FormatId::XxHash3_64 => Some(Self::XxHash3_64),
            FormatId::XxHash3_128 => Some(Self::XxHash3_128),
            FormatId::Sha256 => Some(Self::Sha256),
            _ => None,
        }
    }

    /// Retrieves format metadata from the shared registry.
    #[must_use]
    pub fn format_info(&self) -> Option<&'static FormatInfo> {
        ctb_formats_utilities::get_format_info_by_id(self.to_format_id())
    }
}

impl From<HashAlgorithm> for FormatId {
    fn from(algo: HashAlgorithm) -> Self {
        algo.to_format_id()
    }
}

impl TryFrom<FormatId> for HashAlgorithm {
    type Error = anyhow::Error;

    fn try_from(id: FormatId) -> Result<Self, Self::Error> {
        Self::from_format_id(id).ok_or_else(|| {
            anyhow::anyhow!(
                "FormatId is not a supported HashAlgorithm: {id:?}"
            )
        })
    }
}

/// Generates a help table of supported hash algorithms and their aliases.
pub fn csum_help_table() -> String {
    let format_ids: Vec<FormatId> = HashAlgorithm::ALL_ALGORITHMS
        .iter()
        .map(HashAlgorithm::to_format_id)
        .collect();
    format_help_table("Supported hash algorithms:", &format_ids)
}

/// Global static help table for `csum` CLI command.
pub static CSUM_AFTER_HELP: LazyLock<String> = LazyLock::new(csum_help_table);

impl TryFrom<&str> for HashAlgorithm {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let clean = s.trim().to_ascii_lowercase();
        if let Some(format_id) = FormatId::from_ident(&clean) {
            if let Some(algo) = Self::from_format_id(format_id) {
                return Ok(algo);
            }
        }
        anyhow::bail!("Unknown hash algorithm: {s}")
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

/// Streaming SHA-256 hasher for incremental hashing.
#[derive(Clone, Default)]
pub struct Sha256Stream {
    hasher: Sha256Digest,
}

impl Sha256Stream {
    /// Creates a new SHA-256 streaming hasher.
    #[must_use]
    pub fn new() -> Self {
        Self {
            hasher: Sha256Digest::new(),
        }
    }

    /// Feeds additional bytes into the hasher.
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        self.hasher.update(data.as_ref());
    }

    /// Finalizes the hash computation and returns the raw 32-byte digest.
    #[must_use]
    pub fn finalize(self) -> [u8; 32] {
        let mut out = [0_u8; 32];
        out.copy_from_slice(&self.hasher.finalize());
        out
    }

    /// Finalizes the hash computation and returns the lowercase hex string.
    #[must_use]
    pub fn finalize_hex(self) -> String {
        to_hex(&self.finalize())
    }
}

impl std::io::Write for Sha256Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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

    #[crate::ctb_test]
    fn test_sha256_stream() {
        let mut stream = Sha256Stream::new();
        stream.update(b"hello ");
        stream.update(b"world");
        assert_eq!(
            stream.finalize_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        use std::io::Write;
        let mut stream2 = Sha256Stream::default();
        stream2.write_all(b"hello world").expect("write to stream");
        assert_eq!(
            stream2.finalize_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[crate::ctb_test]
    fn test_hash_algorithm_parsing_and_metadata() {
        assert_eq!(
            HashAlgorithm::try_from("xxh32").unwrap(),
            HashAlgorithm::XxHash32
        );
        assert_eq!(
            HashAlgorithm::try_from("xxhash32").unwrap(),
            HashAlgorithm::XxHash32
        );
        assert_eq!(
            HashAlgorithm::try_from("xxh64").unwrap(),
            HashAlgorithm::XxHash64
        );
        assert_eq!(
            HashAlgorithm::try_from("xxh3").unwrap(),
            HashAlgorithm::XxHash3_64
        );
        assert_eq!(
            HashAlgorithm::try_from("xxhash3-64").unwrap(),
            HashAlgorithm::XxHash3_64
        );
        assert_eq!(
            HashAlgorithm::try_from("xxh128").unwrap(),
            HashAlgorithm::XxHash3_128
        );
        assert_eq!(
            HashAlgorithm::try_from("sha256").unwrap(),
            HashAlgorithm::Sha256
        );
        assert_eq!(
            HashAlgorithm::try_from("sha-256").unwrap(),
            HashAlgorithm::Sha256
        );
        assert!(HashAlgorithm::try_from("nonexistent_hash").is_err());

        let help = csum_help_table();
        assert!(help.contains("Supported hash algorithms:"));
        assert!(help.contains("xxh32, xxhash32: xxHash32"));
        assert!(help.contains("sha256"));
    }
}
