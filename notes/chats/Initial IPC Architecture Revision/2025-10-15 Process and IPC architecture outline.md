Hi Copilot,

I'm building a program that uses a "workspace" that can open and display documents, somewhat similar to a web browser. To isolate untrusted documents, I'd like each document to get its own subprocess.

Here's a high-level overview of the likely architecture:

1) Process model and supervision:
- Workspace = supervisor + IPC router
  - Spawns and tracks child processes and their metadata.
  - Enforces authorization by capability token per child.
  - Handles routing for anything that must be centrally arbitrated (e.g., access to shared services).
- Service processes (stable small set)
  - storage service: sole owner of the DB (only this process touches the DB).
  - io/network service(s)
  - renderer processes: one per document (and per embedded document), sandboxed.
- Renderer subprocesses should not spawn more subprocesses directly
  - Instead, renderers ask the workspace to spawn a child (renderer or worker), passing a capability bundle. This preserves policy enforcement and supervision.
- OS-level supervision
  - Unix: create a new process group/session for each renderer subtree (setpgid) and set PR_SET_PDEATHSIG to ensure children die if parent dies.
  - Windows: attach each subtree to a Job Object with “kill on close” semantics. On shutdown, terminate the job to kill the whole subtree.
  - Keep explicit Child handles and wait on exit to reclaim resources and update ProcessManager.

2) IPC transport and protocol
- Transport
  - A persistent, duplex, framed connection per process: this will use local_socket from the `interprocess` crate.
  - Maintain a single connection and multiplex logical channels over it.
- Framing & serialization
  - Use length-delimited framing (tokio-util::codec::LengthDelimitedCodec) or an established RPC protocol.
  - Serialize with Serde+Postcard (non-self-describing wire format) for compactness, potentially with an option to view JSON for debugging.
  - Define a tagged enum for messages: Request { id, method, args }, Response { id, result|error }, Event { … }, Stream { id, chunk|eof }, etc.
- Multiplexing
  - Implement request-response with correlation IDs.
  - Support cancellation (client can send “cancel id”).
  - For high-volume streams (frames), create a substream concept or move the payload into shared memory (see below).
- Authorization
  - On connection setup, client sends a workspace-minted capability token.
  - The server binds that connection to a capability set: allowed methods and quotas. Enforce on every request by connection ID.
  - Eliminate the current “add_channel” by name; the workspace registers processes internally, and clients only authenticate once per connection with an unguessable token.
- Typed API surface
  - Compile-time-checked API: define traits/enums representing methods. Use serde with enums to generate client/server stubs.

3) Data plane for large payloads (frames)
- Use shared memory or memory-mapped files for large binary blobs.
  - Unix: memfd + pass FD over Unix socket (SCM_RIGHTS).
  - Windows: CreateFileMapping + DuplicateHandle or named shared memory.
  - Cross-platform fallback: temporary memory-mapped files via memmap2; pass a handle/path/token via control channel; readers map and read.
- Wrap blobs with a lifecycle token so the server can clean up if the producer crashes.
- Control plane (RPC) sends metadata and a token; data plane (shared memory) carries the bytes.

4) Capability and permission model
- Capability tokens:
  - Minted by workspace per process, and verified when requests are received by the workspace.
  - Bind to allowed method set (e.g., renderer A can call IO.readFile, Network.getUrl; cannot call Storage.put).
  - Optionally include per-method quotas and rate limits (e.g., bytes/sec for network, frame rate).
  - A renderer can request a change in permissions, and the workspace can change the permissions of a renderer, if a user grants or revokes a permission for the specific document.

5) Concurrency and runtime
- Use `tracing` for structured logs with spans carrying process and request IDs.

6) Process lifecycle, heartbeats, and cleanup
- Heartbeats: child->workspace periodic heartbeats over the control connection. If missed for N intervals, the workspace treats the child as dead and tears down resources.
- Graceful shutdown protocol:
  - Workspace sends shutdown to child; child tears down and acks; then the workspace kills if not exited in timeout (SIGKILL/TerminateJobObject).
- Kill trees:
  - Unix: killpg to send SIGTERM/SIGKILL to a renderer’s entire process group.
  - Windows: TerminateJobObject to kill all descendants in the job.
- Panic handling:
  - Keep the panic hook to notify the workspace and attempt local cleanup.
  - But rely primarily on the parent’s supervision (job/process group) to guarantee cleanup.

