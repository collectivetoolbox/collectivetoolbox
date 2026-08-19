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

//! RPC client helpers for the multiplexed IPC layer.

use crate::multiplex::session::Session;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::error::Error;
use crate::error::to_ipc_error;
use crate::protocol::{
    Cancel, Event, Message, Request, Response, RpcError, StreamControl,
};
use crate::types::{ConnectionId, RequestId, StreamId};
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tracing::Instrument as _;

#[expect(dead_code, reason = "potentially unused API helper")]
pub(crate) const DEFAULT_EVENTS_CAPACITY: usize = 64;

/// RPC client interface layered on a Session, with request/response correlation and cancellation.
#[async_trait]
pub trait RpcClient: Send + Sync {
    /// Send a request; returns immediately after sending.
    async fn send_request(&self, req: Request) -> Result<(), Error>;

    /// Await a response with the given id.
    async fn recv_response(&self, id: RequestId) -> Result<Response, Error>;

    /// Cancel a request by id.
    async fn cancel(&self, cancel: Cancel) -> Result<(), Error>;

    /// Receive server-initiated events.
    fn events(&self) -> Pin<Box<dyn Stream<Item = Event> + Send>>;
}

/// Streaming control plane to coordinate substreams or blob-backed flows.
/// TODO (maybe): wire up streaming
#[async_trait]
pub trait StreamManager: Send + Sync {
    async fn start(&self, ctl: StreamControl) -> Result<(), Error>;
    async fn next(&self, ctl: StreamControl) -> Result<(), Error>;
    async fn end(&self, ctl: StreamControl) -> Result<(), Error>;

    /// Optional helper for control-plane streaming (chunked via frames).
    fn stream_incoming(
        &self,
        id: StreamId,
    ) -> Pin<Box<dyn Stream<Item = Bytes> + Send>>;
}

// ---- Default RpcClient implementation ----

/// Default `RpcClient` built over a Session. It:
/// - Correlates responses by `RequestId`.
/// - Forwards cancellation requests (server-driven cancellation is expected).
/// - Emits server Events via a bounded mpsc channel with backpressure.
///
/// The client spawns a background task to dispatch incoming Messages.
pub struct DefaultRpcClient<S: Session> {
    session: Arc<S>,
    connection_id: Option<ConnectionId>,
    inflight: tokio::sync::Mutex<HashMap<RequestId, oneshot::Sender<Response>>>,
    pending: tokio::sync::Mutex<HashMap<RequestId, Response>>,
    events_tx: mpsc::Sender<Event>,
    events_rx: std::sync::Mutex<Option<mpsc::Receiver<Event>>>,
    streams: Arc<tokio::sync::Mutex<HashMap<StreamId, IncomingStreamState>>>,
}

impl<S: Session + 'static> DefaultRpcClient<S> {
    /// Construct a new `RpcClient` over the given Session. The events channel is
    /// bounded by `events_capacity` to provide backpressure.
    pub fn new(
        session: std::sync::Arc<S>,
        events_capacity: usize,
    ) -> std::sync::Arc<Self> {
        Self::new_with_connection_id(session, None, events_capacity)
    }

    /// Construct a new `RpcClient` with an optional connection id for logging.
    pub fn new_with_connection_id(
        session: std::sync::Arc<S>,
        connection_id: Option<ConnectionId>,
        events_capacity: usize,
    ) -> std::sync::Arc<Self> {
        let (events_tx, events_rx) = mpsc::channel(events_capacity);
        let client = std::sync::Arc::new(Self {
            session: session.clone(),
            connection_id,
            inflight: tokio::sync::Mutex::new(HashMap::new()),
            pending: tokio::sync::Mutex::new(HashMap::new()),
            events_tx,
            events_rx: std::sync::Mutex::new(Some(events_rx)),
            streams: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        });

        Self::spawn_dispatcher(client.clone());
        client
    }

    async fn handle_stream_control(
        &self,
        ctl: StreamControl,
    ) -> Result<(), Error> {
        let id = match &ctl {
            StreamControl::Start { id, .. } => *id,
            StreamControl::Next { id, .. } => *id,
            StreamControl::End { id, .. } => *id,
        };

        let span = if let Some(conn_id) = self.connection_id {
            tracing::info_span!("ipc.rpc.stream_control", conn_id = %conn_id, stream_id = id)
        } else {
            tracing::info_span!("ipc.rpc.stream_control", stream_id = id)
        };

        async move {
            match ctl {
                StreamControl::Start { .. } => {
                    let mut guard = self.streams.lock().await;
                    guard
                        .entry(id)
                        .or_insert_with(|| IncomingStreamState::new(64));
                    Ok(())
                }
                StreamControl::Next { chunk, .. } => {
                    let Some(chunk) = chunk else {
                        // Blob-backed or out-of-band; nothing to deliver on control plane.
                        return Ok(());
                    };

                    let mut guard = self.streams.lock().await;
                    let st = guard
                        .entry(id)
                        .or_insert_with(|| IncomingStreamState::new(64));
                    st.tx.send(Bytes::from(chunk)).await.map_err(|e| {
                        to_ipc_error(anyhow!("stream send failed: {e}"))
                    })?;
                    Ok(())
                }
                StreamControl::End { .. } => {
                    let mut guard = self.streams.lock().await;
                    guard.remove(&id);
                    Ok(())
                }
            }
        }
        .instrument(span)
        .await
    }

    #[cfg(test)]
    async fn stream_count(&self) -> usize {
        let guard = self.streams.lock().await;
        guard.len()
    }
}

