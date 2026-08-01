#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub use ctb_utilities::ipc::service_prelude::*;

use ctb_utilities::ipc::service_traits::{
    ChildIpcContext, renderer::RenderSettings,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    data: Vec<u8>,
}

const NETWORK_SERVICE_NAME: &str = "network";
const NETWORK_METHOD_ECHO: &str = "echo";

/// Start a document runtime with the provided document data.
///
/// This initializes the runtime environment for document processing,
/// setting up the event loop and IPC orchestration infrastructure.
#[ipc_method]
pub fn start(#[ipc(shm)] document: Vec<u8>) -> i32 {
    let doc = Document { data: document };
    log_fmt!("runtime started with {} bytes", doc.data.len());
    0
}

/// Post a rendered frame to the workspace.
///
/// The workspace can attribute the frame to a subprocess via the sender
/// context included in the IPC message envelope.
pub async fn post_frame_to_workspace(
    ipc: &dyn ChildIpcContext,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<()> {
    ipc.send_data_plane_message(bytes, content_type).await
}

/// Test function demonstrating nested document runtime spawning via
/// dependency-injected IPC.
///
/// In real use, the document runtime would send a request to the workspace to
/// spawn a sub-document process and pass the workspace the node ID to render
/// in a single `start_document()` (or something like that) call. The workspace
/// would then handle most interaction with the subprocess (including the final
/// render of multiple documents composed together), because the runtime for one
/// document shouldn't have access to the runtimes for nested documents data or
/// state beyond the initial node ID or passed through secure message passing.
///
/// # Arguments
///
/// * `document` - The document content to process
/// * `ipc` - The IPC context for communicating with the workspace
///
/// # Returns
///
/// A formatted string combining the original document with the subprocess
/// response.
#[ipc_method]
pub async fn test_simple_nested_document(
    #[ipc(shm)] document: String,
    settings: RenderSettings,
) -> Result<i32> {
    let ipc = ipc!();

    let renderer = ipc.request_spawn_renderer(None).await?;
    let response_str = renderer
        .render_from_string(document.as_str(), settings)
        .await?;
    let subruntime = ipc.request_spawn_runtime(None).await?;
    let subruntime_response = subruntime
        .test_prepend(
            response_str.clone(),
            "Prepend example 12345: ".to_string(),
        )
        .await?;

    // First, send a rendered frame back to the workspace.
    let frame = format!(
        "Runtime input document: {document}. Rendered: {response_str}. With subdocument: {subruntime_response}."
    );
    post_frame_to_workspace(ipc, frame.into_bytes(), "text/plain").await?;

    // Denial test: attempt an unauthorized direct call to the network service.
    // The workspace capability router should reject this and log an ERROR.
    let denial_args = postcard_helpers::encode(
        &b"unauthorized network request".to_vec(),
        "network echo request",
    )?;

    // Best-effort: we expect this to fail due to runtime capabilities.
    let _ = ipc
        .call_raw(NETWORK_SERVICE_NAME, NETWORK_METHOD_ECHO, denial_args)
        .await;

    // Then, request that the workspace shuts down.
    ipc.request_workspace_shutdown(Some(
        "runtime requested shutdown after posting frame".into(),
    ))
    .await?;

    // Finally, return an exit code.
    Ok(0)
}

#[expect(clippy::unused_async, reason = "IPC method interface requirement")]
#[ipc_method]
/// Test method: prepend a string to the document.
///
/// This is a test helper for IPC routing verification during example
/// development.
pub async fn test_prepend(document: String, prepend: String) -> String {
    format!("{prepend}{document}")
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
    fn can_start() {
        // Basic test that start() doesn't panic
        start(vec![1, 2, 3]);
    }
}
