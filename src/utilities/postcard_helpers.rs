// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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
