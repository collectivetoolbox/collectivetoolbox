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
use process_wrap::tokio::{CommandWrap, JobObject};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::error::Error;
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

use super::{ChildHandle, ProcessManager, SpawnParams};

#[derive(Debug)]
struct ChildEntry {
    child: Arc<Mutex<Box<dyn process_wrap::tokio::ChildWrapper>>>,
    handle: ChildHandle,
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
        cmd.wrap(JobObject);

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
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(Error::from)?;
            }
        }

        let pid = ProcessId::new();

        let handle = ChildHandle {
            pid,
            kind: params.kind,
            parent: params.parent,
            connection: None,
        };

        let child_arc = Arc::new(Mutex::new(child));
        {
            let mut inner = self.inner.lock().await;
            inner.children.insert(
                pid,
                ChildEntry {
                    child: Arc::clone(&child_arc),
                    handle: handle.clone(),
                },
            );
        }

        let inner_arc = Arc::clone(&self.inner);
        let child_for_reaper = Arc::clone(&child_arc);
        tokio::spawn(async move {
            {
                let mut c = child_for_reaper.lock().await;
                let _ = c.wait().await;
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
        let child = {
            let inner = self.inner.lock().await;
            let Some(entry) = inner.children.get(&pid) else {
                return Ok(());
            };
            Arc::clone(&entry.child)
        };

        if !force {
            sleep(Duration::from_millis(200)).await;
        }

        let mut c = child.lock().await;
        c.start_kill()
            .context("failed to terminate job")
            .map_err(Self::to_err)?;

        Ok(())
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
        let params = SpawnParams {
            kind: ChildKind::External,
            parent: None,
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
        };

        let ch = pm.spawn_child(params).await?;
        pm.terminate_tree(ch.pid, true).await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        let list = pm.list_children().await?;
        assert!(!list.iter().any(|h| h.pid == ch.pid));
        Ok(())
    }
}
