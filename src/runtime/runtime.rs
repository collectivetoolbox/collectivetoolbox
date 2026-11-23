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

pub mod cli;

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
    DcChar, DcMstr, DcMstring, DcRole, DcShorthand, DcString,
    DcWasmResourceLimits, DcWasmSession, DcWasmWatchdogLimits, SeedGraphIndex,
    SyntaxRuleResolver, WASM_RESOURCE_ACCOUNTING_VERSION,
    WasmResourceInterruption, WasmStepOutcome,
    converters::eval_pure_graph_capability_request, dc,
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
    /// Canonical unpinned 1-Short-Dc bootstrap entry for the standalone Dc document execution
    /// kernel (`Dc 1936` `"Dc document execution kernel"`).
    pub const EXECUTION_KERNEL_ENTRY_DC: DcChar =
        dc::DC_DC_DOCUMENT_EXECUTION_KERNEL;

    /// Canonical unpinned 1-Short-Dc bootstrap entry for the full Collective Toolbox workspace
    /// supervisor (`Dc 1937` `"Collective Toolbox Workspace"`).
    pub const WORKSPACE_SUPERVISOR_ENTRY_DC: DcChar =
        dc::DC_COLLECTIVE_TOOLBOX_WORKSPACE;

    /// Resolves and authenticates the standalone execution kernel (`Dc 1936`
    /// `"Dc document execution kernel"`) from the application-bundled System graph
    /// (`SeedGraphIndex::shared()`), validates `user_limits` through the supervisor Dc-WASM
    /// definition, and clamps emergency host `watchdog` bounds.
    ///
    /// # Errors
    /// Returns an error if loading the bundled System graph fails or `user_limits` is rejected.
    pub fn admit_execution_kernel(
        user_limits: DcWasmResourceLimits,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let seed_graph = SeedGraphIndex::shared()?;
        let entry = DcShorthand::from_global_id(
            Self::EXECUTION_KERNEL_ENTRY_DC.to_u128(),
        );
        Self::admit_from_bootstrap_tokens(
            seed_graph,
            &[entry],
            user_limits,
            watchdog,
        )
    }

    /// Resolves and authenticates the standalone execution kernel (`Dc 1936`
    /// `"Dc document execution kernel"`) from the application-bundled System graph with default
    /// resource and watchdog limits.
    ///
    /// # Errors
    /// Returns an error if loading or authenticating the bundled System supervisor fails.
    pub fn admit_execution_kernel_default() -> Result<Self> {
        Self::admit_execution_kernel(
            DcWasmResourceLimits::default(),
            DcWasmWatchdogLimits::default(),
        )
    }

    /// Resolves and validates a graph-backed user quota configuration node (`user_config_node_id`)
    /// through the standalone execution kernel (`Dc 1936` `"Dc document execution kernel"`).
    ///
    /// # Errors
    /// Returns an error if the supervisor or configuration node cannot be resolved or validated.
    pub fn admit_execution_kernel_from_graph_config(
        seed_graph: &SeedGraphIndex,
        user_config_node_id: u128,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let entry = DcShorthand::from_global_id(
            Self::EXECUTION_KERNEL_ENTRY_DC.to_u128(),
        );
        Self::admit_from_graph_config_node(
            seed_graph,
            &[entry],
            user_config_node_id,
            watchdog,
        )
    }

    /// Resolves and authenticates the full workspace supervisor (`Dc 1937`
    /// `"Collective Toolbox Workspace"`) from the application-bundled System graph
    /// (`SeedGraphIndex::shared()`), validates `user_limits` through the supervisor Dc-WASM
    /// definition, and clamps emergency host `watchdog` bounds.
    ///
    /// # Errors
    /// Returns an error if loading the bundled System graph fails or `user_limits` is rejected.
    pub fn admit_workspace_supervisor(
        user_limits: DcWasmResourceLimits,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let seed_graph = SeedGraphIndex::shared()?;
        let entry = DcShorthand::from_global_id(
            Self::WORKSPACE_SUPERVISOR_ENTRY_DC.to_u128(),
        );
        Self::admit_from_bootstrap_tokens(
            seed_graph,
            &[entry],
            user_limits,
            watchdog,
        )
    }

    /// Resolves and authenticates the full workspace supervisor (`Dc 1937`
    /// `"Collective Toolbox Workspace"`) from the application-bundled System graph with default
    /// resource and watchdog limits.
    ///
    /// # Errors
    /// Returns an error if loading or authenticating the bundled System supervisor fails.
    pub fn admit_workspace_supervisor_default() -> Result<Self> {
        Self::admit_workspace_supervisor(
            DcWasmResourceLimits::default(),
            DcWasmWatchdogLimits::default(),
        )
    }

    /// Resolves and validates a graph-backed user quota configuration node (`user_config_node_id`)
    /// through the full workspace supervisor (`Dc 1937` `"Collective Toolbox Workspace"`).
    ///
    /// # Errors
    /// Returns an error if the supervisor or configuration node cannot be resolved or validated.
    pub fn admit_workspace_supervisor_from_graph_config(
        seed_graph: &SeedGraphIndex,
        user_config_node_id: u128,
        watchdog: DcWasmWatchdogLimits,
    ) -> Result<Self> {
        let entry = DcShorthand::from_global_id(
            Self::WORKSPACE_SUPERVISOR_ENTRY_DC.to_u128(),
        );
        Self::admit_from_graph_config_node(
            seed_graph,
            &[entry],
            user_config_node_id,
            watchdog,
        )
    }

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
            .context("Graph user quota configuration node is too short")?;
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
        ctb_wasm_substitutes::ensure_registered();
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

    /// Scans `doc` Dc-by-Dc (skipping escaped tokens, literal strings, identifiers, binary
    /// payloads, and source comments) to locate the first unquoted `[pin_description]`
    /// (`Dc 641` System-block revision pin), returning `(pin_tokens, pin_node_id)`.
    ///
    /// # Errors
    /// Returns an error if `doc` contains invalid DcTF bytes or a malformed `[pin_description]`.
    pub fn scan_first_pin_description(
        doc: &DcMstr,
        resolver: &dyn ctb_formats_dcstring::SyntaxRuleResolver,
    ) -> Result<Option<(Vec<DcChar>, u128)>> {
        use ctb_formats_dcstring::global_graph_layout::{
            SYSTEM_REGION_END, SYSTEM_REGION_START,
            unsupported_graph_block_description,
        };
        use ctb_formats_dcstring::syntax::{DcRole, SyntaxTerm};
        use ctb_formats_dcstring::{
            DcByteSpan, DcCharExt, DcDocumentNavigator, SyntaxEvalContext,
        };

        let chars: Vec<DcChar> = doc.to_dc_string()?.chars().collect();
        let mut idx = 0usize;
        while let Some(&ch) = chars.get(idx) {
            if ch.fills_role_with(DcRole::Escape, resolver) {
                idx = idx.saturating_add(2);
                continue;
            }
            let skip_closer_role = if ch
                .fills_role_with(DcRole::BeginLiteral, resolver)
            {
                Some(DcRole::EndLiteral)
            } else if ch.fills_role_with(DcRole::BeginIdentifier, resolver) {
                Some(DcRole::EndIdentifier)
            } else if ch
                .fills_role_with(DcRole::StartEncapsulationBinary, resolver)
            {
                Some(DcRole::EndEncapsulationBinary)
            } else if ch
                .fills_role_with(DcRole::BeginSingleLineComment, resolver)
                || ch.fills_role_with(
                    DcRole::BeginSingleLineCommentNewLine,
                    resolver,
                )
            {
                Some(DcRole::EndSingleLineComment)
            } else if ch.fills_role_with(DcRole::BeginBlockComment, resolver)
                || ch
                    .fills_role_with(DcRole::BeginBlockCommentNewLine, resolver)
            {
                Some(DcRole::EndBlockComment)
            } else {
                None
            };
            if let Some(closer_role) = skip_closer_role {
                idx = idx.saturating_add(1);
                while let Some(&inner_ch) = chars.get(idx) {
                    if inner_ch.fills_role_with(DcRole::Escape, resolver) {
                        idx = idx.saturating_add(2);
                        continue;
                    }
                    idx = idx.saturating_add(1);
                    if inner_ch.fills_role_with(closer_role, resolver) {
                        break;
                    }
                }
                continue;
            }
            if ch.fills_role_with(
                DcRole::GlobalGraphSystemBlockMostRecentNodeId,
                resolver,
            ) {
                let mut tail_mstr = DcMstring::new();
                for &tok in chars.get(idx..).into_iter().flatten() {
                    tail_mstr.push_char(tok);
                }
                let mut nav = DcDocumentNavigator::from_mstr(&tail_mstr);
                let start_pos = nav.position();
                let mut ctx = SyntaxEvalContext::new(resolver, None);
                let matched = nav.eval_term_single(
                    &SyntaxTerm::NamedConstruct {
                        name: "pin_description".to_owned(),
                        subtype: None,
                        capture_var: None,
                    },
                    &mut ctx,
                )?;
                ensure!(
                    matched && !ctx.has_errors && nav.position() > start_pos,
                    "Malformed [pin_description] starting at Dc 641 in document"
                );
                let pin_end = nav.position();
                let pin_u128s =
                    nav.read_tokens_in_span(DcByteSpan::new(start_pos, pin_end))?;
                let pin_tokens: Vec<DcChar> =
                    pin_u128s.into_iter().map(DcChar::from_u128).collect();
                let second = pin_tokens.get(1).copied().context(
                    "Missing System-block node ID operand after Dc 641",
                )?;
                let pin_id = if second.fills_role_with(DcRole::Escape, resolver)
                    || second.fills_role_with(
                        DcRole::QuoteFollowingDcAsAtom,
                        resolver,
                    )
                {
                    pin_tokens
                        .get(2)
                        .copied()
                        .context("Dangling escape in Dc 641 pin_description")?
                        .to_u128()
                } else {
                    second.to_u128()
                };
                if let Some(block_desc) =
                    unsupported_graph_block_description(pin_id)
                {
                    bail!(
                        "Unsupported graph block pin in Dc 641: node ID l{pin_id} targets {block_desc}"
                    );
                }
                ensure!(
                    (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&pin_id),
                    "Invalid System-block most recent node ID {pin_id} for Dc 641: expected System block range {SYSTEM_REGION_START}..={SYSTEM_REGION_END}"
                );
                return Ok(Some((pin_tokens, pin_id)));
            }
            idx = idx.saturating_add(1);
        }
        Ok(None)
    }

    /// Quotes `doc` as a typed Dc `[string]` literal (`Dc 260 "Begin literal value"`,
    /// `Dc 262 "Begin type definition"`, `Dc 264 "Built-in type identifier: String"`,
    /// `Dc 263 "End type definition"`, escaped payload, `Dc 261 "End literal value"`).
    ///
    /// # Errors
    /// Returns an error if `doc` contains malformed DcTF bytes.
    pub fn quote_dc_document_as_string_literal(
        doc: &DcMstr,
        resolver: &dyn ctb_formats_dcstring::SyntaxRuleResolver,
    ) -> Result<Vec<DcChar>> {
        use ctb_formats_dcstring::DcCharExt;
        use ctb_formats_dcstring::syntax::DcRole;

        let chars: Vec<DcChar> = doc.to_dc_string()?.chars().collect();
        let mut out = Vec::with_capacity(chars.len().saturating_add(6));
        out.push(dc::DC_BEGIN_LITERAL);
        out.push(dc::DC_BEGIN_TYPE_DEF);
        out.push(dc::DC_TYPE_STRING);
        out.push(dc::DC_END_TYPE_DEF);
        for ch in chars {
            if ch.fills_role_with(DcRole::Escape, resolver)
                || ch.fills_role_with(DcRole::EndLiteral, resolver)
                || ch.fills_role_with(DcRole::QuoteFollowingDcAsAtom, resolver)
            {
                out.push(dc::DC_ESCAPE);
            }
            out.push(ch);
        }
        out.push(dc::DC_END_LITERAL);
        Ok(out)
    }

    /// Admits the `Dc 1936` (`Dc document execution kernel`) supervisor with
    /// bounded CLI compilation and execution quotas sufficient to self-host
    /// StageR 3 (`f1681`) and StageL 3 (`f1680`) pipelines while still enforcing
    /// deterministic compute, memory, and output limits.
    ///
    /// # Errors
    /// Returns an error if resolving or executing `Dc 1936` fails.
    pub fn admit_cli_execution_kernel() -> Result<Self> {
        let limits = DcWasmResourceLimits::default()
            .with_max_instructions_per_step(4_000_000_000)
            .with_max_memory_pages(8192)
            .with_max_step_output_bytes(32 * 1024 * 1024)
            .with_max_session_output_bytes(64 * 1024 * 1024)
            .with_max_capability_payload_bytes(32 * 1024 * 1024);
        let watchdog = DcWasmWatchdogLimits::default()
            .with_physical_max_memory_pages(8192)
            .with_hard_max_instructions_per_step(4_000_000_000);
        Self::admit_execution_kernel(limits, watchdog)
    }

    /// Compiles an arbitrary executable Dc document `doc` into a `Dc 1927`..`Dc 1928` Dc-WASM
    /// program by evaluating `[Dc 1938 <pin_description> [Dc 2608 <quoted_doc>]]` (or unpinned
    /// `[Dc 2608 <quoted_doc>]`) under `shared_quota`, ensuring compilation runs under the
    /// exact same resource quotas as target execution.
    ///
    /// # Errors
    /// Returns an error if `doc` attempts privilege elevation, fails pin/converter resolution,
    /// traps during compilation, or exhausts `shared_quota`.
    pub fn compile_dc_document_to_wasm_with_shared_quota(
        &self,
        doc: &DcMstr,
        resolver: &dyn ctb_formats_dcstring::SyntaxRuleResolver,
        shared_quota: ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota,
    ) -> Result<(DcMstring, Option<u128>)> {
        use ctb_formats_dcstring::{
            DcExpressionEvaluator, SharedQuotaDcExpressionEvaluator,
        };

        Self::verify_ordinary_dc_document_cannot_elevate(doc, self.entry_dc)?;
        let opt_pin = Self::scan_first_pin_description(doc, resolver)?;
        let quoted_doc =
            Self::quote_dc_document_as_string_literal(doc, resolver)?;
        let mut dispatch_tokens =
            Vec::with_capacity(quoted_doc.len().saturating_add(6));
        let pin_id = if let Some((pin_tokens, id)) = opt_pin {
            dispatch_tokens.push(dc::DC_RESOLVE_TO_PIN);
            dispatch_tokens.extend(pin_tokens);
            Some(id)
        } else {
            None
        };
        dispatch_tokens.push(dc::DC_CONVERT_DC_STRING_TO_WASM);
        dispatch_tokens.extend(quoted_doc);

        let evaluator = SharedQuotaDcExpressionEvaluator::with_shared_task_quota(
            shared_quota,
        );
        let wasm_chars = evaluator
            .eval_stream_splice(&dispatch_tokens, resolver)?
            .context("ConvertDcStringToWasm did not produce a Dc-WASM program")?;
        let mut wasm_dc_str =
            DcString::with_capacity(wasm_chars.len().saturating_mul(4));
        for ch in wasm_chars {
            wasm_dc_str.push(ch);
        }
        let wasm_doc = DcMstring::from_dc_str_with_policy_and_resolver(
            &wasm_dc_str,
            &ctb_formats_dcstring::DcProjectionPolicy::strict(),
            resolver,
        )?;
        Ok((wasm_doc, pin_id))
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
    /// Read-only minimal query of the active System graph (`Dc 1939`).
    SystemGraphQuery,
    /// Pure child WebAssembly execution (`Dc 1940`) under shared task quotas.
    ExecutePureWasm,
    /// Custom privileged operation.
    Custom(String),
}

