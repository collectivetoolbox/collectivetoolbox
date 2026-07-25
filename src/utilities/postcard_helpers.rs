//! Helpers for postcard serialization/deserialization with consistent error
//! handling.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Encode a value to bytes using postcard serialization.
///
/// # Arguments
///
/// * `val` - The value to encode
/// * `context` - A description of what's being encoded, used in error messages
///
/// # Errors
///
/// Returns an error if serialization fails, with context about what was being
/// encoded.
pub fn encode<T: Serialize>(val: &T, context: &str) -> Result<Vec<u8>> {
    postcard::to_stdvec(val)
        .map_err(|e| anyhow::anyhow!("failed to serialize {context}: {e}"))
}

/// Decode bytes to a value using postcard deserialization.
///
/// # Arguments
///
/// * `bytes` - The bytes to decode
/// * `context` - A description of what's being decoded, used in error messages
///
/// # Errors
///
/// Returns an error if deserialization fails, with context about what was
/// being decoded.
pub fn decode<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    context: &str,
) -> Result<T> {
    postcard::from_bytes(bytes)
        .map_err(|e| anyhow::anyhow!("failed to deserialize {context}: {e}"))
}
