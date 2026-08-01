//! Provides xxHash, a fast, non-cryptographic hash.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod xxhash;

use ctb_utilities::string::{to_hex, to_hex_0x};

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
}

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

/// Computes the hash of the given data using the specified algorithm.
pub fn hash(data: &[u8], algo: HashAlgorithm) -> Vec<u8> {
    match algo {
        HashAlgorithm::XxHash32 => xxhash::xxhash32(data).to_vec(),
        HashAlgorithm::XxHash64 => xxhash::xxhash64(data).to_vec(),
        HashAlgorithm::XxHash3_64 => xxhash::xxhash3_64(data).to_vec(),
        HashAlgorithm::XxHash3_128 => xxhash::xxhash3_128(data).to_vec(),
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
    }
}
