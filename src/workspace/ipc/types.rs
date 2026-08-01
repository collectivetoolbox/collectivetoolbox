#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

/// Monotonic request identifier for RPC calls on a connection.
pub type RequestId = u64;

/// Identifier for a logical stream.
pub type StreamId = u64;

/// Opaque identifier for a process managed by the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub Uuid);

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessId {
    /// Generate a new random `ProcessId`.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse a `ProcessId` from a UUID string.
    pub fn parse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Identifier for a connection (control plane) bound to a process/capability set.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct ConnectionId(pub Uuid);

impl ConnectionId {
    /// Generate a new random `ConnectionId`.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Create a span that identifies a workspace-managed process.
///
/// This is intended for attaching `process_id` to log lines emitted while
/// handling process-scoped work.
#[must_use]
pub fn ipc_process_span(process_id: ProcessId) -> tracing::Span {
    tracing::info_span!("ipc.process", process_id = %process_id)
}

/// Create a span that identifies an IPC connection.
///
/// This is intended for attaching `conn_id` to log lines emitted while
/// servicing a connection.
#[must_use]
pub fn ipc_connection_span(connection_id: ConnectionId) -> tracing::Span {
    tracing::info_span!("ipc.connection", conn_id = %connection_id)
}

/// Create a span that identifies a single RPC request.
///
/// This is intentionally independent of protocol types (only uses `RequestId`).
#[must_use]
pub fn ipc_request_span(request_id: RequestId) -> tracing::Span {
    tracing::info_span!("ipc.request", request_id)
}

/// Create a span that identifies a logical stream.
#[must_use]
pub fn ipc_stream_span(stream_id: StreamId) -> tracing::Span {
    tracing::info_span!("ipc.stream", stream_id)
}

/// Platform-neutral timeout settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeouts {
    pub handshake: Duration,
    pub request: Duration,
    pub graceful_shutdown: Duration,
    pub heartbeat_interval: Duration,
    pub heartbeat_miss_tolerance: u32,
}
