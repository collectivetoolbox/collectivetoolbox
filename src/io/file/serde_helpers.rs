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

//! Custom serde serialization and deserialization helpers for file entities.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Deserializer, Serializer};

/// Serde helper for `[u8; 32]` SHA-256 digests.
pub mod hex_sha256 {
    use super::*;

    /// Serializes a 32-byte digest into a 64-character lowercase hexadecimal string.
    pub fn serialize<S>(val: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = String::with_capacity(64);
        for byte in val {
            use std::fmt::Write;
            let _ = write!(s, "{byte:02x}");
        }
        serializer.serialize_str(&s)
    }

    /// Deserializes a 32-byte digest from either a hex string or a byte sequence.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum HashInput {
            Hex(String),
            Bytes(Vec<u8>),
        }

        match HashInput::deserialize(deserializer)? {
            HashInput::Hex(s) => {
                let trimmed = s.trim();
                if trimmed.len() != 64 {
                    return Err(serde::de::Error::custom(format!(
                        "Invalid SHA-256 hex string length: expected 64, got {}",
                        trimmed.len()
                    )));
                }
                let mut digest = [0_u8; 32];
                for i in 0usize..32usize {
                    let start = i.saturating_mul(2);
                    let end = start.saturating_add(2);
                    let Some(hex_byte) = trimmed.get(start..end) else {
                        return Err(serde::de::Error::custom(
                            "Hex slice index out of bounds",
                        ));
                    };
                    let byte_val = u8::from_str_radix(hex_byte, 16)
                        .map_err(serde::de::Error::custom)?;
                    let Some(slot) = digest.get_mut(i) else {
                        return Err(serde::de::Error::custom(
                            "Digest index out of bounds",
                        ));
                    };
                    *slot = byte_val;
                }
                Ok(digest)
            }
            HashInput::Bytes(b) => {
                if b.len() != 32 {
                    return Err(serde::de::Error::custom(format!(
                        "Invalid SHA-256 byte array length: expected 32, got {}",
                        b.len()
                    )));
                }
                let mut digest = [0_u8; 32];
                digest.copy_from_slice(&b);
                Ok(digest)
            }
        }
    }
}

/// Serde helper for optional Base64 byte payloads (`Option<Vec<u8>>`).
pub mod opt_base64 {
    use super::*;

    /// Serializes optional bytes as a Base64 string or none.
    pub fn serialize<S>(
        val: &Option<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(bytes) => {
                serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes optional bytes from Base64 string, byte array, or null.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ByteInput {
            String(String),
            Bytes(Vec<u8>),
            None,
        }

        match Option::<ByteInput>::deserialize(deserializer)? {
            Some(ByteInput::String(s)) => {
                let decoded = BASE64_STANDARD
                    .decode(s.trim())
                    .map_err(serde::de::Error::custom)?;
                Ok(Some(decoded))
            }
            Some(ByteInput::Bytes(b)) => Ok(Some(b)),
            Some(ByteInput::None) | None => Ok(None),
        }
    }
}

/// Serde helper for byte sequences that prefer UTF-8 strings when losslessly encodable.
pub mod text_or_base64 {
    use super::*;

    /// Serializes bytes as a UTF-8 string if valid, otherwise as Base64.
    pub fn serialize<S>(val: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Ok(utf8_str) = std::str::from_utf8(val) {
            serializer.serialize_str(utf8_str)
        } else {
            serializer.serialize_str(&BASE64_STANDARD.encode(val))
        }
    }

    /// Deserializes bytes from a UTF-8 string or byte array.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrBytes {
            String(String),
            Bytes(Vec<u8>),
        }

        match StringOrBytes::deserialize(deserializer)? {
            StringOrBytes::String(s) => Ok(s.into_bytes()),
            StringOrBytes::Bytes(b) => Ok(b),
        }
    }
}

/// Serde helper for optional byte sequences (`Option<Vec<u8>>`) that prefer UTF-8 strings when losslessly encodable.
pub mod opt_text_or_base64 {
    use super::*;

    /// Serializes optional bytes as a UTF-8 string, Base64, or none.
    pub fn serialize<S>(val: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(bytes) => {
                if let Ok(utf8_str) = std::str::from_utf8(bytes) {
                    serializer.serialize_str(utf8_str)
                } else {
                    serializer.serialize_str(&BASE64_STANDARD.encode(bytes))
                }
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes optional bytes from a UTF-8 string, byte array, or null.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrBytes {
            String(String),
            Bytes(Vec<u8>),
            None,
        }

        match Option::<StringOrBytes>::deserialize(deserializer)? {
            Some(StringOrBytes::String(s)) => Ok(Some(s.into_bytes())),
            Some(StringOrBytes::Bytes(b)) => Ok(Some(b)),
            Some(StringOrBytes::None) | None => Ok(None),
        }
    }
}
