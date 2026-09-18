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

//! Subprocess runtime, job supervisor, and execution environment manager.

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

use std::collections::HashMap;
use ctb_formats_dcstring::DcString;

/// Native Document Character runtime value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeValue {
    /// Document Character string representation without premature UTF-8
    /// decoding.
    DcStr(DcString),
    /// Numeric value.
    Number(i64),
    /// Structured list of runtime values.
    List(Vec<RuntimeValue>),
    /// 0-bit unit value for statements or void routines.
    Unit,
    /// Unreachable value.
    Never,
}

/// Capability privilege requested by a running document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Privilege {
    /// Filesystem input/output.
    FileIo,
    /// Network communication.
    Network,
    /// High-resolution timers or performance counters.
    HighResolutionTimers,
    /// Excessive compute budget exhausted before the document is fully parsed.
    ExcessiveCompute,
    /// Custom privileged operation.
    Custom(String),
}

/// Multi-state capability permission decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecision {
    /// Deny the permission visibly to the document.
    Deny,
    /// Automatically return spoofed/mocked data.
    AutoMock,
    /// Provide a custom replacement interface with user-specified mock
    /// methods.
    ManualMock,
    /// Allow the permission (gated behind warning UI when parse errors exist).
    Allow,
    /// Break out of infinite loops or excessive compute and resume executing
    /// and printing the document on the next Dc.
    BreakOutResumeNext,
}

/// Execution context for document evaluation and capability permission gating.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Active variable bindings.
    pub bindings: HashMap<String, RuntimeValue>,
    /// Whether the parsed document contains syntax or framing errors.
    pub has_errors: bool,
    /// Count of compute steps consumed.
    pub compute_steps: usize,
    /// Maximum allowed compute steps before raising a budget interruption.
    pub max_compute_steps: usize,
}

impl ExecutionContext {
    /// Creates a new execution context with default limits.
    #[must_use]
    pub fn new(has_errors: bool) -> Self {
        Self {
            bindings: HashMap::new(),
            has_errors,
            compute_steps: 0,
            max_compute_steps: 1_000_000,
        }
    }

    /// Evaluates a permission request, strongly steering toward mocking or
    /// denying if `has_errors` is true.
    #[must_use]
    pub fn request_permission(&self, privilege: &Privilege) -> PermissionDecision {
        if matches!(privilege, Privilege::ExcessiveCompute) {
            // For compute exhaustion, default to offering breakout to resume
            // execution/printing on the next Dc.
            return PermissionDecision::BreakOutResumeNext;
        }

        if self.has_errors {
            // Strongly steer away from Allow when document has errors
            PermissionDecision::AutoMock
        } else {
            PermissionDecision::Allow
        }
    }

    /// Increments compute step count, checking if the compute budget is
    /// exhausted. If exhausted, raises a permission request for
    /// `Privilege::ExcessiveCompute`.
    pub fn step(&mut self) -> Result<PermissionDecision, anyhow::Error> {
        self.compute_steps = self.compute_steps.saturating_add(1);
        if self.compute_steps > self.max_compute_steps {
            let decision = self.request_permission(&Privilege::ExcessiveCompute);
            if decision == PermissionDecision::BreakOutResumeNext {
                // Reset or allow caller to break out and resume at the next Dc
                return Ok(PermissionDecision::BreakOutResumeNext);
            }
            anyhow::bail!(
                "Compute budget exhausted ({} steps); permission denied to continue",
                self.max_compute_steps
            );
        }
        Ok(PermissionDecision::Allow)
    }
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
    fn can_start() {
        // Basic test that start() doesn't panic
        start(vec![1, 2, 3]);
    }

    #[crate::ctb_test]
    fn test_permission_steering_on_document_errors() {
        let valid_ctx = ExecutionContext::new(false);
        assert_eq!(
            valid_ctx.request_permission(&Privilege::FileIo),
            PermissionDecision::Allow
        );

        let error_ctx = ExecutionContext::new(true);
        assert_eq!(
            error_ctx.request_permission(&Privilege::FileIo),
            PermissionDecision::AutoMock
        );
    }

    #[crate::ctb_test]
    fn test_excessive_compute_breakout() {
        let mut ctx = ExecutionContext::new(false);
        ctx.max_compute_steps = 2;
        assert_eq!(ctx.step().unwrap(), PermissionDecision::Allow);
        assert_eq!(ctx.step().unwrap(), PermissionDecision::Allow);
        // Exceeding budget triggers breakout to resume on next Dc
        assert_eq!(ctx.step().unwrap(), PermissionDecision::BreakOutResumeNext);
    }
}