7) Services organization
- Storage service (new process)
  - Owns the DB and exposes typed RPC: get/put/delete variants.
  - The storage module becomes a client stub; remove DB calls from IPC module.
- IO and network services
- Renderer
  - Single process per document. Don’t spawn per-module worker processes unless you have strong isolation needs; prefer shared services.
- Embedded documents
  - Embedded renderer is just another renderer child, with a capability set that includes “message to parent” and limited outbound calls.
  - Two-way communication via the workspace router so policies can apply.

8) Protocol shape (sketch)
- Request { id: u64, method: enum, args: Value } serialized with postcard.
- Response { id: u64, ok: bool, result: Value | error: String }.
- Stream { id, kind: “start/next/end”, blob_token? } for frame flows.
- Auth handshake: Hello { token }, HelloOk { capabilities }.

Please:
- Outline the API for this (all needed modules, functions and signatures, structs, etc.) with brief descriptions, omitting the actual implementation
- Break it down into step-by-step LLM prompts, each containing the necessary information for the specific part of the system, that can be used to implement this incrementally. If a prompt would be aided by additional documentation, for instance example code for Postcard, please mention that.

Thank you!




Below is a proposed Rust crate layout with API-only skeletons (traits, structs, enums, type aliases). There is no implementation, only signatures and documentation comments to define responsibilities and boundaries.

After the files, you’ll find a step-by-step set of LLM prompts to implement this system incrementally, with notes on helpful external documentation/examples for each step.

Files

```toml name=Cargo.toml
[package]
name = "workspace_runtime"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[features]
default = []
unix = ["nix"]
windows = ["windows-sys"]

[dependencies]
async-trait = "0.1"
bytes = "1"
cfg-if = "1"
memmap2 = "0.9"
postcard = { version = "1", features = ["use-std"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "io-util", "sync", "time", "process", "signal"] }
tokio-util = { version = "0.7", features = ["codec"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }

# Transport
interprocess = { version = "2", features = ["tokio_support"] }

# Platform specific
nix = { version = "0.29", optional = true, features = ["process", "signal"] }
windows-sys = { version = "0.59", optional = true, features = [
  "Win32_Foundation",
  "Win32_System_Threading",
  "Win32_System_Memory",
  "Win32_System_JobObjects",
  "Win32_System_SystemServices",
  "Win32_System_Pipes",
] }

[dev-dependencies]
serde_json = "1"
```

```rust name=src/workspace/ipc/types.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Monotonic request identifier for RPC calls on a connection.
pub type RequestId = u64;

/// Identifier for a logical stream.
pub type StreamId = u64;

/// Opaque identifier for a process managed by the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub Uuid);

/// Identifier for a connection (control plane) bound to a process/capability set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub Uuid);

/// Opaque identifier for a shared blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobId(pub Uuid);

/// Child process kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChildKind {
    Renderer,
    EmbeddedRenderer,
    StorageService,
    IoService,
    NetworkService,
    Worker,
}

/// A blob token that authorizes mapping/reading/writing to a blob on the data plane.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlobToken {
    pub id: BlobId,
    pub size: u64,
    /// Optional hint for cleanup or lifetime control.
    pub lease_ms: Option<u64>,
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
```

```rust name=src/workspace/ipc/error.rs
use thiserror::Error;

/// Canonical error type for this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Authorization error: {0}")]
    Authorization(String),
    #[error("Capability denied: {0}")]
    CapabilityDenied(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Timeout")]
    Timeout,
    #[error("Canceled")]
    Canceled,
    #[error("Not found")]
    NotFound,
    #[error("Unsupported: {0}")]
    Unsupported(String),
    #[error("Internal: {0}")]
    Internal(String),
}
```

```rust name=src/workspace/ipc/auth/mod.rs
pub mod capability;
```

