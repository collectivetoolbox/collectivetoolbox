//! Parent service API definitions.
//!
//! This service provides the ability for child processes (renderers, etc.)
//! to send messages to their parent process. Messages can be data plane
//! references (for shared-memory transfers), control requests (like spawning
//! a child), or status notifications.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use async_trait::async_trait;
use ctb_utilities::shared_memory::BlobToken;
use serde::{Deserialize, Serialize};

use crate::data_plane::shared_memory::SharedBlobDescriptor;
use crate::error::Error;
use crate::types::{ConnectionId, ProcessId};
use ipc::ChildKind;

/// Method name for sending a message to the parent.
pub const METHOD_MESSAGE_PARENT: &str = "message_parent";

/// Method name for requesting that the parent spawn a child process.
pub const METHOD_REQUEST_SPAWN_CHILD: &str = "request_spawn_child";

/// Method name for asking the parent to proxy an IPC call to one of the
/// caller's own child processes.
pub const METHOD_PROXY_CALL: &str = "proxy_call";

/// A message that can be sent from a child process to its parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentMessage {
    /// The kind of message being sent.
    pub kind: ParentMessageKind,
    /// Optional payload (postcard-encoded based on kind).
    pub payload: Vec<u8>,
}

impl ParentMessage {
    /// Create a data plane message (shared memory reference).
    pub fn data_plane(data_ref: &DataPlaneRef) -> Result<Self, Error> {
        let payload = postcard_helpers::encode(data_ref, "data plane ref")
            .map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(Self {
            kind: ParentMessageKind::DataPlane,
            payload,
        })
    }

    /// Create a text message.
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self {
            kind: ParentMessageKind::Text,
            payload: text.into().into_bytes(),
        }
    }

    /// Create a shutdown request message.
    pub fn shutdown_request(reason: Option<String>) -> Result<Self, Error> {
        let payload = postcard_helpers::encode(
            &ShutdownRequest { reason },
            "shutdown request",
        )
        .map_err(|e| Error::Serialization(e.to_string()))?;
        Ok(Self {
            kind: ParentMessageKind::ShutdownRequest,
            payload,
        })
    }

    /// Decode the payload as a data plane reference.
    pub fn as_data_plane_ref(&self) -> Result<DataPlaneRef, Error> {
        if self.kind != ParentMessageKind::DataPlane {
            return Err(Error::Internal("not a data plane message".into()));
        }
        postcard_helpers::decode(&self.payload, "data plane ref")
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Decode the payload as text.
    pub fn as_text(&self) -> Result<String, Error> {
        if self.kind != ParentMessageKind::Text {
            return Err(Error::Internal("not a text message".into()));
        }
        String::from_utf8(self.payload.clone())
            .map_err(|e| Error::Serialization(e.to_string()))
    }

    /// Decode the payload as a shutdown request.
    pub fn as_shutdown_request(&self) -> Result<ShutdownRequest, Error> {
        if self.kind != ParentMessageKind::ShutdownRequest {
            return Err(Error::Internal("not a shutdown request".into()));
        }
        postcard_helpers::decode(&self.payload, "shutdown request")
            .map_err(|e| Error::Serialization(e.to_string()))
    }
}

/// Kinds of messages that can be sent to a parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParentMessageKind {
    /// A data plane reference (shared memory) for display or processing.
    DataPlane,
    /// A simple text message.
    Text,
    /// A request to shut down the entire workspace.
    ShutdownRequest,
    /// A status notification (e.g., loading progress).
    Status,
    /// An error notification.
    Error,
}

/// A reference to shared memory data on the data plane.
///
/// Rather than embedding data inline, this struct references a blob in
/// shared memory that can be mapped by the recipient process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPlaneRef {
    /// Frame sequence number (optional, for ordering).
    pub sequence: Option<u64>,
    /// The content type (e.g., "text/html", "application/octet-stream").
    pub content_type: String,
    /// Token authorizing access to the shared memory blob.
    pub token: BlobToken,
    /// Descriptor for mapping the shared memory.
    pub descriptor: SharedBlobDescriptor,
}

impl DataPlaneRef {
    /// Create a data plane reference from a blob token and descriptor.
    pub fn new(
        token: BlobToken,
        descriptor: SharedBlobDescriptor,
        content_type: impl Into<String>,
    ) -> Self {
        Self {
            sequence: None,
            content_type: content_type.into(),
            token,
            descriptor,
        }
    }

    /// Create a data plane reference with a sequence number.
    #[must_use]
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.sequence = Some(seq);
        self
    }
}

/// Shutdown request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    /// Optional reason for the shutdown.
    pub reason: Option<String>,
}

/// Request to send a message to the parent process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToParentRequest {
    /// The message to send.
    pub message: ParentMessage,
}

/// Response from sending a message to the parent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageToParentResponse {
    /// Whether the message was accepted.
    pub accepted: bool,
    /// Optional response data from the parent.
    pub response: Option<Vec<u8>>,
}

/// Request to spawn a child process owned by the requesting process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnChildRequest {
    /// The kind of child to spawn.
    pub kind: ChildKind,
    /// Initial document or configuration data for the child.
    pub init_data: Option<Vec<u8>>,
}

/// Response from requesting a child spawn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnChildResponse {
    /// Whether the spawn was accepted.
    pub accepted: bool,
    /// The process ID of the newly spawned child (if accepted).
    pub child_pid: Option<ProcessId>,
    /// Error message if not accepted.
    pub error: Option<String>,
}

