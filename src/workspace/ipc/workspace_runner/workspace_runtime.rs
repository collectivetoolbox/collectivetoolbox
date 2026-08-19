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

//! Workspace runtime loop supervising child processes and IPC services.

use crate::services::renderer::RendererClient;
use crate::services::runtime::RuntimeClient;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use crate::workspace_runner::WorkspaceRunnerConfig;
use crate::workspace_runner::process::{ChildCommand, ChildProcess};

use async_trait::async_trait;
use ctb_utilities::ipc::service_traits::RuntimeClientTrait;
use ctb_utilities::ipc::service_traits::runtime::RuntimeSpawner;

use std::collections::HashMap;
use std::sync::Arc;

use ctb_utilities::ipc::IPC_ARG;
use uuid::Uuid;

use crate::auth::capability::{
    CapabilityBundle, CapabilitySet, CapabilityToken, InMemoryTokenValidator,
};
use crate::process_manager::{
    ProcessManager, SpawnParams, TokioProcessManager,
};
use crate::services::network::NetworkClient;
use crate::services::parent::api::DataPlaneRef;
use crate::services::parent::api::{ParentMessageKind, ShutdownRequest};
use crate::services::parent::channel::ParentMessageEvent;
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

use crate::peer::IpcPeer;

#[derive(Debug, Clone, Default)]
pub(crate) struct ShutdownState {
    pub(crate) requested: bool,
    pub(crate) reason: Option<String>,
}

/// A small runtime handle provided to the workspace implementation.
#[derive(Debug, Clone)]
pub struct WorkspaceRuntime {
    pub(crate) socket_path: String,
    pub(crate) validator: Arc<InMemoryTokenValidator>,
    pub(crate) process_manager: Arc<TokioProcessManager>,
    pub(crate) exe: Arc<std::path::PathBuf>,
    pub(crate) config: WorkspaceRunnerConfig,
    pub(crate) pending_processes:
        Arc<tokio::sync::Mutex<HashMap<String, ProcessId>>>,
    pub(crate) peers: Arc<tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>>,
    pub(crate) pid_by_conn:
        Arc<tokio::sync::Mutex<HashMap<ConnectionId, ProcessId>>>,
    pub(crate) root_pids: Arc<tokio::sync::Mutex<Vec<ProcessId>>>,
    pub(crate) singleton_pids:
        Arc<tokio::sync::Mutex<HashMap<ChildKind, ProcessId>>>,
    pub(crate) shutdown_tx: tokio::sync::mpsc::Sender<Option<String>>,
    pub(crate) shutdown_state: Arc<tokio::sync::Mutex<ShutdownState>>,
    pub(crate) shutdown_notify: Arc<tokio::sync::Notify>,
}