impl Privilege {
    /// Classifies a capability request `DcMstr` emitted by a WASM module
    /// (`WasmStepStatus::RequestCapability`) against `resolver`.
    #[must_use]
    pub fn from_request_dc_with_resolver(
        request: &DcMstr,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Self {
        if let Ok(dc_str) = request.to_dc_string()
            && let Some(first) = dc_str.chars().next()
        {
            let first_u = first.to_u128();
            if resolver
                .token_fills_role(first_u, DcRole::MinimalSystemGraphQuery)
            {
                return Self::SystemGraphQuery;
            }
            if resolver
                .token_fills_role(first_u, DcRole::ExecutePureWasmProgram)
            {
                return Self::ExecutePureWasm;
            }
        }
        Self::from_request_dc_ascii_fallback(request)
    }

    /// Classifies a capability request `DcMstr` emitted by a WASM module
    /// (`WasmStepStatus::RequestCapability`) using the default syntax resolver.
    #[must_use]
    pub fn from_request_dc(request: &DcMstr) -> Self {
        let default_res =
            ctb_formats_dcstring::syntax::default_syntax_resolver();
        Self::from_request_dc_with_resolver(request, default_res.as_ref())
    }

    fn from_request_dc_ascii_fallback(request: &DcMstr) -> Self {
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
        ctb_wasm_substitutes::ensure_registered();
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

    /// Creates a new execution context authenticated against the bundled standalone execution
    /// kernel (`Dc 1936` `"Dc document execution kernel"`).
    ///
    /// # Errors
    /// Returns an error if loading or authenticating the bundled System execution kernel fails.
    pub fn new_execution_kernel(has_errors: bool) -> Result<Self> {
        let admission =
            SystemSupervisorAdmission::admit_execution_kernel_default()?;
        Ok(Self::new(has_errors).with_system_supervisor_admission(admission))
    }

    /// Creates a new execution context authenticated against the bundled workspace supervisor
    /// (`Dc 1937` `"Collective Toolbox Workspace"`).
    ///
    /// # Errors
    /// Returns an error if loading or authenticating the bundled workspace supervisor fails.
    pub fn new_workspace_supervisor(has_errors: bool) -> Result<Self> {
        let admission =
            SystemSupervisorAdmission::admit_workspace_supervisor_default()?;
        Ok(Self::new(has_errors).with_system_supervisor_admission(admission))
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
        } else if matches!(
            privilege,
            Privilege::SystemGraphQuery | Privilege::ExecutePureWasm
        ) {
            PermissionDecision::Allow
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
        // Reason for fallback: when supervisor admission was not configured or failed before admission, trace metadata reports the caller's fallback_limits.
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
        // Reason for fallback: when session initialization failed before a DcWasmSession was constructed, trace budget enforcement uses fallback_limits.max_trace_bytes.
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
        capability_handler: F,
    ) -> Result<WasmExecutionTrace>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        self.execute_wasm_session_with_trace_inner(
            session,
            input,
            None,
            capability_handler,
        )
    }

    fn execute_wasm_session_with_trace_inner<F>(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        resolver: Option<&dyn SyntaxRuleResolver>,
        mut capability_handler: F,
    ) -> Result<WasmExecutionTrace>
    where
        F: FnMut(&Privilege, PermissionDecision, &DcMstr) -> Result<DcMstring>,
    {
        ctb_wasm_substitutes::ensure_registered();
        if let Some(adm) = &self.supervisor_admission
            && !session.has_shared_task_quota()
        {
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
                    let privilege = if let Some(res) = resolver {
                        Privilege::from_request_dc_with_resolver(&req_dc, res)
                    } else {
                        Privilege::from_request_dc(&req_dc)
                    };
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
                        match capability_handler(&privilege, decision, &req_dc)
                        {
                            Ok(r) => r,
                            Err(err)
                                if err
                                    .downcast_ref::<WasmResourceInterruption>()
                                    .is_some() =>
                            {
                                return Ok(self
                                    .handle_session_error_into_trace(
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
                            Err(err) => return Err(err),
                        };
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

    /// Drives a [`DcWasmSession`] with the default capability response encoder
    /// and pure `Dc 1939`/`Dc 1940` capability dispatch against `resolver`.
    ///
    /// # Errors
    /// Returns an error if a non-resource capability handler fails.
    pub fn execute_wasm_session_default_with_resolver_and_trace(
        &mut self,
        session: &mut DcWasmSession,
        input: &DcMstr,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<WasmExecutionTrace> {
        let shared_quota = session.shared_task_quota();
        self.execute_wasm_session_with_trace_inner(
            session,
            input,
            Some(resolver),
            move |privilege, decision, req| {
                if decision == PermissionDecision::Allow
                    && matches!(
                        privilege,
                        Privilege::SystemGraphQuery
                            | Privilege::ExecutePureWasm
                    )
                {
                    return eval_pure_graph_capability_request(
                        req,
                        resolver,
                        shared_quota.clone(),
                        0,
                    );
                }
                Ok(Self::default_capability_response(decision))
            },
        )
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
        let default_res =
            ctb_formats_dcstring::syntax::default_syntax_resolver();
        self.execute_wasm_session_default_with_resolver_and_trace(
            session,
            input,
            default_res.as_ref(),
        )
    }

    /// Compiles and executes an arbitrary ordinary Dc document `ordinary_doc` through the
    /// `Dc 1936` execution kernel pipeline (`[Dc 1938 <pin> [Dc 2608 <quoted_doc>]]`), sharing
    /// one [`ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota`] across both
    /// compilation (`Dc 2608` + any graph converter routine) and target `.WASM` execution.
    ///
    /// # Errors
    /// Returns an error if `ordinary_doc` attempts privilege elevation, fails compilation/assembly,
    /// or fails capability handling.
    pub fn execute_dc_document_default_with_trace(
        &mut self,
        ordinary_doc: &DcMstr,
        input: &DcMstr,
        resolver: &dyn ctb_formats_dcstring::SyntaxRuleResolver,
    ) -> Result<WasmExecutionTrace> {
        use ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota;

        let adm = if let Some(existing) = &self.supervisor_admission {
            existing.clone()
        } else {
            let default_adm =
                SystemSupervisorAdmission::admit_execution_kernel_default()?;
            self.supervisor_admission = Some(default_adm.clone());
            default_adm
        };
        let mut active_limits = adm.compiled_limits;
        let mut compile_events = Vec::new();
        let mut compile_trace_bytes = 0usize;

        let (wasm_doc, opt_pin_id, shared_quota) = loop {
            let shared_quota = SharedWasmTaskQuota::new(active_limits);
            match adm.compile_dc_document_to_wasm_with_shared_quota(
                ordinary_doc,
                resolver,
                shared_quota.clone(),
            ) {
                Ok((wasm_doc, opt_pin_id)) => {
                    break (wasm_doc, opt_pin_id, shared_quota);
                }
                Err(err) => {
                    if let Some(
                        WasmResourceInterruption::ComputeQuotaExhausted {
                            limit,
                        },
                    ) = err.downcast_ref::<WasmResourceInterruption>().cloned()
                    {
                        let decision =
                            self.request_permission(&Privilege::ExcessiveCompute);
                        if self
                            .push_trace_event_bounded(
                                &mut compile_events,
                                &mut compile_trace_bytes,
                                active_limits.max_trace_bytes,
                                0,
                                WasmTraceEvent::ResourceInterruption {
                                    after_logical_steps: 0,
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
                                None,
                                active_limits,
                                WasmExecutionResult {
                                    output: DcMstring::new(),
                                    frames: Vec::new(),
                                    broke_out_early: true,
                                },
                                compile_events,
                                compile_trace_bytes,
                                None,
                            ));
                        }
                        if decision == PermissionDecision::Allow {
                            active_limits.max_instructions_per_step =
                                active_limits
                                    .max_instructions_per_step
                                    .saturating_add(
                                        self.compute_quota_grant.max(1),
                                    );
                            continue;
                        }
                        return Ok(self.build_trace(
                            None,
                            active_limits,
                            WasmExecutionResult {
                                output: DcMstring::new(),
                                frames: Vec::new(),
                                broke_out_early: true,
                            },
                            compile_events,
                            compile_trace_bytes,
                            None,
                        ));
                    }
                    if err.downcast_ref::<WasmResourceInterruption>().is_some() {
                        return Ok(self.handle_session_error_into_trace(
                            None,
                            active_limits,
                            err,
                            0,
                            Vec::new(),
                            compile_events,
                            compile_trace_bytes,
                            None,
                        ));
                    }
                    return Err(err);
                }
            }
        };

        let most_recent_id = opt_pin_id.or_else(|| {
            resolver
                .seed_graph()
                .and_then(SeedGraphIndex::system_block_most_recent_node_id)
        });
        let exec_pinned = if let Some(pin_id) = opt_pin_id {
            if let Some(sg) = resolver.seed_graph()
                && !sg.is_from_asset_bundle()
            {
                let pinned_sg = std::sync::Arc::new(
                    SeedGraphIndex::from_nodes_at_most_recent(
                        sg.nodes().to_vec(),
                        pin_id,
                    ),
                );
                Some(std::sync::Arc::new(
                    ctb_formats_dcstring::syntax::DatasetRuleResolver::from_seed_graph_arc(
                        pinned_sg,
                    )?,
                ))
            } else {
                Some(
                    ctb_formats_dcstring::syntax::syntax_resolver_at_most_recent(
                        pin_id,
                    )?,
                )
            }
        } else {
            None
        };
        let exec_res: &dyn ctb_formats_dcstring::SyntaxRuleResolver =
            if let Some(ref pinned) = exec_pinned {
                pinned.as_ref()
            } else {
                resolver
            };

        let mut session = match DcWasmSession::from_dc_wasm_with_limits(
            &wasm_doc,
            input,
            most_recent_id,
            exec_res,
            active_limits,
        )
        .and_then(|s| s.with_shared_task_quota(shared_quota))
        {
            Ok(s) => s,
            Err(err) => {
                if err.downcast_ref::<WasmResourceInterruption>().is_some() {
                    return Ok(self.handle_session_error_into_trace(
                        None,
                        active_limits,
                        err,
                        0,
                        Vec::new(),
                        compile_events,
                        compile_trace_bytes,
                        None,
                    ));
                }
                return Err(err);
            }
        };

        let mut trace = self.execute_wasm_session_default_with_resolver_and_trace(
            &mut session,
            input,
            exec_res,
        )?;
        if !compile_events.is_empty() {
            compile_events.extend(trace.events);
            trace.events = compile_events;
        }
        Ok(trace)
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
        ctb_wasm_substitutes::ensure_registered();
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
        let routine_v2_extra_instrs = build_cap_routine(300)?;

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
            DcWasmResourceLimits::default().with_max_instructions_per_step(1);
        let mut interrupted_session = DcWasmSession::from_dc_wasm(
            &routine_v2_extra_instrs,
            &file_io_input,
            None,
        )?
        .with_resource_limits(low_compute_quota);
        let mut int_ctx = ExecutionContext::new(false);
        let interrupted_trace = int_ctx
            .execute_wasm_session_default_with_trace(
                &mut interrupted_session,
                &file_io_input,
            )?;
        assert!(interrupted_trace.result.broke_out_early);
        assert_eq!(interrupted_trace.last_interrupted_op_ordinal, Some(2));
        assert_eq!(
            interrupted_trace.last_interrupted_op_required_cost,
            Some(1)
        );

        let mut fast_replay_session = DcWasmSession::from_dc_wasm(
            &routine_v2_extra_instrs,
            &file_io_input,
            None,
        )?;
        let replayed_int = replay_ctx.replay_wasm_session_trace(
            &mut fast_replay_session,
            &file_io_input,
            &interrupted_trace,
        )?;
        assert_eq!(replayed_int, interrupted_trace);
        assert_eq!(fast_replay_session.total_instructions_executed(), 1);
        assert_eq!(
            fast_replay_session.last_interrupted_op_ordinal(),
            Some(2)
        );
        assert_eq!(
            fast_replay_session.last_interrupted_op_required_cost(),
            Some(1)
        );

        // Tampering with `last_interrupted_op_ordinal` in the recorded trace must fail replay verification.
        let mut forged_ordinal_trace = interrupted_trace.clone();
        forged_ordinal_trace.last_interrupted_op_ordinal = Some(99);
        let mut forged_replay_session = DcWasmSession::from_dc_wasm(
            &routine_v2_extra_instrs,
            &file_io_input,
            None,
        )?;
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

        // Step 0 consumes 1 V2 checkpoint unit; step 1 consumes 1 unit (2 total).
        // Start with quota 1 so step 0 succeeds (`YieldFrame`), step 1 triggers `ComputeQuotaExhausted`,
        // and granting +50 via `PermissionDecision::Allow` resumes step 1 to completion.
        let tight_limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(1);
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
                interruption: Some(WasmResourceInterruption::ComputeQuotaExhausted { limit: 1 }),
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
                created_ns: None,
            },
            LedgerNode {
                id: sup_has_a_id,
                tokens: vec![
                    DcShorthand::Short(584),
                    sh_sup,
                    DcShorthand::Short(611),
                ],
                comment: String::new(),
                created_ns: None,
            },
            LedgerNode {
                id: sup_has_val_id,
                tokens: vec![
                    DcShorthand::Short(610),
                    DcShorthand::Long(sup_has_a_id),
                    DcShorthand::Long(sup_def_id),
                ],
                comment: String::new(),
                created_ns: None,
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
                created_ns: None,
            },
            LedgerNode {
                id: prov_has_val_id,
                tokens: vec![
                    DcShorthand::Short(610),
                    DcShorthand::Long(prov_has_a_id),
                    DcShorthand::Short(636),
                ],
                comment: String::new(),
                created_ns: None,
            },
            LedgerNode {
                id: user_cfg_node_id,
                tokens:
                    SystemSupervisorAdmission::encode_graph_quota_config_tokens(
                        &user_limits,
                    ),
                comment: String::new(),
                created_ns: None,
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

        // 6. Production bundled System supervisor entries `Dc 1936 "Dc document execution kernel"`
        //    and `Dc 1937 "Collective Toolbox Workspace"` resolve directly from `SeedGraphIndex::shared()`,
        //    execute their bundled `.WASM` supervisor definitions, and record `Dc 1937`'s dependency
        //    on `Dc 1936` in `dependency_snapshot_node_ids`.
        let prod_kernel_adm =
            SystemSupervisorAdmission::admit_execution_kernel_default()?;
        assert_eq!(
            prod_kernel_adm.entry_dc,
            dc::DC_DC_DOCUMENT_EXECUTION_KERNEL
        );
        let prod_workspace_adm =
            SystemSupervisorAdmission::admit_workspace_supervisor_default()?;
        assert_eq!(
            prod_workspace_adm.entry_dc,
            dc::DC_COLLECTIVE_TOOLBOX_WORKSPACE
        );
        assert_eq!(
            prod_kernel_adm.supervisor_revision_node_id,
            prod_workspace_adm.supervisor_revision_node_id
        );
        let mut kernel_ctx = ExecutionContext::new_execution_kernel(false)?;
        let mut prod_session =
            DcWasmSession::from_dc_wasm(&trap_after_frame_doc, &input, None)?;
        let prod_trace = kernel_ctx.execute_wasm_session_default_with_trace(
            &mut prod_session,
            &input,
        )?;
        assert_eq!(
            prod_trace.supervisor_revision_node_id,
            Some(prod_kernel_adm.supervisor_revision_node_id)
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate3_async_event_ordering_prefix_invariance_and_vertical_migration()
    -> Result<()> {
        use ctb_formats_dcstring::{
            DcChar, DcMixedEncode, DcWasmResourceLimits, LedgerNode,
        };

        let i32_dc =
            DcChar::from_u128(crate::FormatId::I32.dc_id().unwrap());

        // Build a 5-step Dc-WASM routine using mem[64] as phase counter (0..=4):
        // - Step 0 (phase 0): sets mem[64]=1, echoes input mailbox to output, returns 1 (YieldFrame #0)
        // - Step 1 (phase 1): sets mem[64]=2, echoes input mailbox to output, returns 2 (RequestCapability #1)
        // - Step 2 (phase 2): sets mem[64]=3, echoes capability #1 response to output, returns 1 (YieldFrame #1)
        // - Step 3 (phase 3): sets mem[64]=4, writes saved request at mem[128..136] to output, returns 2 (RequestCapability #2)
        // - Step 4 (phase 4): echoes capability #2 response to output, returns 0 (Complete)
        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_short(260));
        doc.push_char(DcChar::from_short(262));
        doc.push_char(DcChar::from_short(280));
        doc.push_char(i32_dc);
        doc.push_char(DcChar::from_short(263));

        // Default output mailbox (8..16) = current input mailbox (0..8)
        for (dst, src) in [(8i32, 0i32), (12, 4)] {
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&dst, &mut doc)?;
            doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&src, &mut doc)?;
            doc.push_char(DcChar::from_short(687));
            for imm in [0u32, 2, 0] {
                u32::encode_dc_mixed(&imm, &mut doc)?;
            }
            doc.push_char(DcChar::from_short(701));
            for imm in [0u32, 2, 0] {
                u32::encode_dc_mixed(&imm, &mut doc)?;
            }
        }

        // Helper closure to emit: if (mem[64] == phase) { mem[64] = phase + 1; ...; return status; } else { ... }
        let emit_phase_check = |out: &mut DcMstring, phase: i32| -> Result<()> {
            out.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&64, out)?;
            out.push_char(DcChar::from_short(687));
            for imm in [0u32, 2, 0] {
                u32::encode_dc_mixed(&imm, out)?;
            }
            out.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&phase, out)?;
            out.push_char(DcChar::from_short(717)); // i32.eq
            out.push_char(DcChar::from_short(651)); // if (result i32)
            out.push_char(i32_dc);
            out.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&64, out)?;
            out.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&(phase + 1), out)?;
            out.push_char(DcChar::from_short(701));
            for imm in [0u32, 2, 0] {
                u32::encode_dc_mixed(&imm, out)?;
            }
            Ok(())
        };

        // Phase 0: return 1 (YieldFrame #0 with initial input)
        emit_phase_check(&mut doc, 0)?;
        doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut doc)?;
        doc.push_char(DcChar::from_short(652)); // else

        // Phase 1: return 2 (RequestCapability #1)
        emit_phase_check(&mut doc, 1)?;
        doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut doc)?;
        doc.push_char(DcChar::from_short(652)); // else

