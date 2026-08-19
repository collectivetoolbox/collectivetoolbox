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

//! Session-layer helpers for framed IPC transports.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::auth::capability::TokenValidator;
use crate::error::Error;
use crate::error::to_ipc_error;
use crate::protocol::{Hello, HelloErr, HelloOk, Message};
use crate::router::{ConnectionContext, Router};
use crate::transport::FramedConnection;
use crate::types::ConnectionId;
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use std::fmt::Debug;

/// A higher-level session that speaks the Message protocol over a `FramedConnection`.
#[async_trait]
pub trait Session: Send + Sync + std::fmt::Debug {
    /// Send a structured message.
    async fn send(&self, msg: &Message) -> Result<(), Error>;

    /// Receive the next structured message (deserializing from frames). None on EOF.
    async fn recv(&self) -> Result<Option<Message>, Error>;

    /// Send a file descriptor via `SCM_RIGHTS` (Unix only).
    ///
    /// This is used for transferring memfd-backed shared memory blobs.
    /// The FD should be sent after the control message that references it.
    #[cfg(unix)]
    async fn send_fd(&self, fd: std::os::unix::io::RawFd) -> Result<(), Error> {
        let _ = fd;
        Err(Error::Unsupported(
            "FD passing not supported on this session".to_string(),
        ))
    }

    /// Receive a file descriptor via `SCM_RIGHTS` (Unix only).
    ///
    /// This is used for receiving memfd-backed shared memory blobs.
    /// The FD should be received after the control message that references it.
    #[cfg(unix)]
    async fn recv_fd(&self) -> Result<std::os::unix::io::RawFd, Error> {
        Err(Error::Unsupported(
            "FD passing not supported on this session".to_string(),
        ))
    }

    /// Client-side handshake helper.
    ///
    /// Sends `Hello` and waits for `HelloOk` or `HelloErr`. Returns bound capability
    /// set on success. Avoids panics; errors map to [`Error`].
    async fn client_handshake(
        &self,
        hello: Hello,
    ) -> Result<crate::auth::capability::CapabilitySet, Error>
    where
        Self: Sized,
    {
        self.send(&Message::Hello(hello)).await?;
        loop {
            let Some(msg) = self.recv().await? else {
                return Err(to_ipc_error(anyhow!("eof before HelloOk")));
            };
            match msg {
                Message::HelloOk(ok) => {
                    return Ok(ok.bound_capabilities);
                }
                Message::HelloErr(err) => {
                    return Err(to_ipc_error(anyhow!(
                        "handshake failed: {}",
                        err.message
                    )));
                }
                // Ignore unrelated messages during handshake.
                _ => {}
            }
        }
    }

    /// Server-side handshake helper.
    ///
    /// Waits for Hello, validates the token, replies with HelloOk/HelloErr,
    /// and registers the connection on success.
    async fn server_handshake(
        &self,
        validator: &dyn TokenValidator,
        router: &dyn Router,
        connection_id: ConnectionId,
    ) -> Result<crate::auth::capability::CapabilitySet, Error>
    where
        Self: Sized,
    {
        let Some(msg) = self.recv().await? else {
            return Err(to_ipc_error(anyhow!("eof awaiting Hello")));
        };
        let Message::Hello(hello) = msg else {
            return Err(to_ipc_error(anyhow!("expected Hello")));
        };

        let bound = match validator.validate(&hello.token) {
            Ok(set) => set,
            Err(e) => {
                // Send HelloErr and return error.
                let _ = self
                    .send(&Message::HelloErr(HelloErr::new(format!("{e:#}"))))
                    .await;
                return Err(to_ipc_error(anyhow!("token invalid: {e:#}")));
            }
        };

        // Send success response.
        self.send(&Message::HelloOk(HelloOk::new(bound.clone())))
            .await?;

        // Register connection context.
        let ctx = ConnectionContext {
            id: connection_id,
            capabilities: bound.clone(),
            metadata: hello.client_info.as_ref().map(|ci| {
                serde_json::json!({
                    "name": ci.name,
                    "version": ci.version,
                    "process_kind": ci.process_kind
                })
            }),
        };
        router.register_connection(ctx).await?;

        Ok(bound)
    }
}

