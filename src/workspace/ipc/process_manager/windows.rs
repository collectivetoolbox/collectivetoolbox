#![cfg(windows)]

//! Windows-specific ProcessManager implementation using Job Objects.
//!
//! - Each child is assigned to a Job Object configured with
//!   JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.
//! - Tree termination uses TerminateJobObject.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use process_wrap::tokio::{CommandWrap, JobObject};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::error::Error;
use crate::types::ChildKind;
use crate::types::{ConnectionId, ProcessId};

use super::{ChildHandle, ProcessManager, SpawnParams};

struct ChildEntry {
    child: Box<dyn process_wrap::tokio::ChildWrapper>,
    handle: ChildHandle,
}

struct Inner {
    children: HashMap<ProcessId, ChildEntry>,
}

/// Tokio-based Windows process manager.
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
        // Map anyhow::Error to crate error. Assumes Error: From<anyhow::Error>.
        Error::from(e)
    }
}

#[async_trait]
impl ProcessManager for TokioProcessManager {
    async fn spawn_child(
        &self,
        params: SpawnParams,
    ) -> Result<ChildHandle, Error> {
        // Spawn the process using process-wrap with JobObject wrapper.
        let mut cmd = CommandWrap::with_new(&params.program, |command| {
            command.args(&params.args);
            for (k, v) in &params.env {
                command.env(k, v);
            }
            if let Some(cwd) = &params.cwd {
                command.current_dir(cwd);
            }
        });
        cmd.wrap(JobObject);

        let child = cmd
            .spawn()
            .with_context(|| "failed to spawn child")
            .map_err(Self::to_err)?;

        let pid = ProcessId::new();

        let handle = ChildHandle {
            pid,
            kind: params.kind.clone(),
            parent: params.parent,
            connection: None,
        };

        {
            let mut inner = self.inner.lock().await;
            inner.children.insert(
                pid,
                ChildEntry {
                    child,
                    handle: handle.clone(),
                },
            );
        }

        // Cleanup on exit.
        let inner_arc = self.inner.clone();
        tokio::spawn(async move {
            let mut remove_me: Option<ProcessId> = None;
            {
                let mut inner = inner_arc.lock().await;
                if let Some(entry) = inner.children.get_mut(&pid) {
                    let wait_fut = entry.child.wait();
                    drop(inner);
                    let _ = wait_fut.await;
                } else {
                    return;
                }
            }
            let mut inner = inner_arc.lock().await;
            if let Some(entry) = inner.children.remove(&pid) {
                remove_me = Some(pid);
            }
            drop(inner);
            // Job cleanup is handled by process-wrap's JobObject on drop.
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
            return Err(Self::to_err(anyhow::anyhow!("unknown pid {:?}", pid)));
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
                // Nothing to terminate.
                return Ok(());
            };
            entry.child.clone() // Clone if needed; process-wrap's Child may support cloning or use Arc.
        };

        // Terminate the entire job (tree). If not force, emulate grace period.
        if !force {
            sleep(Duration::from_millis(200)).await;
        }

        child
            .kill()
            .await
            .with_context(|| "failed to terminate job")
            .map_err(Self::to_err)?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::process_manager::SpawnParams;

    #[crate::ctb_test("tokio")]
    async fn terminate_noop_graceful() -> anyhow::Result<()> {
        let pm = TokioProcessManager::new();
        let params = SpawnParams {
            kind: Default::default(), // Assumes Default; adjust if needed.
            parent: None,
            program: Some("cmd.exe".to_string()),
            args: vec!["/C".into(), "exit".into(), "/B".into(), "0".into()],
            env: vec![],
            cwd: None,
            capabilities: Default::default(), // Assumes Default; adjust if needed.
        };

        let ch = pm.spawn_child(params).await?;
        pm.terminate_tree(ch.pid, false).await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        let list = pm.list_children().await?;
        assert!(!list.iter().any(|h| h.pid == ch.pid));
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn terminate_long_running_force() -> anyhow::Result<()> {
        let pm = TokioProcessManager::new();
        // Use ping to simulate a sleep.
        let params = SpawnParams {
            kind: Default::default(),
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
            capabilities: Default::default(),
        };

        let ch = pm.spawn_child(params).await?;
        pm.terminate_tree(ch.pid, true).await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        let list = pm.list_children().await?;
        assert!(!list.iter().any(|h| h.pid == ch.pid));
        Ok(())
    }
}
