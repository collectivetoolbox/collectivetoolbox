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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

use std::ffi::OsString;

use thiserror::Error;

use crate::protocol::Response;

/// Canonical error type for this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Timeout")]
    Timeout,
    #[error("Canceled")]
    Canceled,
    #[error("Not found")]
    NotFound,
    #[error("Unsupported: {0}")]
    Unsupported(String),
    #[error("Internal: {0}")]
    Internal(String),
}

impl From<std::io::Error> for Error {
    /// Converts a `std::io::Error` into the canonical Error type.
    fn from(e: std::io::Error) -> Self {
        Error::Transport(e.to_string())
    }
}

impl From<anyhow::Error> for Error {
    /// Converts a `anyhow::Error` into the canonical Error type.
    fn from(e: anyhow::Error) -> Self {
        Error::Internal(e.to_string())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(e: std::num::TryFromIntError) -> Self {
        Error::Internal(e.to_string())
    }
}

impl From<crate::protocol::RpcError> for Error {
    fn from(e: crate::protocol::RpcError) -> Self {
        Error::Internal(format!("RPC error {}: {}", e.code, e.message))
    }
}

impl From<Response> for Error {
    /// Converts an error `Response` into the canonical Error type.
    fn from(resp: Response) -> Self {
        match resp.error {
            Some(error) => Error::from(error),
            _ => Error::Internal("Expected error response".to_string()),
        }
    }
}

impl From<OsString> for Error {
    fn from(e: OsString) -> Self {
        Error::Internal(format!("OS string error: {e:?}"))
    }
}

/// Convert `anyhow::Error` into crate Error, preserving context.
pub(crate) fn to_ipc_error(e: anyhow::Error) -> Error {
    e.into()
}
