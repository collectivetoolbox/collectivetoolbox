//! Copies of code from `formats::base64` to avoid circular dependencies.

use anyhow::{Result, anyhow};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

pub(super) fn bytes_to_standard_base64(bytes: &[u8]) -> String {
    BASE64_STANDARD.encode(bytes)
}

pub(super) fn standard_base64_to_bytes(base64: String) -> Result<Vec<u8>> {
    BASE64_STANDARD
        .decode(base64)
        .map_err(|e| anyhow!("Failed to decode base64: {e}"))
}