struct IncomingStreamState {
    tx: mpsc::Sender<Bytes>,
    rx: Option<mpsc::Receiver<Bytes>>,
}

impl IncomingStreamState {
    fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self { tx, rx: Some(rx) }
    }
}

impl<S: Session + 'static> DefaultRpcClient<S> {
    fn spawn_dispatcher(client: std::sync::Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let msg = match client.session.recv().await {
                    Ok(Some(m)) => m,
                    Ok(None) => {
                        // EOF: fail all inflight waiters with an error.
                        let mut inflight = client.inflight.lock().await;
                        for (id, sender) in inflight.drain() {
                            let _ = sender.send(Response {
                                id,
                                ok: false,
                                result: None,
                                error: Some(RpcError {
                                    code: "eof".to_string(),
                                    message:
                                        "session closed while awaiting response"
                                            .to_string(),
                                }),
                            });
                        }
                        break;
                    }
                    Err(e) => {
                        // On session error, fail all inflight waiters.
                        let mut inflight = client.inflight.lock().await;
                        for (id, sender) in inflight.drain() {
                            let _ = sender.send(Response {
                                id,
                                ok: false,
                                result: None,
                                error: Some(RpcError {
                                    code: "session_recv_error".to_string(),
                                    message: format!("{e:#}"),
                                }),
                            });
                        }
                        break;
                    }
                };

                match msg {
                    Message::Response(resp) => {
                        let id = response_id(&resp);
                        let span = if let Some(conn_id) = client.connection_id {
                            tracing::info_span!("ipc.rpc.recv_response_msg", conn_id = %conn_id, request_id = id)
                        } else {
                            tracing::info_span!(
                                "ipc.rpc.recv_response_msg",
                                request_id = id
                            )
                        };
                        async {
                            if let Some(sender) =
                                client.inflight.lock().await.remove(&id)
                            {
                                let _ = sender.send(resp);
                            } else {
                                client.pending.lock().await.insert(id, resp);
                            }
                        }
                        .instrument(span)
                        .await;
                    }
                    Message::Event(ev) => {
                        // Apply backpressure by awaiting send.
                        if let Err(_e) = client.events_tx.send(ev).await {
                            // Receiver dropped; ignore but keep draining input.
                        }
                    }
                    Message::Stream(ctl) => {
                        let _ = client.handle_stream_control(ctl).await;
                    }
                    // Not handled by client:
                    Message::Hello(_)
                    | Message::HelloOk(_)
                    | Message::HelloErr(_)
                    | Message::Request(_)
                    | Message::Cancel(_) => {}
                }
            }
        });
    }

    async fn handle_pending_response(
        &self,
        id: RequestId,
    ) -> Result<Response, Error> {
        if let Some(resp) = self.pending.lock().await.remove(&id) {
            return Ok(resp);
        }

        let (tx, rx) = oneshot::channel();
        {
            let mut inflight = self.inflight.lock().await;
            if let Some(resp) = self.pending.lock().await.remove(&id) {
                return Ok(resp);
            }
            inflight.insert(id, tx);
        }

        match rx.await {
            Ok(resp) => Ok(resp),
            Err(e) => Err(to_ipc_error(anyhow!(
                "response waiter dropped for id={id:?}: {e}"
            ))),
        }
    }
}

