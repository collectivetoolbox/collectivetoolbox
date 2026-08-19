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

//! Tokio-based process management with platform-specific supervision.
//!
//! On Unix, children are placed in their own process group and (on Linux) are
//! configured with a parent-death signal. Tree termination uses killpg.
//!
//! On Windows, children are placed into a Job Object configured to kill all
//! processes on close; tree termination uses `TerminateJobObject`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::auth::capability::CapabilityBundle;
use crate::error::Error;
use crate::peer::IpcPeer;
use crate::services::process::SERVICE_NAME as PROCESS_SERVICE_NAME;
use crate::services::process::api::{
    METHOD_SHUTDOWN_TREE, ShutdownTreeRequest, ShutdownTreeResponse,
};
use crate::types::{ConnectionId, ProcessId};
use async_trait::async_trait;
use ipc::ChildKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

pub mod unix;
pub mod windows;

/// Parameters for spawning a child.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnParams {
    pub kind: ChildKind,
    /// Optional parent process id for tree tracking.
    pub parent: Option<ProcessId>,
    /// Executable or command line.
    pub program: Option<String>,
    /// Arguments for the child.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: Vec<(String, String)>,
    /// Working directory.
    pub cwd: Option<String>,
    /// Capability bundle bound to the child's control connection.
    pub capabilities: CapabilityBundle,
}

/// A handle with metadata tracked by the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildHandle {
    pub pid: ProcessId,
    pub kind: ChildKind,
    pub parent: Option<ProcessId>,
    pub connection: Option<ConnectionId>,
}

/// Supervisor/process manager for spawning and tracking child trees with
/// OS-level supervision.
#[async_trait]
pub trait ProcessManager: Send + Sync + std::fmt::Debug {
    async fn spawn_child(
        &self,
        params: SpawnParams,
    ) -> Result<ChildHandle, Error>;

    async fn attach_connection(
        &self,
        pid: ProcessId,
        conn: ConnectionId,
    ) -> Result<(), Error>;

    async fn list_children(&self) -> Result<Vec<ChildHandle>, Error>;

    /// Terminate a process tree. This is invoked by liveness checks such as
    /// the heartbeat tracker when a connection is considered dead.
    async fn terminate_tree(
        &self,
        pid: ProcessId,
        force: bool,
    ) -> Result<(), Error>;

    /// Wait for a process to exit, up to `timeout`.
    ///
    /// Returns `Ok(true)` if the process is known to have exited within the
    /// timeout, `Ok(false)` if the wait timed out.
    ///
    /// Platform managers should override this to actually reap/wait on the
    /// underlying child handle. The default implementation is conservative and
    /// simply waits out the timeout.
    async fn wait_for_exit(
        &self,
        _pid: ProcessId,
        timeout: Duration,
    ) -> Result<bool, Error> {
        time::sleep(timeout).await;
        Ok(false)
    }

    /// Force kill a process tree.
    ///
    /// Default behavior delegates to `terminate_tree(pid, true)`.
    async fn kill_tree(&self, pid: ProcessId) -> Result<(), Error> {
        self.terminate_tree(pid, true).await
    }
}

/// Outcome of a graceful shutdown attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GracefulShutdownOutcome {
    /// Whether the shutdown request was acknowledged within `ack_timeout`.
    pub acknowledged: bool,
    /// Whether the process exited within the relevant timeout.
    pub exited: bool,
    /// Whether the shutdown path required a forced tree kill.
    pub forced: bool,
}

