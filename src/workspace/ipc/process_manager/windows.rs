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

//! Windows-specific ProcessManager implementation using Job Objects.
//!
//! - Each child is assigned to a Job Object configured with
//!   JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.
//! - Tree termination uses TerminateJobObject.

#![cfg(windows)]

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use process_wrap::tokio::{CommandWrap, JobObject, KillOnDrop};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::time::{Duration, sleep};

use crate::error::Error;
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

use super::{ChildHandle, ProcessManager, SpawnParams};

#[derive(Debug)]
struct ChildEntry {
    terminate: mpsc::UnboundedSender<oneshot::Sender<Result<(), Error>>>,
    handle: ChildHandle,
    ancestors: Vec<ProcessId>,
}

#[derive(Debug)]
struct Inner {
    children: HashMap<ProcessId, ChildEntry>,
}

/// Tokio-based Windows process manager.
#[derive(Debug)]
pub struct TokioProcessManager {
    inner: Arc<Mutex<Inner>>,
}

impl TokioProcessManager {
    /// Create a new manager instance.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(Mutex::new(Inner {
                children: HashMap::new(),
            })),
        })
    }

    fn to_err(e: anyhow::Error) -> Error {
        Error::from(e)
    }
}

#[async_trait]
impl ProcessManager for TokioProcessManager {
    async fn spawn_child(
        &self,
        params: SpawnParams,
    ) -> Result<ChildHandle, Error> {
        let program = if let Some(p) = params.program {
            p
        } else {
            if params.kind == ChildKind::External {
                return Err(Error::from(anyhow!(
                    "No program specified for child process"
                )));
            }
            std::env::current_exe()?.into_os_string().into_string()?
        };

        let mut cmd = CommandWrap::with_new(&program, |command| {
            command.args(&params.args);
            command.stdin(Stdio::piped());
            for (k, v) in &params.env {
                command.env(k, v);
            }
            if let Some(cwd) = &params.cwd {
                command.current_dir(cwd);
            }
        });
        cmd.wrap(KillOnDrop).wrap(JobObject);

        let mut child = cmd
            .spawn()
            .context("failed to spawn child")
            .map_err(Self::to_err)?;

        let token = params.capabilities.token.0.clone();
        if !token.is_empty() {
            if let Some(stdin) = child.stdin() {
                stdin
                    .write_all(token.as_bytes())
                    .await
                    .map_err(Error::from)?;
                stdin.write_all(b"\n").await.map_err(Error::from)?;
            }
        }

        let pid = ProcessId::new();

        let handle = ChildHandle {
            pid,
            kind: params.kind,
            parent: params.parent,
            connection: None,
        };

        let (terminate, mut terminate_rx) =
            mpsc::unbounded_channel::<oneshot::Sender<Result<(), Error>>>();
        {
            let mut inner = self.inner.lock().await;
            let mut ancestors = Vec::new();
            if let Some(parent) = handle.parent {
                ancestors.push(parent);
                if let Some(entry) = inner.children.get(&parent) {
                    ancestors.extend_from_slice(&entry.ancestors);
                }
            }
            inner.children.insert(
                pid,
                ChildEntry {
                    terminate,
                    handle: handle.clone(),
                    ancestors,
                },
            );
        }

        let inner_arc = Arc::clone(&self.inner);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    status = child.wait() => {
                        if let Err(error) = status {
                            warn_fmt!("failed to reap child {pid:?}: {error}");
                        }
                        break;
                    }
                    Some(response) = terminate_rx.recv() => {
                        let result = child.start_kill()
                            .context("failed to terminate job")
                            .map_err(Self::to_err);
                        let _ = response.send(result);
                    }
                }
            }
            let mut inner = inner_arc.lock().await;
            let _ = inner.children.remove(&pid);
        });

        Ok(handle)
    }

    async fn attach_connection(
        &self,
        pid: ProcessId,
        conn: ConnectionId,
    ) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        let Some(entry) = inner.children.get_mut(&pid) else {
            return Err(Self::to_err(anyhow!("unknown pid {:?}", pid)));
        };
        entry.handle.connection = Some(conn);
        Ok(())
    }

    async fn list_children(&self) -> Result<Vec<ChildHandle>, Error> {
        let inner = self.inner.lock().await;
        Ok(inner.children.values().map(|e| e.handle.clone()).collect())
    }

    async fn terminate_tree(
        &self,
        pid: ProcessId,
        force: bool,
    ) -> Result<(), Error> {
        let targets = {
            let inner = self.inner.lock().await;
            inner.children.iter()
                .filter(|(child_pid, entry)| **child_pid == pid || entry.ancestors.contains(&pid))
                .map(|(child_pid, entry)| (*child_pid, entry.terminate.clone()))
                .collect::<Vec<_>>()
        };

        if !force {
            sleep(Duration::from_millis(200)).await;
        }

        for (child_pid, terminate) in targets {
            let (response, response_rx) = oneshot::channel();
            if terminate.send(response).is_ok() {
                if let Ok(result) = response_rx.await {
                    result?;
                    continue;
                }
            }
            if !self.wait_for_exit(child_pid, Duration::ZERO).await? {
                return Err(Self::to_err(anyhow!(
                    "child termination channel closed"
                )));
            }
        }
        Ok(())
    }

    async fn wait_for_exit(
        &self,
        pid: ProcessId,
        timeout: Duration,
    ) -> Result<bool, Error> {
        let is_tree_gone = |inner: &Inner| -> bool {
            !inner.children.iter().any(|(child_pid, entry)| {
                *child_pid == pid || entry.ancestors.contains(&pid)
            })
        };

        let step = Duration::from_millis(20);
        let mut waited = Duration::from_millis(0);
        loop {
            {
                let inner = self.inner.lock().await;
                if is_tree_gone(&inner) {
                    return Ok(true);
                }
            }
            if waited >= timeout {
                break;
            }
            sleep(step).await;
            let Some(next_waited) = waited.checked_add(step) else {
                return Err(Self::to_err(anyhow!("timeout duration overflow")));
            };
            waited = next_waited;
        }

        let inner = self.inner.lock().await;
        Ok(is_tree_gone(&inner))
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
    use crate::auth::capability::CapabilityBundle;
    use crate::process_manager::SpawnParams;

    fn long_running_params(parent: Option<ProcessId>) -> SpawnParams {
        SpawnParams {
            kind: ChildKind::External,
            parent,
            program: Some("cmd.exe".to_string()),
            args: vec![
                "/C".into(),
                "ping".into(),
                "127.0.0.1".into(),
                "-n".into(),
                "30".into(),
            ],
            env: vec![],
            cwd: None,
            capabilities: CapabilityBundle::default(),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn terminate_noop_graceful() -> Result<()> {
        let pm = TokioProcessManager::new();
        let params = SpawnParams {
            kind: ChildKind::External,
            parent: None,
            program: Some("cmd.exe".to_string()),
            args: vec!["/C".into(), "exit".into(), "/B".into(), "0".into()],
            env: vec![],
            cwd: None,
            capabilities: CapabilityBundle::default(),
        };

        let ch = pm.spawn_child(params).await?;
        pm.terminate_tree(ch.pid, false).await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        let list = pm.list_children().await?;
        assert!(!list.iter().any(|h| h.pid == ch.pid));
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn terminate_long_running_force() -> Result<()> {
        let pm = TokioProcessManager::new();
        let ch = pm.spawn_child(long_running_params(None)).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        tokio::time::timeout(
            Duration::from_secs(2),
            pm.terminate_tree(ch.pid, true),
        )
        .await??;
        anyhow::ensure!(
            pm.wait_for_exit(ch.pid, Duration::from_secs(2)).await?,
            "child did not exit after forced termination"
        );
        let list = pm.list_children().await?;
        assert!(!list.iter().any(|h| h.pid == ch.pid));
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn terminate_managed_descendants() -> Result<()> {
        let pm = TokioProcessManager::new();
        let parent = pm.spawn_child(long_running_params(None)).await?;
        let child = pm
            .spawn_child(long_running_params(Some(parent.pid)))
            .await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        tokio::time::timeout(
            Duration::from_secs(2),
            pm.terminate_tree(parent.pid, true),
        )
        .await??;
        anyhow::ensure!(
            pm.wait_for_exit(parent.pid, Duration::from_secs(2)).await?,
            "parent tree did not exit"
        );
        anyhow::ensure!(
            pm.wait_for_exit(child.pid, Duration::ZERO).await?,
            "managed descendant survived parent termination"
        );
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn terminate_descendant_after_intermediate_exit() -> Result<()> {
        let pm = TokioProcessManager::new();
        let root = pm.spawn_child(long_running_params(None)).await?;
        let intermediate = pm
            .spawn_child(long_running_params(Some(root.pid)))
            .await?;
        let descendant = pm
            .spawn_child(long_running_params(Some(intermediate.pid)))
            .await?;
        let terminate = {
            let inner = pm.inner.lock().await;
            inner.children.get(&intermediate.pid)
                .context("missing intermediate child")?.terminate.clone()
        };
        let (response, response_rx) = oneshot::channel();
        terminate.send(response)?;
        response_rx.await??;
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if !pm.list_children().await?.iter()
                    .any(|child| child.pid == intermediate.pid) {
                    return Ok::<(), anyhow::Error>(());
                }
                sleep(Duration::from_millis(20)).await;
            }
        }).await??;
        pm.terminate_tree(root.pid, true).await?;
        let exited = pm.wait_for_exit(descendant.pid, Duration::from_secs(2)).await?;
        pm.terminate_tree(descendant.pid, true).await?;
        anyhow::ensure!(exited, "descendant survived after intermediate exit");
        Ok(())
    }
}
