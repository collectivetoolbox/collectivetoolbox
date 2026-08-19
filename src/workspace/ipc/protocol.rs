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

//! IPC protocol types and message envelopes for workspace communication.
//!
//! This module defines transport-level envelopes and streaming controls. It
//! intentionally avoids embedding business logic; services should delegate to
//! external modules for actual work.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::auth::capability::{CapabilitySet, CapabilityToken};
use crate::shared_memory::BlobToken;
use crate::types::{RequestId, StreamId};
use serde::{Deserialize, Serialize};

/// Top-level message envelope for framed transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Initial handshake from client to server (child -> workspace or workspace -> service).
    Hello(Hello),
    /// Response to Hello with bound capabilities or an error.
    HelloOk(HelloOk),
    HelloErr(HelloErr),

    /// RPC request with correlation id.
    Request(Request),
    /// RPC response associated with a request id.
    Response(Response),

    /// Event (server-initiated notification).
    Event(Event),

    /// Stream control messages for high-volume data.
    Stream(StreamControl),

    /// Cancel a pending request id.
    Cancel(Cancel),
}

/// Authentication handshake payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    pub token: CapabilityToken,
    /// Optional client info (version, process kind).
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub process_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloOk {
    pub bound_capabilities: CapabilitySet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloErr {
    pub message: String,
}

/// Request envelope. The `method` is a typed identifier, and `args` is a postcard-encoded payload for the method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: RequestId,
    pub method: MethodId,
    /// Postcard-encoded args for the method.
    pub args: Vec<u8>,
}

/// Response envelope. Either ok with a postcard-encoded result or error with a message/code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: RequestId,
    pub ok: bool,
    /// If ok, postcard-encoded result.
    pub result: Option<Vec<u8>>,
    /// If error, a short machine-readable code and message.
    pub error: Option<RpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: String,
    pub message: String,
}

/// Events are server-initiated notifications (e.g., `child_exit`,
/// `permission_changed`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub topic: EventTopic,
    /// Postcard-encoded payload per topic.
    pub payload: Vec<u8>,
}

impl Event {
    /// Construct an empty heartbeat event payload.
    pub fn heartbeat() -> Self {
        Self {
            topic: EventTopic::Heartbeat,
            payload: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventTopic {
    Heartbeat,
    ChildExited,
    PermissionChanged,
    StorageCompacted,
    Custom(String),
}

/// Stream controls for large flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamControl {
    Start {
        id: StreamId,
        /// Optionally pass a blob token for data plane.
        blob: Option<BlobToken>,
    },
    Next {
        id: StreamId,
        /// Raw chunk if using control-plane streaming, else omitted when using blob.
        chunk: Option<Vec<u8>>,
    },
    End {
        id: StreamId,
        ok: bool,
        error: Option<String>,
    },
}

/// A compact method identifier (service + method).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodId {
    pub service: String,
    pub method: String,
}

/// Request cancellation for a given id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    pub id: RequestId,
}

impl Hello {
    /// Construct a Hello message payload.
    pub fn new(
        token: CapabilityToken,
        client_info: Option<ClientInfo>,
    ) -> Self {
        Self { token, client_info }
    }
}

impl HelloOk {
    /// Construct a successful handshake response.
    pub fn new(bound_capabilities: CapabilitySet) -> Self {
        Self { bound_capabilities }
    }
}

impl HelloErr {
    /// Construct a failed handshake response with a reason.
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Compact helper for building a `MethodId`.
impl MethodId {
    pub fn new<S1, S2>(service: S1, method: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            service: service.into(),
            method: method.into(),
        }
    }
}

impl RpcError {
    /// Create an "unauthorized" error.
    pub fn unauthorized<S: Into<String>>(message: S) -> Self {
        Self {
            code: RPC_ERROR_CODE_UNAUTHORIZED.into(),
            message: message.into(),
        }
    }

    /// Create a "`capability_denied`" error (e.g., quota/rate limit exceeded).
    pub fn capability_denied<S: Into<String>>(message: S) -> Self {
        Self {
            code: RPC_ERROR_CODE_CAPABILITY_DENIED.into(),
            message: message.into(),
        }
    }

    /// Create a "cancelled" error.
    pub fn cancelled<S: Into<String>>(message: S) -> Self {
        Self {
            code: RPC_ERROR_CODE_CANCELLED.into(),
            message: message.into(),
        }
    }

    /// Create an "internal" error.
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self {
            code: RPC_ERROR_CODE_INTERNAL.into(),
            message: message.into(),
        }
    }
}

impl Cancel {
    /// Construct a Cancel payload.
    pub fn new(id: RequestId) -> Self {
        Self { id }
    }
}

/// Standard RPC error codes used across the IPC boundary.
pub const RPC_ERROR_CODE_UNAUTHORIZED: &str = "unauthorized";
pub const RPC_ERROR_CODE_CAPABILITY_DENIED: &str = "capability_denied";
pub const RPC_ERROR_CODE_RATE_LIMITED: &str = "rate_limited";
pub const RPC_ERROR_CODE_CANCELLED: &str = "cancelled";
pub const RPC_ERROR_CODE_INTERNAL: &str = "internal";
pub const RPC_ERROR_CODE_NOT_IMPLEMENTED: &str = "not_implemented";

