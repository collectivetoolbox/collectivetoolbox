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

use crate::ipc::in_process_support::InProcessIpcCaller;
use crate::ipc::registry::IpcCaller;
use crate::ipc::service_traits::{
    ChildIpcContext, FormatsClientTrait, IoClientTrait, NetworkClientTrait,
    RendererClientTrait, RuntimeClientTrait, StorageClientTrait,
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub(crate) struct InProcessChildIpcContext {
    caller: InProcessIpcCaller,
}

impl InProcessChildIpcContext {
    pub(crate) fn new() -> Self {
        Self {
            caller: InProcessIpcCaller,
        }
    }
}

#[async_trait]
impl ChildIpcContext for InProcessChildIpcContext {
    async fn request_spawn_formats(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn FormatsClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::formats::InProcessFormatsClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn request_spawn_io(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn IoClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::io::InProcessIoClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn request_spawn_network(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn NetworkClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::network::InProcessNetworkClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn request_spawn_renderer(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RendererClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::renderer::InProcessRendererClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn request_spawn_runtime(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RuntimeClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::runtime::InProcessRuntimeClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn request_spawn_storage(
        &self,
        _init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn StorageClientTrait>> {
        Ok(Box::new(
            crate::ipc::service_traits::storage::InProcessStorageClient::new(
                self.caller.clone(),
            ),
        ))
    }

    async fn send_to_parent(&self, _message: &str) -> Result<()> {
        Ok(())
    }

    async fn send_data_plane_message(
        &self,
        _bytes: Vec<u8>,
        _content_type: &str,
    ) -> Result<()> {
        Ok(())
    }

    async fn request_workspace_shutdown(
        &self,
        _reason: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    async fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>> {
        self.caller.call_raw(service, method, args).await
    }
}

/// A wrapper context that transparently redirects requests to local in-process registry-based
/// handlers when a singleton service (like Storage, Network, or Io) makes a call to itself,
/// bypassing the socket loopback and parent workspace.
#[derive(Debug)]
pub struct BypassingChildIpcContext {
    inner: Arc<dyn ChildIpcContext>,
    local_kind: Option<crate::ipc::ChildKind>,
}

impl BypassingChildIpcContext {
    pub fn new(
        inner: Arc<dyn ChildIpcContext>,
        local_kind: Option<crate::ipc::ChildKind>,
    ) -> Self {
        Self { inner, local_kind }
    }
}

#[async_trait]
impl ChildIpcContext for BypassingChildIpcContext {
    async fn request_spawn_formats(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn FormatsClientTrait>> {
        if self.local_kind == Some(crate::ipc::ChildKind::Formats) {
            Ok(Box::new(
                crate::ipc::service_traits::formats::InProcessFormatsClient::new(
                    InProcessIpcCaller,
                ),
            ))
        } else {
            self.inner.request_spawn_formats(init_data).await
        }
    }

    async fn request_spawn_io(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn IoClientTrait>> {
        if self.local_kind == Some(crate::ipc::ChildKind::Io) {
            Ok(Box::new(
                crate::ipc::service_traits::io::InProcessIoClient::new(
                    InProcessIpcCaller,
                ),
            ))
        } else {
            self.inner.request_spawn_io(init_data).await
        }
    }

    async fn request_spawn_network(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn NetworkClientTrait>> {
        if self.local_kind == Some(crate::ipc::ChildKind::Network) {
            Ok(Box::new(
                crate::ipc::service_traits::network::InProcessNetworkClient::new(
                    InProcessIpcCaller,
                ),
            ))
        } else {
            self.inner.request_spawn_network(init_data).await
        }
    }

    async fn request_spawn_renderer(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RendererClientTrait>> {
        // Renderer is a non-singleton service. Do NOT bypass IPC.
        self.inner.request_spawn_renderer(init_data).await
    }

    async fn request_spawn_runtime(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RuntimeClientTrait>> {
        // Runtime is a non-singleton service. Do NOT bypass IPC.
        self.inner.request_spawn_runtime(init_data).await
    }

    async fn request_spawn_storage(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn StorageClientTrait>> {
        if self.local_kind == Some(crate::ipc::ChildKind::Storage) {
            Ok(Box::new(
                crate::ipc::service_traits::storage::InProcessStorageClient::new(
                    InProcessIpcCaller,
                ),
            ))
        } else {
            self.inner.request_spawn_storage(init_data).await
        }
    }

    async fn send_to_parent(&self, message: &str) -> Result<()> {
        self.inner.send_to_parent(message).await
    }

    async fn send_data_plane_message(
        &self,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<()> {
        self.inner
            .send_data_plane_message(bytes, content_type)
            .await
    }

    async fn request_workspace_shutdown(
        &self,
        reason: Option<String>,
    ) -> Result<()> {
        self.inner.request_workspace_shutdown(reason).await
    }

    async fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>> {
        self.inner.call_raw(service, method, args).await
    }
}
