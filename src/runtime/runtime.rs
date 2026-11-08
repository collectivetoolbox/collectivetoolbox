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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[allow(
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

use ctb_formats_dcstring::{
    DcChar, DcMstr, DcMstring, DcShorthand, DcString, DcWasmResourceLimits,
    DcWasmSession, DcWasmWatchdogLimits, SeedGraphIndex,
    WASM_RESOURCE_ACCOUNTING_VERSION, WasmResourceInterruption,
    WasmStepOutcome, dc,
};
use std::collections::HashMap;

/// Authenticated System supervisor bootstrap admission and compiled local resource counters.
///
/// The runtime supervisor is not an ordinary user document: it is instantiated only by the host
/// bootstrap entrypoint from an unpinned single-Short-Dc document whose active definition and
/// transitive dependency subgraph carry authenticated bundled System provenance. User quota
/// configuration is validated and compiled once at admission (or at an explicit task/step
/// boundary) into compact local integer counters (`compiled_limits`) so execution never reads
/// mutable graph state per instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSupervisorAdmission {
    /// Short Dc entry point of the System supervisor base document.
    pub entry_dc: DcChar,
    /// Active System-block graph node ID holding the supervisor's canonical decomposition.
    pub supervisor_revision_node_id: u128,
    /// Sorted, deduplicated System-block node IDs in the supervisor's transitive dependency snapshot.
    pub dependency_snapshot_node_ids: Vec<u128>,
    /// Deterministic resource-accounting rule version (`WASM_RESOURCE_ACCOUNTING_VERSION`).
    pub accounting_version: u32,
    /// Compiled local integer resource counters (clamped by emergency host watchdog bounds).
    pub compiled_limits: DcWasmResourceLimits,
    /// Canonical 76-byte serialized snapshot of `compiled_limits` recorded in execution traces.
    pub effective_config_bytes: Vec<u8>,
}

impl SystemSupervisorAdmission {
    /// Resolves and authenticates the unpinned 1-Short-Dc System supervisor bootstrap document
    /// from `seed_graph`, validates `user_limits` (executing the System supervisor Dc-WASM
    /// definition when non-empty), clamps emergency host `watchdog` bounds,
    /// and compiles the local integer counters and canonical config snapshot.
    ///
    /// # Errors
    /// Returns an error if `bootstrap_tokens` is pinned with `Dc 641`, contains multiple tokens
    /// or a non-Short Dc, lacks authenticated bundled System provenance, or specifies an invalid
    /// `user_limits` configuration.
    pub fn admit_from_bootstrap_tokens(
        seed_graph: &SeedGraphIndex,
        bootstrap_tokens: &[DcShorthand],
        user_limits: DcWasmResourceLimits,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let resolved = seed_graph
            .resolve_trusted_system_supervisor_bootstrap_tokens(
                bootstrap_tokens,
            )?;
        let validated_user_limits =
            Self::validate_limits_with_supervisor_tokens(
                &resolved.definition_tokens,
                user_limits,
            )?;
        ensure!(
            validated_user_limits.accounting_version
                == WASM_RESOURCE_ACCOUNTING_VERSION,
            "Unsupported WASM resource accounting version {} in user quota configuration (expected {})",
            validated_user_limits.accounting_version,
            WASM_RESOURCE_ACCOUNTING_VERSION
        );
        ensure!(
            validated_user_limits.max_instructions_per_step > 0,
            "Supervisor compute_quota (max_instructions_per_step) must be > 0"
        );
        ensure!(
            validated_user_limits.max_memory_pages > 0,
            "Supervisor memory_quota_pages (max_memory_pages) must be > 0"
        );
        let compiled_limits =
            validated_user_limits.apply_watchdog_bounds(watchdog);
        let effective_config_bytes =
            compiled_limits.encode_canonical_config_bytes();
        Ok(Self {
            entry_dc: resolved.entry_dc,
            supervisor_revision_node_id: resolved.supervisor_revision_node_id,
            dependency_snapshot_node_ids: resolved.dependency_snapshot_node_ids,
            accounting_version: compiled_limits.accounting_version,
            compiled_limits,
            effective_config_bytes,
        })
    }

    /// Resolves and validates a graph-backed user quota configuration node (`user_config_node_id`)
    /// through the trusted System supervisor bootstrap document (`bootstrap_tokens`).
    ///
    /// # Errors
    /// Returns an error if the supervisor bootstrap tokens fail provenance verification, if the
    /// graph configuration node is missing/retracted or malformed, or if the supervisor rejects
    /// the configuration.
    pub fn admit_from_graph_config_node(
        seed_graph: &SeedGraphIndex,
        bootstrap_tokens: &[DcShorthand],
        user_config_node_id: u128,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let active_config_id = seed_graph
            .resolve_latest_node_id(user_config_node_id)
            .with_context(|| {
                format!(
                    "User quota configuration graph node l{user_config_node_id} is retracted or missing"
                )
            })?;
        let config_node = seed_graph
            .get_node(active_config_id)
            .with_context(|| {
                format!(
                    "User quota configuration graph node l{active_config_id} not found in graph"
                )
            })?;
        let parsed_limits =
            Self::decode_graph_quota_config_tokens(&config_node.tokens)?;
        Self::admit_from_bootstrap_tokens(
            seed_graph,
            bootstrap_tokens,
            parsed_limits,
            watchdog,
        )
    }

    /// Encodes `limits` as a graph node token sequence (`Dc 310 "Begin list"` of 76 canonical
    /// config bytes as `DcChar::from_unicode`, terminated by `Dc 311 "End list"`).
    #[must_use]
    pub fn encode_graph_quota_config_tokens(
        limits: &DcWasmResourceLimits,
    ) -> Vec<DcShorthand> {
        let bytes = limits.encode_canonical_config_bytes();
        let mut tokens = Vec::with_capacity(bytes.len().saturating_add(2));
        tokens.push(DcShorthand::from_global_id(dc::DC_BEGIN_LIST.to_u128()));
        for b in bytes {
            tokens.push(DcShorthand::Unicode(u32::from(b)));
        }
        tokens.push(DcShorthand::from_global_id(dc::DC_END_LIST.to_u128()));
        tokens
    }

    /// Decodes a graph-backed user quota configuration token sequence produced by
    /// [`Self::encode_graph_quota_config_tokens`].
    ///
    /// # Errors
    /// Returns an error if `tokens` is not a `Dc 310`..`Dc 311` list of 76 canonical config bytes.
    pub fn decode_graph_quota_config_tokens(
        tokens: &[DcShorthand],
    ) -> Result<DcWasmResourceLimits> {
        let open_list =
            DcShorthand::from_global_id(dc::DC_BEGIN_LIST.to_u128());
        let close_list = DcShorthand::from_global_id(dc::DC_END_LIST.to_u128());
        ensure!(
            tokens.first() == Some(&open_list)
                && tokens.last() == Some(&close_list),
            "Graph user quota configuration node must be enclosed in Dc 310 (Begin list) .. Dc 311 (End list)"
        );
        let inner = tokens
            .get(1..tokens.len().saturating_sub(1))
            .unwrap_or(&[]); // Reason: slice bounds verified by first/last check above.
        let mut bytes = Vec::with_capacity(inner.len());
        for &tok in inner {
            let DcShorthand::Unicode(code) = tok else {
                bail!(
                    "Invalid token {tok} in graph user quota configuration node"
                );
            };
            let byte = u8::try_from(code).with_context(|| {
                format!(
                    "Out-of-range byte U+{code:04X} in graph user quota configuration node"
                )
            })?;
            bytes.push(byte);
        }
        DcWasmResourceLimits::decode_canonical_config_bytes(&bytes)
    }

    fn validate_limits_with_supervisor_tokens(
        definition_tokens: &[DcShorthand],
        user_limits: DcWasmResourceLimits,
    ) -> Result<DcWasmResourceLimits> {
        if definition_tokens.len() <= 2 {
            return Ok(user_limits);
        }
        let mut sup_doc = DcMstring::new();
        for &tok in definition_tokens {
            let ch = DcChar::from_u128(tok.to_global_id()?);
            sup_doc.push_char(ch);
        }
        let mut input_doc = DcMstring::new();
        for b in user_limits.encode_canonical_config_bytes() {
            let ch = DcChar::from_unicode(u32::from(b))?;
            input_doc.push_char(ch);
        }
        let mut session =
            DcWasmSession::from_dc_wasm(&sup_doc, &input_doc, None)
                .context("Failed to instantiate System supervisor Dc-WASM document")?;
        match session.step()? {
            WasmStepOutcome::Complete(out_doc) => {
                if out_doc.as_bytes().is_empty() {
                    return Ok(user_limits);
                }
                let out_str = out_doc.to_dc_string()?;
                let mut out_bytes = Vec::with_capacity(76);
                for ch in out_str.chars() {
                    let code = ch.to_unicode()?;
                    let b = u8::try_from(code).context(
                        "Supervisor output config contains non-byte character",
                    )?;
                    out_bytes.push(b);
                }
                DcWasmResourceLimits::decode_canonical_config_bytes(&out_bytes)
            }
            other => bail!(
                "System supervisor validation step must return Complete(config), got {other:?}"
            ),
        }
    }

    /// Verifies that an ordinary user document does not attempt to claim System supervisor
    /// privilege.
    ///
    /// An ordinary document's leading `Dc 641` graph pin or reference to `supervisor_entry_dc`
    /// is **not** an elevation attempt: valid pinned ordinary documents and ordinary documents
    /// referencing `supervisor_entry_dc` execute normally without supervisor authority.
    /// Actual attempts by an ordinary document to assert workspace supervisor role
    /// (`Dc 527 "Execution environment process role is workspace supervisor"` = `Dc 517 "Boolean true"`)
    /// or inline bundled seed-graph provenance (`Dc 635` = `Dc 636`) are denied.
    ///
    /// # Errors
    /// Returns an error if `ordinary_doc_tokens` explicitly claims supervisor privilege.
    pub fn verify_ordinary_document_cannot_elevate(
        ordinary_doc_tokens: &[DcShorthand],
        _supervisor_entry_dc: DcChar,
    ) -> Result<()> {
        let role_supervisor = DcShorthand::Short(527);
        let bool_true =
            DcShorthand::from_global_id(dc::DC_BOOLEAN_TRUE.to_u128());
        let seed_source = DcShorthand::from_global_id(
            dc::DC_COLLECTIVE_TOOLBOX_SEED_GRAPH_DATA_SOURCE.to_u128(),
        );
        let bundled_csv = DcShorthand::from_global_id(
            dc::DC_COLLECTIVE_TOOLBOX_DC_CSV_SEED_DATABASE.to_u128(),
        );
        for window in ordinary_doc_tokens.windows(2) {
            if window == [role_supervisor, bool_true] {
                bail!(
                    "Ordinary user document cannot elevate privilege by asserting Dc 527 (Execution environment process role is workspace supervisor) = true"
                );
            }
            if window == [seed_source, bundled_csv] {
                bail!(
                    "Ordinary user document cannot elevate privilege by asserting inline Dc 635/636 bundled seed-graph provenance"
                );
            }
        }
        Ok(())
    }

    /// Verifies that a raw [`DcMstr`] ordinary document does not attempt to claim System
    /// supervisor privilege.
    ///
    /// # Errors
    /// Returns an error if `ordinary_doc` contains an inline supervisor privilege claim.
    pub fn verify_ordinary_dc_document_cannot_elevate(
        ordinary_doc: &DcMstr,
        supervisor_entry_dc: DcChar,
    ) -> Result<()> {
        let dc_str = ordinary_doc.to_dc_string()?;
        let mut tokens = Vec::with_capacity(dc_str.len());
        for ch in dc_str.chars() {
            tokens.push(DcShorthand::from_global_id(ch.to_u128()));
        }
        Self::verify_ordinary_document_cannot_elevate(
            &tokens,
            supervisor_entry_dc,
        )
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl Privilege {
    /// Classifies a capability request `DcMstr` emitted by a WASM module (`WasmStepStatus::RequestCapability`).
    #[must_use]
    pub fn from_request_dc(request: &DcMstr) -> Self {
        let mut ascii = String::new();
        if let Ok(dc_str) = request.to_dc_string() {
            for ch in dc_str.chars() {
                if let Ok(code) = ch.to_unicode()
                    && let Some(c) = char::from_u32(code)
                    && (c.is_ascii_alphanumeric() || c == '_' || c == ':')
                {
                    ascii.push(c);
                }
            }
        }
        match ascii.as_str() {
            "FileIo" | "file_io" => Self::FileIo,
            "Network" | "network" => Self::Network,
            "HighResolutionTimers" | "timers" => Self::HighResolutionTimers,
            "ExcessiveCompute" | "excessive_compute" => Self::ExcessiveCompute,
            other => Self::Custom(other.to_owned()),
        }
    }
}

/// Result of running a [`DcWasmSession`] under [`ExecutionContext`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionResult {
    /// Final output `DcMstring` (or empty if execution broke out early via `BreakOutResumeNext`).
    pub output: DcMstring,
    /// Intermediate frame outputs emitted via `WasmStepStatus::YieldFrame` (`1`).
    pub frames: Vec<DcMstring>,
    /// True if execution broke out early via `PermissionDecision::BreakOutResumeNext`
    /// (e.g., due to `Privilege::ExcessiveCompute`).
    pub broke_out_early: bool,
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

/// Observable execution trace event recorded during a [`DcWasmSession`] run.
///
/// Distinguishes logical document progress (`YieldFrame`, `CapabilityInteraction`, `Complete`)
/// from external physical VM resource interruptions (`ResourceInterruption`). A changed
/// compiler's physical instruction count is never treated as proof that replay reached the
/// same logical point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmTraceEvent {
    /// Emitted a streaming frame (`WasmStepStatus::YieldFrame`) at the given logical step index.
    YieldFrame {
        /// 0-based logical step index (`YieldFrame` / `RequestCapability` / `Complete` ordinal).
        logical_step: usize,
        /// Emitted frame document.
        frame: DcMstring,
    },
    /// Requested a host capability (`WasmStepStatus::RequestCapability`) and received a
    /// host authorization decision and response payload.
    CapabilityInteraction {
        /// 0-based logical step index at which the capability was requested.
        logical_step: usize,
        /// Classified capability privilege.
        privilege: Privilege,
        /// Raw capability request document emitted by the WASM module.
        request: DcMstring,
        /// Host capability decision applied during this run.
        decision: PermissionDecision,
        /// Response/mock document written back to the module's input mailbox (empty if broke out early).
        response: DcMstring,
    },
    /// External resource interruption raised by VM budgets (`ComputeQuotaExhausted`,
    /// `MemoryLimitExceeded`, `TableLimitExceeded`, `StepOutputSliceTooLarge`,
    /// `SessionOutputBudgetExceeded`, `CapabilityPayloadTooLarge`,
    /// `TraceBufferBudgetExceeded`) or host step budget exhaustion.
    ResourceInterruption {
        /// Number of completed logical steps before the interruption occurred.
        after_logical_steps: usize,
        /// Physical VM resource interruption (or `None` if triggered by the outer host step counter).
        interruption: Option<WasmResourceInterruption>,
        /// Host decision for `Privilege::ExcessiveCompute`.
        decision: PermissionDecision,
    },
    /// Non-resource guest WebAssembly trap (e.g., `unreachable`, integer divide by zero,
    /// out-of-bounds memory access) captured after `after_logical_steps` completed logical steps
    /// while preserving all prior yielded frames and capability events in the trace.
    NonResourceTrap {
        /// Number of completed logical steps before the trap occurred.
        after_logical_steps: usize,
        /// Diagnostic trap message captured from the WebAssembly runtime.
        trap_message: String,
    },
    /// Completed execution (`WasmStepStatus::Complete`) with the final output document.
    Complete {
        /// 0-based logical step index of completion.
        logical_step: usize,
        /// Final output document.
        output: DcMstring,
    },
}

impl WasmTraceEvent {
    /// Deterministic byte size of this trace event for `max_trace_bytes` accounting.
    #[must_use]
    pub fn estimated_trace_bytes(&self) -> usize {
        match self {
            Self::YieldFrame { frame, .. } => {
                16usize.saturating_add(frame.as_bytes().len())
            }
            Self::CapabilityInteraction {
                request, response, ..
            } => 24usize
                .saturating_add(request.as_bytes().len())
                .saturating_add(response.as_bytes().len()),
            Self::ResourceInterruption { .. } => 32usize,
            Self::NonResourceTrap { trap_message, .. } => {
                24usize.saturating_add(trap_message.len())
            }
            Self::Complete { output, .. } => {
                16usize.saturating_add(output.as_bytes().len())
            }
        }
    }
}

/// Complete observable execution trace and result of a [`DcWasmSession`] run,
/// including the active System supervisor revision, dependency snapshot, accounting version,
/// and canonical resource configuration snapshot required for deterministic replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionTrace {
    /// Final execution result (`output`, `frames`, `broke_out_early`).
    pub result: WasmExecutionResult,
    /// Ordered sequence of logical trace events, resource interruptions, and non-resource traps.
    pub events: Vec<WasmTraceEvent>,
    /// Active System-block graph node ID of the System supervisor base document (if admitted).
    pub supervisor_revision_node_id: Option<u128>,
    /// Sorted, deduplicated System-block node IDs in the supervisor's transitive dependency snapshot.
    pub dependency_snapshot_node_ids: Vec<u128>,
    /// Deterministic resource-accounting rule version (`WASM_RESOURCE_ACCOUNTING_VERSION`).
    pub accounting_version: u32,
    /// Canonical 76-byte serialized snapshot of effective [`DcWasmResourceLimits`].
    pub effective_config_bytes: Vec<u8>,
    /// 1-based ordinal of the last operation interrupted by `ComputeQuotaExhausted` (if any).
    pub last_interrupted_op_ordinal: Option<u64>,
    /// Required cost of the last operation interrupted by `ComputeQuotaExhausted` (if any).
    pub last_interrupted_op_required_cost: Option<u64>,
    /// Cumulative trace buffer bytes consumed across `events`.
    pub cumulative_trace_bytes: usize,
}

impl WasmExecutionTrace {
    /// Returns the terminal `ResourceInterruption` event if execution was halted by a denied
    /// resource interruption (`decision != PermissionDecision::BreakOutResumeNext` and
    /// `decision != PermissionDecision::Allow`).
    #[must_use]
    pub fn denied_resource_interruption(
        &self,
    ) -> Option<(usize, Option<&WasmResourceInterruption>, PermissionDecision)>
    {
        if let Some(WasmTraceEvent::ResourceInterruption {
            after_logical_steps,
            interruption,
            decision,
        }) = self.events.last()
            && *decision != PermissionDecision::BreakOutResumeNext
            && *decision != PermissionDecision::Allow
        {
            return Some((
                *after_logical_steps,
                interruption.as_ref(),
                *decision,
            ));
        }
        None
    }

    /// Returns the terminal `NonResourceTrap` event if execution terminated due to a non-resource
    /// guest WebAssembly trap.
    #[must_use]
    pub fn non_resource_trap(&self) -> Option<(usize, &str)> {
        if let Some(WasmTraceEvent::NonResourceTrap {
            after_logical_steps,
            trap_message,
        }) = self.events.last()
        {
            return Some((*after_logical_steps, trap_message.as_str()));
        }
        None
    }

    /// Returns `Err` if the trace terminated due to a denied resource interruption or a
    /// non-resource guest WebAssembly trap, or `Ok(())` if the trace completed or broke out
    /// via `PermissionDecision::BreakOutResumeNext`.
    ///
    /// # Errors
    /// Returns an error describing the denied resource interruption or non-resource trap.
    pub fn ensure_not_denied(&self) -> Result<()> {
        if let Some((steps, interruption, decision)) =
            self.denied_resource_interruption()
        {
            if let Some(intr) = interruption {
                return Err(anyhow::Error::new(intr.clone()).context(format!(
                    "Execution terminated after {steps} logical steps on VM resource interruption ({intr}); Privilege::ExcessiveCompute decision was {decision:?}"
                )));
            }
            bail!(
                "Execution denied by capability policy after {steps} logical steps ({decision:?})"
            );
        }
        if let Some((steps, trap_message)) = self.non_resource_trap() {
            bail!(
                "Execution terminated after {steps} logical steps on non-resource WebAssembly trap: {trap_message}"
            );
        }
        Ok(())
    }
}

/// Execution context for document evaluation and capability permission gating.
///
/// Capability authorization lives strictly in the host (`ExecutionContext`). Historical
/// System-block graph pinning (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`)
/// applies only to language and document semantics and never grants, restores, or modifies
/// host security permissions or capability policies.
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
    /// Portable compute quota units granted each time `Privilege::ExcessiveCompute`
    /// is approved with [`PermissionDecision::Allow`].
    pub compute_quota_grant: u64,
    /// Host-configured per-privilege authorization overrides.
    privilege_overrides: HashMap<Privilege, PermissionDecision>,
    /// Optional authenticated System supervisor admission metadata recorded in execution traces.
    pub supervisor_admission: Option<SystemSupervisorAdmission>,
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
            compute_quota_grant:
                ctb_formats_dcstring::DEFAULT_MAX_WASM_INSTRUCTIONS_PER_STEP,
            privilege_overrides: HashMap::new(),
            supervisor_admission: None,
        }
    }

    /// Builder-style method attaching an authenticated [`SystemSupervisorAdmission`] to this
    /// execution context so all execution traces record the supervisor revision, dependency
    /// snapshot, accounting version, and canonical configuration snapshot.
    #[must_use]
    pub fn with_system_supervisor_admission(
        mut self,
        admission: SystemSupervisorAdmission,
    ) -> Self {
        self.supervisor_admission = Some(admission);
        self
    }

    /// Authenticates and admits the unpinned 1-Short-Dc System supervisor bootstrap document
    /// from `seed_graph` and attaches it to this execution context.
    ///
    /// # Errors
    /// Returns an error if `bootstrap_tokens` is pinned, multi-token, non-Short-Dc, lacks
    /// authenticated bundled System provenance, or specifies invalid resource limits.
    pub fn admit_system_supervisor(
        &mut self,
        seed_graph: &SeedGraphIndex,
        bootstrap_tokens: &[DcShorthand],
        user_limits: DcWasmResourceLimits,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<&SystemSupervisorAdmission> {
        let admission = SystemSupervisorAdmission::admit_from_bootstrap_tokens(
            seed_graph,
            bootstrap_tokens,
            user_limits,
            watchdog,
        )?;
        self.supervisor_admission = Some(admission);
        self.supervisor_admission
            .as_ref()
            .context("supervisor_admission was just populated")
    }

    /// Builder-style method configuring the portable compute quota grant added when
    /// `Privilege::ExcessiveCompute` is approved with [`PermissionDecision::Allow`].
    #[must_use]
    pub const fn with_compute_quota_grant(mut self, grant: u64) -> Self {
        self.compute_quota_grant = grant;
        self
    }

    /// Configures a host capability policy override for `privilege`.
    pub fn set_privilege_decision(
        &mut self,
        privilege: Privilege,
        decision: PermissionDecision,
    ) {
        self.privilege_overrides.insert(privilege, decision);
    }

    /// Builder-style method configuring a host capability policy override for `privilege`.
    #[must_use]
    pub fn with_privilege_decision(
        mut self,
        privilege: Privilege,
        decision: PermissionDecision,
    ) -> Self {
        self.set_privilege_decision(privilege, decision);
        self
    }

    /// Evaluates a permission request against the current host policy, strongly steering
    /// toward mocking or denying if `has_errors` is true.
    ///
    /// Historical `Dc 641` graph pins on documents or WASM sessions are intentionally
    /// excluded from capability evaluation so historical pinning can never grant or restore
    /// obsolete security permissions.
    #[must_use]
    pub fn request_permission(
        &self,
        privilege: &Privilege,
    ) -> PermissionDecision {
        if let Some(&override_decision) =
            self.privilege_overrides.get(privilege)
        {
            if self.has_errors
                && override_decision == PermissionDecision::Allow
                && !matches!(privilege, Privilege::ExcessiveCompute)
            {
                return PermissionDecision::AutoMock;
            }
            return override_decision;
        }

        if matches!(privilege, Privilege::ExcessiveCompute) {
            // For compute exhaustion, default to offering breakout to resume
            // execution/printing on the next Dc.
            return PermissionDecision::BreakOutResumeNext;
        }

        if self.has_errors {
            // Strongly steer away from Allow when document has errors
            PermissionDecision::AutoMock
        } else {
            PermissionDecision::Deny
        }
    }

    /// Increments compute step count, checking if the compute budget is
    /// exhausted. If exhausted, raises a permission request for
    /// `Privilege::ExcessiveCompute`.
    pub fn step(&mut self) -> Result<PermissionDecision, anyhow::Error> {
        self.compute_steps = self.compute_steps.saturating_add(1);
        if self.compute_steps > self.max_compute_steps {
            let decision =
                self.request_permission(&Privilege::ExcessiveCompute);
            if decision == PermissionDecision::Allow {
                self.max_compute_steps = self.compute_steps;
                return Ok(PermissionDecision::Allow);
            }
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

    fn default_capability_response(decision: PermissionDecision) -> DcMstring {
        let mut resp = DcMstring::new();
        match decision {
            PermissionDecision::Allow => {
                resp.push_char(dc::DC_BOOLEAN_TRUE);
            }
            PermissionDecision::AutoMock | PermissionDecision::ManualMock => {
                resp.push_char(dc::DC_OPTIONAL_ABSENT);
            }
            PermissionDecision::Deny
            | PermissionDecision::BreakOutResumeNext => {
                resp.push_char(dc::DC_BOOLEAN_FALSE);
            }
        }
        resp
    }

    fn build_trace(
        &self,
        session: Option<&DcWasmSession>,
        fallback_limits: DcWasmResourceLimits,
        result: WasmExecutionResult,
        events: Vec<WasmTraceEvent>,
        cumulative_trace_bytes: usize,
        interrupted_op_info: Option<(u64, u64)>,
    ) -> WasmExecutionTrace {
        let effective_limits = self
            .supervisor_admission
            .as_ref()
            .map_or(fallback_limits, |adm| adm.compiled_limits);
        let (
            supervisor_revision_node_id,
            dependency_snapshot_node_ids,
            accounting_version,
            effective_config_bytes,
        ) = if let Some(adm) = &self.supervisor_admission {
            (
                Some(adm.supervisor_revision_node_id),
                adm.dependency_snapshot_node_ids.clone(),
                adm.accounting_version,
                adm.effective_config_bytes.clone(),
            )
        } else {
            (
                None,
                Vec::new(),
                effective_limits.accounting_version,
                effective_limits.encode_canonical_config_bytes(),
            )
        };
        let (last_interrupted_op_ordinal, last_interrupted_op_required_cost) =
            if let Some((ord, cost)) = interrupted_op_info {
                (Some(ord), Some(cost))
            } else if let Some(s) = session
                && s.last_interrupted_op_ordinal().is_some()
            {
                (
                    s.last_interrupted_op_ordinal(),
                    s.last_interrupted_op_required_cost(),
                )
            } else {
                (None, None)
            };
        WasmExecutionTrace {
            result,
            events,
            supervisor_revision_node_id,
            dependency_snapshot_node_ids,
            accounting_version,
            effective_config_bytes,
            last_interrupted_op_ordinal,
            last_interrupted_op_required_cost,
            cumulative_trace_bytes,
        }
    }

    /// Appends `event` to `events` if within `max_trace_bytes`; otherwise records a
    /// `WasmResourceInterruption::TraceBufferBudgetExceeded` event and returns `Err(())`.
    fn push_trace_event_bounded(
        &self,
        events: &mut Vec<WasmTraceEvent>,
        cumulative_trace_bytes: &mut usize,
        max_trace_bytes: usize,
        after_logical_steps: usize,
        event: WasmTraceEvent,
    ) -> std::result::Result<(), ()> {
        let event_bytes = event.estimated_trace_bytes();
        let next_bytes = cumulative_trace_bytes.saturating_add(event_bytes);
        if next_bytes > max_trace_bytes {
            let decision =
                self.request_permission(&Privilege::ExcessiveCompute);
            let intr_event = WasmTraceEvent::ResourceInterruption {
                after_logical_steps,
                interruption: Some(
                    WasmResourceInterruption::TraceBufferBudgetExceeded {
                        cumulative_trace_bytes: next_bytes,
                        max_trace_bytes,
                    },
                ),
                decision,
            };
            *cumulative_trace_bytes = cumulative_trace_bytes
                .saturating_add(intr_event.estimated_trace_bytes());
            events.push(intr_event);
            return Err(());
        }
        *cumulative_trace_bytes = next_bytes;
        events.push(event);
        Ok(())
    }

    fn handle_session_error_into_trace(
        &self,
        session: Option<&DcWasmSession>,
        fallback_limits: DcWasmResourceLimits,
        err: anyhow::Error,
        after_logical_steps: usize,
        frames: Vec<DcMstring>,
        mut events: Vec<WasmTraceEvent>,
        mut cumulative_trace_bytes: usize,
        interrupted_op_info: Option<(u64, u64)>,
    ) -> WasmExecutionTrace {
        let max_trace_bytes =
            session.map_or(fallback_limits.max_trace_bytes, |s| {
                s.resource_limits().max_trace_bytes
            });
        if let Some(interruption) =
            err.downcast_ref::<WasmResourceInterruption>().cloned()
        {
            let decision =
                self.request_permission(&Privilege::ExcessiveCompute);
            let _ = self.push_trace_event_bounded(
                &mut events,
                &mut cumulative_trace_bytes,
                max_trace_bytes,
                after_logical_steps,
                WasmTraceEvent::ResourceInterruption {
                    after_logical_steps,
                    interruption: Some(interruption),
                    decision,
                },
            );
        } else {
            let _ = self.push_trace_event_bounded(
                &mut events,
                &mut cumulative_trace_bytes,
                max_trace_bytes,
                after_logical_steps,
                WasmTraceEvent::NonResourceTrap {
                    after_logical_steps,
                    trap_message: format!("{err:#}"),
                },
            );
        }
        self.build_trace(
            session,
            fallback_limits,
            WasmExecutionResult {
                output: DcMstring::new(),
                frames,
                broke_out_early: true,
            },
            events,
            cumulative_trace_bytes,
            interrupted_op_info,
        )
    }

    /// Drives a [`DcWasmSession`] over `input` and records the full [`WasmExecutionTrace`]
    /// (distinguishing logical document progress from physical VM resource interruptions and
    /// non-resource guest traps).
    ///
    /// Both physical VM resource interruptions and non-resource guest traps preserve all
    /// accumulated `frames` and `events` in the returned [`WasmExecutionTrace`] so the finite
    /// prefix can be inspected and deterministically replayed. Call
    /// [`WasmExecutionTrace::ensure_not_denied`] to convert denied resource interruptions or
    /// non-resource traps into `Err`.
    ///
    /// # Errors
    /// Returns an error if `capability_handler` fails.
    pub fn execute_wasm_session_with_trace<F>(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        mut capability_handler: F,
    ) -> Result<WasmExecutionTrace>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        if let Some(adm) = &self.supervisor_admission {
            session.set_resource_limits(adm.compiled_limits);
        }
        let fallback_limits = session.resource_limits();
        let max_trace_bytes = fallback_limits.max_trace_bytes;
        let mut frames = Vec::new();
        let mut events = Vec::new();
        let mut cumulative_trace_bytes = 0usize;
        let mut last_interrupted_op: Option<(u64, u64)> = None;

        if let Err(err) = session.provide_input(input) {
            return Ok(self.handle_session_error_into_trace(
                Some(session),
                fallback_limits,
                err,
                0,
                frames,
                events,
                cumulative_trace_bytes,
                last_interrupted_op,
            ));
        }

        let mut logical_step = 0usize;
        loop {
            let budget_decision = match self.step() {
                Ok(d) => d,
                Err(_err) => {
                    let decision =
                        self.request_permission(&Privilege::ExcessiveCompute);
                    let _ = self.push_trace_event_bounded(
                        &mut events,
                        &mut cumulative_trace_bytes,
                        max_trace_bytes,
                        logical_step,
                        WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: logical_step,
                            interruption: None,
                            decision,
                        },
                    );
                    return Ok(self.build_trace(
                        Some(session),
                        fallback_limits,
                        WasmExecutionResult {
                            output: DcMstring::new(),
                            frames,
                            broke_out_early: true,
                        },
                        events,
                        cumulative_trace_bytes,
                        last_interrupted_op,
                    ));
                }
            };
            if budget_decision == PermissionDecision::BreakOutResumeNext
                || budget_decision == PermissionDecision::Deny
            {
                let _ = self.push_trace_event_bounded(
                    &mut events,
                    &mut cumulative_trace_bytes,
                    max_trace_bytes,
                    logical_step,
                    WasmTraceEvent::ResourceInterruption {
                        after_logical_steps: logical_step,
                        interruption: None,
                        decision: budget_decision,
                    },
                );
                return Ok(self.build_trace(
                    Some(session),
                    fallback_limits,
                    WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                    cumulative_trace_bytes,
                    last_interrupted_op,
                ));
            }

            let step_outcome = match session.step() {
                Ok(outcome) => outcome,
                Err(err) => {
                    if let Some(
                        WasmResourceInterruption::ComputeQuotaExhausted {
                            limit,
                        },
                    ) = err.downcast_ref::<WasmResourceInterruption>().cloned()
                    {
                        if let (Some(ord), Some(cost)) = (
                            session.last_interrupted_op_ordinal(),
                            session.last_interrupted_op_required_cost(),
                        ) {
                            last_interrupted_op = Some((ord, cost));
                        }
                        let decision = self
                            .request_permission(&Privilege::ExcessiveCompute);
                        if self
                            .push_trace_event_bounded(
                                &mut events,
                                &mut cumulative_trace_bytes,
                                max_trace_bytes,
                                logical_step,
                                WasmTraceEvent::ResourceInterruption {
                                    after_logical_steps: logical_step,
                                    interruption: Some(
                                        WasmResourceInterruption::ComputeQuotaExhausted {
                                            limit,
                                        },
                                    ),
                                    decision,
                                },
                            )
                            .is_err()
                        {
                            return Ok(self.build_trace(
                                Some(session),
                                fallback_limits,
                                WasmExecutionResult {
                                    output: DcMstring::new(),
                                    frames,
                                    broke_out_early: true,
                                },
                                events,
                                cumulative_trace_bytes,
                                last_interrupted_op,
                            ));
                        }
                        if decision == PermissionDecision::Allow {
                            session.increase_compute_quota(
                                self.compute_quota_grant.max(1),
                            );
                            continue;
                        }
                        return Ok(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    return Ok(self.handle_session_error_into_trace(
                        Some(session),
                        fallback_limits,
                        err,
                        logical_step,
                        frames,
                        events,
                        cumulative_trace_bytes,
                        last_interrupted_op,
                    ));
                }
            };

            match step_outcome {
                WasmStepOutcome::Complete(output) => {
                    if self
                        .push_trace_event_bounded(
                            &mut events,
                            &mut cumulative_trace_bytes,
                            max_trace_bytes,
                            logical_step,
                            WasmTraceEvent::Complete {
                                logical_step,
                                output: output.clone(),
                            },
                        )
                        .is_err()
                    {
                        return Ok(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    return Ok(self.build_trace(
                        Some(session),
                        fallback_limits,
                        WasmExecutionResult {
                            output,
                            frames,
                            broke_out_early: false,
                        },
                        events,
                        cumulative_trace_bytes,
                        last_interrupted_op,
                    ));
                }
                WasmStepOutcome::YieldFrame(frame) => {
                    if self
                        .push_trace_event_bounded(
                            &mut events,
                            &mut cumulative_trace_bytes,
                            max_trace_bytes,
                            logical_step,
                            WasmTraceEvent::YieldFrame {
                                logical_step,
                                frame: frame.clone(),
                            },
                        )
                        .is_err()
                    {
                        return Ok(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    frames.push(frame);
                    logical_step = logical_step.saturating_add(1);
                }
                WasmStepOutcome::RequestCapability(req_dc) => {
                    let privilege = Privilege::from_request_dc(&req_dc);
                    let decision = self.request_permission(&privilege);
                    if decision == PermissionDecision::BreakOutResumeNext {
                        let _ = self.push_trace_event_bounded(
                            &mut events,
                            &mut cumulative_trace_bytes,
                            max_trace_bytes,
                            logical_step,
                            WasmTraceEvent::CapabilityInteraction {
                                logical_step,
                                privilege,
                                request: req_dc,
                                decision,
                                response: DcMstring::new(),
                            },
                        );
                        return Ok(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    let response =
                        capability_handler(&privilege, decision, &req_dc)?;
                    if self
                        .push_trace_event_bounded(
                            &mut events,
                            &mut cumulative_trace_bytes,
                            max_trace_bytes,
                            logical_step,
                            WasmTraceEvent::CapabilityInteraction {
                                logical_step,
                                privilege,
                                request: req_dc,
                                decision,
                                response: response.clone(),
                            },
                        )
                        .is_err()
                    {
                        return Ok(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    logical_step = logical_step.saturating_add(1);
                    if let Err(err) =
                        session.provide_capability_response(&response)
                    {
                        return Ok(self.handle_session_error_into_trace(
                            Some(session),
                            fallback_limits,
                            err,
                            logical_step,
                            frames,
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                }
            }
        }
    }

    /// Drives a [`DcWasmSession`] with the default capability response encoder and returns
    /// the full [`WasmExecutionTrace`].
    ///
    /// # Errors
    /// Returns an error if the capability handler fails.
    pub fn execute_wasm_session_default_with_trace(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
    ) -> Result<WasmExecutionTrace> {
        self.execute_wasm_session_with_trace(
            session,
            input,
            |_priv, decision, _req| {
                Ok(Self::default_capability_response(decision))
            },
        )
    }

    /// Verifies that an ordinary Dc-WASM document does not attempt to elevate to System
    /// supervisor privilege, then instantiates and executes it without granting supervisor
    /// authority (even if the document carries a leading `Dc 641` pin or references
    /// `supervisor_entry_dc`).
    ///
    /// # Errors
    /// Returns an error if `ordinary_doc` attempts to claim supervisor privilege or fails
    /// to assemble/instantiate.
    pub fn execute_ordinary_dc_wasm_default_with_trace(
        &mut self,
        ordinary_doc: &DcMstr,
        input: &DcMstr,
        most_recent_node_id: Option<u128>,
        supervisor_entry_dc: DcChar,
    ) -> Result<WasmExecutionTrace> {
        SystemSupervisorAdmission::verify_ordinary_dc_document_cannot_elevate(
            ordinary_doc,
            supervisor_entry_dc,
        )?;
        let mut session = DcWasmSession::from_dc_wasm(
            ordinary_doc,
            input,
            most_recent_node_id,
        )?;
        self.execute_wasm_session_default_with_trace(&mut session, input)
    }

    /// Replays a [`DcWasmSession`] against a previously recorded [`WasmExecutionTrace`]:
    /// - Verifies that the active accounting version and canonical resource configuration match.
    /// - Compares every capability request (`Privilege` and `DcMstring` request payload) emitted by
    ///   the document at each logical step, then supplies the recorded allow/deny/mock response.
    /// - Actually executes `session.step()` at every recorded resource interruption boundary and
    ///   verifies that the module triggers the recorded `WasmResourceInterruption` at the exact
    ///   recorded `last_interrupted_op_ordinal` and `last_interrupted_op_required_cost`.
    ///
    /// # Errors
    /// Returns an error if the document's capability requests, yielded frames, completion
    /// output, or trap/interruption behavior diverge from `recorded_trace`.
    pub fn replay_wasm_session_trace(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        recorded_trace: &WasmExecutionTrace,
    ) -> Result<WasmExecutionTrace> {
        self.replay_wasm_session_trace_with_prefix(
            session,
            input,
            recorded_trace,
            Vec::new(),
            0,
            0,
            None,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "Replay state machine threads construction prefix events and config snapshot"
    )]
    fn replay_wasm_session_trace_with_prefix(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        recorded_trace: &WasmExecutionTrace,
        mut events: Vec<WasmTraceEvent>,
        mut cumulative_trace_bytes: usize,
        mut event_idx: usize,
        initial_limits_override: Option<DcWasmResourceLimits>,
    ) -> Result<WasmExecutionTrace> {
        ensure!(
            recorded_trace.accounting_version
                == WASM_RESOURCE_ACCOUNTING_VERSION,
            "Replay accounting_version mismatch: recorded {}, runtime {}",
            recorded_trace.accounting_version,
            WASM_RESOURCE_ACCOUNTING_VERSION
        );
        let decoded_limits =
            DcWasmResourceLimits::decode_canonical_config_bytes(
                &recorded_trace.effective_config_bytes,
            )
            .ok();
        if let Some(adm) = &self.supervisor_admission {
            if initial_limits_override.is_none()
                && session.total_instructions_executed() == 0
            {
                session.set_resource_limits(adm.compiled_limits);
            }
        } else if let Some(rec_limits) = decoded_limits
            && initial_limits_override.is_none()
            && session.total_instructions_executed() == 0
        {
            session.set_resource_limits(rec_limits);
        }
        // Reason for fallback: fallback to session's active limits only if a legacy/synthetic trace omitted canonical config bytes.
        let fallback_limits = initial_limits_override.unwrap_or_else(|| {
            decoded_limits.unwrap_or_else(|| session.resource_limits())
        });
        let max_trace_bytes = fallback_limits.max_trace_bytes;
        let mut frames = Vec::new();
        let mut last_interrupted_op: Option<(u64, u64)> = None;
        let finalize_checked_trace =
            |mut trace: WasmExecutionTrace| -> Result<WasmExecutionTrace> {
                if trace.supervisor_revision_node_id.is_none()
                    && recorded_trace.supervisor_revision_node_id.is_some()
                {
                    trace.supervisor_revision_node_id =
                        recorded_trace.supervisor_revision_node_id;
                    trace.dependency_snapshot_node_ids =
                        recorded_trace.dependency_snapshot_node_ids.clone();
                }
                ensure!(
                    &trace == recorded_trace,
                    "Replay terminated without matching the complete recorded trace and result"
                );
                Ok(trace)
            };

        let record_replay_error =
            |this: &Self,
             session_opt: Option<&DcWasmSession>,
             err: anyhow::Error,
             step_idx: usize,
             cur_event_idx: usize,
             frames_vec: Vec<DcMstring>,
             mut events_vec: Vec<WasmTraceEvent>,
             mut trace_bytes: usize,
             interrupted_op: Option<(u64, u64)>|
             -> Result<WasmExecutionTrace> {
                let expected = recorded_trace
                    .events
                    .get(cur_event_idx)
                    .with_context(|| {
                        format!(
                            "Replay encountered error ({err:#}) at logical step {step_idx} beyond recorded trace length ({})",
                            recorded_trace.events.len()
                        )
                    })?;
                if let Some(actual_intr) =
                    err.downcast_ref::<WasmResourceInterruption>().cloned()
                {
                    let WasmTraceEvent::ResourceInterruption {
                        after_logical_steps,
                        interruption: Some(rec_intr),
                        decision,
                    } = expected
                    else {
                        bail!(
                            "Replay resource interruption mismatch at logical step {step_idx}: expected {expected:?}, got {actual_intr:?}"
                        );
                    };
                    ensure!(
                        *after_logical_steps == step_idx
                            && rec_intr == &actual_intr,
                        "Replay resource interruption mismatch at logical step {step_idx}: expected ({after_logical_steps}, {rec_intr:?}), got ({step_idx}, {actual_intr:?})"
                    );
                    let ev = WasmTraceEvent::ResourceInterruption {
                        after_logical_steps: step_idx,
                        interruption: Some(actual_intr),
                        decision: *decision,
                    };
                    trace_bytes =
                        trace_bytes.saturating_add(ev.estimated_trace_bytes());
                    events_vec.push(ev);
                    return finalize_checked_trace(this.build_trace(
                        session_opt,
                        fallback_limits,
                        WasmExecutionResult {
                            output: DcMstring::new(),
                            frames: frames_vec,
                            broke_out_early: true,
                        },
                        events_vec,
                        trace_bytes,
                        interrupted_op,
                    ));
                }
                let WasmTraceEvent::NonResourceTrap {
                    after_logical_steps,
                    ..
                } = expected
                else {
                    bail!(
                        "Replay non-resource trap mismatch at logical step {step_idx}: expected {expected:?}, got {err:#}"
                    );
                };
                ensure!(
                    *after_logical_steps == step_idx,
                    "Replay non-resource trap logical step mismatch: expected {after_logical_steps}, got {step_idx}"
                );
                let ev = WasmTraceEvent::NonResourceTrap {
                    after_logical_steps: step_idx,
                    trap_message: format!("{err:#}"),
                };
                trace_bytes =
                    trace_bytes.saturating_add(ev.estimated_trace_bytes());
                events_vec.push(ev);
                finalize_checked_trace(this.build_trace(
                    session_opt,
                    fallback_limits,
                    WasmExecutionResult {
                        output: DcMstring::new(),
                        frames: frames_vec,
                        broke_out_early: true,
                    },
                    events_vec,
                    trace_bytes,
                    interrupted_op,
                ))
            };

        if let Err(err) = session.provide_input(input) {
            return record_replay_error(
                self,
                Some(session),
                err,
                0,
                event_idx,
                frames,
                events,
                cumulative_trace_bytes,
                last_interrupted_op,
            );
        }
        let mut logical_step = 0usize;

        loop {
            if let Some(WasmTraceEvent::ResourceInterruption {
                after_logical_steps,
                interruption: None,
                decision,
            }) = recorded_trace.events.get(event_idx)
                && *after_logical_steps == logical_step
            {
                let ev = WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: *after_logical_steps,
                    interruption: None,
                    decision: *decision,
                };
                cumulative_trace_bytes = cumulative_trace_bytes
                    .saturating_add(ev.estimated_trace_bytes());
                events.push(ev);
                if *decision == PermissionDecision::Allow {
                    event_idx = event_idx.saturating_add(1);
                    continue;
                }
                return finalize_checked_trace(self.build_trace(
                    Some(session),
                    fallback_limits,
                    WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                    cumulative_trace_bytes,
                    last_interrupted_op,
                ));
            }

            let expected_event =
                recorded_trace.events.get(event_idx).with_context(|| {
                    format!(
                        "Replay exceeded recorded trace length ({}) at logical step {logical_step}",
                        recorded_trace.events.len()
                    )
                })?;

            let step_outcome = match session.step() {
                Ok(outcome) => outcome,
                Err(err) => {
                    if let Some(
                        WasmResourceInterruption::ComputeQuotaExhausted {
                            limit,
                        },
                    ) = err.downcast_ref::<WasmResourceInterruption>().cloned()
                    {
                        let ord = session
                            .last_interrupted_op_ordinal()
                            .context("ComputeQuotaExhausted during replay missing last_interrupted_op_ordinal")?;
                        let cost = session
                            .last_interrupted_op_required_cost()
                            .context("ComputeQuotaExhausted during replay missing last_interrupted_op_required_cost")?;
                        last_interrupted_op = Some((ord, cost));

                        let WasmTraceEvent::ResourceInterruption {
                            after_logical_steps,
                            interruption:
                                Some(
                                    WasmResourceInterruption::ComputeQuotaExhausted {
                                        limit: rec_limit,
                                    },
                                ),
                            decision,
                        } = expected_event
                        else {
                            bail!(
                                "Replay hit ComputeQuotaExhausted {{ limit: {limit} }} at logical step {logical_step} (op {ord}, cost {cost}), but recorded event was {expected_event:?}"
                            );
                        };
                        ensure!(
                            *after_logical_steps == logical_step
                                && *rec_limit == limit,
                            "Replay ComputeQuotaExhausted mismatch at logical step {logical_step}: expected (step {after_logical_steps}, limit {rec_limit}), got (step {logical_step}, limit {limit})"
                        );
                        // Reason for fallback: an out-of-range tail slice means no further events exist.
                        let has_later_compute_interruption = recorded_trace
                            .events
                            .get(event_idx.saturating_add(1)..)
                            .unwrap_or(&[])
                            .iter()
                            .any(|e| {
                                matches!(
                                    e,
                                    WasmTraceEvent::ResourceInterruption {
                                        interruption: Some(
                                            WasmResourceInterruption::ComputeQuotaExhausted { .. }
                                        ),
                                        ..
                                    }
                                )
                            });
                        if !has_later_compute_interruption {
                            ensure!(
                                recorded_trace.last_interrupted_op_ordinal
                                    == Some(ord)
                                    && recorded_trace
                                        .last_interrupted_op_required_cost
                                        == Some(cost),
                                "Replay interrupted op mismatch at logical step {logical_step}: expected ordinal {:?} (cost {:?}), got ordinal {ord} (cost {cost})",
                                recorded_trace.last_interrupted_op_ordinal,
                                recorded_trace.last_interrupted_op_required_cost
                            );
                        }
                        let ev = WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: logical_step,
                            interruption: Some(
                                WasmResourceInterruption::ComputeQuotaExhausted {
                                    limit,
                                },
                            ),
                            decision: *decision,
                        };
                        cumulative_trace_bytes = cumulative_trace_bytes
                            .saturating_add(ev.estimated_trace_bytes());
                        events.push(ev);
                        if *decision == PermissionDecision::Allow {
                            session.increase_compute_quota(
                                self.compute_quota_grant.max(1),
                            );
                            event_idx = event_idx.saturating_add(1);
                            continue;
                        }
                        return finalize_checked_trace(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    return record_replay_error(
                        self,
                        Some(session),
                        err,
                        logical_step,
                        event_idx,
                        frames,
                        events,
                        cumulative_trace_bytes,
                        last_interrupted_op,
                    );
                }
            };

            if let WasmTraceEvent::ResourceInterruption {
                after_logical_steps: exp_step,
                interruption:
                    Some(
                        WasmResourceInterruption::TraceBufferBudgetExceeded {
                            cumulative_trace_bytes: exp_cum_bytes,
                            max_trace_bytes: exp_max_bytes,
                        },
                    ),
                decision: rec_decision,
            } = expected_event
            {
                let candidate_bytes = match &step_outcome {
                    WasmStepOutcome::YieldFrame(frame) => {
                        WasmTraceEvent::YieldFrame {
                            logical_step,
                            frame: frame.clone(),
                        }
                        .estimated_trace_bytes()
                    }
                    WasmStepOutcome::Complete(output) => {
                        WasmTraceEvent::Complete {
                            logical_step,
                            output: output.clone(),
                        }
                        .estimated_trace_bytes()
                    }
                    WasmStepOutcome::RequestCapability(req_dc) => {
                        let privilege = Privilege::from_request_dc(req_dc);
                        let resp =
                            Self::default_capability_response(*rec_decision);
                        WasmTraceEvent::CapabilityInteraction {
                            logical_step,
                            privilege,
                            request: req_dc.clone(),
                            decision: *rec_decision,
                            response: resp,
                        }
                        .estimated_trace_bytes()
                    }
                };
                let next_bytes =
                    cumulative_trace_bytes.saturating_add(candidate_bytes);
                ensure!(
                    *exp_step == logical_step
                        && next_bytes > max_trace_bytes
                        && *exp_cum_bytes == next_bytes
                        && *exp_max_bytes == max_trace_bytes,
                    "Replay TraceBufferBudgetExceeded mismatch at logical step {logical_step}"
                );
                let ev = WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: logical_step,
                    interruption: Some(
                        WasmResourceInterruption::TraceBufferBudgetExceeded {
                            cumulative_trace_bytes: next_bytes,
                            max_trace_bytes,
                        },
                    ),
                    decision: *rec_decision,
                };
                cumulative_trace_bytes = cumulative_trace_bytes
                    .saturating_add(ev.estimated_trace_bytes());
                events.push(ev);
                return finalize_checked_trace(self.build_trace(
                    Some(session),
                    fallback_limits,
                    WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                    cumulative_trace_bytes,
                    last_interrupted_op,
                ));
            }

            match (step_outcome, expected_event) {
                (
                    WasmStepOutcome::YieldFrame(frame),
                    WasmTraceEvent::YieldFrame {
                        logical_step: exp_step,
                        frame: exp_frame,
                    },
                ) => {
                    ensure!(
                        logical_step == *exp_step && &frame == exp_frame,
                        "Replay YieldFrame mismatch at logical step {logical_step}: expected frame {exp_frame:?}, got {frame:?}"
                    );
                    let _ = self.push_trace_event_bounded(
                        &mut events,
                        &mut cumulative_trace_bytes,
                        max_trace_bytes,
                        logical_step,
                        WasmTraceEvent::YieldFrame {
                            logical_step,
                            frame: frame.clone(),
                        },
                    );
                    frames.push(frame);
                    logical_step = logical_step.saturating_add(1);
                    event_idx = event_idx.saturating_add(1);
                }
                (
                    WasmStepOutcome::RequestCapability(req_dc),
                    WasmTraceEvent::CapabilityInteraction {
                        logical_step: exp_step,
                        privilege: exp_priv,
                        request: exp_req,
                        decision: rec_decision,
                        response: rec_response,
                    },
                ) => {
                    let privilege = Privilege::from_request_dc(&req_dc);
                    ensure!(
                        logical_step == *exp_step
                            && &privilege == exp_priv
                            && &req_dc == exp_req,
                        "Replay capability request mismatch at logical step {logical_step}: expected ({exp_priv:?}, {exp_req:?}), got ({privilege:?}, {req_dc:?})"
                    );
                    let _ = self.push_trace_event_bounded(
                        &mut events,
                        &mut cumulative_trace_bytes,
                        max_trace_bytes,
                        logical_step,
                        WasmTraceEvent::CapabilityInteraction {
                            logical_step,
                            privilege,
                            request: req_dc,
                            decision: *rec_decision,
                            response: rec_response.clone(),
                        },
                    );
                    if *rec_decision == PermissionDecision::BreakOutResumeNext {
                        return finalize_checked_trace(self.build_trace(
                            Some(session),
                            fallback_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        ));
                    }
                    logical_step = logical_step.saturating_add(1);
                    event_idx = event_idx.saturating_add(1);
                    if let Err(err) =
                        session.provide_capability_response(rec_response)
                    {
                        return record_replay_error(
                            self,
                            Some(session),
                            err,
                            logical_step,
                            event_idx,
                            frames,
                            events,
                            cumulative_trace_bytes,
                            last_interrupted_op,
                        );
                    }
                }
                (
                    WasmStepOutcome::Complete(output),
                    WasmTraceEvent::Complete {
                        logical_step: exp_step,
                        output: exp_output,
                    },
                ) => {
                    ensure!(
                        logical_step == *exp_step && &output == exp_output,
                        "Replay Complete output mismatch at logical step {logical_step}: expected {exp_output:?}, got {output:?}"
                    );
                    let _ = self.push_trace_event_bounded(
                        &mut events,
                        &mut cumulative_trace_bytes,
                        max_trace_bytes,
                        logical_step,
                        WasmTraceEvent::Complete {
                            logical_step,
                            output: output.clone(),
                        },
                    );
                    return finalize_checked_trace(self.build_trace(
                        Some(session),
                        fallback_limits,
                        WasmExecutionResult {
                            output,
                            frames,
                            broke_out_early: false,
                        },
                        events,
                        cumulative_trace_bytes,
                        last_interrupted_op,
                    ));
                }
                (actual, expected) => {
                    bail!(
                        "Replay trace divergence at logical step {logical_step}: expected {expected:?}, got {actual:?}"
                    );
                }
            }
        }
    }

    /// Drives a [`DcWasmSession`] over `input` using the standard 32-byte linear-memory
    /// mailbox ABI and `step() -> StatusCode` protocol:
    /// - `0 = Complete`: returns `WasmExecutionResult` with the final `DcMstring` output.
    /// - `1 = YieldFrame`: appends the yielded `DcMstring` frame to `frames` and resumes.
    /// - `2 = RequestCapability`: classifies the requested [`Privilege`], evaluates
    ///   `self.request_permission`, invokes `capability_handler` to produce the response
    ///   `DcMstring`, writes that response into the session's input mailbox, and resumes.
    /// - If `self.step()` or `self.request_permission` returns
    ///   [`PermissionDecision::BreakOutResumeNext`], stops immediately with
    ///   `broke_out_early: true`.
    pub fn execute_wasm_session<F>(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        capability_handler: F,
    ) -> Result<WasmExecutionResult>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        let trace = self.execute_wasm_session_with_trace(
            session,
            input,
            capability_handler,
        )?;
        trace.ensure_not_denied()?;
        Ok(trace.result)
    }

    /// Drives a [`DcWasmSession`] with a default capability response encoder:
    /// - `PermissionDecision::Allow` -> `Dc 517 "Boolean true"`
    /// - `PermissionDecision::AutoMock` / `PermissionDecision::ManualMock` -> `Dc 316 "Optional value absent"`
    /// - `PermissionDecision::Deny` -> `Dc 518 "Boolean false"`
    pub fn execute_wasm_session_default(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
    ) -> Result<WasmExecutionResult> {
        let trace =
            self.execute_wasm_session_default_with_trace(session, input)?;
        trace.ensure_not_denied()?;
        Ok(trace.result)
    }

    /// Instantiates and executes compiled `.wasm` module bytes over `input` with explicit
    /// [`ctb_formats_dcstring::DcWasmResourceLimits`], capturing construction-time resource
    /// interruptions (initial memory/table limits or `start` compute_quota exhaustion) and
    /// `start`-section non-resource traps in the returned [`WasmExecutionTrace`].
    ///
    /// # Errors
    /// Returns an error if the module fails to parse or if `capability_handler` fails.
    pub fn execute_wasm_bytes_with_trace<F>(
        &mut self,
        wasm_bytes: &[u8],
        input: &DcMstr,
        most_recent_node_id: Option<u128>,
        mut resource_limits: ctb_formats_dcstring::DcWasmResourceLimits,
        capability_handler: F,
    ) -> Result<WasmExecutionTrace>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        let initial_limits = resource_limits;
        let mut pre_events = Vec::new();
        let mut pre_bytes = 0usize;
        let mut session = loop {
            match DcWasmSession::from_wasm_bytes_with_limits(
                wasm_bytes,
                input,
                most_recent_node_id,
                resource_limits,
            ) {
                Ok(s) => break s,
                Err(err) => {
                    if let Some(
                        WasmResourceInterruption::ComputeQuotaExhausted {
                            limit,
                        },
                    ) = err.downcast_ref::<WasmResourceInterruption>().cloned()
                    {
                        let decision = self
                            .request_permission(&Privilege::ExcessiveCompute);
                        let ev = WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: 0,
                            interruption: Some(
                                WasmResourceInterruption::ComputeQuotaExhausted {
                                    limit,
                                },
                            ),
                            decision,
                        };
                        pre_bytes = pre_bytes
                            .saturating_add(ev.estimated_trace_bytes());
                        pre_events.push(ev);
                        if decision == PermissionDecision::Allow {
                            resource_limits.max_instructions_per_step =
                                resource_limits
                                    .max_instructions_per_step
                                    .saturating_add(
                                        self.compute_quota_grant.max(1),
                                    );
                            continue;
                        }
                        return Ok(self.build_trace(
                            None,
                            initial_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames: Vec::new(),
                                broke_out_early: true,
                            },
                            pre_events,
                            pre_bytes,
                            None,
                        ));
                    }
                    if err.downcast_ref::<WasmResourceInterruption>().is_some()
                        || err
                            .downcast_ref::<ctb_formats_dcstring::WasmStartSectionTrap>()
                            .is_some()
                    {
                        return Ok(self.handle_session_error_into_trace(
                            None,
                            initial_limits,
                            err,
                            0,
                            Vec::new(),
                            pre_events,
                            pre_bytes,
                            None,
                        ));
                    }
                    return Err(err);
                }
            }
        };
        let mut trace = self.execute_wasm_session_with_trace(
            &mut session,
            input,
            capability_handler,
        )?;
        if !pre_events.is_empty() {
            trace.cumulative_trace_bytes =
                pre_bytes.saturating_add(trace.cumulative_trace_bytes);
            pre_events.extend(trace.events);
            trace.events = pre_events;
            trace.effective_config_bytes =
                initial_limits.encode_canonical_config_bytes();
        }
        Ok(trace)
    }

    /// Replays compiled `.wasm` module bytes against a recorded [`WasmExecutionTrace`],
    /// actually instantiating and running the module to verify construction-time resource
    /// interruptions, `start`-section non-resource traps, and `provide_input` / `step` events.
    ///
    /// # Errors
    /// Returns an error if replay diverges from `recorded_trace`.
    pub fn replay_wasm_bytes_trace(
        &mut self,
        wasm_bytes: &[u8],
        input: &DcMstr,
        most_recent_node_id: Option<u128>,
        mut resource_limits: ctb_formats_dcstring::DcWasmResourceLimits,
        recorded_trace: &WasmExecutionTrace,
    ) -> Result<WasmExecutionTrace> {
        ensure!(
            recorded_trace.accounting_version
                == WASM_RESOURCE_ACCOUNTING_VERSION,
            "Replay accounting_version mismatch: recorded {}, runtime {}",
            recorded_trace.accounting_version,
            WASM_RESOURCE_ACCOUNTING_VERSION
        );
        if let Ok(decoded) =
            DcWasmResourceLimits::decode_canonical_config_bytes(
                &recorded_trace.effective_config_bytes,
            )
        {
            resource_limits = decoded;
        }
        let initial_limits = resource_limits;
        let mut pre_events = Vec::new();
        let mut pre_bytes = 0usize;
        let mut event_cursor = 0usize;

        let mut session = loop {
            match DcWasmSession::from_wasm_bytes_with_limits(
                wasm_bytes,
                input,
                most_recent_node_id,
                resource_limits,
            ) {
                Ok(s) => break s,
                Err(err) => {
                    let expected = recorded_trace
                        .events
                        .get(event_cursor)
                        .with_context(|| {
                            format!(
                                "Replay encountered construction error ({err:#}) beyond recorded trace length ({})",
                                recorded_trace.events.len()
                            )
                        })?;
                    if let Some(actual_intr) =
                        err.downcast_ref::<WasmResourceInterruption>().cloned()
                    {
                        let WasmTraceEvent::ResourceInterruption {
                            after_logical_steps,
                            interruption: Some(rec_intr),
                            decision,
                        } = expected
                        else {
                            bail!(
                                "Replay construction resource interruption mismatch: expected {expected:?}, got {actual_intr:?}"
                            );
                        };
                        ensure!(
                            *after_logical_steps == 0
                                && rec_intr == &actual_intr,
                            "Replay construction resource interruption mismatch: expected ({after_logical_steps}, {rec_intr:?}), got (0, {actual_intr:?})"
                        );
                        let ev = WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: 0,
                            interruption: Some(actual_intr.clone()),
                            decision: *decision,
                        };
                        pre_bytes = pre_bytes
                            .saturating_add(ev.estimated_trace_bytes());
                        pre_events.push(ev);
                        event_cursor = event_cursor.saturating_add(1);
                        if matches!(
                            actual_intr,
                            WasmResourceInterruption::ComputeQuotaExhausted { .. }
                        ) && *decision == PermissionDecision::Allow
                        {
                            resource_limits.max_instructions_per_step =
                                resource_limits
                                    .max_instructions_per_step
                                    .saturating_add(
                                        self.compute_quota_grant.max(1),
                                    );
                            continue;
                        }
                        let mut trace = self.build_trace(
                            None,
                            initial_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames: Vec::new(),
                                broke_out_early: true,
                            },
                            pre_events,
                            pre_bytes,
                            None,
                        );
                        if trace.supervisor_revision_node_id.is_none()
                            && recorded_trace
                                .supervisor_revision_node_id
                                .is_some()
                        {
                            trace.supervisor_revision_node_id =
                                recorded_trace.supervisor_revision_node_id;
                            trace.dependency_snapshot_node_ids = recorded_trace
                                .dependency_snapshot_node_ids
                                .clone();
                        }
                        ensure!(
                            &trace == recorded_trace,
                            "Replay terminated without matching the complete recorded trace and result"
                        );
                        return Ok(trace);
                    }
                    if err
                        .downcast_ref::<ctb_formats_dcstring::WasmStartSectionTrap>()
                        .is_some()
                    {
                        let WasmTraceEvent::NonResourceTrap {
                            after_logical_steps,
                            ..
                        } = expected
                        else {
                            bail!(
                                "Replay construction start trap mismatch: expected {expected:?}, got {err:#}"
                            );
                        };
                        ensure!(
                            *after_logical_steps == 0,
                            "Replay construction start trap logical step mismatch: expected {after_logical_steps}, got 0"
                        );
                        let ev = WasmTraceEvent::NonResourceTrap {
                            after_logical_steps: 0,
                            trap_message: format!("{err:#}"),
                        };
                        pre_bytes = pre_bytes
                            .saturating_add(ev.estimated_trace_bytes());
                        pre_events.push(ev);
                        let mut trace = self.build_trace(
                            None,
                            initial_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames: Vec::new(),
                                broke_out_early: true,
                            },
                            pre_events,
                            pre_bytes,
                            None,
                        );
                        if trace.supervisor_revision_node_id.is_none()
                            && recorded_trace
                                .supervisor_revision_node_id
                                .is_some()
                        {
                            trace.supervisor_revision_node_id =
                                recorded_trace.supervisor_revision_node_id;
                            trace.dependency_snapshot_node_ids = recorded_trace
                                .dependency_snapshot_node_ids
                                .clone();
                        }
                        ensure!(
                            &trace == recorded_trace,
                            "Replay terminated without matching the complete recorded trace and result"
                        );
                        return Ok(trace);
                    }
                    return Err(err);
                }
            }
        };
        self.replay_wasm_session_trace_with_prefix(
            &mut session,
            input,
            recorded_trace,
            pre_events,
            pre_bytes,
            event_cursor,
            Some(initial_limits),
        )
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
            PermissionDecision::Deny
        );
        assert_eq!(
            valid_ctx.request_permission(&Privilege::Network),
            PermissionDecision::Deny
        );
        let allowed_ctx = valid_ctx
            .with_privilege_decision(Privilege::FileIo, PermissionDecision::Allow);
        assert_eq!(allowed_ctx.request_permission(&Privilege::FileIo), PermissionDecision::Allow);

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

    #[crate::ctb_test]
    fn test_execute_wasm_session_yield_capability_and_breakout() -> Result<()> {
        use ctb_formats_dcstring::{DcChar, DcMixedEncode};

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());

        // Build a stateful Dc-WASM routine using mem[64] as phase counter:
        // Phase 0: copies input to output and returns 1 (YieldFrame), sets mem[64] = 1
        // Phase 1: copies input ("FileIo") to output and returns 2 (RequestCapability), sets mem[64] = 2
        // Phase 2: copies capability response (from input mailbox) to output and returns 0 (Complete)
        let mut routine_doc = DcMstring::new();
        routine_doc.push_char(DcChar::from_short(260));
        routine_doc.push_char(DcChar::from_short(262));
        routine_doc.push_char(DcChar::from_short(280));
        routine_doc.push_char(i32_dc);
        routine_doc.push_char(DcChar::from_short(263));

        // Always echo current input mailbox (0..8) to output mailbox (8..16)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        // Check mem[64] == 0 (`i32.eqz` = Dc 716)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(716)); // i32.eqz
        routine_doc.push_char(DcChar::from_short(651)); // if (result i32)
        routine_doc.push_char(i32_dc);
        // mem[64] = 1; return 1 (YieldFrame)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(652)); // else
        // Check mem[64] == 1 (`i32.eq` = Dc 717)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(717)); // i32.eq
        routine_doc.push_char(DcChar::from_short(651)); // if (result i32)
        routine_doc.push_char(i32_dc);
        // mem[64] = 2; return 2 (RequestCapability)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(652)); // else
        // return 0 (Complete)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(658)); // end inner if
        routine_doc.push_char(DcChar::from_short(658)); // end outer if
        routine_doc.push_char(DcChar::from_short(261));

        let mut input = DcMstring::new();
        input.push_str("FileIo");

        // 1. Explicit FileIo grant: response is `Dc 517 "Boolean true"`
        let mut session_ok = DcWasmSession::from_dc_wasm(&routine_doc, &input, None)?;
        let mut valid_ctx = ExecutionContext::new(false)
            .with_privilege_decision(Privilege::FileIo, PermissionDecision::Allow);
        let res_ok = valid_ctx.execute_wasm_session_default(&mut session_ok, &input)?;
        assert!(!res_ok.broke_out_early);
        assert_eq!(res_ok.frames.len(), 1);
        assert_eq!(res_ok.frames[0], input);
        let mut expected_allow = DcMstring::new();
        expected_allow.push_char(dc::DC_BOOLEAN_TRUE);
        assert_eq!(res_ok.output, expected_allow);

        // 2. Document with parse errors (`has_errors = true`): capability steers to `AutoMock` -> `Dc 316 "Optional value absent"`
        let mut session_err = DcWasmSession::from_dc_wasm(&routine_doc, &input, None)?;
        let mut error_ctx = ExecutionContext::new(true);
        let res_err = error_ctx.execute_wasm_session_default(&mut session_err, &input)?;
        assert!(!res_err.broke_out_early);
        assert_eq!(res_err.frames.len(), 1);
        let mut expected_mock = DcMstring::new();
        expected_mock.push_char(dc::DC_OPTIONAL_ABSENT);
        assert_eq!(res_err.output, expected_mock);

        // 3. Budget exhaustion (`max_compute_steps = 1`): yields 1 frame, then breaks out early on step 2
        let mut session_budget = DcWasmSession::from_dc_wasm(&routine_doc, &input, None)?;
        let mut budget_ctx = ExecutionContext::new(false);
        budget_ctx.max_compute_steps = 1;
        let res_budget = budget_ctx.execute_wasm_session_default(&mut session_budget, &input)?;
        assert!(res_budget.broke_out_early);
        assert_eq!(res_budget.frames.len(), 1);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_tight_wasm_loop_triggers_excessive_compute_and_records_resource_event()
    -> Result<()> {
        use ctb_formats_dcstring::{
            DcChar, DcMixedEncode, DcWasmResourceLimits,
        };

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());
        let mut loop_doc = DcMstring::new();
        loop_doc.push_char(DcChar::from_short(260));
        loop_doc.push_char(DcChar::from_short(262));
        loop_doc.push_char(DcChar::from_short(280));
        loop_doc.push_char(i32_dc);
        loop_doc.push_char(DcChar::from_short(263));
        loop_doc.push_char(DcChar::from_short(650)); // WASM loop
        loop_doc.push_char(DcChar::from_short(316)); // empty blocktype (Dc 316 -> 0x40)
        loop_doc.push_char(DcChar::from_short(659)); // WASM br 0
        u32::encode_dc_mixed(&0, &mut loop_doc)?;
        loop_doc.push_char(DcChar::from_short(658)); // WASM end
        loop_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut loop_doc)?;
        loop_doc.push_char(DcChar::from_short(261));

        let input = DcMstring::new();
        let limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(250);

        // 1. Default policy (`BreakOutResumeNext` on `ExcessiveCompute`) breaks out cleanly
        //    and records `WasmTraceEvent::ResourceInterruption` with `ComputeQuotaExhausted`.
        let mut session = DcWasmSession::from_dc_wasm(&loop_doc, &input, None)?
            .with_resource_limits(limits);
        let mut ctx = ExecutionContext::new(false);
        let trace = ctx.execute_wasm_session_default_with_trace(&mut session, &input)?;
        assert!(trace.result.broke_out_early);
        assert_eq!(
            trace.events,
            vec![WasmTraceEvent::ResourceInterruption {
                after_logical_steps: 0,
                interruption: Some(
                    WasmResourceInterruption::ComputeQuotaExhausted {
                        limit: 250
                    }
                ),
                decision: PermissionDecision::BreakOutResumeNext,
            }]
        );

        // 2. Host policy overriding `ExcessiveCompute` to `Deny` preserves the denied
        //    `ResourceInterruption` in `execute_wasm_session_default_with_trace` while
        //    returning an error from `execute_wasm_session_default` / `ensure_not_denied`.
        let mut deny_session =
            DcWasmSession::from_dc_wasm(&loop_doc, &input, None)?
                .with_resource_limits(limits);
        let mut deny_ctx = ExecutionContext::new(false).with_privilege_decision(
            Privilege::ExcessiveCompute,
            PermissionDecision::Deny,
        );
        let denied_trace = deny_ctx
            .execute_wasm_session_default_with_trace(&mut deny_session, &input)?;
        assert!(denied_trace.ensure_not_denied().is_err());
        assert!(
            deny_ctx
                .execute_wasm_session_default(&mut deny_session, &input)
                .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_historical_graph_pin_never_grants_or_restores_capabilities()
    -> Result<()> {
        use ctb_formats_dcstring::{DcChar, DcMixedEncode};

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());
        // Routine that requests capability on step 0 (mem[64] == 0 -> sets mem[64] = 1, returns 2)
        // and echoes the host's capability response on step 1 (returns 0).
        let mut cap_doc = DcMstring::new();
        cap_doc.push_char(DcChar::from_short(260));
        cap_doc.push_char(DcChar::from_short(262));
        cap_doc.push_char(DcChar::from_short(280));
        cap_doc.push_char(i32_dc);
        cap_doc.push_char(DcChar::from_short(263));
        // Echo input mailbox (0..8) -> output mailbox (8..16)
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;

        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;

        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(716)); // i32.eqz
        cap_doc.push_char(DcChar::from_short(651)); // if (result i32)
        cap_doc.push_char(i32_dc);
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_doc)?;
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut cap_doc)?; // RequestCapability
        cap_doc.push_char(DcChar::from_short(652)); // else
        cap_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut cap_doc)?; // Complete
        cap_doc.push_char(DcChar::from_short(658));
        cap_doc.push_char(DcChar::from_short(261));

        let mut input = DcMstring::new();
        input.push_str("Network");

        let mut expected_deny = DcMstring::new();
        expected_deny.push_char(dc::DC_BOOLEAN_FALSE);

        // Even when a session is pinned to an early historical Dc 641 watermark, a host policy
        // denying `Privilege::Network` strictly denies the request.
        for pin in [None, Some(8_589_934_592_u128), Some(8_589_940_000_u128)] {
            let mut session =
                DcWasmSession::from_dc_wasm(&cap_doc, &input, pin)?;
            let mut host_ctx = ExecutionContext::new(false)
                .with_privilege_decision(
                    Privilege::Network,
                    PermissionDecision::Deny,
                );
            let res =
                host_ctx.execute_wasm_session_default(&mut session, &input)?;
            assert_eq!(res.output, expected_deny);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_capability_trace_replay_compares_requests_and_ignores_compiler_instruction_counts()
    -> Result<()> {
        use ctb_formats_dcstring::{
            DcChar, DcMixedEncode, DcWasmResourceLimits,
        };

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());
        let build_cap_routine = |nop_padding: usize| -> Result<DcMstring> {
            let mut doc = DcMstring::new();
            doc.push_char(DcChar::from_short(260));
            doc.push_char(DcChar::from_short(262));
            doc.push_char(DcChar::from_short(280));
            doc.push_char(i32_dc);
            doc.push_char(DcChar::from_short(263));
            for _ in 0..nop_padding {
                doc.push_char(DcChar::from_short(648)); // WASM nop (0x01)
            }
            // Echo input mailbox (0..8) -> output mailbox (8..16)
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&8, &mut doc)?;
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&0, &mut doc)?;
            doc.push_char(DcChar::from_short(687));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;
            doc.push_char(DcChar::from_short(701));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;

            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&12, &mut doc)?;
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&4, &mut doc)?;
            doc.push_char(DcChar::from_short(687));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;
            doc.push_char(DcChar::from_short(701));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;

            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&64, &mut doc)?;
            doc.push_char(DcChar::from_short(687));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;
            doc.push_char(DcChar::from_short(716)); // i32.eqz
            doc.push_char(DcChar::from_short(651)); // if (result i32)
            doc.push_char(i32_dc);
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&64, &mut doc)?;
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&1, &mut doc)?;
            doc.push_char(DcChar::from_short(701));
            u32::encode_dc_mixed(&0, &mut doc)?;
            u32::encode_dc_mixed(&2, &mut doc)?;
            u32::encode_dc_mixed(&0, &mut doc)?;
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&2, &mut doc)?; // RequestCapability
            doc.push_char(DcChar::from_short(652)); // else
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&0, &mut doc)?; // Complete
            doc.push_char(DcChar::from_short(658));
            doc.push_char(DcChar::from_short(261));
            Ok(doc)
        };

        let routine_v1 = build_cap_routine(0)?;
        let routine_v2_extra_instrs = build_cap_routine(75)?;

        let mut file_io_input = DcMstring::new();
        file_io_input.push_str("FileIo");

        // Record trace under `Allow` policy
        let mut rec_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        let mut rec_ctx = ExecutionContext::new(false)
            .with_privilege_decision(Privilege::FileIo, PermissionDecision::Allow);
        let recorded_trace = rec_ctx
            .execute_wasm_session_default_with_trace(
                &mut rec_session,
                &file_io_input,
            )?;
        assert!(!recorded_trace.result.broke_out_early);
        assert_eq!(recorded_trace.events.len(), 2);

        // 1. Replay against a recompiled module with different physical instruction counts
        //    succeeds and reproduces the exact logical trace and recorded response, even if
        //    the current `ExecutionContext` has `Deny` configured for live runs.
        let mut replay_session = DcWasmSession::from_dc_wasm(
            &routine_v2_extra_instrs,
            &file_io_input,
            None,
        )?;
        let mut replay_ctx = ExecutionContext::new(false)
            .with_privilege_decision(Privilege::FileIo, PermissionDecision::Deny);
        let replayed = replay_ctx.replay_wasm_session_trace(
            &mut replay_session,
            &file_io_input,
            &recorded_trace,
        )?;
        assert_eq!(replayed, recorded_trace);
        let mut trailing_trace = recorded_trace.clone();
        trailing_trace.events.push(recorded_trace.events.last().unwrap().clone());
        let mut trailing_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        assert!(replay_ctx.replay_wasm_session_trace(
            &mut trailing_session, &file_io_input, &trailing_trace,
        ).is_err());
        let mut wrong_result_trace = recorded_trace.clone();
        wrong_result_trace.result.broke_out_early = true;
        let mut wrong_result_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        assert!(replay_ctx.replay_wasm_session_trace(
            &mut wrong_result_session, &file_io_input, &wrong_result_trace,
        ).is_err());
        assert_ne!(
            rec_session.total_instructions_executed(),
            replay_session.total_instructions_executed()
        );

        // 2. Replay detects a changed capability request (`Network` instead of `FileIo`) and errors.
        let mut net_input = DcMstring::new();
        net_input.push_str("Network");
        let mut bad_replay_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &net_input, None)?;
        let mismatch_err = replay_ctx
            .replay_wasm_session_trace(
                &mut bad_replay_session,
                &net_input,
                &recorded_trace,
            )
            .expect_err("Divergent capability request during replay must fail");
        assert!(
            mismatch_err
                .to_string()
                .contains("Replay capability request mismatch"),
            "Unexpected error: {mismatch_err}"
        );

        // 3. Record a trace interrupted by compute_quota exhaustion and verify replay actually
        //    executes `session.step()` under the recorded configuration and triggers the exact
        //    recorded `WasmResourceInterruption` at `last_interrupted_op_ordinal`.
        let low_compute_quota =
            DcWasmResourceLimits::default().with_max_instructions_per_step(5);
        let mut interrupted_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?
                .with_resource_limits(low_compute_quota);
        let mut int_ctx = ExecutionContext::new(false);
        let interrupted_trace = int_ctx
            .execute_wasm_session_default_with_trace(
                &mut interrupted_session,
                &file_io_input,
            )?;
        assert!(interrupted_trace.result.broke_out_early);
        assert_eq!(interrupted_trace.last_interrupted_op_ordinal, Some(6));
        assert_eq!(
            interrupted_trace.last_interrupted_op_required_cost,
            Some(1)
        );

        let mut fast_replay_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        let replayed_int = replay_ctx.replay_wasm_session_trace(
            &mut fast_replay_session,
            &file_io_input,
            &interrupted_trace,
        )?;
        assert_eq!(replayed_int, interrupted_trace);
        assert_eq!(fast_replay_session.total_instructions_executed(), 5);
        assert_eq!(
            fast_replay_session.last_interrupted_op_ordinal(),
            Some(6)
        );
        assert_eq!(
            fast_replay_session.last_interrupted_op_required_cost(),
            Some(1)
        );

        // Tampering with `last_interrupted_op_ordinal` in the recorded trace must fail replay verification.
        let mut forged_ordinal_trace = interrupted_trace.clone();
        forged_ordinal_trace.last_interrupted_op_ordinal = Some(99);
        let mut forged_replay_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        assert!(
            replay_ctx
                .replay_wasm_session_trace(
                    &mut forged_replay_session,
                    &file_io_input,
                    &forged_ordinal_trace,
                )
                .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_denied_and_construction_resource_interruptions_preserve_and_replay_traces()
    -> Result<()> {
        use ctb_formats_dcstring::{
            DcChar, DcMixedEncode, DcWasmResourceLimits,
            WasmResourceInterruption,
        };

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());

        // Routine that:
        // - Step 0 (mem[64] == 0): sets mem[64] = 1, echoes input to output, returns 1 (YieldFrame)
        // - Step 1 (mem[64] == 1): sets mem[64] = 2, echoes input to output, returns 2 (RequestCapability)
        // - Step 2 (mem[64] == 2): enters a tight loop that exhausts instruction compute_quota
        let mut multi_doc = DcMstring::new();
        multi_doc.push_char(DcChar::from_short(260));
        multi_doc.push_char(DcChar::from_short(262));
        multi_doc.push_char(DcChar::from_short(280));
        multi_doc.push_char(i32_dc);
        multi_doc.push_char(DcChar::from_short(263));
        // Echo input mailbox (0..8) -> output mailbox (8..16)
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;

        // Read phase = mem[64]
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(716)); // i32.eqz (phase == 0)
        multi_doc.push_char(DcChar::from_short(651)); // if (result i32)
        multi_doc.push_char(i32_dc);
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut multi_doc)?; // return 1 (YieldFrame)
        multi_doc.push_char(DcChar::from_short(652)); // else
        // Check if mem[64] == 1
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(717)); // i32.eq
        multi_doc.push_char(DcChar::from_short(651)); // if (result i32)
        multi_doc.push_char(i32_dc);
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        u32::encode_dc_mixed(&2, &mut multi_doc)?;
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut multi_doc)?; // return 2 (RequestCapability)
        multi_doc.push_char(DcChar::from_short(652)); // else (phase == 2: tight loop)
        multi_doc.push_char(DcChar::from_short(650)); // loop
        multi_doc.push_char(DcChar::from_short(316)); // empty blocktype
        multi_doc.push_char(DcChar::from_short(659)); // br 0
        u32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(658)); // end loop
        multi_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut multi_doc)?;
        multi_doc.push_char(DcChar::from_short(658)); // end inner if
        multi_doc.push_char(DcChar::from_short(658)); // end outer if
        multi_doc.push_char(DcChar::from_short(261));

        let mut input = DcMstring::new();
        input.push_str("FileIo");

        let limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(200);
        let mut session = DcWasmSession::from_dc_wasm(&multi_doc, &input, None)?
            .with_resource_limits(limits);
        let mut deny_ctx = ExecutionContext::new(false)
            .with_privilege_decision(Privilege::FileIo, PermissionDecision::Allow)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );

        // 1. `execute_wasm_session_default_with_trace` preserves the yielded frame (step 0),
        //    the capability interaction (step 1), and the denied ResourceInterruption (step 2).
        let denied_trace = deny_ctx
            .execute_wasm_session_default_with_trace(&mut session, &input)?;
        assert!(denied_trace.result.broke_out_early);
        assert_eq!(denied_trace.result.frames, vec![input.clone()]);
        assert_eq!(denied_trace.events.len(), 3);
        assert_eq!(
            denied_trace.denied_resource_interruption(),
            Some((
                2,
                Some(&WasmResourceInterruption::ComputeQuotaExhausted {
                    limit: 200
                }),
                PermissionDecision::Deny,
            ))
        );
        assert!(denied_trace.ensure_not_denied().is_err());

        // 2. Replaying `denied_trace` reproduces the exact finite prefix and denied interruption.
        let mut replay_session =
            DcWasmSession::from_dc_wasm(&multi_doc, &input, None)?;
        let mut replay_ctx = ExecutionContext::new(false);
        let replayed = replay_ctx.replay_wasm_session_trace(
            &mut replay_session,
            &input,
            &denied_trace,
        )?;
        assert_eq!(replayed, denied_trace);

        // 3. Construction-time resource interruption (looping `start` section) is captured in
        //    `execute_wasm_bytes_with_trace` and replayed via `replay_wasm_bytes_trace`.
        let looping_start = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            0x08, 0x01, 0x00,
            0x0a, 0x09, 0x01, 0x07, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b,
        ];
        let start_trace = deny_ctx.execute_wasm_bytes_with_trace(
            &looping_start,
            &input,
            None,
            limits,
            |_p, d, _r| Ok(ExecutionContext::default_capability_response(d)),
        )?;
        assert_eq!(
            start_trace.denied_resource_interruption(),
            Some((
                0,
                Some(&WasmResourceInterruption::ComputeQuotaExhausted {
                    limit: 200
                }),
                PermissionDecision::Deny,
            ))
        );
        let replayed_start = replay_ctx.replay_wasm_bytes_trace(
            &looping_start,
            &input,
            None,
            limits,
            &start_trace,
        )?;
        assert_eq!(replayed_start, start_trace);

        // 4. Granting additional compute quota (`PermissionDecision::Allow` on `Privilege::ExcessiveCompute`
        //    with `with_compute_quota_grant`) resumes a paused multi-step module without repeating
        //    earlier yielded frames, and replays identically.
        let mut bounded_doc = DcMstring::new();
        bounded_doc.push_char(DcChar::from_short(260));
        bounded_doc.push_char(DcChar::from_short(262));
        bounded_doc.push_char(DcChar::from_short(280));
        bounded_doc.push_char(i32_dc);
        bounded_doc.push_char(DcChar::from_short(263));
        // Echo input mailbox (0..8) to output mailbox (8..16)
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        // Check phase at mem[64]: if 0 -> set mem[64]=1 and return 1 (YieldFrame); else -> return 0 (Complete)
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(716)); // i32.eqz
        bounded_doc.push_char(DcChar::from_short(651)); // if (result i32)
        bounded_doc.push_char(i32_dc);
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        u32::encode_dc_mixed(&2, &mut bounded_doc)?;
        u32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(652)); // else
        bounded_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut bounded_doc)?;
        bounded_doc.push_char(DcChar::from_short(658)); // end if
        bounded_doc.push_char(DcChar::from_short(261));

        // Step 0 consumes ~21 instructions; step 1 consumes ~16 instructions (~37 total).
        // Start with quota 25 so step 0 succeeds (`YieldFrame`), step 1 triggers `ComputeQuotaExhausted`,
        // and granting +50 via `PermissionDecision::Allow` resumes step 1 to completion.
        let tight_limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(25);
        let mut resumable_session =
            DcWasmSession::from_dc_wasm(&bounded_doc, &input, None)?
                .with_resource_limits(tight_limits);
        let mut allow_ctx = ExecutionContext::new(false)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Allow,
            )
            .with_compute_quota_grant(50);
        let resumed_trace = allow_ctx
            .execute_wasm_session_default_with_trace(&mut resumable_session, &input)?;
        assert!(!resumed_trace.result.broke_out_early);
        assert_eq!(resumed_trace.result.frames, vec![input.clone()]);
        assert_eq!(resumed_trace.result.output, input);
        assert!(resumed_trace.events.iter().any(|e| matches!(
            e,
            WasmTraceEvent::ResourceInterruption {
                after_logical_steps: 1,
                interruption: Some(WasmResourceInterruption::ComputeQuotaExhausted { limit: 25 }),
                decision: PermissionDecision::Allow,
            }
        )));

        let mut replay_resumed_session =
            DcWasmSession::from_dc_wasm(&bounded_doc, &input, None)?
                .with_resource_limits(tight_limits);
        let mut replay_allow_ctx =
            ExecutionContext::new(false).with_compute_quota_grant(50);
        let replayed_resumed = replay_allow_ctx.replay_wasm_session_trace(
            &mut replay_resumed_session,
            &input,
            &resumed_trace,
        )?;
        assert_eq!(replayed_resumed, resumed_trace);

        // 5. Non-resource trap inside module `start` (`unreachable` = 0x00) is captured as
        //    `WasmTraceEvent::NonResourceTrap` at `after_logical_steps: 0` in `execute_wasm_bytes_with_trace`
        //    and verified during `replay_wasm_bytes_trace`.
        let trapping_start = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            0x08, 0x01, 0x00,
            0x0a, 0x05, 0x01, 0x03, 0x00, 0x00, 0x0b,
        ];
        let start_trap_trace = deny_ctx.execute_wasm_bytes_with_trace(
            &trapping_start,
            &input,
            None,
            limits,
            |_p, d, _r| Ok(ExecutionContext::default_capability_response(d)),
        )?;
        assert!(start_trap_trace.result.broke_out_early);
        assert!(matches!(
            start_trap_trace.non_resource_trap(),
            Some((0, msg)) if msg.contains("start section")
        ));
        assert!(start_trap_trace.ensure_not_denied().is_err());
        let replayed_start_trap = replay_ctx.replay_wasm_bytes_trace(
            &trapping_start,
            &input,
            None,
            limits,
            &start_trap_trace,
        )?;
        assert_eq!(replayed_start_trap, start_trap_trace);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_supervisor_admission_trace_metadata_non_resource_trap_and_budgets()
    -> Result<()> {
        use ctb_formats_dcstring::{DcMixedEncode, LedgerNode};

        let sup_dc = DcChar::from_short(999);
        let sh_sup = DcShorthand::from_global_id(sup_dc.to_u128());
        let base_id: u128 = 8_589_934_592;
        let sup_def_id = base_id;
        let sup_has_a_id = base_id + 1;
        let sup_has_val_id = base_id + 2;
        let prov_has_a_id = base_id + 3;
        let prov_has_val_id = base_id + 4;
        let user_cfg_node_id = base_id + 5;
        let user_limits = DcWasmResourceLimits::default()
            .with_max_instructions_per_step(5_000)
            .with_max_capability_payload_bytes(8)
            .with_max_trace_bytes(512);
        let bundled_nodes = vec![
            LedgerNode {
                id: sup_def_id,
                tokens: vec![
                    DcShorthand::Short(1927),
                    DcShorthand::Short(1928),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: sup_has_a_id,
                tokens: vec![
                    DcShorthand::Short(584),
                    sh_sup,
                    DcShorthand::Short(611),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: sup_has_val_id,
                tokens: vec![
                    DcShorthand::Short(610),
                    DcShorthand::Long(sup_has_a_id),
                    DcShorthand::Long(sup_def_id),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: prov_has_a_id,
                tokens: vec![
                    DcShorthand::Short(638),
                    DcShorthand::Short(584),
                    DcShorthand::Short(639),
                    DcShorthand::Short(635),
                    DcShorthand::Short(310),
                    DcShorthand::Short(640),
                    DcShorthand::Long(sup_def_id),
                    DcShorthand::Short(312),
                    DcShorthand::Long(sup_has_a_id),
                    DcShorthand::Short(312),
                    DcShorthand::Long(sup_has_val_id),
                    DcShorthand::Short(311),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: prov_has_val_id,
                tokens: vec![
                    DcShorthand::Short(610),
                    DcShorthand::Long(prov_has_a_id),
                    DcShorthand::Short(636),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: user_cfg_node_id,
                tokens:
                    SystemSupervisorAdmission::encode_graph_quota_config_tokens(
                        &user_limits,
                    ),
                comment: String::new(),
            },
        ];
        let seed_graph = SeedGraphIndex::from_bundled_nodes(bundled_nodes);

        // 1. Valid pinned ordinary documents and ordinary documents referencing `sup_dc`
        //    are allowed as ordinary documents (neither grants supervisor authority), while
        //    actual attempts to claim supervisor privilege (`Dc 527` = `Dc 517`, `Dc 635` = `Dc 636`,
        //    or passing pinned/multi-token documents to `admit_from_bootstrap_tokens`) are denied.
        SystemSupervisorAdmission::verify_ordinary_document_cannot_elevate(
            &[
                DcShorthand::from_global_id(
                    dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID
                        .to_u128(),
                ),
                DcShorthand::Long(prov_has_val_id),
                sh_sup,
            ],
            sup_dc,
        )?;
        SystemSupervisorAdmission::verify_ordinary_document_cannot_elevate(
            &[sh_sup, DcShorthand::Short(517)],
            sup_dc,
        )?;
        assert!(
            SystemSupervisorAdmission::verify_ordinary_document_cannot_elevate(
                &[DcShorthand::Short(527), DcShorthand::Short(517)],
                sup_dc,
            )
            .is_err()
        );
        assert!(
            SystemSupervisorAdmission::verify_ordinary_document_cannot_elevate(
                &[DcShorthand::Short(635), DcShorthand::Short(636)],
                sup_dc,
            )
            .is_err()
        );
        assert!(
            SystemSupervisorAdmission::admit_from_bootstrap_tokens(
                &seed_graph,
                &[
                    DcShorthand::from_global_id(
                        dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID
                            .to_u128()
                    ),
                    DcShorthand::Long(prov_has_val_id),
                    sh_sup,
                ],
                user_limits,
                DcWasmWatchdogLimits::default(),
            )
            .is_err()
        );

        // 2. Trusted System supervisor admission resolves graph-backed user quota configuration,
        //    compiles local limits clamped by watchdog bounds, and records supervisor revision +
        //    dependency snapshot + canonical config in traces.
        let watchdog = DcWasmWatchdogLimits {
            hard_max_instructions_per_step: 1_000,
            ..DcWasmWatchdogLimits::default()
        };
        let adm_from_node =
            SystemSupervisorAdmission::admit_from_graph_config_node(
                &seed_graph,
                &[sh_sup],
                user_cfg_node_id,
                watchdog,
            )?;
        let mut ctx = ExecutionContext::new(false);
        let adm = ctx
            .admit_system_supervisor(
                &seed_graph,
                &[sh_sup],
                user_limits,
                watchdog,
            )?
            .clone();
        assert_eq!(adm, adm_from_node);
        assert_eq!(adm.supervisor_revision_node_id, sup_def_id);
        assert_eq!(adm.dependency_snapshot_node_ids, vec![sup_def_id]);
        assert_eq!(adm.compiled_limits.max_instructions_per_step, 1_000);

        // Build a module that yields a frame on step 0, then traps with `unreachable` (Dc 648) on step 1.
        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());
        let mut trap_after_frame_doc = DcMstring::new();
        trap_after_frame_doc.push_char(DcChar::from_short(260));
        trap_after_frame_doc.push_char(DcChar::from_short(262));
        trap_after_frame_doc.push_char(DcChar::from_short(280));
        trap_after_frame_doc.push_char(i32_dc);
        trap_after_frame_doc.push_char(DcChar::from_short(263));
        // Copy input (0..8) to output (8..16)
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        // Check mem[64]: if 0 -> set mem[64]=1 and return 1 (YieldFrame); else -> unreachable (Dc 648)
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(716)); // i32.eqz
        trap_after_frame_doc.push_char(DcChar::from_short(651)); // if (result i32)
        trap_after_frame_doc.push_char(i32_dc);
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&2, &mut trap_after_frame_doc)?;
        u32::encode_dc_mixed(&0, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut trap_after_frame_doc)?;
        trap_after_frame_doc.push_char(DcChar::from_short(652)); // else
        trap_after_frame_doc.push_char(DcChar::from_short(647)); // unreachable (Dc 647 = 0x00)
        trap_after_frame_doc.push_char(DcChar::from_short(658)); // end if
        trap_after_frame_doc.push_char(DcChar::from_short(261));

        let mut input = DcMstring::new();
        input.push_char(dc::DC_BOOLEAN_TRUE);

        // 3. NonResourceTrap preserves the yielded frame at step 0 in `WasmExecutionTrace`,
        //    fails `ensure_not_denied()`, and replays deterministically.
        let mut trap_session =
            DcWasmSession::from_dc_wasm(&trap_after_frame_doc, &input, None)?;
        let trap_trace = ctx
            .execute_wasm_session_default_with_trace(&mut trap_session, &input)?;
        assert_eq!(trap_trace.supervisor_revision_node_id, Some(sup_def_id));
        assert_eq!(trap_trace.dependency_snapshot_node_ids, vec![sup_def_id]);
        assert_eq!(
            trap_trace.accounting_version,
            WASM_RESOURCE_ACCOUNTING_VERSION
        );
        assert_eq!(
            trap_trace.effective_config_bytes,
            adm.effective_config_bytes
        );
        assert_eq!(trap_trace.result.frames, vec![input.clone()]);
        assert!(trap_trace.non_resource_trap().is_some());
        assert!(trap_trace.ensure_not_denied().is_err());

        let mut replay_trap_session =
            DcWasmSession::from_dc_wasm(&trap_after_frame_doc, &input, None)?;
        let mut replay_ctx =
            ExecutionContext::new(false).with_system_supervisor_admission(adm);
        let replayed_trap = replay_ctx.replay_wasm_session_trace(
            &mut replay_trap_session,
            &input,
            &trap_trace,
        )?;
        assert_eq!(replayed_trap, trap_trace);

        // 4. TraceBufferBudgetExceeded triggers when `max_trace_bytes` is smaller than the first YieldFrame event.
        let tiny_trace_limits =
            DcWasmResourceLimits::default().with_max_trace_bytes(8);
        let mut tiny_trace_session =
            DcWasmSession::from_dc_wasm(&trap_after_frame_doc, &input, None)?
                .with_resource_limits(tiny_trace_limits);
        let mut tiny_ctx = ExecutionContext::new(false).with_privilege_decision(
            Privilege::ExcessiveCompute,
            PermissionDecision::Deny,
        );
        let tiny_trace = tiny_ctx.execute_wasm_session_default_with_trace(
            &mut tiny_trace_session,
            &input,
        )?;
        assert!(matches!(
            tiny_trace.events.last(),
            Some(WasmTraceEvent::ResourceInterruption {
                after_logical_steps: 0,
                interruption: Some(
                    WasmResourceInterruption::TraceBufferBudgetExceeded {
                        max_trace_bytes: 8,
                        ..
                    }
                ),
                decision: PermissionDecision::Deny,
            })
        ));

        // 5. Ordinary pinned documents execute via `execute_ordinary_dc_wasm_default_with_trace`
        //    without supervisor privilege, while documents attempting inline elevation are denied.
        let head_pin = ctb_formats_dcstring::SeedGraphIndex::shared()?
            .system_block_most_recent_node_id()
            .unwrap();
        let mut pinned_ordinary_doc = DcMstring::new();
        pinned_ordinary_doc.push_char(
            dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
        );
        pinned_ordinary_doc.push_char(DcChar::from_u128(head_pin));
        for ch in trap_after_frame_doc.to_dc_string()?.chars() {
            pinned_ordinary_doc.push_char(ch);
        }
        let mut unprivileged_ctx = ExecutionContext::new(false);
        let ordinary_trace = unprivileged_ctx
            .execute_ordinary_dc_wasm_default_with_trace(
                &pinned_ordinary_doc,
                &input,
                Some(head_pin),
                sup_dc,
            )?;
        assert_eq!(ordinary_trace.supervisor_revision_node_id, None);
        assert_eq!(ordinary_trace.result.frames, vec![input.clone()]);

        let mut elevating_ordinary_doc = DcMstring::new();
        elevating_ordinary_doc.push_char(DcChar::from_short(527));
        elevating_ordinary_doc.push_char(dc::DC_BOOLEAN_TRUE);
        assert!(
            unprivileged_ctx
                .execute_ordinary_dc_wasm_default_with_trace(
                    &elevating_ordinary_doc,
                    &input,
                    None,
                    sup_dc,
                )
                .is_err()
        );
        Ok(())
    }
}
