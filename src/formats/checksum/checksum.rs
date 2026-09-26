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

/// Result of identifying a checksum digest or manifest format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumDetection {
    /// Primary detected `FormatId` (e.g. `FormatId::Sha256`, `FormatId::Md5`, etc.).
    pub format_id: FormatId,
    /// Possible candidate hash algorithm format IDs of equal digest length.
    pub candidate_algorithms: Vec<FormatId>,
    /// Indicates whether the input is a checksum manifest file/line (e.g. `sha256sum`).
    pub is_manifest: bool,
    /// Digest length in characters if standalone hex digest.
    pub digest_len: usize,
    /// Human-readable description.
    pub description: String,
}

/// Parses a line in GNU coreutils checksum format (`<hex>  <file>` or `<hex> *<file>`).
fn parse_coreutils_line(line: &str) -> Option<(&str, &str)> {
    let (digest, rest) = line.split_once(' ')?;
    let filename = if let Some(bin_file) = rest.strip_prefix('*') {
        bin_file.trim()
    } else if let Some(text_file) = rest.strip_prefix(' ') {
        text_file.trim()
    } else {
        return None;
    };
    if filename.is_empty() {
        return None;
    }
    if digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some((digest, filename))
    } else {
        None
    }
}

/// Parses a line in BSD checksum format (`<ALGO> (<file>) = <hex>`).
fn parse_bsd_line(line: &str) -> Option<(&str, &str, &str)> {
    let (algo_and_file, digest) = line.split_once(" = ")?;
    let (algo, rest) = algo_and_file.split_once(" (")?;
    let filename = rest.strip_suffix(')')?;
    if filename.is_empty() || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some((algo.trim(), filename.trim(), digest.trim()))
}

/// Detects whether `s` represents a standalone cryptographic/non-cryptographic
/// hex digest or a checksum manifest file/line.
#[must_use]
pub fn detect_checksum(s: &str) -> Option<ChecksumDetection> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Check if it's a checksum manifest file / line
    let lines: Vec<&str> = trimmed
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    if !lines.is_empty() {
        let all_coreutils = lines.iter().all(|l| parse_coreutils_line(l).is_some());
        if all_coreutils {
            if let Some((first_digest, _)) = parse_coreutils_line(lines.first().unwrap_or(&"")) {
                let dlen = first_digest.len();
                let (fmt, candidates, name) = match dlen {
                    64 => (FormatId::Sha256, vec![FormatId::Sha256, FormatId::Blake3], "SHA-256"),
                    32 => (FormatId::Md5, vec![FormatId::Md5, FormatId::XxHash3_128], "MD5"),
                    40 => (FormatId::Sha1, vec![FormatId::Sha1], "SHA-1"),
                    128 => (FormatId::Sha512, vec![FormatId::Sha512], "SHA-512"),
                    _ => (FormatId::Sha256, vec![FormatId::Sha256], "checksum"),
                };
                return Some(ChecksumDetection {
                    format_id: fmt,
                    candidate_algorithms: candidates,
                    is_manifest: true,
                    digest_len: dlen,
                    description: format!("{} checksum manifest", name),
                });
            }
        }

        let all_bsd = lines.iter().all(|l| parse_bsd_line(l).is_some());
        if all_bsd {
            if let Some((algo, _, digest)) = parse_bsd_line(lines.first().unwrap_or(&"")) {
                let algo_clean = algo.to_ascii_lowercase();
                let fmt = match algo_clean.as_str() {
                    "md5" => FormatId::Md5,
                    "sha1" => FormatId::Sha1,
                    "sha256" => FormatId::Sha256,
                    "sha512" => FormatId::Sha512,
                    _ => FormatId::Sha256,
                };
                return Some(ChecksumDetection {
                    format_id: fmt,
                    candidate_algorithms: vec![fmt],
                    is_manifest: true,
                    digest_len: digest.len(),
                    description: format!("BSD {} checksum file", algo),
                });
            }
        }
    }

    // 2. Standalone single hex digest
    if !trimmed.contains('\n') && !trimmed.contains(' ') {
        let clean = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
        if !clean.is_empty() && clean.bytes().all(|b| b.is_ascii_hexdigit()) {
            let len = clean.len();
            match len {
                64 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::Sha256,
                        candidate_algorithms: vec![FormatId::Sha256, FormatId::Blake3],
                        is_manifest: false,
                        digest_len: 64,
                        description: "SHA-256 (256-bit hex digest)".to_string(),
                    });
                }
                32 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::Md5,
                        candidate_algorithms: vec![FormatId::Md5, FormatId::XxHash3_128],
                        is_manifest: false,
                        digest_len: 32,
                        description: "MD5 (128-bit hex digest)".to_string(),
                    });
                }
                40 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::Sha1,
                        candidate_algorithms: vec![FormatId::Sha1],
                        is_manifest: false,
                        digest_len: 40,
                        description: "SHA-1 (160-bit hex digest)".to_string(),
                    });
                }
                128 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::Sha512,
                        candidate_algorithms: vec![FormatId::Sha512],
                        is_manifest: false,
                        digest_len: 128,
                        description: "SHA-512 (512-bit hex digest)".to_string(),
                    });
                }
                8 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::Crc32,
                        candidate_algorithms: vec![
                            FormatId::Crc32,
                            FormatId::Adler32,
                            FormatId::XxHash32,
                        ],
                        is_manifest: false,
                        digest_len: 8,
                        description: "CRC-32 / 32-bit hex checksum".to_string(),
                    });
                }
                16 => {
                    return Some(ChecksumDetection {
                        format_id: FormatId::XxHash64,
                        candidate_algorithms: vec![FormatId::XxHash64, FormatId::XxHash3_64],
                        is_manifest: false,
                        digest_len: 16,
                        description: "xxHash64 / 64-bit hex checksum".to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    None
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

    #[crate::ctb_test]
    fn test_detect_checksum_cases() {
        // SHA-256
        let sha256_hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let det = detect_checksum(sha256_hex).unwrap();
        assert_eq!(det.format_id, FormatId::Sha256);
        assert!(!det.is_manifest);
        assert_eq!(det.digest_len, 64);

        // MD5
        let md5_hex = "d41d8cd98f00b204e9800998ecf8427e";
        let det = detect_checksum(md5_hex).unwrap();
        assert_eq!(det.format_id, FormatId::Md5);
        assert!(!det.is_manifest);

        // SHA-1
        let sha1_hex = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
        let det = detect_checksum(sha1_hex).unwrap();
        assert_eq!(det.format_id, FormatId::Sha1);

        // CRC32
        let crc_hex = "00000000";
        let det = detect_checksum(crc_hex).unwrap();
        assert_eq!(det.format_id, FormatId::Crc32);

        // Coreutils manifest
        let manifest = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  empty.txt\n\
                        ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad *abc.txt";
        let det = detect_checksum(manifest).unwrap();
        assert_eq!(det.format_id, FormatId::Sha256);
        assert!(det.is_manifest);

        // BSD format
        let bsd = "SHA256 (file.tar.gz) = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let det = detect_checksum(bsd).unwrap();
        assert_eq!(det.format_id, FormatId::Sha256);
        assert!(det.is_manifest);

        assert!(detect_checksum("").is_none());
        assert!(detect_checksum("not-a-hash").is_none());
    }
}