#[async_trait]
impl<S: Session + 'static> RpcClient for DefaultRpcClient<S> {
    /// Send the request envelope to the remote endpoint.
    async fn send_request(&self, req: Request) -> Result<(), Error> {
        let span = if let Some(conn_id) = self.connection_id {
            tracing::info_span!("ipc.rpc.send_request", conn_id = %conn_id, request_id = req.id, service = %req.method.service, method = %req.method.method)
        } else {
            tracing::info_span!("ipc.rpc.send_request", request_id = req.id, service = %req.method.service, method = %req.method.method)
        };
        async move { self.session.send(&Message::Request(req)).await }
            .instrument(span)
            .await
    }

    /// Await the response correlated by `RequestId`. If the response arrived
    /// before this method was called, it is returned immediately.
    async fn recv_response(&self, id: RequestId) -> Result<Response, Error> {
        let span = if let Some(conn_id) = self.connection_id {
            tracing::info_span!("ipc.rpc.recv_response", conn_id = %conn_id, request_id = id)
        } else {
            tracing::info_span!("ipc.rpc.recv_response", request_id = id)
        };
        async move { self.handle_pending_response(id).await }
            .instrument(span)
            .await
    }

    /// Forward the cancellation to the server. Server-driven cancellation is
    /// expected to materialize as a Response that the dispatcher routes to the
    /// waiter. This method does not fail the waiter eagerly.
    async fn cancel(&self, cancel: Cancel) -> Result<(), Error> {
        let span = if let Some(conn_id) = self.connection_id {
            tracing::info_span!("ipc.rpc.cancel", conn_id = %conn_id, request_id = cancel.id)
        } else {
            tracing::info_span!("ipc.rpc.cancel", request_id = cancel.id)
        };
        async move {
            // Resolve any local waiter immediately.
            if let Some(sender) = self.inflight.lock().await.remove(&cancel.id)
            {
                let _ = sender.send(Response {
                    id: cancel.id,
                    ok: false,
                    result: None,
                    error: Some(RpcError::cancelled(
                        "request cancelled by client",
                    )),
                });
            }
            let _ = self.pending.lock().await.remove(&cancel.id);

            self.session.send(&Message::Cancel(cancel)).await
        }
        .instrument(span)
        .await
    }

    fn events(&self) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
        let mut guard = match self.events_rx.lock() {
            Ok(g) => g,
            Err(_) => return Box::pin(futures_util::stream::empty()),
        };

        if let Some(rx) = guard.take() {
            Box::pin(ReceiverStream::new(rx))
        } else {
            Box::pin(futures_util::stream::empty())
        }
    }
}

#[async_trait]
impl<S: Session + 'static> StreamManager for DefaultRpcClient<S> {
    async fn start(&self, ctl: StreamControl) -> Result<(), Error> {
        self.session.send(&Message::Stream(ctl)).await
    }

    async fn next(&self, ctl: StreamControl) -> Result<(), Error> {
        self.session.send(&Message::Stream(ctl)).await
    }

    async fn end(&self, ctl: StreamControl) -> Result<(), Error> {
        self.session.send(&Message::Stream(ctl)).await
    }

    fn stream_incoming(
        &self,
        id: StreamId,
    ) -> Pin<Box<dyn Stream<Item = Bytes> + Send>> {
        let streams = self.streams.clone();
        let (tx, rx) = mpsc::channel::<Bytes>(1);

        tokio::spawn(async move {
            let mut guard = streams.lock().await;
            if let Some(state) = guard.get_mut(&id)
                && let Some(mut original_rx) = state.rx.take()
            {
                drop(guard);
                while let Some(bytes) = original_rx.recv().await {
                    if tx.send(bytes).await.is_err() {
                        break;
                    }
                }
            }
        });

        Box::pin(ReceiverStream::new(rx))
    }
}

// ---- Helpers (minimal assumptions about protocol types) ----

/// Extract the `RequestId` from a Response.
fn response_id(resp: &Response) -> RequestId {
    resp.id
}