/// Adapter implementing Session on top of a framed connection that transports
/// raw Bytes. Messages are encoded/decoded with postcard.
#[derive(Debug)]
pub struct FramedSession<T> {
    inner: T,
    #[cfg(unix)]
    fd_queue: tokio::sync::Mutex<Vec<std::os::unix::io::RawFd>>,
    #[cfg(unix)]
    fd_notify: tokio::sync::Notify,
}

impl<T: Clone + std::fmt::Debug> FramedSession<T> {
    /// Create a new framed session from a lower-level transport.
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            #[cfg(unix)]
            fd_queue: tokio::sync::Mutex::new(Vec::new()),
            #[cfg(unix)]
            fd_notify: tokio::sync::Notify::new(),
        }
    }
}

#[async_trait]
impl<T> Session for FramedSession<T>
where
    T: FramedConnection + Clone + std::fmt::Debug + 'static,
{
    /// Serialize Message with postcard and send as a single frame.
    async fn send(&self, msg: &Message) -> Result<(), Error> {
        let bytes = postcard_helpers::encode(msg, "Message")
            .context("postcard serialize Message")
            .map_err(to_ipc_error)?;
        self.inner.send_frame(Bytes::from(bytes)).await
    }

    /// Receive one frame, deserialize with postcard into Message.
    async fn recv(&self) -> Result<Option<Message>, Error> {
        loop {
            #[cfg(unix)]
            let maybe = self.inner.recv_frame_with_fds().await?;
            #[cfg(not(unix))]
            let maybe = self
                .inner
                .recv_frame()
                .await?
                .map(|frame| (frame, Vec::new()));

            let Some((frame, fds)) = maybe else {
                return Ok(None);
            };

            #[cfg(unix)]
            {
                if !fds.is_empty() {
                    let mut q = self.fd_queue.lock().await;
                    q.extend(fds);
                    self.fd_notify.notify_waiters();
                }

                // 0xFD is a transport-only marker used by `send_fd`.
                if frame.as_ref() == [0xFD] {
                    continue;
                }
            }

            let msg = postcard_helpers::decode::<Message>(&frame, "Message")
                .context("postcard deserialize Message")
                .map_err(to_ipc_error)?;
            return Ok(Some(msg));
        }
    }

    /// Send a file descriptor via `SCM_RIGHTS` (Unix only).
    ///
    /// This delegates to the underlying `FramedConnection`'s FD passing support.
    /// The FD is sent alongside a dummy frame (FD passing requires at least 1
    /// byte of data in the sendmsg call).
    #[cfg(unix)]
    async fn send_fd(&self, fd: std::os::unix::io::RawFd) -> Result<(), Error> {
        self.inner
            .send_frame_with_fds(Bytes::from_static(&[0xFD]), &[fd])
            .await
    }

    /// Receive a file descriptor via `SCM_RIGHTS` (Unix only).
    ///
    /// This delegates to the underlying `FramedConnection`'s FD passing support.
    #[cfg(unix)]
    async fn recv_fd(&self) -> Result<std::os::unix::io::RawFd, Error> {
        loop {
            if let Some(fd) = self.fd_queue.lock().await.pop() {
                return Ok(fd);
            }
            self.fd_notify.notified().await;
        }
    }
}

// NOTE:
// `Arc<T>` implements `FramedConnection` via a blanket impl in `transport.rs`,
// including `send_frame_with_fds` / `recv_frame_with_fds` on Unix.
//
// Keep FD passing support on the `Session` trait (and on `FramedSession<T>`)
// rather than adding transport-specific inherent helpers here.