```rust name=src/workspace/ipc/auth/capability.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An unguessable capability token bound to a single connection/process.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityToken(pub String);

/// A fully-resolved capability set derived from a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Allowed methods per service, including optional quotas/limits.
    pub allowed: HashMap<ServiceName, Vec<MethodRule>>,
    /// Optional global quotas or ceilings.
    pub global_limits: Option<GlobalLimits>,
}

/// Logical service identifier (human-readable, stable across versions).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceName(pub String);

/// Pattern/rule controlling access to a method with optional quotas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodRule {
    pub method: MethodSelector,
    pub quotas: Option<QuotaSet>,
}

/// A method selector can be exact or wildcard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodSelector {
    /// Exact service.method string, e.g., "storage.get".
    Exact(String),
    /// Prefix match, e.g., "network." allows "network.get" and "network.post".
    Prefix(String),
    /// Allow all methods in the service.
    Any,
}

/// Quotas applicable to a method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSet {
    /// Optional bytes/sec rate limit.
    pub bytes_per_sec: Option<u64>,
    /// Optional ops/sec limit.
    pub ops_per_sec: Option<u64>,
    /// Optional burst capacity.
    pub burst: Option<u64>,
}

/// Global limits across the connection/process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLimits {
    pub max_concurrent_requests: Option<u32>,
    pub max_streams: Option<u32>,
    pub max_blob_bytes: Option<u64>,
}

/// A capability bundle given to the workspace to spawn a child process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityBundle {
    /// Initial capability token to authenticate the child’s control connection.
    pub token: CapabilityToken,
    /// Initial capability set bound to the connection.
    pub capabilities: CapabilitySet,
}
```

```rust name=src/workspace/ipc/mod.rs
pub mod auth;
pub mod data_plane;
pub mod error;
pub mod multiplex;
pub mod process_manager;
pub mod protocol;
pub mod router;
pub mod transport;
pub mod types;
```

```rust name=src/workspace/ipc/protocol.rs
use crate::auth::capability::{CapabilitySet, CapabilityToken};
use crate::ipc::types::{BlobToken, RequestId, StreamId};
use serde::{Deserialize, Serialize};

/// Top-level message envelope for framed transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
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

/// Events are server-initiated notifications (e.g., child_exit, permission_changed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub topic: EventTopic,
    /// Postcard-encoded payload per topic.
    pub payload: Vec<u8>,
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
        kind: StreamKind,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamKind {
    VideoFrames,
    ImageBytes,
    FileRead,
    FileWrite,
    Custom(String),
}

/// A compact method identifier (service + method).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodId {
    pub service: String,
    pub method: String,
}

/// Typed service API marker to allow compile-time mapping between Rust methods and MethodId.
pub trait RpcService {
    /// The name of the service for routing and authorization, e.g., "storage".
    const SERVICE_NAME: &'static str;

    /// Handle an incoming postcard-encoded request for a method id, producing a postcard-encoded response or error.
    ///
    /// Implementations should:
    /// - decode args to the appropriate request type
    /// - execute the method
    /// - encode the result to postcard
    /// - return `Ok(Some(Vec<u8>))` for non-void results, `Ok(None)` for void, or `Err(RpcError)`
    fn dispatch(&self, method: &str, args: &[u8]) -> Result<Option<Vec<u8>>, RpcError>;
}

/// Request cancellation for a given id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cancel {
    pub id: RequestId,
}
```

```rust name=src/workspace/ipc/transport.rs
use async_trait::async_trait;
use bytes::Bytes;
use crate::ipc::error::Error;

/// A framed, length-delimited, duplex connection. Multiplexing is layered above this.
#[async_trait]
pub trait FramedConnection: Send + Sync {
    /// Send one frame.
    async fn send_frame(&self, data: Bytes) -> Result<(), Error>;
    /// Receive one frame. Returns None on EOF.
    async fn recv_frame(&self) -> Result<Option<Bytes>, Error>;
    /// Close half or full connection gracefully.
    async fn close(&self) -> Result<(), Error>;
}

/// Factory to connect or accept connections using local sockets.
#[async_trait]
pub trait TransportFactory: Send + Sync {
    type Conn: FramedConnection;

    /// Client side: connect to an endpoint (e.g., path or named pipe).
    async fn connect(&self, endpoint: &str) -> Result<Self::Conn, Error>;

    /// Server side: bind and accept incoming connections.
    async fn bind(&self, endpoint: &str) -> Result<Box<dyn TransportListener<Conn = Self::Conn>>, Error>;
}

#[async_trait]
pub trait TransportListener: Send + Sync {
    type Conn: FramedConnection;

    /// Accept the next connection. Returns None when listener is closed.
    async fn accept(&self) -> Result<Option<Self::Conn>, Error>;

    /// Close the listener.
    async fn close(&self) -> Result<(), Error>;
}
```

