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

//! Bidirectional IPC peer.
//!
//! The IPC stack supports a single multiplexed session that can carry both
//! Requests and Responses. Many processes need to act as both:
//! - a server (dispatch incoming Requests through an `IpcRouter`)
//! - a client (send Requests and await correlated Responses)
//!
//! `IpcPeer` runs a background receive loop that handles both directions over
//! one connection.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::error::Error;
use crate::multiplex::session::Session;
use crate::protocol::{Cancel, Message, MethodId, Request, Response, RpcError};
use crate::router::{ConnectionContext, IpcRouter, Router};
use crate::types::RequestId;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::{Mutex, Notify, oneshot};

/// A bidirectional peer over a single [`Session`].
///
/// The peer owns a background receive loop that:
/// - dispatches incoming requests through the provided router
/// - correlates incoming responses with inflight client calls
#[derive(Debug)]
pub struct IpcPeer {
    session: Arc<dyn Session>,
    router: Arc<IpcRouter>,
    ctx: ConnectionContext,
    send_lock: Mutex<()>,
    next_id: AtomicU64,
    inflight: Mutex<HashMap<RequestId, oneshot::Sender<Response>>>,
    pending: Mutex<HashMap<RequestId, Response>>,
    closed: AtomicBool,
    closed_notify: Notify,
}

impl IpcPeer {
    pub fn session(&self) -> &dyn Session {
        self.session.as_ref()
    }

    /// Create a new peer and spawn its background dispatcher.
    pub fn new(
        session: Arc<dyn Session>,
        router: Arc<IpcRouter>,
        ctx: ConnectionContext,
    ) -> Arc<Self> {
        let peer = Arc::new(Self {
            session,
            router,
            ctx,
            send_lock: Mutex::new(()),
            next_id: AtomicU64::new(1),
            inflight: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashMap::new()),
            closed: AtomicBool::new(false),
            closed_notify: Notify::new(),
        });

