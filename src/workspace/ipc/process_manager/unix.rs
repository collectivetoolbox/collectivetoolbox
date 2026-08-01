#![cfg(unix)]

//! Unix-specific `ProcessManager` implementation.
//!
//! - Each child is started in its own process group via setpgid(0, 0).
//! - On Linux, `PR_SET_PDEATHSIG` is set to SIGTERM.
//! - Tree termination uses kill on the process group id.
//!
//! Process reaping is based partly on <https://github.com/fpco/pid1-rs>
//!
//! Those parts used under the MIT license, see full license at end.
//! pid1-rs license: Copyright (c) 2023 FP Complete

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use nix::errno::Errno;
use nix::libc;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use process_wrap::tokio::{CommandWrap, ProcessGroup};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};

use crate::error::Error;
use crate::types::{ConnectionId, ProcessId};

use super::{ChildHandle, ProcessManager, SpawnParams};

/// Unix-specific process supervision primitives (to be implemented with nix):
/// - setpgid to create process groups
/// - `prctl(PR_SET_PDEATHSIG)`
/// - killpg for tree termination
pub struct UnixProcessSupervisor;

/// Unix shared memory helpers (memfd, `SCM_RIGHTS`) should be implemented here as needed.
pub struct UnixSharedMemory;

#[derive(Debug)]
struct ChildEntry {
    // child handle is now managed by a background reaper task
    handle: ChildHandle,
    // On Unix, the process group id equals pid when setpgid(0, 0) is used.
    pgid: libc::pid_t,
}

#[derive(Debug)]
struct Inner {
    children: HashMap<ProcessId, ChildEntry>,
}

/// Tokio-based Unix process manager.
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
        // Map anyhow::Error to crate error. Assumes Error: From<anyhow::Error>.
        Error::from(e)
    }

    /// Send a signal to the given process group id, using nix.
    fn signal_pgid(
        pgid: libc::pid_t,
        sig: libc::c_int,
    ) -> Result<(), anyhow::Error> {
        // Use kill with negative pgid to signal the whole process group.
        // nix::unistd::Pid supports negative values for process groups.
        let res = kill(Pid::from_raw(pgid), Signal::try_from(sig)?);
        match res {
            Ok(()) | Err(Errno::EPERM) => Ok(()),
            Err(e) => {
                Err(anyhow::anyhow!("failed to signal process group: {e}"))
            }
        }
    }

    /// Check whether a process group with the given pgid exists, using nix.
    /// We send signal 0 to -pgid to test for any member of that group.
    fn pid_exists(pid: libc::pid_t) -> bool {
        // NOTE: treat `pid` as a process group id (pgid) and test the group.
        let e = kill(Pid::from_raw(pid), None);
        match e {
            Ok(()) | Err(Errno::EPERM) => true,
            Err(_) => false,
        }
    }
}