```rust name=src/workspace/ipc/multiplex.rs
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use crate::workspace::ipc::protocol::{Message, Request, Response, Event, Cancel, StreamControl};
use crate::ipc::types::{RequestId, StreamId};
use crate::ipc::error::Error;
use std::pin::Pin;

/// A higher-level session that speaks the Message protocol over a FramedConnection.
#[async_trait]
pub trait Session: Send + Sync {
    /// Send a structured message.
    async fn send(&self, msg: &Message) -> Result<(), Error>;

    /// Receive the next structured message (deserializing from frames). None on EOF.
    async fn recv(&self) -> Result<Option<Message>, Error>;
}

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
#[async_trait]
pub trait StreamManager: Send + Sync {
    async fn start(&self, ctl: StreamControl) -> Result<(), Error>;
    async fn next(&self, ctl: StreamControl) -> Result<(), Error>;
    async fn end(&self, ctl: StreamControl) -> Result<(), Error>;

    /// Optional helper for control-plane streaming (chunked via frames).
    fn stream_incoming(&self, id: StreamId) -> Pin<Box<dyn Stream<Item = Bytes> + Send>>;
}
```

```rust name=src/workspace/ipc/data_plane/mod.rs
pub mod shared_memory;
```

```rust name=src/workspace/ipc/data_plane/shared_memory.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::ipc::types::{BlobId, BlobToken};
use async_trait::async_trait;
use crate::ipc::error::Error;

/// Platform-neutral description of a shared memory handle that can be sent via control plane metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharedBlobDescriptor {
    /// Unix file descriptor (sent via SCM_RIGHTS on Unix sockets).
    #[cfg(unix)]
    UnixFd(i32),
    /// Windows HANDLE (duplicated to the target process).
    #[cfg(windows)]
    WindowsHandle(u64),
    /// Cross-platform fallback via temporary file path.
    FilePath(PathBuf),
    /// Opaque handle by name (e.g., named shared memory).
    Named(String),
}

/// A producer-created blob that can be shared with other processes.
pub struct ProducerBlob {
    pub id: BlobId,
    pub size: u64,
    pub descriptor: SharedBlobDescriptor,
    pub token: BlobToken,
}

/// A mapped read-only view of a blob.
pub struct MappedRead<'a> {
    /// Platform-backed mapping pointer and length are implementation details.
    pub len: usize,
    /// Lifetime-bound marker to prevent use-after-free.
    pub _marker: std::marker::PhantomData<&'a ()>,
}

/// Blob allocator for creating and managing shared blobs.
#[async_trait]
pub trait BlobAllocator: Send + Sync {
    /// Create a new blob of the given size and return a producer handle with a lifecycle token.
    async fn create(&self, size: u64) -> Result<ProducerBlob, Error>;

    /// Cleanup a blob proactively using its token (server-side GC).
    async fn cleanup(&self, token: &BlobToken) -> Result<(), Error>;
}

/// Reader side to map an incoming blob by token/descriptor.
#[async_trait]
pub trait BlobReader: Send + Sync {
    /// Map a blob for reading using its token and descriptor metadata.
    async fn map_read<'a>(&'a self, token: &BlobToken, desc: &SharedBlobDescriptor) -> Result<MappedRead<'a>, Error>;
}
```

```rust name=src/workspace/ipc/workspace/mod.rs
pub mod process_manager;
pub mod router;
```

```rust name=src/workspace/ipc/workspace/process_manager.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::ipc::types::{ProcessId, ConnectionId};
use crate::ipc::types::ChildKind;
use crate::auth::capability::CapabilityBundle;
use crate::ipc::error::Error;

/// Parameters for spawning a child.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnParams {
    pub kind: ChildKind,
    /// Executable or command line.
    pub program: String,
    /// Arguments for the child.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: Vec<(String, String)>,
    /// Working directory.
    pub cwd: Option<String>,
    /// Capability bundle bound to the child's control connection.
    pub capabilities: CapabilityBundle,
}

/// A handle with metadata tracked by the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildHandle {
    pub pid: ProcessId,
    pub kind: ChildKind,
    pub connection: Option<ConnectionId>,
}

/// Supervisor/process manager for spawning and tracking child trees with OS-level supervision.
#[async_trait]
pub trait ProcessManager: Send + Sync {
    async fn spawn_child(&self, params: SpawnParams) -> Result<ChildHandle, Error>;
    async fn attach_connection(&self, pid: ProcessId, conn: ConnectionId) -> Result<(), Error>;
    async fn list_children(&self) -> Result<Vec<ChildHandle>, Error>;
    async fn terminate_tree(&self, pid: ProcessId, force: bool) -> Result<(), Error>;
}
```

