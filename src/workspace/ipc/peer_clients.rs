// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! Cross-service IPC helper utilities.
//!
//! `connection.rs` is intended to be transport/handshake focused. Service-
//! specific client implementations should live under `services/`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::Arc;

use crate::data_plane::send_blob_fd;
use crate::data_plane::shared_memory::{
    BlobAllocator, BlobBackend, SharedMemoryBlobs,
    descriptor_requires_fd_transfer,
};
use crate::peer::IpcPeer;
use crate::protocol::MethodId;
use crate::services::parent::SERVICE_NAME as PARENT_SERVICE_NAME;
use crate::services::parent::api::{
    DataPlaneRef, METHOD_MESSAGE_PARENT, METHOD_PROXY_CALL,
    METHOD_REQUEST_SPAWN_CHILD, MessageToParentRequest, ParentMessage,
    ProxyCallRequest, ProxyCallResponse, SpawnChildRequest, SpawnChildResponse,
};
use crate::types::ProcessId;
use ipc::ChildKind;

#[cfg(unix)]
use ctb_utilities::ipc::registry::IpcCallerWithFds;
use ctb_utilities::ipc::registry::{IpcCallFuture, IpcCaller};

/// Send a text message to the parent process.
pub(crate) async fn send_to_parent(
    peer: &Arc<IpcPeer>,
    message: &str,
) -> Result<()> {
    send_parent_message(peer, ParentMessage::text(message)).await
}

/// Send a data-plane message to the parent process.
///
/// This allocates a shared-memory blob, writes `bytes`, sends a parent message
/// containing a `DataPlaneRef`, and performs FD transfer if required.
pub(crate) async fn send_data_plane_message(
    peer: &Arc<IpcPeer>,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<()> {
    let blobs = SharedMemoryBlobs::new(BlobBackend::PlatformDefault);
    let size = u64::try_from(bytes.len())
        .map_err(|e| anyhow::anyhow!("blob too large: {e}"))?;

    let blob = blobs
        .create(size)
        .await
        .map_err(|e| anyhow::anyhow!("failed to allocate blob: {e}"))?;

    blob.write_all(&bytes)
        .map_err(|e| anyhow::anyhow!("failed to write blob: {e}"))?;

    let data_ref = DataPlaneRef::new(
        blob.token.clone(),
        blob.descriptor.clone(),
        content_type.to_string(),
    );

    let msg = ParentMessage::data_plane(&data_ref).map_err(|e| {
        anyhow::anyhow!("failed to encode data plane message: {e}")
    })?;

    send_parent_message(peer, msg).await?;

    if descriptor_requires_fd_transfer(&blob.descriptor) {
        send_blob_fd(peer.session(), &blob.descriptor)
            .await
            .map_err(|e| anyhow::anyhow!("failed to send blob fd: {e}"))?;
    }

    Ok(())
}

/// Send a shutdown request to the workspace parent.
pub(crate) async fn request_workspace_shutdown(
    peer: &Arc<IpcPeer>,
    reason: Option<String>,
) -> Result<()> {
    let msg = ParentMessage::shutdown_request(reason).map_err(|e| {
        anyhow::anyhow!("failed to encode shutdown request: {e}")
    })?;
    send_parent_message(peer, msg).await
}

/// Request that the parent spawn a child process and return its PID.
pub(crate) async fn request_spawn_child(
    peer: &Arc<IpcPeer>,
    kind: ChildKind,
    init_data: Option<Vec<u8>>,
) -> Result<ProcessId> {
    let spawn_req = SpawnChildRequest { kind, init_data };
    let args = postcard_helpers::encode(&spawn_req, "spawn request")?;

    let resp = peer
        .call(
            MethodId {
                service: PARENT_SERVICE_NAME.into(),
                method: METHOD_REQUEST_SPAWN_CHILD.into(),
            },
            args,
        )
        .await
        .map_err(|e| anyhow::anyhow!("spawn request IPC call failed: {e}"))?;

    let bytes = crate::response_result_bytes(resp, "spawn request")?;
    let result: SpawnChildResponse =
        postcard_helpers::decode(&bytes, "spawn response")?;

    if !result.accepted {
        anyhow::bail!(
            "spawn request rejected: {}",
            // Reason for fallback: spawn rejection without explicit message defaults to "no reason"
            result.error.unwrap_or_else(|| "no reason".into())
        );
    }

    result
        .child_pid
        .ok_or_else(|| anyhow::anyhow!("spawn response missing child pid"))
}

async fn send_parent_message(
    peer: &Arc<IpcPeer>,
    message: ParentMessage,
) -> Result<()> {
    let req = MessageToParentRequest { message };
    let args = postcard_helpers::encode(&req, "parent message")?;

    let resp = peer
        .call(
            MethodId {
                service: PARENT_SERVICE_NAME.into(),
                method: METHOD_MESSAGE_PARENT.into(),
            },
            args,
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!("send_parent_message IPC call failed: {e}")
        })?;

    crate::ensure_response_ok(&resp, "send_parent_message")?;

    Ok(())
}

/// Shared helper for proxied child IPC calls (via the parent `proxy_call`).
#[derive(Debug, Clone)]
pub(crate) struct PeerProxiedClient {
    peer: Arc<IpcPeer>,
    target_pid: ProcessId,
}

impl PeerProxiedClient {
    pub(crate) fn new(peer: Arc<IpcPeer>, target_pid: ProcessId) -> Self {
        Self { peer, target_pid }
    }

    async fn proxy_call(
        &self,
        method: MethodId,
        args: Vec<u8>,
    ) -> Result<ProxyCallResponse> {
        let req = ProxyCallRequest {
            target_pid: self.target_pid,
            method,
            args,
            fd_count: 0,
        };
        let payload = postcard_helpers::encode(&req, "proxy request")?;

        let resp = self
            .peer
            .call(
                MethodId {
                    service: PARENT_SERVICE_NAME.into(),
                    method: METHOD_PROXY_CALL.into(),
                },
                payload,
            )
            .await
            .map_err(|e| anyhow::anyhow!("proxy_call IPC call failed: {e}"))?;

        let bytes = crate::response_result_bytes(resp, "proxy_call")?;
        let result: ProxyCallResponse =
            postcard_helpers::decode(&bytes, "proxy_call response")?;
        Ok(result)
    }

    #[cfg(unix)]
    async fn proxy_call_with_fds(
        &self,
        method: MethodId,
        args: Vec<u8>,
        fds: Vec<std::os::unix::io::RawFd>,
    ) -> Result<ProxyCallResponse> {
        let fd_count = u32::try_from(fds.len())
            .map_err(|e| anyhow::anyhow!("too many fds for proxy call: {e}"))?;

        let req = ProxyCallRequest {
            target_pid: self.target_pid,
            method,
            args,
            fd_count,
        };

        let payload = postcard_helpers::encode(&req, "proxy request")?;

        let resp = self
            .peer
            .call_raw_with_fds(
                MethodId {
                    service: PARENT_SERVICE_NAME.into(),
                    method: METHOD_PROXY_CALL.into(),
                },
                payload,
                fds,
            )
            .await
            .map_err(|e| anyhow::anyhow!("proxy_call IPC call failed: {e}"))?;

        let bytes = crate::response_result_bytes(resp, "proxy_call")?;
        let result: ProxyCallResponse =
            postcard_helpers::decode(&bytes, "proxy_call response")?;
        Ok(result)
    }

    pub(crate) async fn call_postcard<Req, Resp>(
        &self,
        service: &str,
        method: &str,
        req: &Req,
    ) -> Result<Resp>
    where
        Req: serde::Serialize + Send + Sync,
        Resp: serde::de::DeserializeOwned,
    {
        let args = postcard_helpers::encode(req, "request")?;

        let proxied = self
            .proxy_call(
                MethodId {
                    service: service.to_string(),
                    method: method.to_string(),
                },
                args,
            )
            .await?;

        if !proxied.ok {
            // Reason for fallback: proxied IPC call failure without error message defaults to "unknown error"
            let msg = proxied
                .error
                .map_or_else(|| "unknown error".to_string(), |e| e.message);
            anyhow::bail!("proxied call failed: {msg}");
        }

        let result = proxied
            .result
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("response missing result"))
            .and_then(|bytes| postcard_helpers::decode(bytes, "response"))?;

        Ok(result)
    }
}

impl IpcCaller for PeerProxiedClient {
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();

        Box::pin(async move {
            let proxied =
                self.proxy_call(MethodId { service, method }, args).await?;

            proxied_result_bytes(proxied, "proxied call")
        })
    }
}

#[cfg(unix)]
impl IpcCallerWithFds for PeerProxiedClient {
    fn call_raw_with_fds(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
        fds: Vec<std::os::unix::io::RawFd>,
    ) -> IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();

        Box::pin(async move {
            let proxied = self
                .proxy_call_with_fds(MethodId { service, method }, args, fds)
                .await?;

            proxied_result_bytes(proxied, "proxied call")
        })
    }
}

fn proxied_result_bytes(
    proxied: ProxyCallResponse,
    context: &str,
) -> Result<Vec<u8>> {
    if !proxied.ok {
        // Reason for fallback: proxied response failure without error message defaults to "unknown error"
        let msg = proxied
            .error
            .as_ref()
            .map_or_else(|| "unknown error".to_string(), |e| e.message.clone());
        anyhow::bail!("{context} failed: {msg}");
    }

    proxied
        .result
        .ok_or_else(|| anyhow::anyhow!("{context} missing result"))
}
