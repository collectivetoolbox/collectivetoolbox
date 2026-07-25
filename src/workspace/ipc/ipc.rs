#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
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