```rust name=src/workspace/ipc/workspace/router.rs
use async_trait::async_trait;
use crate::auth::capability::CapabilitySet;
use crate::workspace::ipc::protocol::{Message, Request, Response, Event, MethodId, RpcError};
use crate::ipc::types::{ConnectionId, RequestId};
use crate::ipc::error::Error;

/// Context bound to a connection for authorization and audit.
#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub id: ConnectionId,
    pub capabilities: CapabilitySet,
    /// Optional additional metadata (process kind, user, document id, etc.)
    pub metadata: Option<serde_json::Value>,
}

/// Central router interface responsible for:
/// - binding connections to capability sets
/// - enforcing authorization
/// - dispatching requests to services
/// - emitting events
#[async_trait]
pub trait Router: Send + Sync {
    /// Register a new connection after handshake.
    async fn register_connection(&self, ctx: ConnectionContext) -> Result<(), Error>;

    /// Resolve and dispatch a request to a target service method.
    async fn dispatch(&self, ctx: &ConnectionContext, request: Request) -> Result<Response, Error>;

    /// Emit an event to a connection or broadcast to all with appropriate policies.
    async fn emit_event(&self, event: Event) -> Result<(), Error>;

    /// Check whether a given method is allowed by a connection’s capabilities.
    fn is_authorized(&self, ctx: &ConnectionContext, method: &MethodId) -> Result<(), RpcError>;
}
```

```rust name=src/workspace/ipc/services/mod.rs
pub mod storage;
pub mod io_network;
```

```rust name=src/workspace/ipc/services/storage/mod.rs
pub mod api;
```

```rust name=src/workspace/ipc/services/storage/api.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::workspace::ipc::protocol::{RpcService, RpcError};

/// Storage service request/response types.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequest {
    pub table: String,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResponse {
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutRequest {
    pub table: String,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutResponse {
    pub replaced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub table: String,
    pub key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub existed: bool,
}

/// Storage service trait with typed methods. The IPC layer will map these to MethodIds.
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn get(&self, req: GetRequest) -> Result<GetResponse, RpcError>;
    async fn put(&self, req: PutRequest) -> Result<PutResponse, RpcError>;
    async fn delete(&self, req: DeleteRequest) -> Result<DeleteResponse, RpcError>;
}

/// Method name constants for routing/authorization.
pub mod methods {
    pub const SERVICE: &str = "storage";
    pub const GET: &str = "get";
    pub const PUT: &str = "put";
    pub const DELETE: &str = "delete";
}

/// A server-side adapter to expose a StorageService behind RpcService.
pub struct StorageRpcAdapter<S: StorageService>(pub S);

impl<S: StorageService> RpcService for StorageRpcAdapter<S> {
    const SERVICE_NAME: &'static str = methods::SERVICE;

    fn dispatch(&self, method: &str, args: &[u8]) -> Result<Option<Vec<u8>>, RpcError> {
        match method {
            m if m == methods::GET => {
                let req: GetRequest = postcard::from_bytes(args).map_err(|e| RpcError{ code: "decode".into(), message: e.to_string() })?;
                let fut = self.0.get(req);
                // Implementation should await and encode result; omitted here by design.
                Err(RpcError{ code: "unimplemented".into(), message: "adapter body to be implemented".into() })
            }
            m if m == methods::PUT => Err(RpcError{ code: "unimplemented".into(), message: "adapter body to be implemented".into() }),
            m if m == methods::DELETE => Err(RpcError{ code: "unimplemented".into(), message: "adapter body to be implemented".into() }),
            _ => Err(RpcError{ code: "not_found".into(), message: "unknown method".into() }),
        }
    }
}

/// Client-side trait for typed storage calls (stubs will use RpcClient underneath).
#[async_trait]
pub trait StorageClient: Send + Sync {
    async fn get(&self, req: GetRequest) -> Result<GetResponse, RpcError>;
    async fn put(&self, req: PutRequest) -> Result<PutResponse, RpcError>;
    async fn delete(&self, req: DeleteRequest) -> Result<DeleteResponse, RpcError>;
}
```

```rust name=src/workspace/ipc/services/io_network/mod.rs
pub mod api;
```

