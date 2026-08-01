//! Generic workspace runner.
//!
//! This module provides a reusable [`WorkspaceRunner`] which encapsulates the
//! IPC event loop for:
//! - binding an IPC socket listener
//! - accepting and handshaking child connections
//! - routing child-to-parent messages into a workspace loop
//! - handling spawn requests from children
//! - requesting graceful shutdown of the workspace process tree
//!
//! It runs the workspace's [`Workspace::run`] concurrently with this IPC loop.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;
use crate::workspace_runner::workspace_runtime::{
    ShutdownState, WorkspaceRuntime,
};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::auth::capability::{CapabilitySet, InMemoryTokenValidator};
use crate::connection::WorkspaceConnectionHandler;
use crate::multiplex::session::FramedSession;
use crate::process_manager::{
    ProcessManager, TokioProcessManager, shutdown_for_process_tree,
};
use crate::router::IpcRouter;
use crate::services::io::IoClient;
use crate::services::network::NetworkClient;
use crate::services::parent::api::{ProxyCallRequest, ProxyCallResponse};
use crate::services::parent::channel::ProxyCallWithResponse;
use crate::services::parent::{ChannelParentMessenger, ParentRequestContext};
use crate::services::parent::{
    ParentMessageEvent, ParentMessageKind, SpawnChildRequest,
    SpawnChildResponse, SpawnRequestWithResponse,
};
use crate::services::process::ShutdownChannelProcessService;
use crate::services::storage::StorageClient;
use crate::transport::{
    LocalSocketFramedConnection, LocalSocketTransportFactory, TransportFactory,
    TransportListener,
};
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

use crate::peer::IpcPeer;

pub mod cli;
pub mod process;
pub mod workspace_runtime;

/// Container for singleton service dependencies injected into a workspace.
///
/// Each field corresponds to a [`ChildKind`] singleton service that can be
/// requested via [`Workspace::services_needed`]. The runner will start the
/// requested services and populate this struct before calling
/// [`Workspace::boot`].
#[derive(Debug, Clone, Default)]
pub struct WorkspaceServices {
    pub network: Option<NetworkClient>,
    pub io: Option<IoClient>,
    pub storage: Option<StorageClient>,
}

impl WorkspaceServices {
    /// Return the network client.
    ///
    /// This returns an error if the workspace did not declare
    /// [`ChildKind::Network`] in `services_needed()`.
    pub fn network(&self) -> Result<&NetworkClient> {
        self.network.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "NetworkClient not available. Did you request ChildKind::Network in services_needed()?"
            )
        })
    }

    /// Return the IO client.
    ///
    /// This returns an error if the workspace did not declare [`ChildKind::Io`]
    /// in `services_needed()`.
    pub fn io(&self) -> Result<&IoClient> {
        self.io.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "IoClient not available. Did you request ChildKind::Io in services_needed()?"
            )
        })
    }

    /// Return the storage client.
    ///
    /// This returns an error if the workspace did not declare
    /// [`ChildKind::Storage`] in `services_needed()`.
    pub fn storage(&self) -> Result<&StorageClient> {
        self.storage.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "StorageClient not available. Did you request ChildKind::Storage in services_needed()?"
            )
        })
    }
}

/// Extension trait providing ergonomic service access for workspaces.
///
/// Any type implementing [`Workspace`] automatically gets these helper methods.
pub trait WorkspaceExt: Workspace {
    /// Access the network client with clean syntax.
    fn network(&self) -> Result<&NetworkClient> {
        self.services().network()
    }

    /// Access the IO client with clean syntax.
    fn io(&self) -> Result<&IoClient> {
        self.services().io()
    }

    /// Access the storage client with clean syntax.
    fn storage(&self) -> Result<&StorageClient> {
        self.services().storage()
    }
}

impl<W: Workspace> WorkspaceExt for W {}

