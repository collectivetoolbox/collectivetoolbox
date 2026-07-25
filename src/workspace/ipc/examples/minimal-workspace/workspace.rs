//! Workspace logic for the minimal workspace example.
//!
//! The reusable IPC plumbing (listener, connection acceptance, message + spawn
//! routing, shutdown handling) lives in `ctb_workspace_ipc::workspace_runner`.
//! This module contains only example-specific policy:
//! - which subprocesses to spawn at startup
//! - how to evaluate spawn requests from children
//! - how to print demo output

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

import_all_ipc_client_ext_traits!();

use std::path::PathBuf;

use anyhow::Result;
use ctb_workspace_ipc::auth::capability::CapabilitySet;

use ctb_utilities::ipc::service_traits::renderer::{
    RenderMode, RenderSettings, RenderTarget,
};
#[cfg(unix)]
use ctb_workspace_ipc::services::parent::ParentMessageEvent;
use ctb_workspace_ipc::workspace_runner::workspace_runtime::{
    ResolvedParentMessage, WorkspaceRuntime,
};
use ctb_workspace_ipc::workspace_runner::{
    SpawnRequestDecision, SpawnRequester, Workspace, WorkspaceExt,
    WorkspaceRunner, WorkspaceRunnerConfig, WorkspaceServices,
};
use ipc::ChildKind;

use crate::capabilities::{
    create_network_capabilities, create_renderer_capabilities,
    create_runtime_capabilities,
};

use ctb_utilities::testing::binary_path::resolve_binary_path_supporting_tests_or_example;

#[derive(Debug, Default)]
struct MinimalWorkspace {
    runtime_pid: std::sync::OnceLock<ctb_workspace_ipc::types::ProcessId>,
    services: WorkspaceServices,
}

#[async_trait::async_trait]
impl Workspace for MinimalWorkspace {
    fn services_needed(&self) -> Vec<(ChildKind, CapabilitySet)> {
        vec![(ChildKind::Network, create_network_capabilities())]
    }

    fn services(&self) -> &WorkspaceServices {
        &self.services
    }

    fn set_services(&mut self, services: WorkspaceServices) {
        self.services = services;
    }

    async fn boot(&mut self, rt: &WorkspaceRuntime) -> Result<()> {
        println!("=== Minimal Workspace Example ===\n");
        println!("Starting workspace on socket: {}", rt.socket_path());

        println!("\n[workspace] --- Network service started ---");

        Ok(())
    }

    async fn run(&self, rt: &WorkspaceRuntime) -> Result<()> {
        println!("\n[workspace] --- Spawning runtime subprocess ---");
        let runtime = rt
            .start_runtime_service(create_runtime_capabilities(), None)
            .await?;

        let pid = runtime.pid();

        println!("[workspace] Spawned runtime with pid: {pid}");
        if self.runtime_pid.set(pid).is_err() {
            warn_fmt!("runtime pid already set; ignoring");
        }

        // Call the network service's echo method.
        println!("\n[workspace] --- Requesting data from network service ---");
        let response = self
            .network()?
            .echo("Hello from network module".as_bytes().to_vec())
            .await?;
        let response_str = String::from_utf8_lossy(&response);
        println!("[workspace] Received echo from network: {response_str}");

        // Send the document to the runtime via the data plane.
        println!(
            "\n[workspace] --- Sending document to runtime (data plane) ---"
        );

        // Important: the runtime's nested-document demo will synchronously
        // request spawns and proxy calls back through the workspace.
        //
        // Simply explained: the runner's IPC routing loop is already running while
        // `run()` is executing, so awaiting this call inline is safe.
        let document = response_str.to_string();
        let settings = RenderSettings {
            mode: RenderMode::Immediate,
            target: RenderTarget::Teletype,
        };

        let code = runtime
            .test_simple_nested_document(document, settings)
            .await?;
        println!("[workspace] runtime returned exit code: {code}");

        // The runtime requests workspace shutdown after posting the data-plane
        // frame. Waiting here ensures the runner doesn't begin tearing down the
        // process tree prematurely.
        rt.wait_for_shutdown().await;

        Ok(())
    }