```rust name=src/workspace/ipc/services/io_network/api.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::workspace::ipc::protocol::{RpcService, RpcError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileRequest {
    pub path: String,
    pub offset: Option<u64>,
    pub len: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileResponse {
    /// For large reads, this may be None and data is delivered via Stream/Blob.
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// May be None if body is streamed via Stream/Blob.
    pub body: Option<Vec<u8>>,
}

#[async_trait]
pub trait IoNetworkService: Send + Sync {
    async fn read_file(&self, req: ReadFileRequest) -> Result<ReadFileResponse, RpcError>;
    async fn fetch(&self, req: FetchRequest) -> Result<FetchResponse, RpcError>;
}

pub mod methods {
    pub const SERVICE: &str = "io_net";
    pub const READ_FILE: &str = "read_file";
    pub const FETCH: &str = "fetch";
}

pub struct IoNetworkRpcAdapter<S: IoNetworkService>(pub S);

impl<S: IoNetworkService> RpcService for IoNetworkRpcAdapter<S> {
    const SERVICE_NAME: &'static str = methods::SERVICE;

    fn dispatch(&self, method: &str, _args: &[u8]) -> Result<Option<Vec<u8>>, RpcError> {
        match method {
            m if m == methods::READ_FILE => Err(RpcError{ code: "unimplemented".into(), message: "adapter body to be implemented".into() }),
            m if m == methods::FETCH => Err(RpcError{ code: "unimplemented".into(), message: "adapter body to be implemented".into() }),
            _ => Err(RpcError{ code: "not_found".into(), message: "unknown method".into() }),
        }
    }
}

#[async_trait]
pub trait IoNetworkClient: Send + Sync {
    async fn read_file(&self, req: ReadFileRequest) -> Result<ReadFileResponse, RpcError>;
    async fn fetch(&self, req: FetchRequest) -> Result<FetchResponse, RpcError>;
}
```

```rust name=src/workspace/ipc/renderer/mod.rs
pub mod api;
```

```rust name=src/workspace/ipc/renderer/api.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::ipc::types::{ProcessId};
use crate::workspace::ipc::protocol::{RpcError};

/// Renderer context provided by the workspace upon startup/handshake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendererContext {
    pub process_id: ProcessId,
    /// Arbitrary document metadata.
    pub document_id: String,
    /// Initial permissions summary for the renderer.
    pub permissions: serde_json::Value,
}

/// Entry-point interface for a renderer process. The workspace will spawn the process and
/// the renderer will run until shutdown.
#[async_trait]
pub trait RendererMain: Send + Sync {
    async fn run(&self, ctx: RendererContext) -> Result<(), RpcError>;
}
```

```rust name=src/workspace/ipc/platform/mod.rs
pub mod unix;
pub mod windows;
```

```rust name=src/workspace/ipc/platform/unix.rs
/// Unix-specific process supervision primitives (to be implemented with nix):
/// - setpgid to create process groups
/// - prctl(PR_SET_PDEATHSIG)
/// - killpg for tree termination
pub struct UnixProcessSupervisor;

/// Unix shared memory helpers (memfd, SCM_RIGHTS) should be implemented here as needed.
pub struct UnixSharedMemory;
```

```rust name=src/workspace/ipc/platform/windows.rs
/// Windows-specific process supervision primitives:
/// - Job Objects with kill-on-close
/// - TerminateJobObject for tree termination
pub struct WindowsProcessSupervisor;

/// Windows shared memory helpers (CreateFileMapping, DuplicateHandle).
pub struct WindowsSharedMemory;
```


Implementation plan: step-by-step LLM prompts

Use these prompts to incrementally implement the system. Each step includes the scope, acceptance criteria, and helpful references.

1) Crate scaffolding and common types
- Prompt:
  Implement crate scaffolding and common types. Add dependencies in Cargo.toml as specified. Implement src/workspace/ipc/types.rs and src/workspace/ipc/error.rs exactly as defined. Ensure the crate compiles (cargo check).
- Acceptance criteria:
  - cargo check succeeds on Unix and Windows feature sets (even if platform modules are stubs).
- Helpful docs:
  - Cargo manifest basics and feature flags.

2) IPC protocol: messages and handshake
- Prompt:
  Implement src/workspace/ipc/protocol.rs: define Message, Hello/HelloOk/HelloErr, Request/Response, Event, StreamControl, MethodId, RpcError, Cancel exactly as outlined. Derive serde traits. Add unit tests that serialize/deserialize each message with postcard and assert round-trips. Show how to encode/decode using postcard::to_stdvec and from_bytes.
- Acceptance criteria:
  - Unit tests pass for postcard round-tripping.
