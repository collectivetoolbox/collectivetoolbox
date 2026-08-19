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
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

use crate::ipc::registry::{IpcCallFuture, IpcCaller};
use crate::shared_memory::{
    BlobAllocator as _, BlobBackend, BlobToken, SharedBlobDescriptor,
    SharedMemoryBlobs, descriptor_requires_fd_transfer,
};
use std::collections::VecDeque;
use std::sync::Arc;

#[cfg(unix)]
use crate::ipc::registry::IpcCallerWithFds;

#[cfg(unix)]
pub(crate) type InProcessIpcFd = std::os::unix::io::RawFd;

#[cfg(not(unix))]
pub(crate) type InProcessIpcFd = ();

/// Encode a request payload using postcard.
#[inline]
pub(crate) fn encode_req<T: serde::Serialize>(
    value: &T,
    label: &str,
) -> Result<Vec<u8>> {
    postcard_helpers::encode(value, label)
}

/// Decode a response payload using postcard.
#[inline]
pub(crate) fn decode_resp<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T> {
    postcard_helpers::decode(bytes, label)
}

/// Allocate a shared-memory blob and write `bytes` to it.
///
/// Returns the (token, descriptor) pair to send over IPC, plus any file
/// descriptors that must be passed out-of-band on Unix.
pub(crate) async fn make_shm_param(
    bytes: &[u8],
) -> Result<((BlobToken, SharedBlobDescriptor), Vec<InProcessIpcFd>)> {
    let blobs = SharedMemoryBlobs::new(BlobBackend::PlatformDefault);
    let size = u64::try_from(bytes.len())
        .map_err(|e| anyhow::anyhow!("blob too large: {e}"))?;

    let blob = blobs
        .create(size)
        .await
        .map_err(|e| anyhow::anyhow!("blob alloc failed: {e}"))?;

    blob.write_all(bytes)
        .map_err(|e| anyhow::anyhow!("blob write failed: {e}"))?;

    #[cfg(unix)]
    {
        let mut fds: Vec<InProcessIpcFd> = Vec::new();
        if descriptor_requires_fd_transfer(&blob.descriptor) {
            let SharedBlobDescriptor::UnixFd(fd) = blob.descriptor else {
                bail!("expected UnixFd descriptor for FD-transfer backend");
            };
            fds.push(fd);
        }

        let token_and_desc = (blob.token.clone(), blob.descriptor.clone());
        Ok((token_and_desc, fds))
    }

    #[cfg(not(unix))]
    {
        let token_and_desc = (blob.token.clone(), blob.descriptor.clone());
        Ok((token_and_desc, Vec::new()))
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct InProcessIpcCaller;

impl InProcessIpcCaller {
    async fn dispatch(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
        #[cfg(unix)] fds: Vec<std::os::unix::io::RawFd>,
    ) -> Result<Vec<u8>> {
        let Some(reg) = crate::ipc::registry::find(service, method) else {
            bail!("unregistered IPC method {service}.{method}");
        };

        let ipc_ctx = Arc::new(InProcessRequestContext {
            #[cfg(unix)]
            fds: tokio::sync::Mutex::new(VecDeque::from(fds)),
        });

        (reg.handler)(ipc_ctx, &args).await
    }
}

impl IpcCaller for InProcessIpcCaller {
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();
        Box::pin(async move {
            self.dispatch(
                service.as_str(),
                method.as_str(),
                args,
                #[cfg(unix)]
                Vec::new(),
            )
            .await
        })
    }
}

#[cfg(unix)]
impl IpcCallerWithFds for InProcessIpcCaller {
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
            self.dispatch(service.as_str(), method.as_str(), args, fds)
                .await
        })
    }
}

#[derive(Debug, Default)]
struct InProcessRequestContext {
    #[cfg(unix)]
    fds: tokio::sync::Mutex<VecDeque<std::os::unix::io::RawFd>>,
}

impl crate::ipc::registry::IpcRequestContext for InProcessRequestContext {
    #[cfg(unix)]
    fn recv_fd(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<std::os::unix::io::RawFd>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let mut guard = self.fds.lock().await;
            guard
                .pop_front()
                .ok_or_else(|| anyhow::anyhow!("no fd queued for request"))
        })
    }
}