/// Attempt a graceful shutdown of `pid`.
///
/// The caller provides `send_shutdown`, which should represent the RPC to the
/// target process' `process.shutdown_tree` handler, returning when the target
/// acknowledges the request.
///
/// Behavior:
/// - wait for ack up to `ack_timeout`
/// - wait for exit up to `exit_timeout`
/// - if either step times out, force-kill the tree and wait again up to
///   `exit_timeout`
///
/// This is intentionally transport-agnostic: IPC wiring lives elsewhere.
pub async fn graceful_shutdown_tree<F, Fut>(
    process_manager: &dyn ProcessManager,
    pid: ProcessId,
    send_shutdown: F,
    ack_timeout: Duration,
    exit_timeout: Duration,
) -> Result<GracefulShutdownOutcome, Error>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = Result<ShutdownTreeResponse, Error>> + Send,
{
    // If the process tree has already exited (common when a child initiates
    // shutdown and the workspace observes the request after the fact), treat
    // this as a clean shutdown.
    let already_exited = process_manager
        .wait_for_exit(pid, Duration::from_millis(0))
        .await?;
    if already_exited {
        return Ok(GracefulShutdownOutcome {
            acknowledged: false,
            exited: true,
            forced: false,
        });
    }

    let acknowledged = match time::timeout(ack_timeout, send_shutdown()).await {
        Ok(Ok(resp)) => resp.acknowledged,
        Ok(Err(_)) => false,
        Err(_elapsed) => false,
    };

    if acknowledged {
        let exited = process_manager.wait_for_exit(pid, exit_timeout).await?;
        if exited {
            return Ok(GracefulShutdownOutcome {
                acknowledged,
                exited,
                forced: false,
            });
        }
    }

    process_manager.kill_tree(pid).await?;
    let exited = process_manager.wait_for_exit(pid, exit_timeout).await?;

    Ok(GracefulShutdownOutcome {
        acknowledged,
        exited,
        forced: true,
    })
}

/// Sends a shutdown request via IPC to the specified peer and its subtree.
///
/// This function handles the IPC communication for graceful shutdown, including
/// notifying the target peer (e.g., renderer) and best-effort notifications to
/// other peers in the map.
async fn send_shutdown_request(
    #[expect(
        clippy::implicit_hasher,
        reason = "uniform peer dictionary mapping type interface"
    )]
    peers: Arc<tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>>,
    pid: ProcessId,
    reason: Option<String>,
) -> Result<ShutdownTreeResponse, Error> {
    let req = ShutdownTreeRequest { reason };
    let args = postcard_helpers::encode(&req, "shutdown request")
        .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;

    let (parent_peer, other_peers) = {
        let guard = peers.lock().await;
        let parent_peer = guard.get(&pid).cloned().ok_or_else(|| {
            crate::error::Error::Internal("target peer not connected".into())
        })?;

        let mut others: Vec<Arc<IpcPeer>> = Vec::new();
        for (peer_pid, peer) in &*guard {
            if *peer_pid != pid {
                others.push(Arc::clone(peer));
            }
        }

        (parent_peer, others)
    };

    // Best-effort: ask all other known children to shut down too, so that
    // the target subtree can exit cleanly.
    for peer in other_peers {
        let args_clone = args.clone();
        drop(tokio::spawn(async move {
            let _ = peer
                .call(
                    crate::protocol::MethodId {
                        service: PROCESS_SERVICE_NAME.into(),
                        method: METHOD_SHUTDOWN_TREE.into(),
                    },
                    args_clone,
                )
                .await;
        }));
    }

    let resp = parent_peer
        .call(
            crate::protocol::MethodId {
                service: PROCESS_SERVICE_NAME.into(),
                method: METHOD_SHUTDOWN_TREE.into(),
            },
            args,
        )
        .await?;

    if !resp.ok {
        return Ok(ShutdownTreeResponse {
            acknowledged: false,
        });
    }

    let Some(bytes) = resp.result else {
        return Ok(ShutdownTreeResponse {
            acknowledged: false,
        });
    };

    postcard_helpers::decode::<ShutdownTreeResponse>(
        &bytes,
        "shutdown response",
    )
    .map_err(|e| crate::error::Error::Serialization(e.to_string()))
}

/// Performs a graceful shutdown for the specified process using IPC peers.
///
/// This function wraps `graceful_shutdown_tree` with IPC-based shutdown logic,
/// allowing shutdown requests to be sent via connected peers.
pub async fn shutdown_for_process_tree(
    #[expect(
        clippy::implicit_hasher,
        reason = "uniform peer dictionary mapping type interface"
    )]
    peers: Arc<tokio::sync::Mutex<HashMap<ProcessId, Arc<IpcPeer>>>>,
    process_manager: &dyn ProcessManager,
    pid: ProcessId,
    ack_timeout: Duration,
    exit_timeout: Duration,
    reason: Option<String>,
) -> Result<GracefulShutdownOutcome, Error> {
    graceful_shutdown_tree(
        process_manager,
        pid,
        || send_shutdown_request(Arc::clone(&peers), pid, reason.clone()),
        ack_timeout,
        exit_timeout,
    )
    .await
}

