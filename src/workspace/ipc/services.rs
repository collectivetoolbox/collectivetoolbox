#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::Arc;

use crate::connection::DeferredChildIpcContext;
use crate::connection::connect_to_workspace;
use anyhow::{Result, anyhow};
use ipc::{ChildKind, child_kind_from_string, format_child_kind};
use tokio::sync::oneshot;

use crate::protocol::ClientInfo;
use crate::router::IpcRouter;
use crate::services::process::OneshotShutdownProcessService;

use serde::{Deserialize, Serialize};

use crate::types::ProcessId;
use crate::workspace_runner::process::ChildProcess;

pub mod formats;
pub mod io;
pub mod network;
pub mod parent;
pub mod process;
pub mod renderer;
pub mod runtime;
pub mod storage;

pub mod macros;

/// Generic response payloads (postcard-encoded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringResponse {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytesResponse {
    /// Raw bytes result.
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeResponse {
    pub code: i32,
}

pub trait IpcServiceClient {
    /// Access the service process associated with this IPC client.
    fn proc(&self) -> &ChildProcess;
}

/// Extension trait providing ergonomic service access for `IpcServiceClient`s.
///
/// Any type implementing [`IpcServiceClient`] automatically gets these helper
/// methods.
pub trait IpcServiceClientExt: IpcServiceClient {
    fn pid(&self) -> ProcessId {
        self.proc().pid
    }
}

impl<C: IpcServiceClient> IpcServiceClientExt for C {}

/// Start a service as a subprocess based on mode.
pub async fn run_as_subprocess(
    mode: &str,
    socket_path: &str,
    token: &str,
) -> Result<()> {
    crate::services::run_service(
        child_kind_from_string(mode)?,
        socket_path,
        token,
    )
    .await
}

/// Run the runtime subprocess logic.
///
/// # Arguments
/// * `service` - the service type to start
/// * `socket_path` - Path to the workspace's IPC socket
/// * `token` - Capability token for authentication
pub async fn run_service(
    service: ChildKind,
    socket_path: &str,
    token: &str,
) -> Result<()> {
    let process_kind: &'static str = format_child_kind(&service);
    debug_fmt!("[{process_kind}] Starting...");

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<Option<String>>();

    let router = IpcRouter::new();
    let mut deferred_ipc: Option<Arc<DeferredChildIpcContext>> = None;
    let router = match service {
        ChildKind::Network => {
            let ipc = Arc::new(DeferredChildIpcContext::new());
            deferred_ipc = Some(Arc::clone(&ipc));
            let ipc_ctx: Arc<
                dyn ctb_utilities::ipc::service_traits::ChildIpcContext,
            > = ipc.clone();
            ctb_network::init_ipc_context(&ipc_ctx, Some(ChildKind::Network))?;

            let backend: Arc<dyn ctb_network::NetworkBackend> =
                Arc::new(ctb_network::DefaultNetworkBackend);
            ctb_network::init_backend(&backend)?;

            Ok(router)
        }
        ChildKind::Io => {
            let ipc = Arc::new(DeferredChildIpcContext::new());
            deferred_ipc = Some(Arc::clone(&ipc));
            let ipc_ctx: Arc<
                dyn ctb_utilities::ipc::service_traits::ChildIpcContext,
            > = ipc.clone();
            ctb_io::init_ipc_context(&ipc_ctx, Some(ChildKind::Io))?;
            Ok(router)
        }
        ChildKind::Renderer => {
            let ipc = Arc::new(DeferredChildIpcContext::new());
            deferred_ipc = Some(Arc::clone(&ipc));
            let ipc_ctx: Arc<
                dyn ctb_utilities::ipc::service_traits::ChildIpcContext,
            > = ipc.clone();
            ctb_renderer::init_ipc_context(
                &ipc_ctx,
                Some(ChildKind::Renderer),
            )?;
            Ok(router)
        }
        ChildKind::Runtime => {
            let ipc = Arc::new(DeferredChildIpcContext::new());
            deferred_ipc = Some(Arc::clone(&ipc));
            let ipc_ctx: Arc<
                dyn ctb_utilities::ipc::service_traits::ChildIpcContext,
            > = ipc.clone();
            ctb_runtime::init_ipc_context(&ipc_ctx, Some(ChildKind::Runtime))?;
            Ok(router)
        }
        ChildKind::Storage => {
            let ipc = Arc::new(DeferredChildIpcContext::new());
            deferred_ipc = Some(Arc::clone(&ipc));
            let ipc_ctx: Arc<
                dyn ctb_utilities::ipc::service_traits::ChildIpcContext,
            > = ipc.clone();
            ctb_storage::init_ipc_context(&ipc_ctx, Some(ChildKind::Storage))?;
            Ok(router)
        }
        _ => Err(anyhow!(
            "unsupported or unimplemented service kind for runtime"
        )),
    }?;
    let router = router.with_process_service(Arc::new(
        OneshotShutdownProcessService::new(shutdown_tx),
    ));

    let client_info = ClientInfo {
        name: format!("example-{process_kind}"),
        version: "0.1.0".into(),
        process_kind: process_kind.into(),
    };

    let peer =
        connect_to_workspace(socket_path, token, client_info, router).await?;

    if let Some(ipc) = deferred_ipc.as_ref() {
        ipc.attach_peer(Arc::clone(&peer)).map_err(|e| {
            anyhow!("failed to attach IPC peer to context: {e}")
        })?;
    }

    debug_fmt!("[{process_kind}] Listening for requests...");

    tokio::select! {
        reason = shutdown_rx => {
            let reason = match reason {
                Ok(reason) => reason,
                Err(_canceled) => Some("shutdown channel closed".into()),
            };
            debug_fmt!(
                "[{process_kind}] Shutdown acknowledged; exiting: {}",
                reason.unwrap_or_else(|| "no reason".into())
            );
        }
        () = peer.wait_closed() => {
            debug_fmt!("[{process_kind}] Connection closed; exiting.");
        }
    }

    debug_fmt!("[{process_kind}] Done.");
    Ok(())
}
