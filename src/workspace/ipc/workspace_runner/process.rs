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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::types::ProcessId;

use crate::data_plane::shared_memory::SharedBlobDescriptor;
use crate::peer::IpcPeer;

use crate::workspace_runner::workspace_runtime::WorkspaceRuntime;

use ctb_utilities::ipc::registry::{IpcCallFuture, IpcCaller};

/// Command line / program specification for a child process.
#[derive(Debug, Clone)]
pub struct ChildCommand {
    pub program: std::path::PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: Option<String>,
}

#[derive(Debug)]
pub struct CommandBuilder {
    pub exe: PathBuf,
}

/// A handle to a spawned child process.
#[derive(Debug, Clone)]
pub struct ChildProcess {
    pub pid: ProcessId,
    pub(crate) rt: WorkspaceRuntime,
}

impl ChildProcess {
    /// Wait for the child's control connection and return its peer.
    pub async fn peer(&self) -> Result<Arc<IpcPeer>> {
        let pid = self.pid;
        let peers = Arc::clone(&self.rt.peers);
        let timeout = self.rt.config.connect_timeout;

        tokio::time::timeout(timeout, async move {
            loop {
                if let Some(peer) = peers.lock().await.get(&pid).cloned() {
                    return Ok(peer);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_elapsed| {
            anyhow::anyhow!("timed out waiting for child to connect")
        })?
    }

    /// Make a postcard RPC call and decode the postcard response.
    pub async fn call_postcard<Req, Resp>(
        &self,
        service: &str,
        method: &str,
        req: &Req,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: for<'de> serde::Deserialize<'de>,
    {
        let args = postcard_helpers::encode(req, "request")?;
        let resp = self
            .peer()
            .await?
            .call(
                crate::protocol::MethodId {
                    service: service.into(),
                    method: method.into(),
                },
                args,
            )
            .await?;

        let bytes = crate::response_result_bytes(resp, "IPC call_postcard")?;

        postcard_helpers::decode::<Resp>(&bytes, "response")
    }

    #[cfg(unix)]
    pub async fn call_postcard_with_blob_fd<Req, Resp>(
        &self,
        service: &str,
        method: &str,
        req: &Req,
        blob_descriptor: &SharedBlobDescriptor,
    ) -> Result<Resp>
    where
        Req: serde::Serialize,
        Resp: for<'de> serde::Deserialize<'de>,
    {
        let resp = self
            .peer()
            .await?
            .call_postcard_with_blob_fd(service, method, req, blob_descriptor)
            .await?;
        Ok(resp)
    }
}

impl IpcCaller for ChildProcess {
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();
        Box::pin(async move {
            let resp = self
                .peer()
                .await?
                .call(crate::protocol::MethodId { service, method }, args)
                .await?;

            crate::response_result_bytes(resp, "IPC call_raw")
        })
    }
}

#[cfg(unix)]
impl ctb_utilities::ipc::registry::IpcCallerWithFds for ChildProcess {
    fn call_raw_with_fds(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
        fds: Vec<std::os::unix::io::RawFd>,
    ) -> ctb_utilities::ipc::registry::IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();
        Box::pin(async move {
            let resp = self
                .peer()
                .await?
                .call_raw_with_fds(
                    crate::protocol::MethodId { service, method },
                    args,
                    fds,
                )
                .await?;

            crate::response_result_bytes(resp, "IPC call_raw_with_fds")
        })
    }
}
