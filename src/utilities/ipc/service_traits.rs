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

//! Abstract service client traits for dependency injection.
//!
//! These traits define the interface for service clients without depending on
//! the IPC crate. The concrete implementations live in `ctb-workspace-ipc` and
//! implement these traits. Service modules (like `ctb-runtime`) can accept
//! `&dyn ServiceTrait` parameters to call IPC operations without creating a
//! circular dependency.
//!
//! # Architecture
//!
//! ```text
//! ctb-utilities (defines traits)
//!       ↑
//!       │ depends on
//!       │
//! ctb-runtime (uses &dyn traits)
//!       ↑
//!       │ depends on
//!       │
//! ctb-workspace-ipc (implements traits)
//! ```
//!
//! # Usage Contexts
//!
//! There are two main contexts where these traits are used:
//!
//! 1. **Workspace context** (`RuntimeSpawner`): The workspace process can
//!    directly spawn child runtime processes and get a client handle.
//!
//! 2. **Child process context** (`ChildIpcContext`): A child process (like a
//!    runtime) can request the workspace to spawn sub-processes via IPC,
//!    and gets back a client handle when the spawn is accepted.

use crate::Result;
pub use crate::ipc::service_traits::formats::FormatsClientTrait;
pub use crate::ipc::service_traits::io::IoClientTrait;
pub use crate::ipc::service_traits::network::NetworkClientTrait;
pub use crate::ipc::service_traits::renderer::{
    RenderMode, RenderSettings, RenderTarget, RendererClientTrait,
};
pub use crate::ipc::service_traits::runtime::RuntimeClientTrait;
pub use crate::ipc::service_traits::storage::StorageClientTrait;
use async_trait::async_trait;

use crate::ipc::registry::{IpcCallFuture, IpcCaller};

pub mod formats;
pub mod io;
pub mod network;
pub mod renderer;
pub mod runtime;
pub mod storage;

/// Abstract IPC context for child processes.
///
/// This trait provides child processes (like runtimes) with the ability to
/// communicate with the workspace and request operations like spawning
/// sub-processes, without depending on concrete IPC types.
///
/// This is implemented by the IPC peer connection infrastructure and passed
/// to service implementations.
#[async_trait]
pub trait ChildIpcContext: Send + Sync + std::fmt::Debug {
    async fn request_spawn_formats(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn FormatsClientTrait>>;

    /// Convenience wrapper for `request_spawn_formats(None)`.
    async fn formats(&self) -> Result<Box<dyn FormatsClientTrait>> {
        self.request_spawn_formats(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::formats`].
    fn formats_b(&self) -> Result<Box<dyn FormatsClientTrait>> {
        crate::unasync(self.formats())?
    }

    async fn request_spawn_io(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn IoClientTrait>>;

    /// Convenience wrapper for `request_spawn_io(None)`.
    async fn io(&self) -> Result<Box<dyn IoClientTrait>> {
        self.request_spawn_io(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::io`].
    fn io_b(&self) -> Result<Box<dyn IoClientTrait>> {
        crate::unasync(self.io())?
    }

    async fn request_spawn_network(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn NetworkClientTrait>>;

    /// Convenience wrapper for `request_spawn_network(None)`.
    async fn network(&self) -> Result<Box<dyn NetworkClientTrait>> {
        self.request_spawn_network(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::network`].
    fn network_b(&self) -> Result<Box<dyn NetworkClientTrait>> {
        crate::unasync(self.network())?
    }

    /// Request the workspace to spawn a renderer process.
    ///
    /// The child process asks the workspace to spawn a new renderer. The
    /// workspace evaluates the request based on the requester's capabilities
    /// and policy, and if accepted, spawns the child and returns a client
    /// handle.
    ///
    /// # Arguments
    ///
    /// * `init_data` - Optional initialization data for the renderer
    ///
    /// # Returns
    ///
    /// A boxed trait object for communicating with the spawned renderer, or an
    /// error if the spawn was denied.
    async fn request_spawn_renderer(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RendererClientTrait>>;

    /// Convenience wrapper for `request_spawn_renderer(None)`.
    async fn renderer(&self) -> Result<Box<dyn RendererClientTrait>> {
        self.request_spawn_renderer(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::renderer`].
    fn renderer_b(&self) -> Result<Box<dyn RendererClientTrait>> {
        crate::unasync(self.renderer())?
    }

    /// Request the workspace to spawn a sub-runtime process.
    ///
    /// The child process asks the workspace to spawn a new runtime. The
    /// workspace evaluates the request based on the requester's capabilities
    /// and policy, and if accepted, spawns the child and returns a client
    /// handle.
    ///
    /// # Arguments
    ///
    /// * `init_data` - Optional initialization data for the new runtime
    ///
    /// # Returns
    ///
    /// A boxed trait object for communicating with the spawned runtime, or an
    /// error if the spawn was denied.
    async fn request_spawn_runtime(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn RuntimeClientTrait>>;

    /// Convenience wrapper for `request_spawn_runtime(None)`.
    async fn runtime(&self) -> Result<Box<dyn RuntimeClientTrait>> {
        self.request_spawn_runtime(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::runtime`].
    fn runtime_b(&self) -> Result<Box<dyn RuntimeClientTrait>> {
        crate::unasync(self.runtime())?
    }

    async fn request_spawn_storage(
        &self,
        init_data: Option<Vec<u8>>,
    ) -> Result<Box<dyn StorageClientTrait>>;

    /// Convenience wrapper for `request_spawn_storage(None)`.
    async fn storage(&self) -> Result<Box<dyn StorageClientTrait>> {
        self.request_spawn_storage(None).await
    }

    /// Blocking wrapper for [`ChildIpcContext::storage`].
    fn storage_b(&self) -> Result<Box<dyn StorageClientTrait>> {
        crate::unasync(self.storage())?
    }

    /// Send a text message to the parent process.
    async fn send_to_parent(&self, message: &str) -> Result<()>;

    /// Send a data-plane message (opaque bytes) to the parent process.
    ///
    /// Implementations are responsible for selecting an appropriate transport
    /// (e.g. shared memory) and ensuring the parent can read the payload.
    async fn send_data_plane_message(
        &self,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<()>;

    /// Request that the parent workspace shuts down.
    async fn request_workspace_shutdown(
        &self,
        reason: Option<String>,
    ) -> Result<()>;

    /// Make a raw IPC call to a service method.
    ///
    /// This is intentionally low-level and is primarily intended for
    /// integration tests and examples that need to exercise capability
    /// denials (e.g., attempting a forbidden network call).
    ///
    /// Implementations should:
    /// - perform the IPC call,
    /// - return the response result bytes on success,
    /// - return an error if the call is rejected or fails.
    async fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>>;
}

impl IpcCaller for dyn ChildIpcContext {
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        let service = service.to_string();
        let method = method.to_string();
        Box::pin(async move {
            ChildIpcContext::call_raw(
                self,
                service.as_str(),
                method.as_str(),
                args,
            )
            .await
        })
    }
}