    async fn on_parent_message(
        &self,
        rt: &WorkspaceRuntime,
        event: ParentMessageEvent,
    ) -> Result<()> {
        println!("[workspace] message from {:?}", event.ctx.process_kind);
        handle_parent_message(rt, &event).await?;
        Ok(())
    }

    async fn evaluate_spawn_request(
        &self,
        _rt: &WorkspaceRuntime,
        requester: SpawnRequester,
        request: ctb_workspace_ipc::services::parent::api::SpawnChildRequest,
    ) -> Result<SpawnRequestDecision> {
        // Example policy:
        // - only the top-level runtime may request spawns
        // - runtimes may request a renderer or a subruntime
        // - spawned processes are owned by the requesting runtime
        let Some(runtime_pid) = self.runtime_pid.get().copied() else {
            return Ok(SpawnRequestDecision::Reject {
                error: Some("runtime not started".into()),
            });
        };

        if requester.pid != Some(runtime_pid)
            || requester.ctx.process_kind.as_deref() != Some("runtime")
        {
            return Ok(SpawnRequestDecision::Reject {
                error: Some("only the top-level runtime may spawn".into()),
            });
        }

        if request.kind != ChildKind::Runtime
            && request.kind != ChildKind::Renderer
        {
            return Ok(SpawnRequestDecision::Reject {
                error: Some(
                    "only runtime + renderer spawn requests are allowed".into(),
                ),
            });
        }

        let caps = match request.kind {
            ChildKind::Runtime => create_runtime_capabilities(),
            ChildKind::Renderer => create_renderer_capabilities(),
            _ => {
                return Ok(SpawnRequestDecision::Reject {
                    error: Some("unsupported child kind".into()),
                });
            }
        };

        Ok(SpawnRequestDecision::Accept {
            parent: requester.pid,
            caps,
            extra_args: vec![],
        })
    }
}

async fn handle_parent_message(
    rt: &WorkspaceRuntime,
    event: &ParentMessageEvent,
) -> Result<()> {
    match rt.resolve_parent_message(event).await? {
        ResolvedParentMessage::DataPlaneBytes { bytes, .. } => {
            println!(
                "Workspace received data plane message: {}",
                String::from_utf8_lossy(&bytes)
            );
        }
        ResolvedParentMessage::DataPlaneReadFailed { error, .. } => {
            println!("Workspace failed to read data plane blob: {error:#}");
        }
        ResolvedParentMessage::ShutdownRequest { reason } => {
            println!(
                "Workspace received shutdown request: {}",
                reason.unwrap_or_else(|| "no reason".into())
            );
        }
        ResolvedParentMessage::Text(text) => {
            println!("Workspace received text message: {text}");
        }
        ResolvedParentMessage::Other { kind } => {
            println!("Workspace received message of kind: {kind:?}");
        }
    }
    Ok(())
}

fn resolve_minimal_workspace_example_exe_path() -> Result<PathBuf> {
    resolve_binary_path_supporting_tests_or_example(
        "minimal-workspace",
        Some("CTB_MINIMAL_WORKSPACE_EXE"),
    )
}

/// Convenience function to run the workspace example.
pub async fn run_workspace_example() -> Result<()> {
    let runner = WorkspaceRunner::new(
        MinimalWorkspace::default(),
        WorkspaceRunnerConfig::default_with_timeout(),
        resolve_minimal_workspace_example_exe_path()?.into(),
        None,
    );
    let stats = runner.run().await?;

    anyhow::ensure!(
        stats.shutdown_received,
        "example timed out or exited without shutdown"
    );
    anyhow::ensure!(
        !stats.forced_termination,
        "workspace required forced termination"
    );
    anyhow::ensure!(
        stats.data_plane_messages_received >= 1,
        "expected >=1 data plane message (final frame), got {}",
        stats.data_plane_messages_received
    );

    println!(
        "\n[workspace] --- Summary: received {} data plane messages, shutdown={} ---",
        stats.data_plane_messages_received, stats.shutdown_received
    );

    println!("\n[workspace] === Example completed successfully ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test("tokio")]
    #[ignore]
    async fn workspace_example_runs() -> Result<()> {
        run_workspace_example().await
    }
}
