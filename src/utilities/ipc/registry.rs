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

//! Cross-crate IPC method registry.
//!
//! This module provides an inventory-based registry of postcard-encoded IPC
//! handlers. Any crate can register a handler via the `#[ipc_method]` proc
//! macro, and the IPC router can dispatch to the handler by `(service, method)`.
//!
//! The key property is that adding a new method only requires changing the
//! crate that defines the method; there is no central routing table to edit.

#[expect(unused_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;

/// The async handler future type.
pub type IpcHandlerFuture =
    Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'static>>;

/// Future returned by an IPC client call.
pub type IpcCallFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

/// Per-request server-side context.
///
/// This is passed to inventory-registered handlers so they can access
/// transport features like FD passing for shared-memory (data plane)
/// parameters.
pub trait IpcRequestContext: Send + Sync {
    /// Receive a file descriptor (Unix only).
    #[cfg(unix)]
    fn recv_fd(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = Result<std::os::unix::io::RawFd>> + Send + '_>,
    >;
}

/// Minimal IPC caller abstraction.
///
/// This trait is implemented in the IPC transport crate (e.g.
/// `ctb-workspace-ipc`) for client types like `ChildProcess`.
///
/// Macro-generated client methods target this trait so callers do not have to
/// type service/method strings.
pub trait IpcCaller: Send + Sync {
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_>;
}

impl<T> IpcCaller for Box<T>
where
    T: IpcCaller + ?Sized,
{
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        (**self).call_raw(service, method, args)
    }
}

impl<T> IpcCaller for Arc<T>
where
    T: IpcCaller + ?Sized,
{
    fn call_raw(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
    ) -> IpcCallFuture<'_> {
        (**self).call_raw(service, method, args)
    }
}

/// IPC caller that can send Unix file descriptors alongside a request.
///
/// This is required for data-plane parameters passed via shared memory
/// (memfd + `SCM_RIGHTS`).
#[cfg(unix)]
pub trait IpcCallerWithFds: IpcCaller {
    fn call_raw_with_fds(
        &self,
        service: &str,
        method: &str,
        args: Vec<u8>,
        fds: Vec<std::os::unix::io::RawFd>,
    ) -> IpcCallFuture<'_>;
}

/// The handler function type.
///
/// The function receives the postcard-encoded request bytes and must return
/// postcard-encoded response bytes.
pub type IpcHandlerFn =
    fn(Arc<dyn IpcRequestContext>, &[u8]) -> IpcHandlerFuture;

/// A single registered IPC method.
#[derive(Clone, Copy)]
pub struct IpcMethodRegistration {
    pub service: &'static str,
    pub method: &'static str,
    pub handler: IpcHandlerFn,
}

inventory::collect!(IpcMethodRegistration);

type RegistryByService = HashMap<
    &'static str,
    HashMap<&'static str, &'static IpcMethodRegistration>,
>;

static REGISTRY_BY_SERVICE: OnceLock<RegistryByService> = OnceLock::new();

fn registry_by_service() -> &'static RegistryByService {
    REGISTRY_BY_SERVICE.get_or_init(|| {
        let mut by_service: RegistryByService = HashMap::new();

        for reg in inventory::iter::<IpcMethodRegistration> {
            by_service
                .entry(reg.service)
                .or_default()
                // Keep the first registration for a given key.
                .entry(reg.method)
                .or_insert(reg);
        }

        by_service
    })
}

/// Find a registered IPC handler.
pub fn find(
    service: &str,
    method: &str,
) -> Option<&'static IpcMethodRegistration> {
    let by_service = registry_by_service();
    by_service
        .get(service)
        .and_then(|methods| methods.get(method))
        .copied()
}