/// Configuration for the workspace runner.
#[derive(Debug, Clone)]
pub struct WorkspaceRunnerConfig {
    /// Maximum time to wait for shutdown. If `None`, no timeout is enforced.
    pub timeout: Option<Duration>,
    /// Time to wait for the listener to start before calling `startup`.
    pub listener_startup_delay: Duration,
    /// Buffer size for message channels.
    pub message_buffer_size: usize,
    /// Buffer size for spawn request channels.
    pub spawn_buffer_size: usize,
    /// Event loop tick interval.
    pub tick_interval: Duration,
    /// Maximum time to wait for a child to connect before failing calls.
    pub connect_timeout: Duration,
    /// Grace period to allow a subtree to exit after acknowledging shutdown.
    pub shutdown_grace: Duration,
    /// How long to wait for a shutdown acknowledgement.
    pub shutdown_ack_timeout: Duration,
}

impl Default for WorkspaceRunnerConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            listener_startup_delay: Duration::from_millis(50),
            message_buffer_size: 32,
            spawn_buffer_size: 8,
            tick_interval: Duration::from_millis(100),
            connect_timeout: Duration::from_secs(5),
            shutdown_grace: Duration::from_secs(30),
            shutdown_ack_timeout: Duration::from_secs(1),
        }
    }
}

impl WorkspaceRunnerConfig {
    /// Returns the default configuration with a timeout set.
    pub fn default_with_timeout() -> Self {
        Self {
            timeout: Some(Duration::from_secs(10)),
            ..Self::default()
        }
    }
}

/// Statistics collected during workspace execution.
#[derive(Debug, Clone, Default)]
pub struct WorkspaceStats {
    /// Number of data-plane messages received from children.
    pub data_plane_messages_received: usize,
    /// Whether a shutdown was received.
    pub shutdown_received: bool,
    /// Reason for shutdown, if one was provided.
    pub shutdown_reason: Option<String>,
    /// Whether the runner had to force-terminate any process tree.
    pub forced_termination: bool,
}

/// Identifies the sender of a spawn request.
#[derive(Debug, Clone)]
pub struct SpawnRequester {
    pub ctx: ParentRequestContext,
    /// Process id for the requester if it has been attached.
    pub pid: Option<ProcessId>,
}

/// Decision returned by a workspace when evaluating a spawn request.
#[derive(Debug, Clone)]
pub enum SpawnRequestDecision {
    Reject {
        error: Option<String>,
    },
    Accept {
        /// Optional override for the parent pid.
        parent: Option<ProcessId>,
        /// Capabilities bound to the spawned child's control connection.
        caps: CapabilitySet,
        /// Extra arguments to pass.
        extra_args: Vec<String>,
    },
}

/// Workspace behavior that can be plugged into [`WorkspaceRunner`].
///
/// Implementors should store a `WorkspaceServices` field and provide access
/// via the `services()` and `set_services()` methods. The runner will call
/// `set_services()` before `boot()` to inject the started services.
#[async_trait::async_trait]
pub trait Workspace: Send + Sync {
    /// Return the list of singleton services this workspace needs.
    ///
    /// The runner will start these services before calling [`Workspace::boot`]
    /// and inject them via `set_services`.
    fn services_needed(&self) -> Vec<(ChildKind, CapabilitySet)> {
        Vec::new()
    }

    /// Return the services container. Used by helper methods.
    fn services(&self) -> &WorkspaceServices;

    /// Set the services container. Called by the runner before `boot()`.
    fn set_services(&mut self, services: WorkspaceServices);

    /// Called after the IPC listener is bound and accepting connections.
    async fn boot(&mut self, rt: &WorkspaceRuntime) -> Result<()>;

    /// Main application task for the workspace.
    ///
    /// Simply explained:
    /// - The runner has an internal IPC loop (accepts connections, routes
    ///   messages, handles spawn/proxy calls).
    /// - Your `run()` executes concurrently with that IPC loop.
    /// - If you want `run()` to stay alive until shutdown, await
    ///   `rt.wait_for_shutdown()` (usually via `tokio::select!`).
    ///
    /// Returning from `run()` will request workspace shutdown.
    async fn run(&self, rt: &WorkspaceRuntime) -> Result<()>;

