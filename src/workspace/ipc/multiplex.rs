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

//! Session and RPC client implementations for framed IPC with postcard.
//!
//! This module provides:
//! - A Session implementation that serializes and deserializes Message values
//!   using postcard over a framed transport.
//! - A default `RpcClient` that correlates request/response pairs by `RequestId`,
//!   forwards cancellations, and exposes an events stream with bounded
//!   backpressure.
//!
//! `StreamManager` is transport-only; data processing is delegated to services.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

pub mod blob_store;
pub mod client;
pub mod session;

// ---- Tests: mocked Session, correlation, out-of-order, cancellation ----
// These tests use a mock Session and the public RpcClient API.

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use crate::error::{Error, to_ipc_error};
    use crate::multiplex::client::{
        DEFAULT_EVENTS_CAPACITY, DefaultRpcClient, RpcClient,
    };
    use crate::multiplex::session::Session;
    use crate::protocol::MethodId;
    use crate::protocol::RpcError;
    use crate::protocol::{Cancel, Message, Request, Response};
    use anyhow::anyhow;
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    use super::*;
    use anyhow::Result;

    /// A simple in-memory Session mock backed by channels carrying Message.
    #[derive(Debug)]
    struct MockSession {
        to_server: mpsc::Sender<Message>,
        from_server: tokio::sync::Mutex<mpsc::Receiver<Message>>,
    }

    #[async_trait]
    impl Session for MockSession {
        async fn send(&self, msg: &Message) -> Result<(), Error> {
            self.to_server
                .send(msg.clone())
                .await
                .map_err(|e| to_ipc_error(anyhow!("send failed: {e}")))
        }

        async fn recv(&self) -> Result<Option<Message>, Error> {
            let mut rx = self.from_server.lock().await;
            Ok(rx.recv().await)
        }
    }

    fn make_mock_session() -> (
        std::sync::Arc<MockSession>,
        mpsc::Receiver<Message>,
        mpsc::Sender<Message>,
    ) {
        let (to_server_tx, to_server_rx) = mpsc::channel(16);
        let (from_server_tx, from_server_rx) = mpsc::channel(16);
        let session = std::sync::Arc::new(MockSession {
            to_server: to_server_tx,
            from_server: tokio::sync::Mutex::new(from_server_rx),
        });
        (session, to_server_rx, from_server_tx)
    }

    /// Sends two concurrent requests and verifies that out-of-order responses
    /// are matched to the correct waiters.
    #[crate::ctb_test("tokio")]
    async fn rpc_correlation_out_of_order() -> Result<()> {
        let (session, mut to_server_rx, from_server_tx) = make_mock_session();
        let client = DefaultRpcClient::new(session, DEFAULT_EVENTS_CAPACITY);

        // Create two requests
        let req1 = Request {
            id: 1,
            method: MethodId {
                service: "test".to_string(),
                method: "method1".to_string(),
            },
            args: vec![],
        };
        let req2 = Request {
            id: 2,
            method: MethodId {
                service: "test".to_string(),
                method: "method2".to_string(),
            },
            args: vec![],
        };

        // Send requests concurrently
        let send1 = client.send_request(req1.clone());
        let send2 = client.send_request(req2.clone());
        tokio::try_join!(send1, send2)?;

        let sent_req1 = to_server_rx.recv().await.context("missing req1")?;
        let sent_req2 = to_server_rx.recv().await.context("missing req2")?;
        assert!(matches!(sent_req1, Message::Request(r) if r.id == 1));
        assert!(matches!(sent_req2, Message::Request(r) if r.id == 2));

        // Spawn tasks to await responses
        let client2 = client.clone();
        let recv1_handle =
            tokio::spawn(async move { client.recv_response(1).await });
        let recv2_handle =
            tokio::spawn(async move { client2.recv_response(2).await });

        // Send responses out of order (id 2 first, then id 1)
        let resp2 = Response {
            id: 2,
            ok: true,
            result: Some(vec![0xBB]),
            error: None,
        };
        let resp1 = Response {
            id: 1,
            ok: true,
            result: Some(vec![0xAA]),
            error: None,
        };
        from_server_tx.send(Message::Response(resp2)).await?;
        from_server_tx.send(Message::Response(resp1)).await?;

        let (resp1_recv, resp2_recv) =
            tokio::try_join!(recv1_handle, recv2_handle)?;
        let resp1_recv = resp1_recv?;
        let resp2_recv = resp2_recv?;
        assert_eq!(resp1_recv.id, 1);
        assert_eq!(resp1_recv.result, Some(vec![0xAA]));
        assert_eq!(resp2_recv.id, 2);
        assert_eq!(resp2_recv.result, Some(vec![0xBB]));
        Ok(())
    }

    /// Cancels a request and verifies downstream behavior (e.g., server sends
    /// a cancellation response routed to the correct waiter).
    #[crate::ctb_test("tokio")]
    async fn rpc_cancellation() -> Result<()> {
        let (session, mut to_server_rx, from_server_tx) = make_mock_session();
        let client = DefaultRpcClient::new(session, DEFAULT_EVENTS_CAPACITY);

        // Send a request
        let req = Request {
            id: 1,
            method: MethodId {
                service: "test".to_string(),
                method: "method".to_string(),
            },
            args: vec![],
        };
        client.send_request(req).await?;

        // Verify request was sent
        let sent_req = to_server_rx.recv().await.context("missing request")?;
        assert!(matches!(sent_req, Message::Request(r) if r.id == 1));

        // Cancel the request
        let cancel = Cancel { id: 1 };
        client.cancel(cancel).await?;

        // Verify cancel was sent
        let sent_cancel =
            to_server_rx.recv().await.context("missing cancel")?;
        assert!(matches!(sent_cancel, Message::Cancel(c) if c.id == 1));

        // Server sends a cancellation response too (client should handle either).
        let resp = Response {
            id: 1,
            ok: false,
            result: None,
            error: Some(RpcError::cancelled("request cancelled")),
        };
        from_server_tx.send(Message::Response(resp)).await?;

        let recv_resp = client.recv_response(1).await?;
        assert_eq!(recv_resp.id, 1);
        assert!(!recv_resp.ok);
        assert_eq!(recv_resp.error.unwrap().code, "cancelled");
        Ok(())
    }

    /// Cancel an in-flight "fetch" and ensure work stops and a cancelled response is sent.
    #[ignore = "test is incomplete"]
    #[crate::ctb_test("tokio")]
    async fn server_cancel_stops_inflight_fetch() -> Result<()> {
        Ok(())
        /*let (session, mut to_server_rx, from_server_tx) = make_mock_session();

        let steps = std::sync::Arc::new(AtomicUsize::new(0));
        let router = SimpleRouter::new();
        router.with_network_service(Arc::new(ctb_network::MockNetworkBackend));

        let ctx = ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet::default(),
            metadata: None,
        };

        let server_task = tokio::spawn(serve_router(session.clone(), router, ctx));

        // Send a fetch request into the server loop.
        from_server_tx
            .send(Message::Request(Request {
                id: 7,
                method: MethodId {
                    service: "network".into(),
                    method: "fetch".into(),
                },
                args: vec![1, 2, 3],
            }))
            .await?;

        // Ensure it started doing work.
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        let before = steps.load(Ordering::SeqCst);

        // Cancel it.
        from_server_tx.send(Message::Cancel(Cancel::new(7))).await?;

        // Observe the server emitting a cancelled response.
        let msg = to_server_rx.recv().await.context("missing response")?;
        let Message::Response(resp) = msg else {
            anyhow::bail!("expected Response");
        };
        assert_eq!(resp.id, 7);
        assert_eq!(resp.error.context("missing error")?.code, "cancelled");

        // Work should stop growing soon after cancellation.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let after = steps.load(Ordering::SeqCst);
        assert!(after >= before);

        // Stop the server task by dropping channels/session.
        drop(from_server_tx);
        let _ = server_task.await?;
        Ok(())*/
    }
}