        // Phase 2: return 1 (YieldFrame #1 with capability #1's response)
        emit_phase_check(&mut doc, 2)?;
        doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut doc)?;
        doc.push_char(DcChar::from_short(652)); // else

        // Phase 3: return 2 (RequestCapability #2 with capability #1's response in mailbox)
        emit_phase_check(&mut doc, 3)?;
        doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut doc)?;
        doc.push_char(DcChar::from_short(652)); // else

        // Phase 4: return 0 (Complete with capability #2's response)
        doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut doc)?;

        // Close all 4 nested `if` blocks and the literal container:
        for _ in 0..4 {
            doc.push_char(DcChar::from_short(658));
        }
        doc.push_char(DcChar::from_short(261));

        let mut req_input = DcMstring::new();
        req_input.push_str("FileIo");

        let mut resp_first = DcMstring::new();
        resp_first.push_str("first_cap_ok");
        let mut resp_second_a = DcMstring::new();
        resp_second_a.push_str("second_cap_branch_A");
        let mut resp_second_b = DcMstring::new();
        resp_second_b.push_str("second_cap_branch_B");
        let mut resp_second_denied = DcMstring::new();
        resp_second_denied.push_char(dc::DC_BOOLEAN_FALSE);

        let run_scenario = |second_resp: DcMstring| -> Result<WasmExecutionTrace> {
            let mut session =
                DcWasmSession::from_dc_wasm(&doc, &req_input, None)?;
            let mut ctx = ExecutionContext::new(false)
                .with_privilege_decision(
                    Privilege::FileIo,
                    PermissionDecision::Allow,
                );
            let mut call_idx = 0usize;
            let first_clone = resp_first.clone();
            ctx.execute_wasm_session_with_trace(
                &mut session,
                &req_input,
                |_priv, _dec, _raw| {
                    call_idx += 1;
                    if call_idx == 1 {
                        Ok(first_clone.clone())
                    } else {
                        Ok(second_resp.clone())
                    }
                },
            )
        };

        let trace_a = run_scenario(resp_second_a.clone())?;
        let trace_b = run_scenario(resp_second_b.clone())?;
        let trace_denied = run_scenario(resp_second_denied.clone())?;

        // 1. Subsequent outputs diverge after delivery point #2 (step 3):
        assert_eq!(trace_a.result.output, resp_second_a);
        assert_eq!(trace_b.result.output, resp_second_b);
        assert_eq!(trace_denied.result.output, resp_second_denied);
        assert_ne!(trace_a.result.output, trace_b.result.output);
        assert_ne!(trace_a.result.output, trace_denied.result.output);

        // 2. Prefix invariance: all frames (`frames[0..2]`), all earlier events (`events[0..3]`),
        //    and the capability request emitted at step 3 (`events[3].request`) are bit-for-bit identical!
        assert_eq!(trace_a.result.frames, vec![req_input.clone(), resp_first.clone()]);
        assert_eq!(trace_a.result.frames, trace_b.result.frames);
        assert_eq!(trace_a.result.frames, trace_denied.result.frames);
        assert_eq!(trace_a.events[..3], trace_b.events[..3]);
        assert_eq!(trace_a.events[..3], trace_denied.events[..3]);
        match (&trace_a.events[3], &trace_b.events[3], &trace_denied.events[3]) {
            (
                WasmTraceEvent::CapabilityInteraction {
                    request: req_a,
                    privilege: priv_a,
                    ..
                },
                WasmTraceEvent::CapabilityInteraction {
                    request: req_b,
                    privilege: priv_b,
                    ..
                },
                WasmTraceEvent::CapabilityInteraction {
                    request: req_c,
                    privilege: priv_c,
                    ..
                },
            ) => {
                assert_eq!(req_a, &resp_first);
                assert_eq!(req_a, req_b);
                assert_eq!(req_a, req_c);
                assert_eq!(
                    priv_a,
                    &Privilege::Custom("first_cap_ok".to_string())
                );
                assert_eq!(priv_a, priv_b);
                assert_eq!(priv_a, priv_c);
            }
            other => panic!("Expected CapabilityInteraction at event[3], got {other:?}"),
        }

        // 3. Replaying each trace reproduces the exact branch output and trace:
        for expected_trace in [&trace_a, &trace_b, &trace_denied] {
            let mut replay_session =
                DcWasmSession::from_dc_wasm(&doc, &req_input, None)?;
            let mut replay_ctx = ExecutionContext::new(false);
            let replayed = replay_ctx.replay_wasm_session_trace(
                &mut replay_session,
                &req_input,
                expected_trace,
            )?;
            assert_eq!(&replayed, expected_trace);
        }

        // 4. Vertical migration across generated System graph snapshots R1 < R2 (and R3 > R2 rejection):
        let shared = ctb_formats_dcstring::SeedGraphIndex::shared()?;
        let head_pin = shared.system_block_most_recent_node_id().unwrap();
        let prev_pin = head_pin.checked_sub(1).unwrap();
        for pin in [prev_pin, head_pin] {
            let mut pinned_doc = DcMstring::new();
            pinned_doc.push_char(
                dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            );
            pinned_doc.push_char(DcChar::from_u128(pin));
            for ch in doc.to_dc_string()?.chars() {
                pinned_doc.push_char(ch);
            }
            let pinned_nav = ctb_formats_dcstring::DcDocumentNavigator::from_mstr(&pinned_doc);
            assert_eq!(pinned_nav.document_most_recent_node_id(), Some(pin));
            let pinned_resolver = pinned_nav.effective_resolver();
            let mut pinned_session =
                DcWasmSession::from_dc_wasm_with_resolver(
                    &doc,
                    &req_input,
                    Some(pin),
                    pinned_resolver.as_ref(),
                )?;
            let mut replay_ctx = ExecutionContext::new(false);
            let replayed_pinned = replay_ctx.replay_wasm_session_trace(
                &mut pinned_session,
                &req_input,
                &trace_a,
            )?;
            assert_eq!(replayed_pinned, trace_a);
        }
        // Future pin R3 = head_pin + 1 must fail before any capability request is issued:
        let future_pin = head_pin.checked_add(1).unwrap();
        let mut future_doc = DcMstring::new();
        future_doc.push_char(
            dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
        );
        future_doc.push_char(DcChar::from_u128(future_pin));
        for ch in doc.to_dc_string()?.chars() {
            future_doc.push_char(ch);
        }
        assert!(
            DcWasmSession::from_dc_wasm(&future_doc, &req_input, None).is_err(),
            "Future Dc 641 snapshot pin must be rejected before execution"
        );

        let _ = (DcWasmResourceLimits::default(), LedgerNode {
            id: 0,
            tokens: Vec::new(),
            comment: String::new(),
            created_ns: None,
        });
        Ok(())
    }

    fn build_minimal_constant_output_wasm_doc(
        output_byte: u8,
        spin_iters: u32,
    ) -> Result<DcMstring> {
        let spin_wat = if spin_iters == 0 {
            String::new()
        } else {
            format!(
                "(local $i i32)\n    (set $i {spin_iters})\n    (block $done (loop $lp (br_if $done (i32.eqz $i)) (set $i (i32.sub $i 1)) (br $lp)))"
            )
        };
        let wat = format!(
            r#"(module
  (memory (export "memory") 1)
  (func (export "step") (result i32)
    {spin_wat}
    (i32.store8 256 {output_byte})
    (i32.store 8 256)
    (i32.store 12 1)
    (i32.const 0)))"#
        );
        let shorthands = ctb_formats_dcstring::wat_to_dc_shorthands(&wat)?;
        let mut doc = DcMstring::new();
        for sh in shorthands {
            doc.push_char(DcChar::from_u128(sh.to_global_id()?));
        }
        Ok(doc)
    }

    fn build_converter_emitting_constant_wasm_shorthands(
        target_wasm_doc: &DcMstr,
        spin_iters: u32,
    ) -> Result<Vec<DcShorthand>> {
        let dc_str = target_wasm_doc.to_dc_string()?;
        let raw_dctf = dc_str.as_bytes();
        let mut stores = String::new();
        for (idx, &b) in raw_dctf.iter().enumerate() {
            let addr = 512usize.saturating_add(idx);
            stores.push_str(&format!("    (i32.store8 {addr} {b})\n"));
        }
        let spin_wat = if spin_iters == 0 {
            String::new()
        } else {
            format!(
                "    (local $i i32)\n    (set $i {spin_iters})\n    (block $done (loop $lp (br_if $done (i32.eqz $i)) (set $i (i32.sub $i 1)) (br $lp)))\n"
            )
        };
        let wat = format!(
            r#"(module
  (memory (export "memory") 1)
  (func (export "step") (result i32)
{spin_wat}{stores}    (i32.store 8 512)
    (i32.store 12 {})
    (i32.const 0)))"#,
            raw_dctf.len()
        );
        ctb_formats_dcstring::wat_to_dc_shorthands(&wat)
    }

    #[crate::ctb_test]
    fn test_dc_2608_convert_dc_string_to_wasm_and_dc_1938_resolve_to_pin()
    -> Result<()> {
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;

        let shared_sg = SeedGraphIndex::shared()?;
        let default_res = ctb_formats_dcstring::syntax::default_syntax_resolver();

        // 1. `scan_first_pin_description` ignores Dc 641 inside comments and literals.
        let head_pin = shared_sg.system_block_most_recent_node_id().unwrap();
        let mut commented_doc = DcMstring::new();
        commented_doc.push_char(dc::DC_BEGIN_SINGLE_LINE_COMMENT);
        commented_doc
            .push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        commented_doc.push_char(DcChar::from_u128(head_pin));
        commented_doc.push_char(dc::DC_END_SINGLE_LINE_COMMENT);
        commented_doc.push_char(dc::DC_BEGIN_LITERAL);
        commented_doc.push_char(dc::DC_BEGIN_TYPE_DEF);
        commented_doc.push_char(dc::DC_TYPE_STRING);
        commented_doc.push_char(dc::DC_END_TYPE_DEF);
        commented_doc
            .push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        commented_doc.push_char(DcChar::from_u128(head_pin));
        commented_doc.push_char(dc::DC_END_LITERAL);
        assert_eq!(
            SystemSupervisorAdmission::scan_first_pin_description(
                &commented_doc,
                default_res.as_ref()
            )?,
            None
        );

        // 2. Construct an in-memory graph snapshot extending `shared_sg` with two revisions
        //    of a standalone `[Dc 302 262 1927]` converter routine (`pin_v1 < pin_v2`).
        let mut nodes = shared_sg.nodes().to_vec();
        let mut next_id = head_pin.checked_add(1).unwrap();
        let mut alloc_node = |tokens: Vec<DcShorthand>| -> u128 {
            let id = next_id;
            next_id += 1;
            nodes.push(ctb_formats_dcstring::LedgerNode {
                id,
                tokens,
                comment: String::new(),
                created_ns: None,
            });
            id
        };

        let wasm_v1 = build_minimal_constant_output_wasm_doc(b'A', 0)?;
        let wasm_v2 = build_minimal_constant_output_wasm_doc(b'B', 0)?;
        let conv_wasm_v1 =
            build_converter_emitting_constant_wasm_shorthands(&wasm_v1, 0)?;
        let conv_wasm_v2 =
            build_converter_emitting_constant_wasm_shorthands(&wasm_v2, 0)?;

        let fmt_node_id = alloc_node(vec![
            DcShorthand::Short(302),
            DcShorthand::Short(262),
            DcShorthand::Short(1927),
        ]);
        let conv_ent_v1 = alloc_node(vec![
            DcShorthand::Short(586),
            DcShorthand::Short(615),
            DcShorthand::Short(280),
        ]);
        let fmt_has_a_v1 = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent_v1),
            DcShorthand::Short(595),
        ]);
        let _fmt_val_v1 = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(fmt_has_a_v1),
            DcShorthand::Long(fmt_node_id),
        ]);
        let wasm_node_v1 = alloc_node(conv_wasm_v1);
        let decomp_has_a_v1 = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent_v1),
            DcShorthand::Short(611),
        ]);
        let decomp_val_v1 = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(decomp_has_a_v1),
            DcShorthand::Long(wasm_node_v1),
        ]);
        let pin_v1 = decomp_val_v1;

        // Revision 2 adds a Preferred (`Dc 570`) standalone converter routine for `262 -> 1927`.
        let conv_ent_v2 = alloc_node(vec![
            DcShorthand::Short(586),
            DcShorthand::Short(615),
            DcShorthand::Short(280),
        ]);
        let fmt_has_a_v2 = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent_v2),
            DcShorthand::Short(595),
        ]);
        let _fmt_val_v2 = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(fmt_has_a_v2),
            DcShorthand::Long(fmt_node_id),
        ]);
        let wasm_node_v2 = alloc_node(conv_wasm_v2);
        let decomp_has_a_v2 = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent_v2),
            DcShorthand::Short(611),
        ]);
        let _decomp_val_v2 = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(decomp_has_a_v2),
            DcShorthand::Long(wasm_node_v2),
        ]);
        let pref_v2 = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(decomp_has_a_v2),
            DcShorthand::Short(570),
        ]);
        let pin_v2 = pref_v2;

        let custom_sg = std::sync::Arc::new(SeedGraphIndex::from_nodes(nodes));
        let custom_res = DatasetRuleResolver::from_seed_graph(&custom_sg)?;
        let empty_in = DcMstring::new();

        // Pinned to `pin_v1` -> executes converter V1 -> outputs "A"
        let mut doc_v1 = DcMstring::new();
        doc_v1.push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_v1.push_char(DcChar::from_u128(pin_v1));
        doc_v1.push_char(dc::DC_BEGIN_TYPE_DEF);
        doc_v1.push_char(dc::DC_TYPE_STRING);
        doc_v1.push_char(dc::DC_END_TYPE_DEF);
        let mut env_v1 = ExecutionContext::new(false);
        let trace_v1 = env_v1.execute_dc_document_default_with_trace(
            &doc_v1,
            &empty_in,
            &custom_res,
        )?;
        assert_eq!(trace_v1.result.output.to_dc_string()?.to_string(), "A");

        // Pinned to `pin_v2` (and unpinned) -> executes preferred converter V2 -> outputs "B"
        let mut doc_v2 = DcMstring::new();
        doc_v2.push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_v2.push_char(DcChar::from_u128(pin_v2));
        doc_v2.push_char(dc::DC_BEGIN_TYPE_DEF);
        doc_v2.push_char(dc::DC_TYPE_STRING);
        doc_v2.push_char(dc::DC_END_TYPE_DEF);
        let mut env_v2 = ExecutionContext::new(false);
        let trace_v2 = env_v2.execute_dc_document_default_with_trace(
            &doc_v2,
            &empty_in,
            &custom_res,
        )?;
        assert_eq!(trace_v2.result.output.to_dc_string()?.to_string(), "B");

        let mut doc_unpinned = DcMstring::new();
        doc_unpinned.push_char(dc::DC_BEGIN_TYPE_DEF);
        doc_unpinned.push_char(dc::DC_TYPE_STRING);
        doc_unpinned.push_char(dc::DC_END_TYPE_DEF);
        let mut env_unpinned = ExecutionContext::new(false);
        let trace_unpinned = env_unpinned
            .execute_dc_document_default_with_trace(
                &doc_unpinned,
                &empty_in,
                &custom_res,
            )?;
        assert_eq!(
            trace_unpinned.result.output.to_dc_string()?.to_string(),
            "B"
        );

        // Future pin `pin_v2 + 1` -> rejected!
        let mut doc_future = DcMstring::new();
        doc_future
            .push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_future.push_char(DcChar::from_u128(pin_v2 + 1));
        doc_future.push_char(dc::DC_BEGIN_TYPE_DEF);
        doc_future.push_char(dc::DC_TYPE_STRING);
        doc_future.push_char(dc::DC_END_TYPE_DEF);
        let mut env_future = ExecutionContext::new(false);
        assert!(
            env_future
                .execute_dc_document_default_with_trace(
                    &doc_future,
                    &empty_in,
                    &custom_res,
                )
                .is_err()
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_1936_shared_quota_across_compilation_and_execution() -> Result<()>
    {
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;

        let shared_sg = SeedGraphIndex::shared()?;
        let head_pin = shared_sg.system_block_most_recent_node_id().unwrap();
        let mut nodes = shared_sg.nodes().to_vec();
        let mut next_id = head_pin.checked_add(1).unwrap();
        let mut alloc_node = |tokens: Vec<DcShorthand>| -> u128 {
            let id = next_id;
            next_id += 1;
            nodes.push(ctb_formats_dcstring::LedgerNode {
                id,
                tokens,
                comment: String::new(),
                created_ns: None,
            });
            id
        };

        // Target WASM spins 120 iterations (> 100 V2 checkpoints); converter spins 80 iterations.
        let target_wasm = build_minimal_constant_output_wasm_doc(b'Q', 120)?;
        let conv_wasm = build_converter_emitting_constant_wasm_shorthands(
            &target_wasm,
            80,
        )?;
        let fmt_node_id = alloc_node(vec![
            DcShorthand::Short(302),
            DcShorthand::Short(262),
            DcShorthand::Short(1927),
        ]);
        let conv_ent = alloc_node(vec![
            DcShorthand::Short(586),
            DcShorthand::Short(615),
            DcShorthand::Short(280),
        ]);
        let fmt_has_a = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent),
            DcShorthand::Short(595),
        ]);
        let _fmt_val = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(fmt_has_a),
            DcShorthand::Long(fmt_node_id),
        ]);
        let wasm_node = alloc_node(conv_wasm);
        let decomp_has_a = alloc_node(vec![
            DcShorthand::Short(584),
            DcShorthand::Long(conv_ent),
            DcShorthand::Short(611),
        ]);
        let decomp_val = alloc_node(vec![
            DcShorthand::Short(610),
            DcShorthand::Long(decomp_has_a),
            DcShorthand::Long(wasm_node),
        ]);
        let pin = decomp_val;

        let custom_sg = std::sync::Arc::new(SeedGraphIndex::from_nodes(nodes));
        let custom_res = DatasetRuleResolver::from_seed_graph(&custom_sg)?;
        let empty_in = DcMstring::new();

        let mut doc = DcMstring::new();
        doc.push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc.push_char(DcChar::from_u128(pin));
        doc.push_char(dc::DC_BEGIN_TYPE_DEF);
        doc.push_char(dc::DC_TYPE_STRING);
        doc.push_char(dc::DC_END_TYPE_DEF);

        // Measure standalone target execution cost:
        let mut probe_session = DcWasmSession::from_dc_wasm_with_resolver(
            &target_wasm,
            &empty_in,
            Some(pin),
            &custom_res,
        )?;
        let _ = probe_session.step()?;
        let exec_cost = probe_session.total_instructions_executed();
        assert!(exec_cost > 100);

        // Measure compilation cost via SharedWasmTaskQuota:
        let default_adm =
            SystemSupervisorAdmission::admit_execution_kernel_default()?;
        let probe_quota =
            ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota::new(
                DcWasmResourceLimits::default(),
            );
        let _ = default_adm.compile_dc_document_to_wasm_with_shared_quota(
            &doc,
            &custom_res,
            probe_quota.clone(),
        )?;
        let compile_cost = probe_quota.snapshot()?.total_instructions_executed;
        assert!(
            compile_cost > 200,
            "Both Dc 2608 unescaping and converter routine must consume metered quota (got {compile_cost})"
        );

        // 1. Budget smaller than `compile_cost`: interrupts with ComputeQuotaExhausted during compilation!
        let mut tiny_limits = DcWasmResourceLimits::default();
        tiny_limits.max_instructions_per_step = compile_cost / 2;
        let tiny_adm = SystemSupervisorAdmission::admit_execution_kernel(
            tiny_limits,
            DcWasmWatchdogLimits::default(),
        )?;
        let mut env_compile_exhaust = ExecutionContext::new(false)
            .with_system_supervisor_admission(tiny_adm)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );
        let trace_compile_exhaust = env_compile_exhaust
            .execute_dc_document_default_with_trace(
                &doc,
                &empty_in,
                &custom_res,
            )?;
        assert!(trace_compile_exhaust.result.broke_out_early);
        assert!(matches!(
            trace_compile_exhaust.events.first(),
            Some(WasmTraceEvent::ResourceInterruption {
                interruption: Some(
                    WasmResourceInterruption::ComputeQuotaExhausted { .. }
                ),
                decision: PermissionDecision::Deny,
                ..
            })
        ));

        // 2. Budget large enough for `compile_cost` OR `exec_cost` individually,
        //    but smaller than `compile_cost + exec_cost`: compilation succeeds, then
        //    target execution exhausts the shared remaining quota!
        let mid_budget = compile_cost + (exec_cost / 2);
        assert!(mid_budget > exec_cost);
        let mut mid_limits = DcWasmResourceLimits::default();
        mid_limits.max_instructions_per_step = mid_budget;
        let mid_adm = SystemSupervisorAdmission::admit_execution_kernel(
            mid_limits,
            DcWasmWatchdogLimits::default(),
        )?;
        let mut env_mid_deny = ExecutionContext::new(false)
            .with_system_supervisor_admission(mid_adm.clone())
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );
        let trace_mid_deny = env_mid_deny
            .execute_dc_document_default_with_trace(
                &doc,
                &empty_in,
                &custom_res,
            )?;
        assert!(trace_mid_deny.result.broke_out_early);

        // Granting `Allow` on `ExcessiveCompute` resumes and finishes cleanly!
        let mut env_mid_allow = ExecutionContext::new(false)
            .with_system_supervisor_admission(mid_adm)
            .with_compute_quota_grant(exec_cost)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Allow,
            );
        let trace_mid_allow = env_mid_allow
            .execute_dc_document_default_with_trace(
                &doc,
                &empty_in,
                &custom_res,
            )?;
        assert!(!trace_mid_allow.result.broke_out_early);
        assert_eq!(
            trace_mid_allow.result.output.to_dc_string()?.to_string(),
            "Q"
        );
        Ok(())
    }

    fn sl3_source_to_dc_document(
        src: &str,
        sg: &SeedGraphIndex,
        pin: Option<u128>,
    ) -> Result<DcMstring> {
        let tokens =
            ctb_formats_dcstring::stagel3::seed_linker::sl3_to_dc_tokens_with_seed_graph(
                src, sg,
            )?;
        let mut dc_str = DcString::with_capacity(tokens.len().saturating_mul(4));
        if let Some(pin_id) = pin {
            dc_str.push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
            dc_str.push(DcChar::from_u128(pin_id));
        }
        for tok in tokens {
            dc_str.push(tok);
        }
        DcMstring::from_dc_str(&dc_str)
    }

    fn admit_high_budget_execution_kernel() -> Result<SystemSupervisorAdmission> {
        SystemSupervisorAdmission::admit_cli_execution_kernel()
    }

    #[crate::ctb_test]
    fn test_dc_1936_executes_real_stagel3_via_pinned_self_hosted_wat_converter()
    -> Result<()> {
        use ctb_formats_dcstring::stagel_dc_alloc::{
            DiscoveredStagelFile, ImplStage, reconcile_stagel_and_csv_in_memory,
        };
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;
        use std::collections::BTreeSet;
        use std::path::PathBuf;

        // 1. Real StageL 3 document importing `std/bytes` & `std/math` and using n-ary tuples
        //    + positional `struct`s, compiled via `stagel3_to_wasm.wat` in `SeedGraphIndex::shared()`.
        let shared_sg = SeedGraphIndex::shared()?;
        let shared_res = DatasetRuleResolver::from_seed_graph(shared_sg)?;
        let head_pin = shared_sg.system_block_most_recent_node_id().unwrap();
        let high_adm = admit_high_budget_execution_kernel()?;

        let std_app_src = r#"
use std/bytes: explode implode ::.
use std/math: canAdd ::.

struct pairTag :: l/u8 u32.

v/l/u8/step l/u8/input
    do
        let tup: l/l/u8 b ::/parts tupNew: explode b"," /input canAdd u32:10 u32:20 ::.
        if not field /parts u32:1 fault nop.
        let pairTag/pt structNew pairTag: implode b":" field /parts u32:0 u32:1 ::.
        let pairTag/pt2 with /pt u32:1 u32:2.
        if ne field /pt2 u32:1 u32:2 fault nop.
        concat field /pt2 u32:0 b"!ok".
    ::.
"#;
        let pinned_std_doc =
            sl3_source_to_dc_document(std_app_src, shared_sg, Some(head_pin))?;
        let unpinned_std_doc =
            sl3_source_to_dc_document(std_app_src, shared_sg, None)?;
        let mut std_in = DcMstring::new();
        for ch in "one,two,three".chars() {
            std_in.push_char(DcChar::from(ch));
        }

        let mut env_std_pinned = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_std_pinned = env_std_pinned
            .execute_dc_document_default_with_trace(
                &pinned_std_doc,
                &std_in,
                &shared_res,
            )?;
        assert!(
            !trace_std_pinned.result.broke_out_early,
            "unexpected early interruption on pinned std execution: {:?}",
            trace_std_pinned.events
        );
        assert_eq!(
            trace_std_pinned.result.output.to_dc_string()?.to_string(),
            "one:two:three!ok"
        );

        let mut env_std_unpinned = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_std_unpinned = env_std_unpinned
            .execute_dc_document_default_with_trace(
                &unpinned_std_doc,
                &std_in,
                &shared_res,
            )?;
        assert!(
            !trace_std_unpinned.result.broke_out_early,
            "unexpected early interruption on unpinned std execution: {:?}",
            trace_std_unpinned.events
        );
        assert_eq!(
            trace_std_unpinned.result.output.to_dc_string()?.to_string(),
            "one:two:three!ok"
        );

        // 2. Multi-revision historical pin fidelity across StageL 3 dependencies (`Dc 591` & `Dc 592`):
        let dep_v1_src = "ns demo/ver.\n\npub v/l/u8/tag :: b\"v1:\".\npub v/l/u8/onlyInV1 :: b\"legacy:\".\n";
        let mut files_v1 = vec![DiscoveredStagelFile {
            stage: ImplStage::Stage1,
            disk_path: PathBuf::from("demo/ver.sl3"),
            repo_rel_path:
                "src/formats/dcdata/data-src/stage_1_impls/demo/ver.sl3"
                    .to_owned(),
            psr4_rel_path: "demo/ver.sl3".to_owned(),
            raw_bytes: dep_v1_src.as_bytes().to_vec(),
            source: dep_v1_src.to_owned(),
        }];
        let (v1_nodes, _, _) = reconcile_stagel_and_csv_in_memory(
            Vec::new(),
            &mut files_v1,
            &[],
            &[],
            &BTreeSet::new(),
            None,
        )?;
        let v1_len = v1_nodes.len();
        assert!(v1_len > 0);

        // Revision 2 supersedes `tag` (`b"v1:"` -> `b"v2:"`) via `Dc 591` and retracts `onlyInV1` via `Dc 592`.
        let dep_v2_src = files_v1[0].source.replace("b\"v1:\"", "b\"v2:\"");
        let v2_lines: Vec<&str> = dep_v2_src.lines().collect();
        let only_idx = v2_lines
            .iter()
            .position(|l| l.contains("onlyInV1"))
            .expect("onlyInV1 line must exist");
        let dc_comment_idx = only_idx.checked_sub(1).unwrap();
        assert!(v2_lines[dc_comment_idx].trim_start().starts_with(": @dc "));
        let mut trimmed_v2 = String::new();
        for (i, line) in v2_lines.iter().enumerate() {
            if i != dc_comment_idx && i != only_idx {
                trimmed_v2.push_str(line);
                trimmed_v2.push('\n');
            }
        }
        let mut files_v2 = vec![DiscoveredStagelFile {
            stage: ImplStage::Stage1,
            disk_path: PathBuf::from("demo/ver.sl3"),
            repo_rel_path:
                "src/formats/dcdata/data-src/stage_1_impls/demo/ver.sl3"
                    .to_owned(),
            psr4_rel_path: "demo/ver.sl3".to_owned(),
            raw_bytes: trimmed_v2.as_bytes().to_vec(),
            source: trimmed_v2,
        }];
        let (v2_nodes, _, v2_stats) = reconcile_stagel_and_csv_in_memory(
            v1_nodes,
            &mut files_v2,
            &[],
            &[],
            &BTreeSet::new(),
            None,
        )?;
        assert!(v2_stats.nodes_superseded >= 1);
        assert!(v2_stats.nodes_retracted >= 1);

        let sys_start =
            ctb_formats_dcstring::global_graph_layout::SYSTEM_REGION_START;
        let id_shift = (head_pin + 1) - sys_start;
        let mut combined_nodes = shared_sg.nodes().to_vec();
        let mut pin_v1 = 0u128;
        let mut pin_v2 = 0u128;
        for (idx, mut node) in v2_nodes.into_iter().enumerate() {
            node.id += id_shift;
            for tok in &mut node.tokens {
                if let DcShorthand::Long(lid) = tok
                    && *lid >= sys_start
                {
                    *lid += id_shift;
                }
            }
            if idx + 1 == v1_len {
                pin_v1 = node.id;
            }
            pin_v2 = node.id;
            combined_nodes.push(node);
        }
        assert!(pin_v1 > head_pin && pin_v2 > pin_v1);

        let v1_only_nodes: Vec<ctb_formats_dcstring::LedgerNode> =
            combined_nodes
                .iter()
                .filter(|n| n.id <= pin_v1)
                .cloned()
                .collect();
        let snapshot_v1 = SeedGraphIndex::from_nodes(v1_only_nodes);
        let custom_sg =
            std::sync::Arc::new(SeedGraphIndex::from_nodes(combined_nodes));
        let custom_res = DatasetRuleResolver::from_seed_graph(&custom_sg)?;

        let caller_tag_src =
            "use demo/ver: tag ::.\nv/l/u8/step l/u8/inp :: concat tag /inp.\n";
        let raw_tag_body =
            sl3_source_to_dc_document(caller_tag_src, &snapshot_v1, None)?;

        let mut doc_tag_v1_str = DcString::new();
        doc_tag_v1_str
            .push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_tag_v1_str.push(DcChar::from_u128(pin_v1));
        doc_tag_v1_str.push_dc_str(&raw_tag_body.to_dc_string()?);
        let doc_tag_v1 = DcMstring::from_dc_str(&doc_tag_v1_str)?;

        let mut doc_tag_v2_str = DcString::new();
        doc_tag_v2_str
            .push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_tag_v2_str.push(DcChar::from_u128(pin_v2));
        doc_tag_v2_str.push_dc_str(&raw_tag_body.to_dc_string()?);
        let doc_tag_v2 = DcMstring::from_dc_str(&doc_tag_v2_str)?;

        let mut world_in = DcMstring::new();
        for ch in "world".chars() {
            world_in.push_char(DcChar::from(ch));
        }

        let mut env_v1 = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_v1 = env_v1.execute_dc_document_default_with_trace(
            &doc_tag_v1,
            &world_in,
            &custom_res,
        )?;
        assert_eq!(
            trace_v1.result.output.to_dc_string()?.to_string(),
            "v1:world"
        );

        let mut env_v2 = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_v2 = env_v2.execute_dc_document_default_with_trace(
            &doc_tag_v2,
            &world_in,
            &custom_res,
        )?;
        assert_eq!(
            trace_v2.result.output.to_dc_string()?.to_string(),
            "v2:world"
        );

        // Calling `onlyInV1` succeeds when pinned to `pin_v1` and fails cleanly when pinned to `pin_v2`:
        let caller_legacy_src =
            "use demo/ver: onlyInV1 ::.\nv/l/u8/step l/u8/inp :: concat onlyInV1 /inp.\n";
        let raw_legacy_body =
            sl3_source_to_dc_document(caller_legacy_src, &snapshot_v1, None)?;

        let mut doc_legacy_v1_str = DcString::new();
        doc_legacy_v1_str
            .push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_legacy_v1_str.push(DcChar::from_u128(pin_v1));
        doc_legacy_v1_str.push_dc_str(&raw_legacy_body.to_dc_string()?);
        let doc_legacy_v1 = DcMstring::from_dc_str(&doc_legacy_v1_str)?;

        let mut env_leg_v1 = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_leg_v1 = env_leg_v1.execute_dc_document_default_with_trace(
            &doc_legacy_v1,
            &world_in,
            &custom_res,
        )?;
        assert_eq!(
            trace_leg_v1.result.output.to_dc_string()?.to_string(),
            "legacy:world"
        );

        let mut doc_legacy_v2_str = DcString::new();
        doc_legacy_v2_str
            .push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        doc_legacy_v2_str.push(DcChar::from_u128(pin_v2));
        doc_legacy_v2_str.push_dc_str(&raw_legacy_body.to_dc_string()?);
        let doc_legacy_v2 = DcMstring::from_dc_str(&doc_legacy_v2_str)?;

        let mut env_leg_v2 = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm);
        assert!(
            env_leg_v2
                .execute_dc_document_default_with_trace(
                    &doc_legacy_v2,
                    &world_in,
                    &custom_res,
                )
                .is_err(),
            "Retracted StageL dependency routine must be rejected when pinned to pin_v2"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_1936_real_stagel3_shared_quota_across_compile_and_exec()
    -> Result<()> {
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;

        let shared_sg = SeedGraphIndex::shared()?;
        let shared_res = DatasetRuleResolver::from_seed_graph(shared_sg)?;
        let head_pin = shared_sg.system_block_most_recent_node_id().unwrap();

        let spin_sl3 = r#"
v/l/u8/step l/u8/inp
    do
        let u32/i u32:0.
        let u64/acc u64:0.
        while lt /i u32:600 do
            set /acc wrapAdd /acc conv /i u64.
            set /i add /i u32:1.
        ::.
        if eq /acc u64:0 fault nop.
        concat /inp b":done".
    ::.
"#;
        let doc = sl3_source_to_dc_document(spin_sl3, shared_sg, Some(head_pin))?;
        let mut input = DcMstring::new();
        for ch in "quota".chars() {
            input.push_char(DcChar::from(ch));
        }

        // Measure `stagel3_to_wasm.wat` compilation cost and resulting target `.WASM` execution cost:
        let high_adm = admit_high_budget_execution_kernel()?;
        let mut probe_limits = DcWasmResourceLimits::default();
        probe_limits.max_instructions_per_step = 100_000_000;
        let probe_quota =
            ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota::new(
                probe_limits,
            );
        let (compiled_wasm, compiled_pin) = high_adm
            .compile_dc_document_to_wasm_with_shared_quota(
                &doc,
                &shared_res,
                probe_quota.clone(),
            )?;
        assert_eq!(compiled_pin, Some(head_pin));
        let compile_cost = probe_quota.snapshot()?.total_instructions_executed;
        assert!(compile_cost > 10_000);

        let mut probe_session = DcWasmSession::from_dc_wasm_with_resolver(
            &compiled_wasm,
            &input,
            Some(head_pin),
            &shared_res,
        )?;
        let _ = probe_session.step()?;
        let exec_cost = probe_session.total_instructions_executed();
        assert!(exec_cost > 1_000);

        // 1. Budget smaller than `compile_cost`: interrupts with ComputeQuotaExhausted during compilation!
        let mut compile_cap_limits = DcWasmResourceLimits::default();
        compile_cap_limits.max_instructions_per_step = compile_cost / 2;
        let compile_cap_adm = SystemSupervisorAdmission::admit_execution_kernel(
            compile_cap_limits,
            DcWasmWatchdogLimits::default(),
        )?;
        let mut env_compile_deny = ExecutionContext::new(false)
            .with_system_supervisor_admission(compile_cap_adm)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );
        let trace_compile_deny = env_compile_deny
            .execute_dc_document_default_with_trace(&doc, &input, &shared_res)?;
        assert!(trace_compile_deny.result.broke_out_early);
        assert!(matches!(
            trace_compile_deny.events.first(),
            Some(WasmTraceEvent::ResourceInterruption {
                interruption: Some(
                    WasmResourceInterruption::ComputeQuotaExhausted { .. }
                ),
                decision: PermissionDecision::Deny,
                ..
            })
        ));

        // 2. Budget covering compilation alone (`compile_cost + exec_cost / 2`), but less than
        //    `compile_cost + exec_cost`: compilation finishes, then target `.WASM` execution
        //    exhausts the shared remaining budget!
        let mid_budget = compile_cost + (exec_cost / 2);
        assert!(mid_budget > compile_cost && mid_budget > exec_cost);
        let mut mid_limits = DcWasmResourceLimits::default();
        mid_limits.max_instructions_per_step = mid_budget;
        let mid_adm = SystemSupervisorAdmission::admit_execution_kernel(
            mid_limits,
            DcWasmWatchdogLimits::default(),
        )?;
        let mut env_mid_deny = ExecutionContext::new(false)
            .with_system_supervisor_admission(mid_adm.clone())
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );
        let trace_mid_deny = env_mid_deny
            .execute_dc_document_default_with_trace(&doc, &input, &shared_res)?;
        assert!(trace_mid_deny.result.broke_out_early);

        // Granting `Allow` on `ExcessiveCompute` resumes and finishes target execution cleanly!
        let mut env_mid_allow = ExecutionContext::new(false)
            .with_system_supervisor_admission(mid_adm)
            .with_compute_quota_grant(exec_cost)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Allow,
            );
        let trace_mid_allow = env_mid_allow
            .execute_dc_document_default_with_trace(&doc, &input, &shared_res)?;
        assert!(!trace_mid_allow.result.broke_out_early);
        assert_eq!(
            trace_mid_allow.result.output.to_dc_string()?.to_string(),
            "quota:done"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_1936_executes_real_stager3_via_pinned_f1681_and_f1680_chain()
    -> Result<()> {
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;

        let shared_sg = SeedGraphIndex::shared()?;
        let shared_res = DatasetRuleResolver::from_seed_graph(shared_sg)?;
        let head_pin = shared_sg.system_block_most_recent_node_id().unwrap();

        let stager_adm = admit_high_budget_execution_kernel()?;

        let sr3_src = r#"ns app/kernel_stager.

brand u32/tagId.

struct jobSpec
    tagId/id.
    i32/offset.

enum jobMode
    echo.
    i32/boost.

v/result/i32/evalJob jobSpec/spec jobMode/mode
    const i32/base conv? unwrap field /spec id i32.
    const i32/sum add? /base field /spec offset.
    match /mode
        case echo result/i32:ok /sum.
        case boost i32/k add /sum /k.
    ::.

pub v/l/u8/step l/u8/input
    const jobSpec/spec structNew jobSpec:
        offset i32:-3
        id wrap u32:40 tagId
    ::.
    const i32/score unwrapOr evalJob /spec jobMode:boost i32:5 i32:0.
    const s/joined implode ":" explode "," "left,right".
    if and eq /score i32:42 eq /joined "left:right"
        concat b"sr3:" /input
        b"bad".
"#;
        let sr_tokens =
            ctb_formats_dcstring::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_src, shared_sg,
            )?;
        let build_sr_doc = |pin: Option<u128>| -> Result<DcMstring> {
            let mut dc_str =
                DcString::with_capacity(sr_tokens.len().saturating_mul(4));
            if let Some(pin_id) = pin {
                dc_str.push(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
                dc_str.push(DcChar::from_u128(pin_id));
            }
            for &tok in &sr_tokens {
                dc_str.push(tok);
            }
            DcMstring::from_dc_str(&dc_str)
        };
        let pinned_sr_doc = build_sr_doc(Some(head_pin))?;
        let unpinned_sr_doc = build_sr_doc(None)?;

        let mut in_doc = DcMstring::new();
        for ch in "payload".chars() {
            in_doc.push_char(DcChar::from(ch));
        }

        let mut env_pinned = ExecutionContext::new(false)
            .with_system_supervisor_admission(stager_adm.clone());
        let trace_pinned = env_pinned.execute_dc_document_default_with_trace(
            &pinned_sr_doc,
            &in_doc,
            &shared_res,
        )?;
        assert!(
            !trace_pinned.result.broke_out_early,
            "pinned StageR 3 execution broke out early (op={:?}, req_cost={:?}); events: {:?}",
            trace_pinned.last_interrupted_op_ordinal,
            trace_pinned.last_interrupted_op_required_cost,
            trace_pinned.events,
        );
        assert_eq!(
            trace_pinned.result.output.to_dc_string()?.to_string(),
            "sr3:payload"
        );

        // Second execution exercises unpinned head resolution (and verifies cached StageL
        // converter quota accounting under `stager_adm`).
        let mut env_unpinned = ExecutionContext::new(false)
            .with_system_supervisor_admission(stager_adm);
        let trace_unpinned = env_unpinned
            .execute_dc_document_default_with_trace(
                &unpinned_sr_doc,
                &in_doc,
                &shared_res,
            )?;
        assert!(
            !trace_unpinned.result.broke_out_early,
            "unpinned StageR 3 execution broke out early (op={:?}, req_cost={:?}); events: {:?}",
            trace_unpinned.last_interrupted_op_ordinal,
            trace_unpinned.last_interrupted_op_required_cost,
            trace_unpinned.events,
        );
        assert_eq!(
            trace_unpinned.result.output.to_dc_string()?.to_string(),
            "sr3:payload"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_option_b_dc_1939_and_dc_1940_execution_context_policies()
    -> Result<()> {
        use ctb_formats_dcstring::converters::encode_dc_string_literal;
        use ctb_formats_dcstring::converters::wat::{
            wat_to_dcmstring, wat_to_wasm,
        };
        use ctb_formats_dcstring::syntax::default_syntax_resolver;

        let resolver = default_syntax_resolver();
        let sg = resolver.seed_graph().context("Missing seed graph")?;
        let watermark = sg
            .system_block_most_recent_node_id()
            .context("Missing seed graph watermark")?;

        let mut wm_req = DcMstring::new();
        wm_req.push_char(dc::DC_MINIMAL_SYSTEM_GRAPH_QUERY);
        wm_req.push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        wm_req.push_char(dc::DC_WILDCARD_PATTERN);

        // Build a looping pure child WASM module so quota exhaustion deterministically occurs inside Dc 1940.
        let child_wat = r#"(module
          (memory (export "memory") 1)
          (func (export "step") (result i32)
            (local $i i32)
            (block $done
              (loop $lp
                (br_if $done (i32.ge_u (local.get $i) (i32.const 400)))
                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br $lp)))
            (i32.store (i32.const 8) (i32.load (i32.const 0)))
            (i32.store (i32.const 12) (i32.load (i32.const 4)))
            (i32.const 0)))"#;
        let child_wasm_doc = wat_to_dcmstring(child_wat)?;
        let lit_in = encode_dc_string_literal(&[
            DcChar::from('H'),
            DcChar::from('i'),
        ]);
        let mut pure_req = DcMstring::new();
        pure_req.push_char(dc::DC_EXECUTE_PURE_WASM_PROGRAM);
        for ch in child_wasm_doc.to_dc_string()?.chars() {
            pure_req.push_char(ch);
        }
        for ch in lit_in.to_dc_string()?.chars() {
            pure_req.push_char(ch);
        }

        let mut combined_input = wm_req.clone();
        let wm_len = combined_input.payload_bytes().len();
        for ch in pure_req.to_dc_string()?.chars() {
            combined_input.push_char(ch);
        }
        let pure_len =
            combined_input.payload_bytes().len().saturating_sub(wm_len);

        let parent_wat = format!(
            r#"(module
              (memory (export "memory") 1)
              (global $__heap_ptr (mut i32) (i32.const 256))
              (export "__heap_ptr" (global $__heap_ptr))
              (func $__ctb_capability_yield (export "__ctb_capability_yield"))
              (func $copy (param $src i32) (param $dst i32) (param $len i32)
                (local $i i32)
                (block $done
                  (loop $lp
                    (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
                    (i32.store8
                      (i32.add (local.get $dst) (local.get $i))
                      (i32.load8_u (i32.add (local.get $src) (local.get $i))))
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br $lp))))
              (func (export "step") (result i32)
                (local $in_ptr i32)
                (local $in_len i32)
                (local $p1 i32)
                (local $l1 i32)
                (local $p2 i32)
                (local $l2 i32)
                (local $out_ptr i32)
                (local.set $in_ptr (i32.load (i32.const 0)))
                (local.set $in_len (i32.load (i32.const 4)))
                (global.set $__heap_ptr
                  (i32.and
                    (i32.add (i32.add (local.get $in_ptr) (local.get $in_len)) (i32.const 63))
                    (i32.const -8)))
                (i32.store (i32.const 8) (local.get $in_ptr))
                (i32.store (i32.const 12) (i32.const {wm_len}))
                (call $__ctb_capability_yield)
                (local.set $p1 (i32.load (i32.const 0)))
                (local.set $l1 (i32.load (i32.const 4)))
                (i32.store (i32.const 8) (i32.add (local.get $in_ptr) (i32.const {wm_len})))
                (i32.store (i32.const 12) (i32.const {pure_len}))
                (call $__ctb_capability_yield)
                (local.set $p2 (i32.load (i32.const 0)))
                (local.set $l2 (i32.load (i32.const 4)))
                (local.set $out_ptr (global.get $__heap_ptr))
                (call $copy (local.get $p1) (local.get $out_ptr) (local.get $l1))
                (call $copy (local.get $p2) (i32.add (local.get $out_ptr) (local.get $l1)) (local.get $l2))
                (i32.store (i32.const 8) (local.get $out_ptr))
                (i32.store (i32.const 12) (i32.add (local.get $l1) (local.get $l2)))
                (i32.const 0)))"#,
        );
        let parent_wasm = wat_to_wasm(&parent_wat)?;

        // 1. Default policy allows both Dc 1939 and Dc 1940.
        let mut expected_allow = DcMstring::new();
        expected_allow
            .push_char(dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
        expected_allow.push_char(DcChar::from_u128(watermark));
        for ch in lit_in.to_dc_string()?.chars() {
            expected_allow.push_char(ch);
        }
        let mut s_ok = DcWasmSession::from_wasm_bytes_with_limits(
            &parent_wasm,
            &combined_input,
            Some(watermark),
            DcWasmResourceLimits::default(),
        )?;
        let mut ctx_ok = ExecutionContext::new(false);
        let tr_ok = ctx_ok
            .execute_wasm_session_default_with_resolver_and_trace(
                &mut s_ok,
                &combined_input,
                resolver.as_ref(),
            )?;
        assert!(!tr_ok.result.broke_out_early);
        assert_eq!(tr_ok.result.output, expected_allow);
        let cap_privs: Vec<(&Privilege, PermissionDecision)> = tr_ok
            .events
            .iter()
            .filter_map(|e| match e {
                WasmTraceEvent::CapabilityInteraction {
                    privilege,
                    decision,
                    ..
                } => Some((privilege, *decision)),
                _ => None,
            })
            .collect();
        assert_eq!(
            cap_privs,
            vec![
                (&Privilege::SystemGraphQuery, PermissionDecision::Allow),
                (&Privilege::ExecutePureWasm, PermissionDecision::Allow),
            ]
        );

        // 2. Explicit Deny override on SystemGraphQuery returns Dc 518 without executing query.
        let mut s_deny = DcWasmSession::from_wasm_bytes_with_limits(
            &parent_wasm,
            &combined_input,
            Some(watermark),
            DcWasmResourceLimits::default(),
        )?;
        let mut ctx_deny = ExecutionContext::new(false)
            .with_privilege_decision(
                Privilege::SystemGraphQuery,
                PermissionDecision::Deny,
            );
        let tr_deny = ctx_deny
            .execute_wasm_session_default_with_resolver_and_trace(
                &mut s_deny,
                &combined_input,
                resolver.as_ref(),
            )?;
        let mut expected_deny = DcMstring::new();
        expected_deny.push_char(dc::DC_BOOLEAN_FALSE);
        for ch in lit_in.to_dc_string()?.chars() {
            expected_deny.push_char(ch);
        }
        assert_eq!(tr_deny.result.output, expected_deny);

        // 3. Exhausting shared compute quota inside Dc 1940 child WASM records ResourceInterruption.
        let tiny_limits = DcWasmResourceLimits::default()
            .with_max_instructions_per_step(5);
        let tiny_quota =
            ctb_formats_dcstring::converters::wasm::SharedWasmTaskQuota::new(
                tiny_limits,
            );
        let mut s_quota = DcWasmSession::from_wasm_bytes_with_limits(
            &parent_wasm,
            &combined_input,
            Some(watermark),
            tiny_limits,
        )?;
        s_quota.set_shared_task_quota(tiny_quota)?;
        let mut ctx_quota = ExecutionContext::new(false)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Deny,
            );
        let tr_quota = ctx_quota
            .execute_wasm_session_default_with_resolver_and_trace(
                &mut s_quota,
                &combined_input,
                resolver.as_ref(),
            )?;
        assert!(tr_quota.result.broke_out_early);
        assert!(tr_quota.events.iter().any(|e| matches!(
            e,
            WasmTraceEvent::ResourceInterruption {
                decision: PermissionDecision::Deny,
                interruption: Some(_),
                ..
            }
        )));
        assert!(tr_quota.ensure_not_denied().is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_2608_graph_stager_and_pre_migration_fallback() -> Result<()> {
        use ctb_formats_dcstring::converters::encode_dc_string_literal;
        use ctb_formats_dcstring::syntax::DatasetRuleResolver;

        let shared_sg = SeedGraphIndex::shared()?;
        let dc2608_sh = DcShorthand::from_global_id(
            dc::DC_CONVERT_DC_STRING_TO_WASM.to_u128(),
        );
        let mut cur_sg = std::sync::Arc::clone(shared_sg);
        let mut pre_pin: Option<u128> = None;
        while let Some((val_id, _)) =
            cur_sg.resolve_stagel_decomposition_tokens(dc2608_sh)?
        {
            let mut min_id = val_id;
            for pred in [
                dc::DC_CANONICAL_DECOMPOSITION,
                dc::DC_DECOMPOSITION,
                dc::DC_SEMANTIC_DECOMPOSITION,
            ] {
                let pred_sh = DcShorthand::from_global_id(pred.to_u128());
                for entry in cur_sg.get_multi_properties(dc2608_sh, pred_sh) {
                    min_id = min_id.min(entry.has_a_id);
                }
            }
            let next_pin = min_id
                .checked_sub(1)
                .context("Dc 2608 pre-migration pin underflow")?;
            pre_pin = Some(next_pin);
            cur_sg = SeedGraphIndex::load_at_most_recent_from_asset_bundle(
                next_pin,
            )?;
        }
        let pre_pin = pre_pin.context(
            "Expected active StageR 3 decomposition on Dc 2608 at HEAD",
        )?;
        assert!(
            cur_sg
                .resolve_stagel_decomposition_tokens(dc2608_sh)?
                .is_none()
        );

        let tiny_sl3 = "v/l/u8/step l/u8/input :: concat b\"ok:\" /input.\n";
        let doc_unpinned =
            sl3_source_to_dc_document(tiny_sl3, shared_sg, None)?;
        let doc_pre_pinned =
            sl3_source_to_dc_document(tiny_sl3, &cur_sg, Some(pre_pin))?;
        let doc_chars: Vec<DcChar> =
            doc_unpinned.to_dc_string()?.chars().collect();
        let encoded_doc_lit = encode_dc_string_literal(&doc_chars);
        let mut call_tokens = vec![dc::DC_CONVERT_DC_STRING_TO_WASM];
        for ch in encoded_doc_lit.to_dc_string()?.chars() {
            call_tokens.push(ch);
        }

        let mut input_hi = DcMstring::new();
        for ch in "hi".chars() {
            input_hi.push_char(DcChar::from(ch));
        }

        let high_adm = admit_high_budget_execution_kernel()?;
        assert!(
            std::ptr::eq(
                ctb_formats_dcstring::syntax::default_syntax_resolver()
                    .seed_graph()
                    .context("Missing default seed graph")?,
                shared_sg.as_ref(),
            ),
            "default_syntax_resolver() must share the process-wide SeedGraphIndex::shared() Arc without cloning"
        );
        let head_res = DatasetRuleResolver::from_seed_graph(shared_sg)?;
        let pre_res = DatasetRuleResolver::from_seed_graph(&cur_sg)?;

        // 1. HEAD pin compiles Dc 2608 via its StageR 3 graph implementation.
        assert!(
            shared_sg
                .resolve_stagel_decomposition_tokens(dc2608_sh)?
                .is_some(),
            "HEAD pin must have an active StageR 3 decomposition on Dc 2608"
        );
        let mut env_head = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_head = env_head.execute_dc_document_default_with_trace(
            &doc_unpinned,
            &input_hi,
            &head_res,
        )?;
        assert!(
            !trace_head.result.broke_out_early,
            "HEAD graph StageR 3 Dc 2608 execution broke out early: {:?}",
            trace_head.events
        );
        assert_eq!(
            trace_head.result.output.to_dc_string()?.to_string(),
            "ok:hi"
        );

        // 2. Pre-migration pin returns None from graph StageR 3 lookup and
        //    succeeds through the historical host Rust fallback.
        assert!(
            ctb_formats_dcstring::converters::wasm::run_graph_stager_dc_2608_with_quota(
                &call_tokens,
                &pre_res,
                None,
            )?
            .is_none(),
            "Pre-migration pin must return None so historical Rust fallback handles Dc 2608"
        );
        let mut env_pre_res = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_pre_res = env_pre_res
            .execute_dc_document_default_with_trace(
                &doc_unpinned,
                &input_hi,
                &pre_res,
            )?;
        assert!(
            !trace_pre_res.result.broke_out_early,
            "Pre-migration resolver execution broke out early: {:?}",
            trace_pre_res.events
        );
        assert_eq!(
            trace_pre_res.result.output.to_dc_string()?.to_string(),
            "ok:hi"
        );

        let mut env_pre_doc = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm.clone());
        let trace_pre_doc = env_pre_doc
            .execute_dc_document_default_with_trace(
                &doc_pre_pinned,
                &input_hi,
                &head_res,
            )?;
        assert!(
            !trace_pre_doc.result.broke_out_early,
            "Pre-migration Dc 641 pinned document execution broke out early: {:?}",
            trace_pre_doc.events
        );
        assert_eq!(
            trace_pre_doc.result.output.to_dc_string()?.to_string(),
            "ok:hi"
        );

        let mut env_pre_doc_default = ExecutionContext::new(false)
            .with_system_supervisor_admission(high_adm);
        let trace_pre_doc_default = env_pre_doc_default
            .execute_dc_document_default_with_trace(
                &doc_pre_pinned,
                &input_hi,
                ctb_formats_dcstring::syntax::default_syntax_resolver()
                    .as_ref(),
            )?;
        assert!(
            !trace_pre_doc_default.result.broke_out_early,
            "Pre-migration Dc 641 pinned document execution against default_syntax_resolver broke out early: {:?}",
            trace_pre_doc_default.events
        );
        assert_eq!(
            trace_pre_doc_default.result.output.to_dc_string()?.to_string(),
            "ok:hi"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_substitute_cross_backend_trace_replay_parity() -> Result<()> {
        use ctb_formats_dcstring::converters::wasm::{
            DcWasmExecutionBackend, compile_system_subject_wasm_bytes,
        };
        use ctb_formats_dcstring::stagel3::seed_linker::{
            bundle_dc_module_for_wasm_converter,
            sl3_to_dc_tokens_with_seed_graph,
        };
        use ctb_formats_dcstring::syntax::default_syntax_resolver;
        use ctb_formats_dcstring::{DcWasmResourceLimits, WasmResourceInterruption};

        ctb_wasm_substitutes::ensure_registered();
        let resolver = default_syntax_resolver();
        let sg = SeedGraphIndex::shared()?;
        let f1680_wasm = compile_system_subject_wasm_bytes(
            DcShorthand::Format(1680),
            resolver.as_ref(),
        )?;

        let tiny_sl3 = "v/l/u8/step l/u8/input :: concat b\"ok:\" /input.\n";
        let entry_tokens = sl3_to_dc_tokens_with_seed_graph(tiny_sl3, sg)?;
        let bundled = bundle_dc_module_for_wasm_converter(
            &entry_tokens,
            sg,
            resolver.as_ref(),
        )?;
        let mut input_doc = DcMstring::new();
        for &tok in &bundled {
            input_doc.push_char(tok);
        }

        let base_limits = DcWasmResourceLimits::default()
            .with_max_instructions_per_step(50_000_000)
            .with_max_memory_pages(512)
            .with_max_step_output_bytes(8 * 1024 * 1024)
            .with_max_session_output_bytes(16 * 1024 * 1024);

        let mut measure_session =
            DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                &f1680_wasm,
                &input_doc,
                None,
                base_limits,
                DcWasmExecutionBackend::Auto,
            )?;
        assert!(measure_session.used_native_substitute());
        let mut measure_ctx = ExecutionContext::new(false);
        let full_trace = measure_ctx.execute_wasm_session_default_with_trace(
            &mut measure_session,
            &input_doc,
        )?;
        let full_total = measure_session.total_instructions_executed();
        assert!(full_total > 10 && !full_trace.result.broke_out_early);

        let tight_quota = (full_total / 2).max(1);
        let tight_limits =
            base_limits.with_max_instructions_per_step(tight_quota);
        let grant = full_total.saturating_mul(2);

        // 1. Record trace on NativeSubstitute (`Auto`), then replay on `ForceTinyWasm`.
        let mut s_native =
            DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                &f1680_wasm,
                &input_doc,
                None,
                tight_limits,
                DcWasmExecutionBackend::Auto,
            )?;
        assert!(s_native.used_native_substitute());
        let mut ctx_native = ExecutionContext::new(false)
            .with_compute_quota_grant(grant)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Allow,
            );
        let trace_native = ctx_native.execute_wasm_session_default_with_trace(
            &mut s_native,
            &input_doc,
        )?;
        assert!(!trace_native.result.broke_out_early);
        assert!(trace_native.events.iter().any(|ev| {
            matches!(
                ev,
                WasmTraceEvent::ResourceInterruption {
                    interruption: Some(
                        WasmResourceInterruption::ComputeQuotaExhausted { .. }
                    ),
                    decision: PermissionDecision::Allow,
                    ..
                }
            )
        }));

        let mut s_replay_tw =
            DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                &f1680_wasm,
                &input_doc,
                None,
                tight_limits,
                DcWasmExecutionBackend::ForceTinyWasm,
            )?;
        assert!(!s_replay_tw.used_native_substitute());
        let mut replay_tw_ctx =
            ExecutionContext::new(false).with_compute_quota_grant(grant);
        let replayed_on_tw = replay_tw_ctx.replay_wasm_session_trace(
            &mut s_replay_tw,
            &input_doc,
            &trace_native,
        )?;
        assert_eq!(replayed_on_tw, trace_native);

        // 2. Record trace on `ForceTinyWasm`, verify equality, and replay on `Auto`.
        let mut s_tw = DcWasmSession::from_wasm_bytes_with_limits_and_backend(
            &f1680_wasm,
            &input_doc,
            None,
            tight_limits,
            DcWasmExecutionBackend::ForceTinyWasm,
        )?;
        assert!(!s_tw.used_native_substitute());
        let mut ctx_tw = ExecutionContext::new(false)
            .with_compute_quota_grant(grant)
            .with_privilege_decision(
                Privilege::ExcessiveCompute,
                PermissionDecision::Allow,
            );
        let trace_tw = ctx_tw.execute_wasm_session_default_with_trace(
            &mut s_tw,
            &input_doc,
        )?;
        assert_eq!(trace_tw, trace_native);

        let mut s_replay_native =
            DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                &f1680_wasm,
                &input_doc,
                None,
                tight_limits,
                DcWasmExecutionBackend::Auto,
            )?;
        assert!(s_replay_native.used_native_substitute());
        let mut replay_native_ctx =
            ExecutionContext::new(false).with_compute_quota_grant(grant);
        let replayed_on_native = replay_native_ctx.replay_wasm_session_trace(
            &mut s_replay_native,
            &input_doc,
            &trace_tw,
        )?;
        assert_eq!(replayed_on_native, trace_tw);

        Ok(())
    }
}