    /// Called for each child-to-parent message.
    async fn on_parent_message(
        &self,
        rt: &WorkspaceRuntime,
        event: ParentMessageEvent,
    ) -> Result<()>;

    /// Evaluate a spawn request.
    async fn evaluate_spawn_request(
        &self,
        rt: &WorkspaceRuntime,
        requester: SpawnRequester,
        request: SpawnChildRequest,
    ) -> Result<SpawnRequestDecision>;
}

/// Encapsulates the workspace's main event loop and lifecycle management.
#[derive(Debug)]
pub struct WorkspaceRunner<W: Workspace> {
    workspace: W,
    config: WorkspaceRunnerConfig,
    socket_path: String,
    validator: Arc<InMemoryTokenValidator>,
    router: Arc<IpcRouter>,
    process_manager: Arc<TokioProcessManager>,
    exe: Arc<std::path::PathBuf>,
    pending_processes: Arc<tokio::sync::Mutex<HashMap<String, ProcessId>>>,
    peers: Arc<tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>>,
    pid_by_conn: Arc<tokio::sync::Mutex<HashMap<ConnectionId, ProcessId>>>,
    root_pids: Arc<tokio::sync::Mutex<Vec<ProcessId>>>,
    singleton_pids: Arc<tokio::sync::Mutex<HashMap<ChildKind, ProcessId>>>,
    messages_rx: mpsc::Receiver<ParentMessageEvent>,
    spawn_requests_rx: mpsc::Receiver<SpawnRequestWithResponse>,
    proxy_calls_rx: mpsc::Receiver<ProxyCallWithResponse>,
    shutdown_tx: mpsc::Sender<Option<String>>,
    shutdown_rx: mpsc::Receiver<Option<String>>,
    shutdown_state: Arc<tokio::sync::Mutex<ShutdownState>>,
    shutdown_notify: Arc<tokio::sync::Notify>,
}

impl<W: Workspace> WorkspaceRunner<W> {
    /// Create a new runner.
    ///
    /// If `router` is provided, the runner will install its own
    /// parent-messenger and process-service into it.
    pub fn new(
        workspace: W,
        config: WorkspaceRunnerConfig,
        exe: Arc<std::path::PathBuf>,
        router: Option<IpcRouter>,
    ) -> Self {
        let (messages_tx, messages_rx) =
            mpsc::channel::<ParentMessageEvent>(config.message_buffer_size);
        let (spawn_requests_tx, spawn_requests_rx) =
            mpsc::channel::<SpawnRequestWithResponse>(config.spawn_buffer_size);
        let (proxy_calls_tx, proxy_calls_rx) =
            mpsc::channel::<ProxyCallWithResponse>(config.spawn_buffer_size);
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<Option<String>>(1);
        let runtime_shutdown_tx = shutdown_tx.clone();

        let shutdown_state =
            Arc::new(tokio::sync::Mutex::new(ShutdownState::default()));
        let shutdown_notify = Arc::new(tokio::sync::Notify::new());

        let parent_messenger = Arc::new(ChannelParentMessenger::new(
            messages_tx,
            spawn_requests_tx,
            proxy_calls_tx,
        ));
        let process_service =
            Arc::new(ShutdownChannelProcessService::new(shutdown_tx));

        let router = router
            .unwrap_or_default()
            .with_parent_messenger(parent_messenger)
            .with_process_service(process_service);

        let validator = Arc::new(InMemoryTokenValidator::new());
        let socket_path = crate::transport::unique_endpoint("workspace");
        let process_manager = TokioProcessManager::new();

        Self {
            workspace,
            config,
            socket_path,
            validator,
            router: Arc::new(router),
            process_manager,
            exe,
            pending_processes: Arc::new(
                tokio::sync::Mutex::new(HashMap::new()),
            ),
            peers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            pid_by_conn: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            root_pids: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            singleton_pids: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            messages_rx,
            spawn_requests_rx,
            proxy_calls_rx,
            shutdown_tx: runtime_shutdown_tx,
            shutdown_rx,
            shutdown_state,
            shutdown_notify,
        }
    }

