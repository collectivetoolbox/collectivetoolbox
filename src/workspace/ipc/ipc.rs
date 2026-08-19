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

//! IPC library root exporting router, auth, connection, and types.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use crate::protocol::Response;

pub mod auth;
pub mod connection;
pub mod data_plane;
pub mod error;
pub mod multiplex;
pub mod peer;
pub mod peer_clients;
pub mod process_manager;
pub mod protocol;
pub mod router;
pub mod services;
pub mod transport;
pub mod types;
pub mod workspace_runner;

pub(crate) fn ensure_response_ok(
    response: &Response,
    context: &str,
) -> Result<()> {
    if response.ok {
        return Ok(());
    }

    // Reason for fallback: JSON-RPC error payload missing a detailed error message defaults to generic "unknown error"
    let msg = response
        .error
        .as_ref()
        .map_or_else(|| "unknown error".to_string(), |e| e.message.clone());
    bail!("{context} failed: {msg}")
}

pub(crate) fn response_result_bytes(
    response: Response,
    context: &str,
) -> Result<Vec<u8>> {
    ensure_response_ok(&response, context)?;
    let Some(bytes) = response.result else {
        bail!("{context} missing result");
    };
    Ok(bytes)
}

#[cfg(test)]
pub fn assert_ipc_response_ok(response: &Response) {
    assert!(response.ok, "IPC response indicates failure: {response:?}");
}

#[cfg(test)]
pub fn assert_ipc_response_error(response: &Response) {
    assert!(
        !response.ok,
        "IPC response indicates success when error expected: {response:?}"
    );
}