#[derive(Debug, Default)]
pub(crate) struct MockProcessManager {
    terminated: std::sync::Mutex<Vec<(ProcessId, bool)>>,
    killed: std::sync::atomic::AtomicUsize,
    waited: std::sync::atomic::AtomicUsize,
    exited: Arc<tokio::sync::Notify>,
}

impl MockProcessManager {
    pub(crate) fn new(exited: Arc<tokio::sync::Notify>) -> Self {
        Self {
            terminated: Default::default(),
            killed: std::sync::atomic::AtomicUsize::new(0),
            waited: std::sync::atomic::AtomicUsize::new(0),
            exited,
        }
    }

    pub(crate) fn terminations(&self) -> Vec<(ProcessId, bool)> {
        if let Ok(guard) = self.terminated.lock() {
            guard.clone()
        } else {
            Vec::new()
        }
    }
}

#[async_trait]
impl ProcessManager for MockProcessManager {
    async fn spawn_child(
        &self,
        _params: SpawnParams,
    ) -> Result<ChildHandle, Error> {
        Ok(ChildHandle {
            pid: ProcessId::default(),
            kind: ChildKind::Renderer,
            parent: None,
            connection: None,
        })
    }

    async fn attach_connection(
        &self,
        _pid: ProcessId,
        _conn: ConnectionId,
    ) -> Result<(), Error> {
        Ok(())
    }

    async fn list_children(&self) -> Result<Vec<ChildHandle>, Error> {
        Ok(Vec::new())
    }

    async fn terminate_tree(
        &self,
        pid: ProcessId,
        force: bool,
    ) -> Result<(), Error> {
        if let Ok(mut guard) = self.terminated.lock() {
            guard.push((pid, force));
        }
        self.killed
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let exited = Arc::new(self.exited.clone());
        tokio::spawn(async move {
            time::sleep(Duration::from_millis(1)).await;
            exited.notify_waiters();
        });
        Ok(())
    }

    async fn wait_for_exit(
        &self,
        _pid: ProcessId,
        timeout: Duration,
    ) -> Result<bool, Error> {
        self.waited
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        match time::timeout(timeout, self.exited.notified()).await {
            Ok(()) => Ok(true),
            Err(_elapsed) => Ok(false),
        }
    }
}

#[cfg(unix)]
pub use unix::TokioProcessManager;
#[cfg(windows)]
pub use windows::TokioProcessManager;

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

    use anyhow::Result;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    #[crate::ctb_test("tokio")]
    async fn graceful_shutdown_cooperative_child() -> Result<()> {
        let exited = Arc::new(tokio::sync::Notify::new());
        let pm = MockProcessManager::new(exited.clone());
        let pid = ProcessId::default();

        let outcome = graceful_shutdown_tree(
            &pm,
            pid,
            || async {
                let exited = exited.clone();
                drop(tokio::spawn(async move {
                    time::sleep(Duration::from_millis(10)).await;
                    exited.notify_waiters();
                }));
                Ok(ShutdownTreeResponse { acknowledged: true })
            },
            Duration::from_millis(50),
            Duration::from_millis(200),
        )
        .await?;

        anyhow::ensure!(
            outcome
                == GracefulShutdownOutcome {
                    acknowledged: true,
                    exited: true,
                    forced: false,
                },
            "unexpected outcome: {outcome:?}"
        );
        anyhow::ensure!(pm.killed.load(Ordering::SeqCst) == 0);
        anyhow::ensure!(pm.waited.load(Ordering::SeqCst) >= 1);
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn graceful_shutdown_ignoring_child_forces_kill() -> Result<()> {
        let exited = Arc::new(tokio::sync::Notify::new());
        let pm = MockProcessManager::new(exited.clone());
        let pid = ProcessId::default();

        let outcome = graceful_shutdown_tree(
            &pm,
            pid,
            || async {
                std::future::pending::<Result<ShutdownTreeResponse, Error>>()
                    .await
            },
            Duration::from_millis(20),
            Duration::from_millis(200),
        )
        .await?;

        anyhow::ensure!(!outcome.acknowledged);
        anyhow::ensure!(outcome.forced);
        anyhow::ensure!(outcome.exited);
        anyhow::ensure!(pm.killed.load(Ordering::SeqCst) == 1);
        anyhow::ensure!(pm.waited.load(Ordering::SeqCst) >= 1);
        Ok(())
    }
}