    /// Socket path for this workspace.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    #[allow(dead_code, reason = "potentially unused API helper")]
    fn runtime(&self) -> WorkspaceRuntime {
        WorkspaceRuntime {
            socket_path: self.socket_path.clone(),
            validator: Arc::clone(&self.validator),
            process_manager: Arc::clone(&self.process_manager),
            exe: Arc::clone(&self.exe),
            config: self.config.clone(),
            pending_processes: Arc::clone(&self.pending_processes),
            peers: Arc::clone(&self.peers),
            pid_by_conn: Arc::clone(&self.pid_by_conn),
            root_pids: Arc::clone(&self.root_pids),
            singleton_pids: Arc::clone(&self.singleton_pids),
            shutdown_tx: self.shutdown_tx.clone(),
            shutdown_state: Arc::clone(&self.shutdown_state),
            shutdown_notify: Arc::clone(&self.shutdown_notify),
        }
    }

    /// Run the workspace to completion.
    ///
    /// Simply explained: there are two things happening at once:
    /// - the IPC loop (accepting connections + routing messages)
    /// - your workspace `run()` function
    ///
    /// The IPC loop starts first, then `run()` is awaited concurrently.
    #[allow(clippy::too_many_lines, reason = "uniform runner run loop is naturally long")]
    pub async fn run(self) -> Result<WorkspaceStats> {
        // Simply explained: this function is the "real" top-level loop.
        //
        // It starts the IPC listener + routing loop, then runs the workspace
        // application logic concurrently.
        let WorkspaceRunner {
            mut workspace,
            config,
            socket_path,
            validator,
            router,
            process_manager,
            exe,
            pending_processes,
            peers,
            pid_by_conn,
            root_pids,
            singleton_pids,
            messages_rx,
            spawn_requests_rx,
            proxy_calls_rx,
            shutdown_tx,
            shutdown_rx,
            shutdown_state,
            shutdown_notify,
        } = self;

        debug_fmt!("Starting workspace runner on socket: {}", socket_path);

        #[cfg(unix)]
        let _ = std::fs::remove_file(&socket_path);

        let factory = LocalSocketTransportFactory;
        let listener = factory.bind(&socket_path).await?;
        let listener: Arc<
            dyn TransportListener<Conn = LocalSocketFramedConnection>,
        > = Arc::from(listener);

        let accept_task = Self::spawn_accept_task(
            Arc::clone(&router),
            Arc::clone(&validator),
            Arc::clone(&listener),
            Arc::clone(&pending_processes),
            Arc::clone(&peers),
            Arc::clone(&pid_by_conn),
            Arc::clone(&process_manager),
        );
        tokio::time::sleep(config.listener_startup_delay).await;

        let rt = WorkspaceRuntime {
            socket_path: socket_path.clone(),
            validator: Arc::clone(&validator),
            process_manager: Arc::clone(&process_manager),
            exe: Arc::clone(&exe),
            config: config.clone(),
            pending_processes: Arc::clone(&pending_processes),
            peers: Arc::clone(&peers),
            pid_by_conn: Arc::clone(&pid_by_conn),
            root_pids: Arc::clone(&root_pids),
            singleton_pids: Arc::clone(&singleton_pids),
            shutdown_tx: shutdown_tx.clone(),
            shutdown_state: Arc::clone(&shutdown_state),
            shutdown_notify: Arc::clone(&shutdown_notify),
        };

        // Start requested singleton services and build WorkspaceServices
        let requested_services = workspace.services_needed();
        let services =
            Self::start_requested_services_static(&rt, &requested_services)
                .await?;

        workspace.set_services(services);

        workspace.boot(&rt).await?;

        // Run the IPC routing loop and the workspace application concurrently.
        //
        // Simply explained: shutdown can arrive (from a child) while the workspace
        // is still awaiting some in-flight RPC. If we immediately proceed to
        // tear down subprocess trees, those RPCs can fail with transport errors
        // (e.g. broken pipe).
        //
        // So if the event loop ends first, we give `run()` a grace period to
        // finish before shutting down process trees.
        let event_loop = Self::run_event_loop_static(
            &workspace,
            &rt,
            config.clone(),
            messages_rx,
            spawn_requests_rx,
            proxy_calls_rx,
            shutdown_rx,
        );
        tokio::pin!(event_loop);

        let app_fut = workspace.run(&rt);
        tokio::pin!(app_fut);

        let mut stats = tokio::select! {
            app_res = &mut app_fut => {
                app_res?;
                // If the app returns, request shutdown so the IPC loop exits.
                let _ = rt
                    .request_shutdown(Some("workspace run() returned".into()))
                    .await;
                event_loop.await?
            }
            stats_res = &mut event_loop => {
                let stats = stats_res?;

                match tokio::time::timeout(config.shutdown_grace, &mut app_fut).await {
                    Ok(app_res) => {
                        app_res?;
                    }
                    Err(_) => {
                        warn_fmt!(
                            "workspace run() did not exit within shutdown grace; canceling"
                        );
                    }
                }

                stats
            }
        };

        // Ask each root process tree to shut down.
        let roots = root_pids.lock().await.clone();
        for pid in roots {
            let outcome = shutdown_for_process_tree(
                Arc::clone(&peers),
                process_manager.as_ref(),
                pid,
                config.shutdown_ack_timeout,
                config.shutdown_grace,
                Some("workspace shutdown".into()),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!("shutdown_for_process_tree failed: {e:#}")
            })?;

            if outcome.forced {
                stats.forced_termination = true;
            }
        }