impl WorkspaceRuntime {
    /// Socket path children should connect to.
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }

    /// Check if a singleton service is running and return its `ProcessId`.
    pub async fn get_singleton_pid(
        &self,
        kind: ChildKind,
    ) -> Option<ProcessId> {
        let guard = self.singleton_pids.lock().await;
        guard.get(&kind).copied()
    }

    /// Wait until the workspace runner requests shutdown.
    ///
    /// Simply explained: the runner has its own internal IPC loop which listens for
    /// shutdown requests (Ctrl-C, child requests, timeouts). Your
    /// `Workspace::run()` should usually `select!` on this signal so it can exit
    /// cleanly.
    pub async fn wait_for_shutdown(&self) -> Option<String> {
        loop {
            {
                let guard = self.shutdown_state.lock().await;
                if guard.requested {
                    return guard.reason.clone();
                }
            }
            self.shutdown_notify.notified().await;
        }
    }

    pub(crate) async fn notify_shutdown(&self, reason: Option<String>) {
        let mut guard = self.shutdown_state.lock().await;
        guard.requested = true;
        guard.reason = reason;
        drop(guard);
        self.shutdown_notify.notify_waiters();
    }

    /// Start a process as a "singleton" service (one instance per `ChildKind`).
    ///
    /// This is meant for workspace-level services like network/io/storage.
    pub async fn start_singleton_service(
        &self,
        kind: ChildKind,
        caps: CapabilitySet,
    ) -> Result<ChildProcess> {
        let mut guard = self.singleton_pids.lock().await;
        if let Some(pid) = guard.get(&kind).copied() {
            return Ok(ChildProcess {
                pid,
                rt: self.clone(),
            });
        }
        let child = self.start_service(kind, caps, None).await?;
        guard.insert(kind, child.pid);
        Ok(child)
    }

    /// Start a child process.
    pub async fn start_service(
        &self,
        kind: ChildKind,
        caps: CapabilitySet,
        parent: Option<ProcessId>,
    ) -> Result<ChildProcess> {
        self.start_service_with_args(kind, caps, parent, vec![])
            .await
    }

    /// Start a child process with extra command-line arguments.
    pub async fn start_service_with_args(
        &self,
        kind: ChildKind,
        caps: CapabilitySet,
        parent: Option<ProcessId>,
        extra_args: Vec<String>,
    ) -> Result<ChildProcess> {
        let mut args = invocation_settings::get_settings().command_line_args();
        args.extend(extra_args);

        let pid = self
            .spawn_child_with_command(kind, parent, caps, args)
            .await?;

        Ok(ChildProcess {
            pid,
            rt: self.clone(),
        })
    }

    /// Convenience: start a network service and return a typed client.
    pub async fn start_network_service(
        &self,
        caps: CapabilitySet,
        parent: Option<ProcessId>,
    ) -> Result<NetworkClient> {
        let proc = self.start_service(ChildKind::Network, caps, parent).await?;
        Ok(NetworkClient::from_child(proc))
    }

    /// Convenience: start a renderer service and return a typed client.
    pub async fn start_renderer_service(
        &self,
        caps: CapabilitySet,
        parent: Option<ProcessId>,
    ) -> Result<RendererClient> {
        let proc = self
            .start_service(ChildKind::Renderer, caps, parent)
            .await?;
        Ok(RendererClient::from_child(proc))
    }

    /// Convenience: start a runtime service and return a typed client.
    pub async fn start_runtime_service(
        &self,
        caps: CapabilitySet,
        parent: Option<ProcessId>,
    ) -> Result<RuntimeClient> {
        let proc = self.start_service(ChildKind::Runtime, caps, parent).await?;
        Ok(RuntimeClient::from_child(proc))
    }

    async fn spawn_child_with_command(
        &self,
        kind: ChildKind,
        parent: Option<ProcessId>,
        caps: CapabilitySet,
        extra_args: Vec<String>,
    ) -> Result<ProcessId> {
        let token_for = ipc::format_child_kind(&kind);
        let token = self.register_token(token_for, caps.clone());

        let mut cmd = ChildCommand {
            program: self.exe.to_path_buf(),
            args: vec![],
            env: vec![],
            cwd: None,
        };

        // Built the command-line arguments, inserting them before any custom
        // arguments. Note that because they are un-shifted onto the beginning of the
        // arguemnt list, they are in reverse order here.
        cmd.args.insert(0, "--".into());
        cmd.args.insert(0, self.socket_path.clone());
        cmd.args.insert(0, "--socket".into());
        cmd.args.insert(0, format!("--{token_for}"));
        cmd.args.insert(0, IPC_ARG.into());

        // Append any extra arguments
        cmd.args.extend(extra_args);

        let program = cmd.program.to_string_lossy().into_owned();

        debug_fmt!("spawning child {:?} {:?}", cmd.program, cmd.args);

        let handle = self
            .process_manager
            .spawn_child(SpawnParams {
                kind,
                parent,
                program: Some(program),
                args: cmd.args,
                env: cmd.env,
                cwd: cmd.cwd,
                capabilities: CapabilityBundle {
                    token: CapabilityToken(token),
                    capabilities: caps,
                },
            })
            .await
            .map_err(|e| anyhow::anyhow!("spawn_child failed: {e:#}"))?;

        self.pending_processes
            .lock()
            .await
            .insert(token_for.to_string(), handle.pid);

        if parent.is_none() {
            self.root_pids.lock().await.push(handle.pid);
        }

        Ok(handle.pid)
    }

    /// Resolve the pid associated with a connection, if known.
    pub async fn pid_for_connection(
        &self,
        conn: ConnectionId,
    ) -> Option<ProcessId> {
        self.pid_by_conn.lock().await.get(&conn).copied()
    }

    /// Wait for the peer associated with `pid` to be connected.
    pub async fn peer_for_pid(&self, pid: ProcessId) -> Result<Arc<IpcPeer>> {
        let peers = Arc::clone(&self.peers);
        let timeout = self.config.connect_timeout;

        tokio::time::timeout(timeout, async move {
            loop {
                if let Some(peer) = peers.lock().await.get(&pid).cloned() {
                    return Ok(peer);
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_elapsed| {
            anyhow::anyhow!("timed out waiting for child peer")
        })?
    }

    /// Wait for the peer associated with `conn` to be connected.
    pub async fn peer_for_connection(
        &self,
        conn: ConnectionId,
    ) -> Result<Arc<IpcPeer>> {
        let Some(pid) = self.pid_for_connection(conn).await else {
            anyhow::bail!("no pid registered for connection {conn}");
        };
        self.peer_for_pid(pid).await
    }

    /// Read a data-plane blob referenced by a parent message.
    ///
    /// This intentionally hides FD-transfer and descriptor handling from
    /// workspace implementations.
    pub async fn read_data_plane_ref_bytes(
        &self,
        conn: ConnectionId,
        data_ref: &DataPlaneRef,
    ) -> Result<Vec<u8>> {
        let mut descriptor = data_ref.descriptor.clone();

        #[cfg(unix)]
        {
            if crate::data_plane::shared_memory::descriptor_requires_fd_transfer(
                &descriptor,
            ) {
                let peer = self.peer_for_connection(conn).await?;
                descriptor = crate::data_plane::recv_blob_fd(
                    peer.session(),
                    &descriptor,
                )
                .await
                .map_err(|e| anyhow::anyhow!("recv_blob_fd failed: {e}"))?;
            }
        }

        ctb_utilities::shared_memory::read_blob_contents(
            &descriptor,
            data_ref.token.size,
        )
        .context("read_blob_contents")
    }

    /// Resolve an incoming parent message into a high-level representation.
    ///
    /// Workspace implementations should prefer this over manually decoding
    /// `DataPlaneRef` payloads to avoid coupling example logic to the IPC
    /// data-plane implementation details (including FD transfer).
    pub async fn resolve_parent_message(
        &self,
        event: &ParentMessageEvent,
    ) -> Result<ResolvedParentMessage> {
        match event.message.kind {
            ParentMessageKind::DataPlane => {
                let data_ref =
                    event.message.as_data_plane_ref().map_err(|e| {
                        anyhow::anyhow!("decode data plane ref failed: {e:#}")
                    })?;

                match self
                    .read_data_plane_ref_bytes(
                        event.ctx.connection_id,
                        &data_ref,
                    )
                    .await
                {
                    Ok(bytes) => Ok(ResolvedParentMessage::DataPlaneBytes {
                        bytes,
                        content_type: data_ref.content_type,
                        sequence: data_ref.sequence,
                    }),
                    Err(error) => {
                        Ok(ResolvedParentMessage::DataPlaneReadFailed {
                            error,
                            content_type: data_ref.content_type,
                            sequence: data_ref.sequence,
                        })
                    }
                }
            }
            ParentMessageKind::Text => {
                let text = event.message.as_text().map_err(|e| {
                    anyhow::anyhow!("decode text message failed: {e:#}")
                })?;
                Ok(ResolvedParentMessage::Text(text))
            }
            ParentMessageKind::ShutdownRequest => {
                let ShutdownRequest { reason } =
                    event.message.as_shutdown_request().map_err(|e| {
                        anyhow::anyhow!("decode shutdown request failed: {e:#}")
                    })?;
                Ok(ResolvedParentMessage::ShutdownRequest { reason })
            }
            other => Ok(ResolvedParentMessage::Other { kind: other }),
        }
    }

    /// Request that the workspace event loop shuts down.
    pub async fn request_shutdown(&self, reason: Option<String>) -> Result<()> {
        self.shutdown_tx.send(reason).await.map_err(|e| {
            anyhow::anyhow!("failed to send shutdown request: {e}")
        })?;
        Ok(())
    }

    /// Register a token with capabilities and return the token string.
    pub fn register_token(
        &self,
        token_for: &str,
        caps: CapabilitySet,
    ) -> String {
        let token = format!("{token_for}-{}", Uuid::new_v4());
        self.validator.register_token(&token, caps);
        token
    }
}

/// A high-level decoded parent message.
#[derive(Debug)]
pub enum ResolvedParentMessage {
    /// A successfully resolved data-plane message.
    DataPlaneBytes {
        bytes: Vec<u8>,
        content_type: String,
        sequence: Option<u64>,
    },
    /// A data-plane message whose reference decoded, but whose bytes could
    /// not be read.
    DataPlaneReadFailed {
        error: anyhow::Error,
        content_type: String,
        sequence: Option<u64>,
    },
    /// A plain text message.
    Text(String),
    /// A workspace shutdown request.
    ShutdownRequest { reason: Option<String> },
    /// Any other message kind.
    Other { kind: ParentMessageKind },
}

#[async_trait]
impl RuntimeSpawner for WorkspaceRuntime {
    async fn spawn_runtime(&self) -> Result<Box<dyn RuntimeClientTrait>> {
        // Use default/empty capabilities for trait-based spawning.
        // Callers needing specific capabilities should use
        // `start_runtime_service` directly.
        let caps = CapabilitySet::default();
        let client = self.start_runtime_service(caps, None).await?;
        Ok(Box::new(client))
    }
}