/// When payloads are larger than this threshold, prefer blob-backed flows.
///
/// This is a protocol-level constant so clients and servers agree on the
/// heuristic, but callers may still override it (e.g., for testing).
pub const IO_NETWORK_BLOB_THRESHOLD_BYTES: u64 = 256 * 1024;

/// Default chunk size used for control-plane streaming via `StreamControl::Next`.
pub const STREAM_CONTROL_PLANE_CHUNK_BYTES: usize = 16 * 1024;

impl Request {
    /// Create a `tracing` span for handling this request.
    ///
    /// The span carries `request_id`, `service`, and `method` so downstream logs
    /// can be correlated without manually repeating those fields.
    #[must_use]
    pub fn span(&self) -> tracing::Span {
        tracing::info_span!(
            "ipc.request",
            request_id = self.id,
            service = %self.method.service,
            method = %self.method.method
        )
    }
}

impl StreamControl {
    /// Return the stream id for all `StreamControl` variants.
    #[must_use]
    pub fn stream_id(&self) -> StreamId {
        match self {
            StreamControl::Start { id, .. } => *id,
            StreamControl::Next { id, .. } => *id,
            StreamControl::End { id, .. } => *id,
        }
    }

    /// Create a `tracing` span for handling this stream control message.
    ///
    /// Carries `stream_id`, `variant`, `kind` (when present), and blob metadata
    /// (when present) to correlate logs across the stream lifecycle.
    #[must_use]
    pub fn span(&self) -> tracing::Span {
        match self {
            StreamControl::Start { id, blob } => tracing::info_span!(
                "ipc.stream",
                stream_id = *id,
                variant = "start",
                blob_id =
                    blob.as_ref().map(|b| b.id).map(tracing::field::display),
                blob_size = blob.as_ref().map(|b| b.size),
                blob_lease_ms = blob.as_ref().and_then(|b| b.lease_ms),
            ),
            StreamControl::Next { id, chunk } => tracing::info_span!(
                "ipc.stream",
                stream_id = *id,
                variant = "next",
                chunk_len = chunk.as_ref().map(std::vec::Vec::len),
            ),
            StreamControl::End { id, ok, error } => tracing::info_span!(
                "ipc.stream",
                stream_id = *id,
                variant = "end",
                ok = *ok,
                error = error.as_deref(),
            ),
        }
    }
}

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
    use super::*;
    use anyhow::{Result, ensure};

    use std::collections::HashMap;
    use std::sync::Mutex;
    use tracing::field::{Field, Visit};
    use tracing_subscriber::layer::{Context as LayerContext, Layer};

    use tracing_subscriber::registry::LookupSpan;

    #[derive(Default)]
    struct FieldCaptureVisitor {
        fields: HashMap<String, String>,
    }

    impl Visit for FieldCaptureVisitor {
        fn record_str(&mut self, field: &Field, value: &str) {
            let _ = self
                .fields
                .insert(field.name().to_string(), value.to_string());
        }

        fn record_u64(&mut self, field: &Field, value: u64) {
            let _ = self
                .fields
                .insert(field.name().to_string(), value.to_string());
        }

        fn record_i64(&mut self, field: &Field, value: i64) {
            let _ = self
                .fields
                .insert(field.name().to_string(), value.to_string());
        }

        fn record_bool(&mut self, field: &Field, value: bool) {
            let _ = self
                .fields
                .insert(field.name().to_string(), value.to_string());
        }

        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            let _ = self
                .fields
                .insert(field.name().to_string(), format!("{value:?}"));
        }
    }

    #[derive(Default)]
    struct CaptureLayer {
        spans: Mutex<HashMap<tracing::span::Id, HashMap<String, String>>>,
        events: Mutex<Vec<HashMap<String, String>>>,
    }

    impl CaptureLayer {
        fn drain_events(&self) -> Vec<HashMap<String, String>> {
            if let Ok(mut guard) = self.events.lock() {
                std::mem::take(&mut *guard)
            } else {
                Vec::new()
            }
        }
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        fn on_new_span(
            &self,
            attrs: &tracing::span::Attributes<'_>,
            id: &tracing::span::Id,
            _ctx: LayerContext<'_, S>,
        ) {
            let mut v = FieldCaptureVisitor::default();
            attrs.record(&mut v);

            if let Ok(mut spans) = self.spans.lock() {
                let _ = spans.insert(id.clone(), v.fields);
            }
        }

        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            ctx: LayerContext<'_, S>,
        ) {
            let mut merged: HashMap<String, String> = HashMap::new();

            if let Some(scope) = ctx.event_scope(event)
                && let Ok(spans) = self.spans.lock()
            {
                for span in scope.from_root() {
                    if let Some(fields) = spans.get(&span.id()) {
                        merged.extend(fields.clone());
                    }
                }
            }

            let mut v = FieldCaptureVisitor::default();
            event.record(&mut v);
            merged.extend(v.fields);

            if let Ok(mut events) = self.events.lock() {
                events.push(merged);
            }
        }
    }

    #[crate::ctb_test]
    fn spans_attach_request_and_stream_fields() -> Result<()> {
        // If this test mysteriously fails, it may be because of an issue with
        // the logging filter. Check what get_workspace_filter is returning;
        // maybe try just running with `trace` as the filter.
        let req = Request {
            id: 42,
            method: MethodId::new("svc", "do_work"),
            args: vec![],
        };

        let stream = StreamControl::Start {
            id: 7,
            blob: Some(BlobToken {
                id: Default::default(),
                size: 123,
                lease_ms: Some(456),
            }),
        };

        req.span().in_scope(|| {
            tracing::info!("inside request span");
        });

        stream.span().in_scope(|| {
            tracing::info!("inside stream span");
        });

        // The macro injects logs_contain() function
        assert!(logs_contain("inside request span"));
        assert!(logs_contain("inside stream span"));

        Ok(())
    }

    /// Round-trip a value via postcard, ensuring serialized bytes are stable.
    fn roundtrip_bytes<T>(value: &T) -> Result<()>
    where
        T: serde::Serialize + for<'de> serde::de::Deserialize<'de>,
    {
        let bytes = postcard_helpers::encode(value, "test value")?;
        let decoded: T = postcard_helpers::decode(&bytes, "test value")?;
        let bytes2 = postcard_helpers::encode(&decoded, "test value")?;
        ensure!(bytes == bytes2, "postcard roundtrip bytes mismatch");
        Ok(())
    }

    /// Round-trip `Message::Hello` via postcard.
    #[crate::ctb_test]
    fn message_hello_roundtrip() -> Result<()> {
        let msg = Message::Hello(Hello {
            token: Default::default(),
            client_info: Some(ClientInfo {
                name: "client".into(),
                version: "1.0.0".into(),
                process_kind: "test".into(),
            }),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::HelloOk` via postcard.
    #[crate::ctb_test]
    fn message_hello_ok_roundtrip() -> Result<()> {
        let msg = Message::HelloOk(HelloOk {
            bound_capabilities: Default::default(),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::HelloErr` via postcard.
    #[crate::ctb_test]
    fn message_hello_err_roundtrip() -> Result<()> {
        let msg = Message::HelloErr(HelloErr {
            message: "handshake failed".into(),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Request` via postcard.
    #[crate::ctb_test]
    fn message_request_roundtrip() -> Result<()> {
        let msg = Message::Request(Request {
            id: Default::default(),
            method: MethodId {
                service: "svc".into(),
                method: "do_work".into(),
            },
            args: vec![0x01, 0x02, 0x03],
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Response(ok)` via postcard.
    #[crate::ctb_test]
    fn message_response_ok_roundtrip() -> Result<()> {
        let msg = Message::Response(Response {
            id: Default::default(),
            ok: true,
            result: Some(vec![0xAA, 0xBB]),
            error: None,
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Response(err)` via postcard.
    #[crate::ctb_test]
    fn message_response_err_roundtrip() -> Result<()> {
        let msg = Message::Response(Response {
            id: Default::default(),
            ok: false,
            result: None,
            error: Some(RpcError {
                code: "bad_request".into(),
                message: "invalid input".into(),
            }),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Event` via postcard.
    #[crate::ctb_test]
    fn message_event_roundtrip() -> Result<()> {
        let msg = Message::Event(Event {
            topic: EventTopic::Custom("custom.topic".into()),
            payload: vec![0x10, 0x20],
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Event(Heartbeat)` via postcard.
    #[crate::ctb_test]
    fn message_event_heartbeat_roundtrip() -> Result<()> {
        let msg = Message::Event(Event::heartbeat());
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Stream(Start)` via postcard.
    #[crate::ctb_test]
    fn message_stream_start_roundtrip() -> Result<()> {
        let msg = Message::Stream(StreamControl::Start {
            id: Default::default(),
            blob: Some(Default::default()),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Stream(Next)` via postcard.
    #[crate::ctb_test]
    fn message_stream_next_roundtrip() -> Result<()> {
        let msg = Message::Stream(StreamControl::Next {
            id: Default::default(),
            chunk: Some(vec![1, 2, 3, 4]),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Stream(End)` via postcard.
    #[crate::ctb_test]
    fn message_stream_end_roundtrip() -> Result<()> {
        let msg = Message::Stream(StreamControl::End {
            id: Default::default(),
            ok: false,
            error: Some("stream error".into()),
        });
        roundtrip_bytes(&msg)
    }

    /// Round-trip `Message::Cancel` via postcard.
    #[crate::ctb_test]
    fn message_cancel_roundtrip() -> Result<()> {
        let msg = Message::Cancel(Cancel {
            id: Default::default(),
        });
        roundtrip_bytes(&msg)
    }
}