        listener.close().await?;
        accept_task.abort();

        #[cfg(unix)]
        let _ = std::fs::remove_file(&socket_path);

        Ok(stats)
    }

    /// Start all requested singleton services and return a `WorkspaceServices`.
    async fn start_requested_services_static(
        rt: &WorkspaceRuntime,
        requested: &[(ChildKind, CapabilitySet)],
    ) -> Result<WorkspaceServices> {
        let mut services = WorkspaceServices::default();

        for (kind, caps) in requested {
            match kind {
                ChildKind::Network => {
                    let proc =
                        rt.start_singleton_service(*kind, caps.clone()).await?;
                    services.network = Some(NetworkClient::from_child(proc));
                }
                ChildKind::Io => {
                    let proc =
                        rt.start_singleton_service(*kind, caps.clone()).await?;
                    services.io = Some(IoClient::from_child(proc));
                }
                ChildKind::Storage => {
                    let proc =
                        rt.start_singleton_service(*kind, caps.clone()).await?;
                    services.storage = Some(StorageClient::from_child(proc));
                }
                _ => {
                    log_fmt!("Unexpected service kind requested: {kind:?}");
                }
            }
        }

        Ok(services)
    }

    fn spawn_accept_task(
        router: Arc<IpcRouter>,
        validator: Arc<InMemoryTokenValidator>,
        listener: Arc<
            dyn TransportListener<Conn = LocalSocketFramedConnection>,
        >,
        pending: Arc<tokio::sync::Mutex<HashMap<String, ProcessId>>>,
        peers: Arc<tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>>,
        pid_by_conn: Arc<tokio::sync::Mutex<HashMap<ConnectionId, ProcessId>>>,
        process_manager: Arc<TokioProcessManager>,
    ) -> tokio::task::JoinHandle<()> {
        let handler = WorkspaceConnectionHandler::new(router, validator);

        tokio::spawn(async move {
            loop {
                let conn = match listener.accept().await {
                    Ok(Some(c)) => c,
                    Ok(None) => break,
                    Err(e) => {
                        warn_fmt!("Accept error: {e:#}");
                        continue;
                    }
                };

                let conn = Arc::new(conn);
                let session = Arc::new(FramedSession::new(conn));
                let conn_id = ConnectionId::new();

                let handler = handler.clone();
                let pending = Arc::clone(&pending);
                let peers = Arc::clone(&peers);
                let pid_by_conn = Arc::clone(&pid_by_conn);
                let process_manager = Arc::clone(&process_manager);

                tokio::spawn(async move {
                    let process_manager: &dyn ProcessManager =
                        process_manager.as_ref();

                    let peer = match handler.accept_peer(session, conn_id).await
                    {
                        Ok(peer) => peer,
                        Err(e) => {
                            warn_fmt!("Handshake failed: {e:#}");
                            return;
                        }
                    };

                    match handler
                        .attach_peer_by_process_kind(
                            conn_id,
                            Arc::clone(&peer),
                            process_manager,
                            pending.as_ref(),
                            peers.as_ref(),
                        )
                        .await
                    {
                        Ok(Some(pid)) => {
                            pid_by_conn.lock().await.insert(conn_id, pid);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            warn_fmt!(
                                "attach_peer_by_process_kind failed: {e:#}"
                            );
                        }
                    }
                });
            }
        })
    }

    async fn run_event_loop_static(
        workspace: &W,
        rt: &WorkspaceRuntime,
        config: WorkspaceRunnerConfig,
        mut messages_rx: mpsc::Receiver<ParentMessageEvent>,
        mut spawn_requests_rx: mpsc::Receiver<SpawnRequestWithResponse>,
        mut proxy_calls_rx: mpsc::Receiver<ProxyCallWithResponse>,
        mut shutdown_rx: mpsc::Receiver<Option<String>>,
    ) -> Result<WorkspaceStats> {
        let mut stats = WorkspaceStats::default();
        let start = std::time::Instant::now();

        while !stats.shutdown_received {
            if let Some(timeout) = config.timeout {
                if start.elapsed() >= timeout {
                    break;
                }
            }

            tokio::select! {
                maybe_msg = messages_rx.recv() => {
                    let Some(event) = maybe_msg else {
                        break;
                    };

                    if event.message.kind == ParentMessageKind::DataPlane {
                        stats.data_plane_messages_received =
                            stats.data_plane_messages_received.saturating_add(1);
                    }

                    if event.message.kind == ParentMessageKind::ShutdownRequest {
                        stats.shutdown_received = true;
                        if let Ok(sr) = event.message.as_shutdown_request() {
                            stats.shutdown_reason = sr.reason;
                        }
                        rt.notify_shutdown(stats.shutdown_reason.clone()).await;
                    }

                    workspace.on_parent_message(rt, event).await?;
                }
                maybe_spawn = spawn_requests_rx.recv() => {
                    let Some(req) = maybe_spawn else {
                        break;
                    };
                    Self::handle_spawn_request_static(workspace, rt, req).await;
                }
                maybe_proxy = proxy_calls_rx.recv() => {
                    let Some(req) = maybe_proxy else {
                        break;
                    };
                    Self::handle_proxy_call_static(rt, req).await;
                }
                maybe_reason = shutdown_rx.recv() => {
                    let Some(reason) = maybe_reason else {
                        break;
                    };
                    stats.shutdown_received = true;
                    stats.shutdown_reason = reason;
                    rt.notify_shutdown(stats.shutdown_reason.clone()).await;
                }
                () = tokio::time::sleep(config.tick_interval) => {}
            }
        }

        Ok(stats)
    }

    async fn handle_proxy_call_static(
        rt: &WorkspaceRuntime,
        req: ProxyCallWithResponse,
    ) {
        let ProxyCallWithResponse {
            ctx,
            request,
            response_tx,
        } = req;

        let requester_pid = rt.pid_for_connection(ctx.connection_id).await;
        let Some(requester_pid) = requester_pid else {
            let _ = response_tx.send(ProxyCallResponse {
                ok: false,
                result: None,
                error: Some(crate::protocol::RpcError {
                    code: "unauthorized".into(),
                    message: "unknown requester pid".into(),
                }),
            });
            return;
        };

        let is_singleton_target = {
            let singleton_pids = rt.singleton_pids.lock().await;
            singleton_pids.values().any(|&pid| pid == request.target_pid)
        };

        let allowed = if is_singleton_target {
            true
        } else {
            match Self::is_owned_child(rt, requester_pid, request.target_pid)
                .await
            {
                Ok(v) => v,
                Err(e) => {
                    let _ = response_tx.send(ProxyCallResponse {
                        ok: false,
                        result: None,
                        error: Some(crate::protocol::RpcError {
                            code: "internal".into(),
                            message: format!("ownership check failed: {e:#}"),
                        }),
                    });
                    return;
                }
            }
        };

        if !allowed {
            let _ = response_tx.send(ProxyCallResponse {
                ok: false,
                result: None,
                error: Some(crate::protocol::RpcError {
                    code: "unauthorized".into(),
                    message: "target pid is not owned by requester".into(),
                }),
            });
            return;
        }

        let resp =
            match Self::forward_call_to_child(rt, ctx.connection_id, &request)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = response_tx.send(ProxyCallResponse {
                        ok: false,
                        result: None,
                        error: Some(crate::protocol::RpcError {
                            code: "internal".into(),
                            message: format!("proxy call failed: {e:#}"),
                        }),
                    });
                    return;
                }
            };

        let _ = response_tx.send(resp);
    }

    async fn is_owned_child(
        rt: &WorkspaceRuntime,
        owner: crate::types::ProcessId,
        target: crate::types::ProcessId,
    ) -> Result<bool> {
        let children = rt
            .process_manager
            .list_children()
            .await
            .map_err(|e| anyhow::anyhow!("list_children failed: {e:#}"))?;
        Ok(children
            .iter()
            .any(|h| h.pid == target && h.parent == Some(owner)))
    }

    async fn forward_call_to_child(
        rt: &WorkspaceRuntime,
        requester_conn: crate::types::ConnectionId,
        request: &ProxyCallRequest,
    ) -> Result<ProxyCallResponse> {
        let peer = rt.peer_for_pid(request.target_pid).await?;

        #[cfg(unix)]
        let resp = {
            let fd_count = usize::try_from(request.fd_count)
                .map_err(|e| anyhow::anyhow!("invalid fd_count: {e}"))?;

            if fd_count == 0 {
                peer.call(request.method.clone(), request.args.clone())
                    .await?
            } else {
                let requester_peer = rt
                    .peer_for_connection(requester_conn)
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!("requester peer lookup failed: {e:#}")
                    })?;

                let mut fds = Vec::with_capacity(fd_count);
                for _ in 0..fd_count {
                    let fd = requester_peer.recv_fd().await.map_err(|e| {
                        anyhow::anyhow!("recv_fd failed: {e:#}")
                    })?;
                    fds.push(fd);
                }

                peer.call_raw_with_fds(
                    request.method.clone(),
                    request.args.clone(),
                    fds,
                )
                .await?
            }
        };

        #[cfg(not(unix))]
        let resp = {
            if request.fd_count != 0 {
                anyhow::bail!(
                    "proxy_call included fds but platform does not support fd passing"
                );
            }
            peer.call(request.method.clone(), request.args.clone())
                .await?
        };

        Ok(ProxyCallResponse {
            ok: resp.ok,
            result: resp.result,
            error: resp.error,
        })
    }

    async fn handle_spawn_request_static(
        workspace: &W,
        rt: &WorkspaceRuntime,
        req: SpawnRequestWithResponse,
    ) {
        let SpawnRequestWithResponse {
            ctx,
            request,
            response_tx,
        } = req;

        let requester_pid = rt.pid_for_connection(ctx.connection_id).await;
        let requester = SpawnRequester {
            ctx,
            pid: requester_pid,
        };

        let decision = workspace
            .evaluate_spawn_request(rt, requester, request.clone())
            .await;

        let (parent, caps, extra_args) = match decision {
            Ok(SpawnRequestDecision::Accept {
                parent,
                caps,
                extra_args,
            }) => (parent, caps, extra_args),
            Ok(SpawnRequestDecision::Reject { error }) => {
                let _ = response_tx.send(SpawnChildResponse {
                    accepted: false,
                    child_pid: None,
                    error,
                });
                return;
            }
            Err(e) => {
                let _ = response_tx.send(SpawnChildResponse {
                    accepted: false,
                    child_pid: None,
                    error: Some(format!("spawn evaluation failed: {e:#}")),
                });
                return;
            }
        };

        // If the service is a singleton and it has already been spawned, return the
        // existing PID instead of spawning a new instance.
        let spawn_result = {
            let singleton_pids = rt.singleton_pids.lock().await;
            if let Some(&pid) = singleton_pids.get(&request.kind) {
                Ok(process::ChildProcess {
                    pid,
                    rt: rt.clone(),
                })
            } else {
                drop(singleton_pids);
                let spawn_parent = parent.or(requester_pid);
                rt.start_service_with_args(
                    request.kind,
                    caps,
                    spawn_parent,
                    extra_args,
                )
                .await
            }
        };

        let child = match spawn_result {
            Ok(child) => child,
            Err(e) => {
                let _ = response_tx.send(SpawnChildResponse {
                    accepted: false,
                    child_pid: None,
                    error: Some(format!("spawn failed: {e:#}")),
                });
                return;
            }
        };

        let pid = child.pid;

        let _ = response_tx.send(SpawnChildResponse {
            accepted: true,
            child_pid: Some(pid),
            error: None,
        });
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[derive(Debug, Default)]
    struct NoopWorkspace {
        services: WorkspaceServices,
    }

    #[async_trait::async_trait]
    impl Workspace for NoopWorkspace {
        fn services(&self) -> &WorkspaceServices {
            &self.services
        }

        fn set_services(&mut self, services: WorkspaceServices) {
            self.services = services;
        }

        async fn boot(&mut self, _rt: &WorkspaceRuntime) -> Result<()> {
            Ok(())
        }

        async fn run(&self, _rt: &WorkspaceRuntime) -> Result<()> {
            Ok(())
        }

        async fn on_parent_message(
            &self,
            _rt: &WorkspaceRuntime,
            _event: ParentMessageEvent,
        ) -> Result<()> {
            Ok(())
        }

        async fn evaluate_spawn_request(
            &self,
            _rt: &WorkspaceRuntime,
            _requester: SpawnRequester,
            _request: SpawnChildRequest,
        ) -> Result<SpawnRequestDecision> {
            Ok(SpawnRequestDecision::Reject { error: None })
        }
    }

    #[crate::ctb_test]
    fn workspace_runner_creates_socket_path() {
        let runner = WorkspaceRunner::new(
            NoopWorkspace::default(),
            WorkspaceRunnerConfig::default_with_timeout(),
            Arc::new(std::path::PathBuf::from("ctb-example")),
            None,
        );
        assert!(runner.socket_path().contains("ctb-workspace-"));
    }

    #[crate::ctb_test]
    fn workspace_runner_registers_tokens() {
        let runner = WorkspaceRunner::new(
            NoopWorkspace::default(),
            WorkspaceRunnerConfig::default_with_timeout(),
            Arc::new(std::path::PathBuf::from("ctb-example")),
            None,
        );
        let rt = runner.runtime();

        let token = rt.register_token("test", CapabilitySet::default());
        assert!(token.starts_with("test-"));
        assert!(runner.validator.has_token(&token));
    }
}