        Self::spawn_dispatch_loop(Arc::clone(&peer));
        peer
    }

    fn spawn_dispatch_loop(peer: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let msg = match peer.session.recv().await {
                    Ok(Some(m)) => m,
                    Ok(None) => break,
                    Err(e) => {
                        tracing::warn!("[peer] recv error: {e:#}");
                        break;
                    }
                };

                match msg {
                    Message::Request(req) => {
                        // Important: request handlers may perform outbound
                        // calls (and await responses) over the same session.
                        // If we await the handler inline here, the receive
                        // loop can't process those responses and can
                        // deadlock. Spawn per-request tasks to keep the recv
                        // loop responsive.
                        let peer = Arc::clone(&peer);
                        tokio::spawn(async move {
                            let request_id = req.id;
                            let resp = match peer
                                .router
                                .dispatch_with_session(
                                    &peer.ctx,
                                    Arc::clone(&peer.session),
                                    req,
                                )
                                .await
                            {
                                Ok(r) => r,
                                Err(e) => {
                                    tracing::warn!(
                                        "[peer] dispatch error for request_id={request_id}: {e:#}"
                                    );
                                    IpcRouter::error_response(
                                        request_id,
                                        "internal",
                                        &format!("dispatch failed: {e:#}"),
                                    )
                                }
                            };

                            let _guard = peer.send_lock.lock().await;
                            if let Err(e) = peer
                                .session
                                .send(&Message::Response(resp))
                                .await
                            {
                                tracing::warn!(
                                    "[peer] send error (response) for request_id={request_id}: {e:#}"
                                );
                            }
                        });
                    }
                    Message::Response(resp) => {
                        let id = resp.id;
                        if let Some(sender) =
                            peer.inflight.lock().await.remove(&id)
                        {
                            let _ = sender.send(resp);
                        } else {
                            peer.pending.lock().await.insert(id, resp);
                        }
                    }
                    Message::Cancel(Cancel { id }) => {
                        let _ = peer.router.observe_cancel(&peer.ctx, id).await;
                    }
                    // Not handled by the peer today.
                    Message::Event(_)
                    | Message::Stream(_)
                    | Message::Hello(_)
                    | Message::HelloOk(_)
                    | Message::HelloErr(_) => {}
                }
            }

            // Fail any inflight waiters on exit.
            let mut inflight = peer.inflight.lock().await;
            for (id, sender) in inflight.drain() {
                let _ = sender.send(Response {
                    id,
                    ok: false,
                    result: None,
                    error: Some(RpcError {
                        code: "eof".to_string(),
                        message: "session closed".to_string(),
                    }),
                });
            }

            peer.closed.store(true, Ordering::Release);
            peer.closed_notify.notify_waiters();
        });
    }

    fn next_request_id(&self) -> RequestId {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn take_pending(&self, id: RequestId) -> Option<Response> {
        self.pending.lock().await.remove(&id)
    }

    /// Wait until the underlying session receive loop exits.
    pub async fn wait_closed(&self) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        self.closed_notify.notified().await;
    }

    /// Send an RPC request and await its response.
    pub async fn call(
        &self,
        method: MethodId,
        args: Vec<u8>,
    ) -> Result<Response, Error> {
        let id = self.next_request_id();

        if let Some(resp) = self.take_pending(id).await {
            return Ok(resp);
        }

        let (tx, rx) = oneshot::channel();
        self.inflight.lock().await.insert(id, tx);

        let req = Request { id, method, args };
        let _guard = self.send_lock.lock().await;
        self.session.send(&Message::Request(req)).await?;

        rx.await.map_err(|e| {
            anyhow::anyhow!("response waiter dropped for id={id}: {e}").into()
        })
    }

    #[cfg(unix)]
    pub async fn recv_fd(&self) -> Result<std::os::unix::io::RawFd, Error> {
        self.session.recv_fd().await
    }

    /// Send an RPC request and associated Unix FDs, then await its response.
    ///
    /// This is used for data-plane parameters (memfd + `SCM_RIGHTS`) where the
    /// control message contains metadata and the actual FD is transferred
    /// out-of-band.
    #[cfg(unix)]
    pub async fn call_raw_with_fds(
        &self,
        method: MethodId,
        args: Vec<u8>,
        fds: Vec<std::os::unix::io::RawFd>,
    ) -> Result<Response, Error> {
        let id = self.next_request_id();

        if let Some(resp) = self.take_pending(id).await {
            return Ok(resp);
        }

        let (tx, rx) = oneshot::channel();
        self.inflight.lock().await.insert(id, tx);

        let req = Request { id, method, args };

        {
            let _guard = self.send_lock.lock().await;
            self.session.send(&Message::Request(req)).await?;

            for fd in fds {
                self.session.send_fd(fd).await?;
            }
        }

        rx.await.map_err(|e| {
            Error::from(anyhow::anyhow!(
                "response waiter dropped for id={id}: {e}"
            ))
        })
    }

    #[cfg(unix)]
    pub async fn call_postcard_with_blob_fd<Req, Resp>(
        &self,
        service: &str,
        method: &str,
        req: &Req,
        blob_descriptor: &crate::data_plane::shared_memory::SharedBlobDescriptor,
    ) -> Result<Resp, Error>
    where
        Req: serde::Serialize,
        Resp: for<'de> serde::Deserialize<'de>,
    {
        let args =
            postcard_helpers::encode(req, "request").map_err(Error::from)?;

        let id = self.next_request_id();

        if let Some(resp) = self.take_pending(id).await {
            if !resp.ok {
                // Reason for fallback: failed IPC call response lacking error detail defaults to "unknown error"
                let msg = resp
                    .error
                    .map_or_else(|| "unknown error".into(), |e| e.message);
                return Err(anyhow::anyhow!("IPC call failed: {msg}").into());
            }
            let Some(bytes) = resp.result else {
                return Err(anyhow::anyhow!("IPC call missing result").into());
            };
            return postcard_helpers::decode::<Resp>(&bytes, "response")
                .map_err(Error::from);
        }

        let (tx, rx) = oneshot::channel();
        self.inflight.lock().await.insert(id, tx);

        let req_msg = Request {
            id,
            method: MethodId {
                service: service.into(),
                method: method.into(),
            },
            args,
        };

        {
            let _guard = self.send_lock.lock().await;
            self.session.send(&Message::Request(req_msg)).await?;

            if crate::data_plane::shared_memory::descriptor_requires_fd_transfer(
                blob_descriptor,
            ) {
                crate::data_plane::send_blob_fd(
                    self.session(),
                    blob_descriptor,
                )
                .await?;
            }
        }

        let resp = rx.await.map_err(|e| {
            Error::from(anyhow::anyhow!(
                "response waiter dropped for id={id}: {e}"
            ))
        })?;
        if !resp.ok {
            // Reason for fallback: failed IPC call response lacking error detail defaults to "unknown error"
            let msg = resp
                .error
                .map_or_else(|| "unknown error".into(), |e| e.message);
            return Err(anyhow::anyhow!("IPC call failed: {msg}").into());
        }

        let Some(bytes) = resp.result else {
            return Err(anyhow::anyhow!("IPC call missing result").into());
        };

        postcard_helpers::decode::<Resp>(&bytes, "response")
            .map_err(Error::from)
    }
}