- Helpful docs:
  - Postcard README and examples.
  - Serde attributes for enums with tag/content.

3) Transport: framed local sockets with tokio-util
- Prompt:
  Implement src/workspace/ipc/transport.rs for Unix and Windows using interprocess::local_socket (tokio support) and tokio_util::codec::LengthDelimitedCodec. Provide a concrete FramedConnection that sends/receives frames (Bytes) and a TransportFactory with connect/bind/accept for endpoints. Include graceful close semantics. Add an integration test that echoes frames between a server and client.
- Acceptance criteria:
  - Echo test passes on each platform.
- See attached supporting reference documentation:
  - interprocess crate local_socket tokio docs.
  - tokio-util LengthDelimitedCodec docs.

4) Session and multiplexing with correlation IDs
- Prompt:
  Build a Session implementation over FramedConnection that encodes/decodes Message with postcard. Implement an RpcClient that manages inflight requests by RequestId, supports Cancel, and exposes an events stream via a tokio::sync::mpsc receiver. Ensure backpressure handling and error propagation. Add tests: request-response pairing, out-of-order responses, cancellation behavior.
- Acceptance criteria:
  - Unit tests simulate concurrent requests and verify correct correlation and cancellation.
- Helpful docs:
  - tokio mpsc channels.
  - Futures streams and pinning patterns.

5) Authorization and router skeleton
- Prompt:
  Implement a Router that:
  - Registers ConnectionContext post-handshake.
  - Authorizes requests by matching MethodId against CapabilitySet rules (Exact/Prefix/Any).
  - For now, only validates authorization and returns a canned “not implemented” Response on success.
  Add tests for authorization matrix (allowed vs denied based on MethodSelector and per-service allowed lists).
- Acceptance criteria:
  - Authorization unit tests pass (denials produce Response with ok=false and error code).
- Helpful docs:
  - Pattern matching and string prefix checks in Rust.

6) Process manager: spawn, attach, terminate tree (platform hooks)
- Prompt:
  Implement a ProcessManager using tokio::process to spawn children and platform-specific supervision:
  - Unix: setpgid/session and prctl(PR_SET_PDEATHSIG). Provide killpg for tree termination.
  - Windows: create Job Object per subtree with kill-on-close; attach child processes; TerminateJobObject on kill.
  Expose spawn_child, attach_connection, list_children, terminate_tree. Include tests that spawn a no-op child process and terminate it with force=false then force=true timeouts.
- Acceptance criteria:
  - Tests pass on each platform (guard platform-specific tests with cfg).
- Helpful docs:
  - nix crate for setpgid, prctl, killpg.
  - Windows Job Objects with windows-sys examples.

7) Handshake flow and capability binding
- Prompt:
  Implement handshake flow:
  - Client sends Hello { token, client_info }.
  - Server validates token -> CapabilitySet, returns HelloOk or HelloErr.
  - On success, Router::register_connection is called with ConnectionContext.
  Wire this into Session to perform a handshake helper at connection setup. Add tests with fake token verifier.
- Acceptance criteria:
  - Handshake unit tests cover success and failure paths; ConnectionContext is stored.
- Helpful docs:
  - Design of capability token verification, potential HMAC contents.

8) Heartbeats and liveness detection
- Prompt:
  Add periodic heartbeat Events from child->workspace. Implement a HeartbeatTracker in the workspace that marks a connection dead if N intervals are missed. On dead, trigger ProcessManager::terminate_tree and cleanup resources. Add tests that simulate missed heartbeats and verify cleanup is invoked.
- Acceptance criteria:
  - Heartbeat miss triggers termination and removal from tracking.
- Helpful docs:
  - tokio time intervals and cancellation.

9) Graceful shutdown protocol
- Prompt:
  Implement graceful shutdown:
  - Workspace sends a shutdown Request to a child (define a workspace method: workspace.shutdown).
  - Child acknowledges and exits voluntarily; workspace waits up to timeout then kills tree if needed.
  Add tests where the child cooperates and where the child ignores shutdown (force kill).
- Acceptance criteria:
  - Both scenarios covered; resources reclaimed in each case.
- Helpful docs:
  - tokio::process child.wait, timeouts.

10) Data plane: shared blob creation, mapping, and lifecycle
- Prompt:
  Implement BlobAllocator and BlobReader:
  - Unix: memfd + SCM_RIGHTS passing.
  - Windows: CreateFileMapping + DuplicateHandle.
  - Fallback: temp files via memmap2.
  Implement ProducerBlob and BlobToken lifecycle, with server GC on crash (e.g., token-based cleanup). Add tests that create a blob, write data, map in reader, verify content round-trip.
