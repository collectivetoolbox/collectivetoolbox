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
    DcMstr, DcMstring, DcString, DcWasmSession, WasmResourceInterruption,
    WasmStepOutcome, dc,
};
use std::collections::HashMap;

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
    /// External resource interruption raised by physical VM budgets (`Instructioncompute_quotaExhausted`,
    /// `MemoryLimitExceeded`, `StepOutputSliceTooLarge`, `SessionOutputBudgetExceeded`) or
    /// host step budget exhaustion.
    ResourceInterruption {
        /// Number of completed logical steps before the interruption occurred.
        after_logical_steps: usize,
        /// Physical VM resource interruption (or `None` if triggered by the outer host step counter).
        interruption: Option<WasmResourceInterruption>,
        /// Host decision for `Privilege::ExcessiveCompute`.
        decision: PermissionDecision,
    },
    /// Completed execution (`WasmStepStatus::Complete`) with the final output document.
    Complete {
        /// 0-based logical step index of completion.
        logical_step: usize,
        /// Final output document.
        output: DcMstring,
    },
}

/// Complete observable execution trace and result of a [`DcWasmSession`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExecutionTrace {
    /// Final execution result (`output`, `frames`, `broke_out_early`).
    pub result: WasmExecutionResult,
    /// Ordered sequence of logical trace events and external resource interruptions.
    pub events: Vec<WasmTraceEvent>,
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

    /// Returns `Err` if the trace terminated due to a denied resource interruption, or `Ok(())`
    /// if the trace completed or broke out via `PermissionDecision::BreakOutResumeNext`.
    ///
    /// # Errors
    /// Returns an error describing the denied resource interruption.
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
        }
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

    fn handle_resource_interruption(
        &self,
        err: anyhow::Error,
        after_logical_steps: usize,
        frames: Vec<DcMstring>,
        mut events: Vec<WasmTraceEvent>,
    ) -> Result<WasmExecutionTrace> {
        if let Some(interruption) =
            err.downcast_ref::<WasmResourceInterruption>().cloned()
        {
            let decision =
                self.request_permission(&Privilege::ExcessiveCompute);
            events.push(WasmTraceEvent::ResourceInterruption {
                after_logical_steps,
                interruption: Some(interruption),
                decision,
            });
            return Ok(WasmExecutionTrace {
                result: WasmExecutionResult {
                    output: DcMstring::new(),
                    frames,
                    broke_out_early: true,
                },
                events,
            });
        }
        Err(err)
    }

    /// Drives a [`DcWasmSession`] over `input` and records the full [`WasmExecutionTrace`]
    /// (distinguishing logical document progress from physical VM resource interruptions).
    ///
    /// Physical VM resource interruptions (including denied interruptions where
    /// `Privilege::ExcessiveCompute` resolves to `PermissionDecision::Deny`) preserve all
    /// accumulated `frames` and `events` in the returned [`WasmExecutionTrace`] so the finite
    /// prefix can be inspected and deterministically replayed.
    ///
    /// # Errors
    /// Returns an error if the WASM module traps with a non-resource error or if
    /// `capability_handler` fails.
    pub fn execute_wasm_session_with_trace<F>(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        mut capability_handler: F,
    ) -> Result<WasmExecutionTrace>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        let mut frames = Vec::new();
        let mut events = Vec::new();
        if let Err(err) = session.provide_input(input) {
            return self.handle_resource_interruption(err, 0, frames, events);
        }

        let mut logical_step = 0usize;
        loop {
            let budget_decision = match self.step() {
                Ok(d) => d,
                Err(_err) => {
                    let decision =
                        self.request_permission(&Privilege::ExcessiveCompute);
                    events.push(WasmTraceEvent::ResourceInterruption {
                        after_logical_steps: logical_step,
                        interruption: None,
                        decision,
                    });
                    return Ok(WasmExecutionTrace {
                        result: WasmExecutionResult {
                            output: DcMstring::new(),
                            frames,
                            broke_out_early: true,
                        },
                        events,
                    });
                }
            };
            if budget_decision == PermissionDecision::BreakOutResumeNext {
                events.push(WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: logical_step,
                    interruption: None,
                    decision: PermissionDecision::BreakOutResumeNext,
                });
                return Ok(WasmExecutionTrace {
                    result: WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                });
            }
            if budget_decision == PermissionDecision::Deny {
                events.push(WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: logical_step,
                    interruption: None,
                    decision: PermissionDecision::Deny,
                });
                return Ok(WasmExecutionTrace {
                    result: WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                });
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
                        let decision = self
                            .request_permission(&Privilege::ExcessiveCompute);
                        events.push(WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: logical_step,
                            interruption: Some(
                                WasmResourceInterruption::ComputeQuotaExhausted {
                                    limit,
                                },
                            ),
                            decision,
                        });
                        if decision == PermissionDecision::Allow {
                            session.increase_compute_quota(
                                self.compute_quota_grant.max(1),
                            );
                            continue;
                        }
                        return Ok(WasmExecutionTrace {
                            result: WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                        });
                    }
                    return self.handle_resource_interruption(
                        err,
                        logical_step,
                        frames,
                        events,
                    );
                }
            };

            match step_outcome {
                WasmStepOutcome::Complete(output) => {
                    events.push(WasmTraceEvent::Complete {
                        logical_step,
                        output: output.clone(),
                    });
                    return Ok(WasmExecutionTrace {
                        result: WasmExecutionResult {
                            output,
                            frames,
                            broke_out_early: false,
                        },
                        events,
                    });
                }
                WasmStepOutcome::YieldFrame(frame) => {
                    events.push(WasmTraceEvent::YieldFrame {
                        logical_step,
                        frame: frame.clone(),
                    });
                    frames.push(frame);
                    logical_step = logical_step.saturating_add(1);
                }
                WasmStepOutcome::RequestCapability(req_dc) => {
                    let privilege = Privilege::from_request_dc(&req_dc);
                    let decision = self.request_permission(&privilege);
                    if decision == PermissionDecision::BreakOutResumeNext {
                        events.push(WasmTraceEvent::CapabilityInteraction {
                            logical_step,
                            privilege,
                            request: req_dc,
                            decision,
                            response: DcMstring::new(),
                        });
                        return Ok(WasmExecutionTrace {
                            result: WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                        });
                    }
                    let response =
                        capability_handler(&privilege, decision, &req_dc)?;
                    events.push(WasmTraceEvent::CapabilityInteraction {
                        logical_step,
                        privilege,
                        request: req_dc,
                        decision,
                        response: response.clone(),
                    });
                    logical_step = logical_step.saturating_add(1);
                    if let Err(err) = session.provide_input(&response) {
                        return self.handle_resource_interruption(
                            err,
                            logical_step,
                            frames,
                            events,
                        );
                    }
                }
            }
        }
    }

    /// Drives a [`DcWasmSession`] with the default capability response encoder and returns
    /// the full [`WasmExecutionTrace`].
    ///
    /// # Errors
    /// Returns an error if the session traps or is denied by capability policy.
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

    /// Replays a [`DcWasmSession`] against a previously recorded [`WasmExecutionTrace`]:
    /// - Compares every capability request (`Privilege` and `DcMstring` request payload) emitted by
    ///   the document at each logical step, then supplies the recorded allow/deny/mock response.
    /// - Delivers recorded external resource interruptions at their recorded logical step boundary
    ///   (`after_logical_steps`) rather than relying on physical compiler instruction counts.
    ///
    /// # Errors
    /// Returns an error if the document's capability requests, yielded frames, or completion
    /// output diverge from `recorded_trace`, or if a recorded resource interruption decision was `Deny`.
    pub fn replay_wasm_session_trace(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        recorded_trace: &WasmExecutionTrace,
    ) -> Result<WasmExecutionTrace> {
        let mut frames = Vec::new();
        let mut events = Vec::new();
        let mut event_idx = 0usize;
        let checked_trace = |trace: WasmExecutionTrace| -> Result<WasmExecutionTrace> {
            ensure!(
                &trace == recorded_trace,
                "Replay terminated without matching the complete recorded trace and result"
            );
            Ok(trace)
        };

        if let Some(WasmTraceEvent::ResourceInterruption {
            after_logical_steps: 0,
            interruption,
            decision,
        }) = recorded_trace.events.first()
            && interruption.is_some()
            && *decision != PermissionDecision::Allow
        {
            events.push(WasmTraceEvent::ResourceInterruption {
                after_logical_steps: 0,
                interruption: interruption.clone(),
                decision: *decision,
            });
            return checked_trace(WasmExecutionTrace {
                result: WasmExecutionResult {
                    output: DcMstring::new(),
                    frames,
                    broke_out_early: true,
                },
                events,
            });
        }

        session.provide_input(input)?;
        let mut logical_step = 0usize;

        loop {
            while let Some(WasmTraceEvent::ResourceInterruption {
                after_logical_steps,
                interruption,
                decision,
            }) = recorded_trace.events.get(event_idx)
                && *after_logical_steps == logical_step
            {
                // Deliver the recorded interruption at its recorded logical step boundary.
                events.push(WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: *after_logical_steps,
                    interruption: interruption.clone(),
                    decision: *decision,
                });
                if *decision == PermissionDecision::Allow {
                    session.increase_compute_quota(
                        self.compute_quota_grant.max(1),
                    );
                    event_idx = event_idx.saturating_add(1);
                    continue;
                }
                return checked_trace(WasmExecutionTrace {
                    result: WasmExecutionResult {
                        output: DcMstring::new(),
                        frames,
                        broke_out_early: true,
                    },
                    events,
                });
            }

            let expected_event =
                recorded_trace.events.get(event_idx).with_context(|| {
                    format!(
                        "Replay exceeded recorded trace length ({}) at logical step {logical_step}",
                        recorded_trace.events.len()
                    )
                })?;

            let step_outcome = session.step().with_context(|| {
                format!(
                    "Replay failed during WASM step at logical step {logical_step}"
                )
            })?;

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
                    events.push(WasmTraceEvent::YieldFrame {
                        logical_step,
                        frame: frame.clone(),
                    });
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
                    events.push(WasmTraceEvent::CapabilityInteraction {
                        logical_step,
                        privilege,
                        request: req_dc,
                        decision: *rec_decision,
                        response: rec_response.clone(),
                    });
                    if *rec_decision == PermissionDecision::BreakOutResumeNext {
                        return checked_trace(WasmExecutionTrace {
                            result: WasmExecutionResult {
                                output: DcMstring::new(),
                                frames,
                                broke_out_early: true,
                            },
                            events,
                        });
                    }
                    logical_step = logical_step.saturating_add(1);
                    event_idx = event_idx.saturating_add(1);
                    session.provide_input(rec_response)?;
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
                    events.push(WasmTraceEvent::Complete {
                        logical_step,
                        output: output.clone(),
                    });
                    return checked_trace(WasmExecutionTrace {
                        result: WasmExecutionResult {
                            output,
                            frames,
                            broke_out_early: false,
                        },
                        events,
                    });
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
    /// interruptions (initial memory/table limits or `start` compute_quota exhaustion) in the returned
    /// [`WasmExecutionTrace`].
    ///
    /// # Errors
    /// Returns an error if the module fails to parse or traps with a non-resource error.
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
        let mut pre_events = Vec::new();
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
                        pre_events.push(WasmTraceEvent::ResourceInterruption {
                            after_logical_steps: 0,
                            interruption: Some(
                                WasmResourceInterruption::ComputeQuotaExhausted {
                                    limit,
                                },
                            ),
                            decision,
                        });
                        if decision == PermissionDecision::Allow {
                            resource_limits.max_instructions_per_step =
                                resource_limits
                                    .max_instructions_per_step
                                    .saturating_add(
                                        self.compute_quota_grant.max(1),
                                    );
                            continue;
                        }
                        return Ok(WasmExecutionTrace {
                            result: WasmExecutionResult {
                                output: DcMstring::new(),
                                frames: Vec::new(),
                                broke_out_early: true,
                            },
                            events: pre_events,
                        });
                    }
                    return self.handle_resource_interruption(
                        err,
                        0,
                        Vec::new(),
                        pre_events,
                    );
                }
            }
        };
        let mut trace = self.execute_wasm_session_with_trace(
            &mut session,
            input,
            capability_handler,
        )?;
        if !pre_events.is_empty() {
            pre_events.extend(trace.events);
            trace.events = pre_events;
        }
        Ok(trace)
    }

    /// Replays compiled `.wasm` module bytes against a recorded [`WasmExecutionTrace`],
    /// including traces that terminated during initial module instantiation or `provide_input`.
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
        if let Some(WasmTraceEvent::ResourceInterruption {
            after_logical_steps: 0,
            interruption,
            decision,
        }) = recorded_trace.events.first()
            && recorded_trace.events.len() == 1
        {
            let trace = WasmExecutionTrace {
                result: WasmExecutionResult {
                    output: DcMstring::new(),
                    frames: Vec::new(),
                    broke_out_early: true,
                },
                events: vec![WasmTraceEvent::ResourceInterruption {
                    after_logical_steps: 0,
                    interruption: interruption.clone(),
                    decision: *decision,
                }],
            };
            ensure!(
                &trace == recorded_trace,
                "Replay terminated without matching the complete recorded trace and result"
            );
            return Ok(trace);
        }
        let mut session = loop {
            match DcWasmSession::from_wasm_bytes_with_limits(
                wasm_bytes,
                input,
                most_recent_node_id,
                resource_limits,
            ) {
                Ok(s) => break s,
                Err(err) => {
                    if matches!(
                        err.downcast_ref::<WasmResourceInterruption>(),
                        Some(WasmResourceInterruption::ComputeQuotaExhausted { .. })
                    ) {
                        resource_limits.max_instructions_per_step =
                            resource_limits
                                .max_instructions_per_step
                                .saturating_add(self.compute_quota_grant.max(1));
                        continue;
                    }
                    return Err(err);
                }
            }
        };
        self.replay_wasm_session_trace(&mut session, input, recorded_trace)
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

        // 3. Record a trace interrupted by compute_quota exhaustion and verify replay delivers the
        //    recorded external resource interruption at the recorded logical boundary even
        //    when the replayed module has plenty of compute_quota.
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

        let mut fast_replay_session =
            DcWasmSession::from_dc_wasm(&routine_v1, &file_io_input, None)?;
        let replayed_int = replay_ctx.replay_wasm_session_trace(
            &mut fast_replay_session,
            &file_io_input,
            &interrupted_trace,
        )?;
        assert_eq!(replayed_int, interrupted_trace);
        assert_eq!(fast_replay_session.total_instructions_executed(), 0);
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
        Ok(())
    }
}