#[async_trait]
impl ProcessManager for TokioProcessManager {
    async fn spawn_child(
        &self,
        params: SpawnParams,
    ) -> Result<ChildHandle, Error> {
        // Use process-wrap's CommandWrap for safe process group setup
        let program = if let Some(p) = params.program {
            p
        } else {
            if params.kind == ipc::ChildKind::External {
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
        // Apply the ProcessGroup wrapper to safely handle setpgid and PR_SET_PDEATHSIG (on Linux)
        cmd.wrap(ProcessGroup::leader());

        // Spawn the child using the wrapped command
        let mut child = cmd.spawn().context("failed to spawn child")?;

        // Pipe the capability token via stdin to avoid exposing it in argv.
        // The child is expected to read a single line token from stdin.
        let token = params.capabilities.token.0.clone();
        if !token.is_empty() {
            if let Some(stdin) = child.stdin() {
                stdin
                    .write_all(token.as_bytes())
                    .await
                    .map_err(|e: std::io::Error| Error::from(e))?;
                stdin
                    .write_all(b"\n")
                    .await
                    .map_err(|e: std::io::Error| Error::from(e))?;
            }
        }
        let os_pid = child.id().ok_or_else(|| anyhow!("child has no pid"))?;
        let process_id = ProcessId::new();
        let handle = ChildHandle {
            pid: process_id,
            kind: params.kind,
            parent: params.parent,
            connection: None,
        };

        // Compute process group id value we use for signaling (negative value to target group)
        let pgid = i32::try_from(os_pid)
            .map_err(|e| Self::to_err(anyhow!("pid conversion error: {e}")))?
            .checked_neg()
            .ok_or_else(|| Self::to_err(anyhow!("pid negation overflow")))?;

        // Spawn a background reaper task that waits on the child and cleans up
        let inner_for_reaper = Arc::clone(&self.inner);
        let pid_for_reaper = process_id;
        tokio::spawn(async move {
            // Wait for the child to exit and be reaped
            let _ = child.wait().await;
            // Best-effort cleanup of the manager's registry (in case terminate_tree wasn't called)
            let mut inner = inner_for_reaper.lock().await;
            let _ = inner.children.remove(&pid_for_reaper);
        });

        let entry = ChildEntry {
            handle: handle.clone(),
            pgid,
        };
        let mut inner = self.inner.lock().await;
        inner.children.insert(process_id, entry);
        Ok(handle)
    }

    async fn attach_connection(
        &self,
        pid: ProcessId,
        conn: ConnectionId,
    ) -> Result<(), Error> {
        let mut inner = self.inner.lock().await;
        let entry = inner.children.get_mut(&pid).ok_or_else(|| {
            Self::to_err(anyhow!("child with pid {pid:?} not found"))
        })?;
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
        // Determine subtree pids and their process groups.
        let targets = {
            let inner = self.inner.lock().await;

            let mut targets: Vec<(ProcessId, libc::pid_t)> = Vec::new();

            // Build a parent index.
            let mut stack = vec![pid];
            while let Some(cur) = stack.pop() {
                if let Some(entry) = inner.children.get(&cur) {
                    targets.push((cur, entry.pgid));
                    // Note: we cannot clone a oneshot receiver, so we only
                    // best-effort reap via `wait_for_exit`.
                    // Keep the receiver only for the direct pid if present.
                }

                for (child_pid, entry) in &inner.children {
                    if entry.handle.parent == Some(cur) {
                        stack.push(*child_pid);
                    }
                }
            }

            targets
        };

        if targets.is_empty() {
            return Ok(());
        }

        let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
        for (_pid, pgid) in &targets {
            let _ = Self::signal_pgid(*pgid, sig);
        }

        Ok(())
    }

    async fn wait_for_exit(
        &self,
        pid: ProcessId,
        timeout: Duration,
    ) -> Result<bool, Error> {
        // Snapshot subtree process groups.
        let pgids: Vec<libc::pid_t> = {
            let inner = self.inner.lock().await;
            let mut pgids = Vec::new();
            let mut stack = vec![pid];
            while let Some(cur) = stack.pop() {
                if let Some(entry) = inner.children.get(&cur) {
                    pgids.push(entry.pgid);
                }
                for (child_pid, entry) in &inner.children {
                    if entry.handle.parent == Some(cur) {
                        stack.push(*child_pid);
                    }
                }
            }
            pgids
        };

        if pgids.is_empty() {
            return Ok(true);
        }

        let step = Duration::from_millis(50);
        let mut waited = Duration::from_millis(0);
        while waited < timeout {
            if pgids.iter().all(|pgid| !Self::pid_exists(*pgid)) {
                return Ok(true);
            }
            sleep(step).await;
            waited = waited
                .checked_add(step)
                .ok_or_else(|| Self::to_err(anyhow!("timeout duration overflow")))?;
        }

        Ok(pgids.iter().all(|pgid| !Self::pid_exists(*pgid)))
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use tokio::time::timeout;

    use crate::auth::capability::CapabilityBundle;
    use ipc::ChildKind;

    use super::*;

    #[crate::ctb_test("tokio")]
    async fn test_spawn_and_terminate_graceful() {
        let manager = TokioProcessManager::new();
        let params = SpawnParams {
            kind: ChildKind::External,
            parent: None,
            program: Some("sleep".to_string()),
            args: vec!["1".to_string()],
            env: vec![],
            cwd: None,
            capabilities: CapabilityBundle::default(),
        };
        let handle = manager.spawn_child(params).await.unwrap();
        // Terminate with force=false, should wait for graceful exit
        let result = timeout(
            Duration::from_secs(3),
            manager.terminate_tree(handle.pid, false),
        )
        .await;
        assert!(
            result.is_ok(),
            "graceful termination should complete within timeout"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_spawn_and_terminate_force() {
        let manager = TokioProcessManager::new();
        let params = SpawnParams {
            kind: ChildKind::External,
            parent: None,
            program: Some("sleep".to_string()),
            args: vec!["10".to_string()], // Long sleep to test force kill
            env: vec![],
            cwd: None,
            capabilities: CapabilityBundle::default(),
        };
        let handle = manager.spawn_child(params).await.unwrap();
        // Terminate with force=true, should kill immediately
        let result = manager.terminate_tree(handle.pid, true).await;
        assert!(result.is_ok(), "force termination should succeed");
    }
}

/* pid1-rs license:
MIT License

Copyright (c) 2023 FP Complete

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */
