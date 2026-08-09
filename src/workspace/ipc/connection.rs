#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use ctb_utilities::ipc::service_traits::{
    ChildIpcContext, FormatsClientTrait, IoClientTrait, NetworkClientTrait,
    RendererClientTrait, RuntimeClientTrait, StorageClientTrait,
};
use once_cell::sync::OnceCell;

use crate::auth::capability::CapabilityToken;
use crate::auth::capability::TokenValidator;
use crate::auth::capability::trusted_workspace_capabilities;
use crate::multiplex::session::{FramedSession, Session};
use crate::peer::IpcPeer;
use crate::process_manager::ProcessManager;
use crate::protocol::{ClientInfo, Hello, MethodId};
use crate::router::{ConnectionContext, IpcRouter};
use crate::transport::{LocalSocketTransportFactory, TransportFactory};
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

pub async fn connect_to_workspace(
    socket_path: &str,
    token: &str,
    client: ClientInfo,
    router: IpcRouter,
) -> Result<Arc<IpcPeer>> {
    // Connect to parent
    let factory = LocalSocketTransportFactory;
    let conn = factory.connect(socket_path).await?;
    let conn = Arc::new(conn);
    let session = Arc::new(FramedSession::new(conn));

    // Perform handshake
    let hello =
        Hello::new(CapabilityToken(token.to_string()), Some(client.clone()));
    let caps = session.client_handshake(hello).await?;
    let cap_keys = format!("{:?}", caps.allowed.keys());

    // NOTE: `caps` returned from the handshake represent what *this child* is
    // allowed to call on the workspace. Incoming requests on this connection
    // originate from the workspace, so we authorize them using a trusted
    // workspace capability set.
    let ctx = ConnectionContext {
        id: ConnectionId::default(),
        capabilities: trusted_workspace_capabilities(),
        metadata: None,
    };

    let session: Arc<dyn Session> = session;
    let peer = IpcPeer::new(session, Arc::new(router), ctx);

    debug_fmt!(
        "[{:?}] Connected with capabilities: {:?}",
        client.process_kind,
        cap_keys
    );

    Ok(peer)
}

/// Shared, reusable server-side IPC connection wiring.
///
/// This encapsulates the handshake + peer creation that most workspaces need.
/// Callers can optionally use [`WorkspaceConnectionHandler::attach_peer_by_process_kind`]
/// to map the connection to a pending child process.
#[derive(Debug, Clone)]
pub struct WorkspaceConnectionHandler {
    router: Arc<IpcRouter>,
    validator: Arc<dyn TokenValidator>,
}

impl WorkspaceConnectionHandler {
    /// Construct a new handler.
    pub fn new(
        router: Arc<IpcRouter>,
        validator: Arc<dyn TokenValidator>,
    ) -> Self {
        Self { router, validator }
    }

    /// Perform server handshake and create a new [`IpcPeer`].
    pub async fn accept_peer<T>(
        &self,
        session: Arc<FramedSession<T>>,
        conn_id: ConnectionId,
    ) -> Result<Arc<IpcPeer>>
    where
        T: crate::transport::FramedConnection
            + Clone
            + std::fmt::Debug
            + 'static,
    {
        let _caps = session
            .server_handshake(
                self.validator.as_ref(),
                self.router.as_ref(),
                conn_id,
            )
            .await
            .context("server handshake")?;

        let ctx = self.router.get(&conn_id).ok_or_else(|| {
            anyhow::anyhow!("connection context missing for {conn_id}")
        })?;

        let session: Arc<dyn Session> = session;
        Ok(IpcPeer::new(session, Arc::clone(&self.router), ctx))
    }

    /// Best-effort attach a peer to a known pending process based on
    /// `process_kind` handshake metadata.
    ///
    /// This is a convenience for the common pattern:
    /// - workspace spawns a child and records its PID under a process kind key
    /// - child connects and identifies itself (`process_kind`)
    /// - workspace attaches the connection id to that PID and stores the peer
    pub async fn attach_peer_by_process_kind(
        &self,
        conn_id: ConnectionId,
        peer: Arc<IpcPeer>,
        process_manager: &dyn ProcessManager,
        pending_processes: &tokio::sync::Mutex<HashMap<String, ProcessId>>,
        peers: &tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>,
    ) -> Result<Option<ProcessId>> {
        let Some(ctx) = self.router.get(&conn_id) else {
            return Ok(None);
        };

        let kind = ctx
            .metadata
            .as_ref()
            .and_then(|m| m.get("process_kind"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let Some(kind) = kind else {
            return Ok(None);
        };

        let maybe_pid = pending_processes.lock().await.remove(&kind);
        let Some(pid) = maybe_pid else {
            return Ok(None);
        };

        if let Err(e) = process_manager.attach_connection(pid, conn_id).await {
            tracing::warn!(
                "attach_connection failed for pid={pid} conn_id={conn_id}: {e:#}"
            );
        }

        peers.lock().await.insert(pid, peer);
        Ok(Some(pid))
    }

    /// End-to-end helper: accept + create peer, then best-effort attach.
    pub async fn handle_connection_with_pending_processes<T>(
        &self,
        session: Arc<FramedSession<T>>,
        conn_id: ConnectionId,
        process_manager: &dyn ProcessManager,
        pending_processes: &tokio::sync::Mutex<HashMap<String, ProcessId>>,
        peers: &tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>,
    ) -> Result<()>
    where
        T: crate::transport::FramedConnection
            + Clone
            + std::fmt::Debug
            + 'static,
    {
        let peer = self.accept_peer(session, conn_id).await?;
        let _ = self
            .attach_peer_by_process_kind(
                conn_id,
                peer,
                process_manager,
                pending_processes,
                peers,
            )
            .await?;
        Ok(())
    }
}

/// IPC context implementation for child processes using an `IpcPeer`.
///
/// This struct wraps an `Arc<IpcPeer>` and implements `ChildIpcContext`,
/// enabling child processes to request spawning of sub-processes and
/// communicate with the workspace.
#[derive(Debug, Clone)]
pub struct PeerChildIpcContext {
    peer: Arc<IpcPeer>,
}

impl PeerChildIpcContext {
    /// Create a new `PeerChildIpcContext` from an `IpcPeer`.
    pub fn new(peer: Arc<IpcPeer>) -> Self {
        Self { peer }
    }

    pub(crate) fn peer(&self) -> Arc<IpcPeer> {
        Arc::clone(&self.peer)
    }
}

/// A `ChildIpcContext` that can be wired up after connecting.
///
/// Child processes need an `IpcRouter` (and thus their services) constructed
/// before the IPC handshake completes. Some services (notably the runtime)
/// need a `ChildIpcContext` to make outbound IPC calls back to the workspace.
///
/// This type enables that wiring without requiring the service constructors to
/// have direct access to the eventually-created `IpcPeer`.
#[derive(Debug, Default)]
pub struct DeferredChildIpcContext {
    inner: OnceCell<PeerChildIpcContext>,
}

impl DeferredChildIpcContext {
    /// Create a new deferred IPC context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach the connected peer.
    ///
    /// This should be called exactly once after `connect_to_workspace`.
    pub fn attach_peer(&self, peer: Arc<IpcPeer>) -> Result<()> {
        if self.inner.set(PeerChildIpcContext::new(peer)).is_err() {
            anyhow::bail!("IPC context already initialized");
        }
        Ok(())
    }

    fn ctx(&self) -> Result<&PeerChildIpcContext> {
        self.inner
            .get()
            .ok_or_else(|| anyhow::anyhow!("IPC context not initialized yet"))
    }

    pub(crate) fn peer(&self) -> Result<Arc<IpcPeer>> {
        Ok(self.ctx()?.peer())
    }
}

#[async_trait]
impl ChildIpcContext for DeferredChildIpcContext {
    async fn request_spawn_formats(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn FormatsClientTrait>> {
        self.ctx()?.request_spawn_formats(init_data).await
    }

    async fn request_spawn_io(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn IoClientTrait>> {
        self.ctx()?.request_spawn_io(init_data).await
    }

    async fn request_spawn_network(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn NetworkClientTrait>> {
        self.ctx()?.request_spawn_network(init_data).await
    }

    async fn request_spawn_renderer(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RendererClientTrait>> {
        self.ctx()?.request_spawn_renderer(init_data).await
    }

    async fn request_spawn_runtime(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RuntimeClientTrait>> {
        self.ctx()?.request_spawn_runtime(init_data).await
    }

    async fn request_spawn_storage(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn StorageClientTrait>> {
        self.ctx()?.request_spawn_storage(init_data).await
    }

    async fn send_to_parent(&self, message: &str) -> Result<()> {
        self.ctx()?.send_to_parent(message).await
    }

    async fn send_data_plane_message(
        &self,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<()> {
        self.ctx()?
            .send_data_plane_message(bytes, content_type)
            .await
    }

    async fn request_workspace_shutdown(
        &self,
        reason: Option<String>,
    ) -> Result<()> {
        self.ctx()?.request_workspace_shutdown(reason).await
    }

    async fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>> {
        self.ctx()?.call_raw(service, method, args).await
    }
}

#[async_trait]
impl ChildIpcContext for PeerChildIpcContext {
    async fn request_spawn_formats(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn FormatsClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Formats,
            init_data,
        )
        .await?;
        Ok(crate::services::formats::peer_formats_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn request_spawn_io(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn IoClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Io,
            init_data,
        )
        .await?;
        Ok(crate::services::io::peer_io_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn request_spawn_network(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn NetworkClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Network,
            init_data,
        )
        .await?;
        Ok(crate::services::network::peer_network_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn request_spawn_runtime(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RuntimeClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Runtime,
            init_data,
        )
        .await?;
        Ok(crate::services::runtime::peer_runtime_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn request_spawn_renderer(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RendererClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Renderer,
            init_data,
        )
        .await?;
        Ok(crate::services::renderer::peer_renderer_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn request_spawn_storage(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn StorageClientTrait>> {
        let pid = crate::peer_clients::request_spawn_child(
            &self.peer,
            ChildKind::Storage,
            init_data,
        )
        .await?;
        Ok(crate::services::storage::peer_storage_client(
            Arc::clone(&self.peer),
            pid,
        ))
    }

    async fn send_to_parent(&self, message: &str) -> Result<()> {
        crate::peer_clients::send_to_parent(&self.peer, message).await
    }

    async fn send_data_plane_message(
        &self,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<()> {
        crate::peer_clients::send_data_plane_message(
            &self.peer,
            bytes,
            content_type,
        )
        .await
    }

    async fn request_workspace_shutdown(
        &self,
        reason: Option<String>,
    ) -> Result<()> {
        crate::peer_clients::request_workspace_shutdown(&self.peer, reason)
            .await
    }

    async fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>> {
        let resp = self
            .peer
            .call(
                MethodId {
                    service: service.to_string(),
                    method: method.to_string(),
                },
                args,
            )
            .await
            .map_err(|e| anyhow::anyhow!("IPC call_raw failed: {e}"))?;

        crate::ensure_response_ok(&resp, "IPC call_raw")?;
        // Reason for fallback: successful IPC response with null result defaults to serde_json::Value::Null
        Ok(resp.result.unwrap_or_default())
    }
}