- Acceptance criteria:
  - Data plane tests pass on supported platforms; fallback path covered.
- Helpful docs:
  - memfd_create and SCM_RIGHTS (Unix).
  - CreateFileMapping/MapViewOfFile and DuplicateHandle (Windows).
  - memmap2 usage.

11) Services: storage typed RPC and client stubs
- Prompt:
  Implement StorageService server and StorageClient over RpcClient:
  - Define a mapping from methods::{GET, PUT, DELETE} to MethodId.
  - Implement server adapter dispatch: decode args via postcard, call service, encode result.
  - Implement client stub methods that build Request, await Response, decode result or map error.
  Add unit tests using a mock StorageService and a loopback RpcClient.
- Acceptance criteria:
  - Typed get/put/delete round-trips pass, errors propagate with RpcError codes.
- Helpful docs:
  - postcard usage patterns for small structs.

12) Services: IO/Network typed RPC and streaming integration
- Prompt:
  Implement IoNetworkService and IoNetworkClient with streaming:
  - read_file may use StreamControl or Blob when len is large; include a threshold constant.
  - fetch supports body streaming similarly.
  Add integration tests that exercise both small inline payloads and large blob-backed flows.
- Acceptance criteria:
  - Tests verify both code paths and correct resource cleanup for blobs/streams.
- Helpful docs:
  - Deciding thresholds; combining control-plane and data-plane coordination.

13) Renderer API, embedded documents, and workspace routing
- Prompt:
  Implement RendererMain contract and workspace routing:
  - Workspace spawns one renderer per document with a capability bundle.
  - Router forwards renderer->service calls based on MethodId and authorization.
  - Implement “message to parent” capability for embedded renderers via a workspace-scoped method (workspace.message_parent).
  Add tests setting up a mock renderer and asserting it can read files but cannot write to storage unless permitted.
- Acceptance criteria:
  - Authorization enforcement in real routing; embedded renderer messaging verified.
- Helpful docs:
  - Capability sets and method selectors, ensuring least privilege by default.

14) Cancellation and quotas/rate-limits enforcement
- Prompt:
  Implement request cancellation wiring end-to-end and basic quotas:
  - RpcClient.cancel sends Cancel; services should observe cancellation via cooperative checks (e.g., context).
  - Enforce bytes/sec on IO and Network via simple token bucket per connection/method.
  Add tests that cancel an in-flight fetch and that exceed rate limits to produce CapabilityDenied errors.
- Acceptance criteria:
  - Cancellation stops work; rate-limits enforced with clear error codes.
- Helpful docs:
  - Token bucket algorithm; tokio time.

15) Telemetry with tracing and IDs
- Prompt:
  Integrate tracing spans for process, connection, request, and stream ids. Propagate correlation ids in logs. Provide helpers to create spans when handling Requests and Streams. Add a test/log capture that asserts span fields are present.
- Acceptance criteria:
  - Logs include ids; tests validate presence of fields.
- Helpful docs:
  - tracing crate spans, fields, subscriber setup.

16) End-to-end example and documentation
- Prompt:
  Create an examples/minimal_workspace and examples/minimal_renderer demonstrating:
  - Workspace spawns a renderer with capability to read a file via IO service.
  - Renderer handshakes, sends heartbeat, issues read_file, receives data, and shuts down.
  Provide a README describing the architecture and how to run the example.
- Acceptance criteria:
  - Example runs locally, prints expected logs, and exits cleanly.
- Helpful docs:
  - tokio main patterns; interprocess endpoint addressing.

Notes and documentation suggestions
- For Postcard serialization, include a small helper module or examples showing to_stdvec/from_bytes and error mapping to RpcError.
- For interprocess local sockets, consult crate examples for both Unix and Windows endpoints (names differ).
- For Unix process supervision, reference prctl(PR_SET_PDEATHSIG) and setpgid; for Windows, Job Objects with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.
- For shared memory fallback via memmap2, use tempfile + map ranges, with careful cleanup on token revoke.
- For rate limiting, implement a per-connection MethodKey = (service, method) token bucket and integrate checks into Router::dispatch before calling services.

If you want, I can start by filling in one of the modules (e.g., Transport or Protocol round-trip tests) following the above plan.