/// Request for the parent to proxy an IPC call to a specific child pid.
///
/// The workspace is responsible for enforcing that the `target_pid` is owned
/// by the requesting process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyCallRequest {
    pub target_pid: ProcessId,
    pub method: crate::protocol::MethodId,
    pub args: Vec<u8>,
    /// Number of Unix file descriptors attached to this request.
    ///
    /// This is used for shared-memory (data plane) parameters where the
    /// descriptor metadata is carried in `args` and the actual FD(s) are sent
    /// out-of-band via the underlying session.
    ///
    /// When `0`, no FD transfer is expected.
    #[serde(default)]
    pub fd_count: u32,
}

/// Response from a proxied IPC call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyCallResponse {
    pub ok: bool,
    pub result: Option<Vec<u8>>,
    pub error: Option<crate::protocol::RpcError>,
}

/// Minimal metadata about a child-to-parent request.
///
/// This is provided by the server-side router, based on the connection's
/// handshake metadata.
#[derive(Debug, Clone)]
pub struct ParentRequestContext {
    /// Connection id the request came from.
    pub connection_id: ConnectionId,
    /// Optional process kind provided during handshake (e.g. "renderer").
    pub process_kind: Option<String>,
}

/// Trait for sending messages to the parent process.
///
/// This is implemented by the IPC infrastructure and injected into child
/// process routers to enable parent communication.
#[async_trait]
pub trait ParentMessenger: Send + Sync + std::fmt::Debug {
    /// Send a message to the parent process.
    async fn send_message(
        &self,
        ctx: ParentRequestContext,
        message: ParentMessage,
    ) -> Result<MessageToParentResponse, Error>;

    /// Request that the parent spawn a child process owned by the caller.
    async fn request_spawn_child(
        &self,
        ctx: ParentRequestContext,
        request: SpawnChildRequest,
    ) -> Result<SpawnChildResponse, Error>;

    /// Ask the workspace to proxy an IPC call to a specific child process.
    async fn proxy_call(
        &self,
        ctx: ParentRequestContext,
        request: ProxyCallRequest,
    ) -> Result<ProxyCallResponse, Error>;
}

/// A mock parent messenger for testing.
#[derive(Debug, Default)]
pub struct MockParentMessenger {
    messages: std::sync::Mutex<Vec<ParentMessage>>,
    spawn_requests: std::sync::Mutex<Vec<SpawnChildRequest>>,
}

impl MockParentMessenger {
    /// Create a new mock messenger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all received messages.
    pub fn messages(&self) -> Vec<ParentMessage> {
        self.messages.lock().map(|g| g.clone()).unwrap_or_default()
    }

    /// Get all spawn requests.
    pub fn spawn_requests(&self) -> Vec<SpawnChildRequest> {
        self.spawn_requests
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}
#[async_trait]
impl ParentMessenger for MockParentMessenger {
    async fn send_message(
        &self,
        _ctx: ParentRequestContext,
        message: ParentMessage,
    ) -> Result<MessageToParentResponse, Error> {
        if let Ok(mut guard) = self.messages.lock() {
            guard.push(message);
        }
        Ok(MessageToParentResponse {
            accepted: true,
            response: None,
        })
    }

    async fn request_spawn_child(
        &self,
        _ctx: ParentRequestContext,
        request: SpawnChildRequest,
    ) -> Result<SpawnChildResponse, Error> {
        if let Ok(mut guard) = self.spawn_requests.lock() {
            guard.push(request);
        }
        Ok(SpawnChildResponse {
            accepted: true,
            child_pid: Some(ProcessId::new()),
            error: None,
        })
    }

    async fn proxy_call(
        &self,
        _ctx: ParentRequestContext,
        _request: ProxyCallRequest,
    ) -> Result<ProxyCallResponse, Error> {
        Ok(ProxyCallResponse {
            ok: false,
            result: None,
            error: Some(crate::protocol::RpcError {
                code: "not_implemented".into(),
                message: "mock does not proxy calls".into(),
            }),
        })
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

    #[crate::ctb_test("tokio")]
    async fn mock_messenger_records_messages() -> Result<()> {
        let messenger = MockParentMessenger::new();

        let ctx = ParentRequestContext {
            connection_id: ConnectionId::default(),
            process_kind: Some("test".into()),
        };

        messenger
            .send_message(ctx.clone(), ParentMessage::text("hello"))
            .await?;
        messenger
            .send_message(ctx.clone(), ParentMessage::text("world"))
            .await?;

        let messages = messenger.messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].as_text()?, "hello");
        assert_eq!(messages[1].as_text()?, "world");
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn mock_messenger_records_spawn_requests() -> Result<()> {
        let messenger = MockParentMessenger::new();

        let ctx = ParentRequestContext {
            connection_id: ConnectionId::default(),
            process_kind: Some("test".into()),
        };

        let resp = messenger
            .request_spawn_child(
                ctx,
                SpawnChildRequest {
                    kind: ChildKind::Renderer,
                    init_data: None,
                },
            )
            .await?;

        assert!(resp.accepted);
        assert!(resp.child_pid.is_some());

        let requests = messenger.spawn_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].kind, ChildKind::Renderer);
        Ok(())
    }

    #[crate::ctb_test]
    fn data_plane_ref_roundtrip() -> Result<()> {
        use crate::data_plane::shared_memory::SharedBlobDescriptor;
        use shared_memory::BlobId;
        use uuid::Uuid;

        let token = BlobToken {
            id: BlobId(Uuid::new_v4()),
            size: 128,
            lease_ms: None,
        };
        let descriptor = SharedBlobDescriptor::Named("test-blob".into());
        let data_ref =
            DataPlaneRef::new(token.clone(), descriptor, "text/plain");
        let msg = ParentMessage::data_plane(&data_ref)?;
        let decoded = msg.as_data_plane_ref()?;
        anyhow::ensure!(decoded.token == token, "tokens should match");
        anyhow::ensure!(
            decoded.content_type == "text/plain",
            "content types should match"
        );
        Ok(())
    }
}
