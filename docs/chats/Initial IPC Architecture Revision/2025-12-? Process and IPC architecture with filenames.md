7) Handshake flow and capability binding
- Prompt:
  Edit the following files to implement handshake flow:
  - `src/workspace/ipc/protocol.rs` (for Hello, HelloOk, HelloErr message handling and handshake logic)
  - `src/workspace/ipc/router.rs` (for Router::register_connection implementation)
  - `src/workspace/ipc/multiplex.rs` (for Session handshake helper method)
  - `src/workspace/ipc/auth/capability.rs` (for token validation and CapabilitySet derivation)
  
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
  Edit the following files to add periodic heartbeat Events from child->workspace:
  - `src/workspace/ipc/protocol.rs` (for Heartbeat Event definition and sending)
  - `src/workspace/ipc/router.rs` (for HeartbeatTracker implementation and connection tracking)
  - `src/workspace/ipc/process_manager.rs` (for ProcessManager::terminate_tree on dead connections)
  
  Implement a HeartbeatTracker in the workspace that marks a connection dead if N intervals are missed. On dead, trigger ProcessManager::terminate_tree and cleanup resources. Add tests that simulate missed heartbeats and verify cleanup is invoked.
- Acceptance criteria:
  - Heartbeat miss triggers termination and removal from tracking.
- Helpful docs:
  - tokio time intervals and cancellation.

9) Graceful shutdown protocol
- Prompt:
  Edit the following files to implement graceful shutdown:
  - `src/workspace/ipc/process_manager.rs` (for waiting on child exit and force killing tree)
  - `src/workspace/ipc/services.rs` (to add a new workspace service module for defining workspace.shutdown method)
  - `src/workspace/ipc/services/workspace.rs` (new file for workspace service API, including shutdown request/response types and trait)
  - `src/workspace/ipc/services/workspace/api.rs` (new file for workspace service API, including shutdown request/response types and trait)
  - `src/workspace/ipc/router.rs` (for routing workspace.shutdown requests)
  
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
  Edit the following files to implement BlobAllocator and BlobReader:
  - `src/workspace/ipc/data_plane/shared_memory.rs` (for BlobAllocator, BlobReader, ProducerBlob, and BlobToken lifecycle)
  - `src/workspace/ipc/platform/unix.rs` (for Unix-specific memfd + SCM_RIGHTS implementation)
  - `src/workspace/ipc/platform/windows.rs` (for Windows-specific CreateFileMapping + DuplicateHandle implementation)
  
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
  Edit the following files to implement StorageService server and StorageClient over RpcClient:
  - `src/workspace/ipc/services/storage/api.rs` (for StorageService trait, StorageRpcAdapter dispatch implementation, and StorageClient stubs)
  - `src/workspace/ipc/protocol.rs` (for MethodId mapping from methods::{GET, PUT, DELETE})
  
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
  Edit the following files to implement IoNetworkService and IoNetworkClient with streaming:
  - `src/workspace/ipc/services/io_network/api.rs` (for IoNetworkService trait, IoNetworkRpcAdapter dispatch, and IoNetworkClient stubs with streaming logic)
  - `src/workspace/ipc/protocol.rs` (for StreamControl integration and threshold constants)
  - `src/workspace/ipc/multiplex.rs` (for StreamManager coordination with blob-backed flows)
  
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
  Edit the following files to implement RendererMain contract and workspace routing:
  - `src/workspace/ipc/renderer/api.rs` (for RendererMain trait and RendererContext)
  - `src/workspace/ipc/process_manager.rs` (for spawning renderer per document with capability bundle)
  - `src/workspace/ipc/router.rs` (for forwarding renderer->service calls based on MethodId and authorization)
  - `src/workspace/ipc/services/workspace/api.rs` (to add workspace.message_parent method for embedded renderers)
  
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
  Edit the following files to implement request cancellation wiring end-to-end and basic quotas:
  - `src/workspace/ipc/protocol.rs` (for Cancel message handling)
  - `src/workspace/ipc/multiplex.rs` (for RpcClient.cancel implementation)
  - `src/workspace/ipc/auth/capability.rs` (for QuotaSet and GlobalLimits definitions)
  - `src/workspace/ipc/router.rs` (for enforcing quotas/rate-limits and cancellation observation)
  
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
  Edit the following files to integrate tracing spans for process, connection, request, and stream ids:
  - `src/workspace/ipc/types.rs` (to add tracing integration for IDs like ProcessId, ConnectionId, RequestId, StreamId)
  - `src/workspace/ipc/protocol.rs` (for span creation in Request and Stream handling)
  - `src/workspace/ipc/router.rs` (for spans in dispatch and emit_event)
  - `src/workspace/ipc/multiplex.rs` (for spans in RpcClient operations)
  
  Integrate tracing spans for process, connection, request, and stream ids. Propagate correlation ids in logs. Provide helpers to create spans when handling Requests and Streams. Add a test/log capture that asserts span fields are present.
- Acceptance criteria:
  - Logs include ids; tests validate presence of fields.
- Helpful docs:
  - tracing crate spans, fields, subscriber setup.

16) End-to-end example and documentation
- Prompt:
  Edit the following files to create an examples/minimal_workspace and examples/minimal_renderer demonstrating:
  - `examples/minimal_workspace/main.rs` (implement workspace setup, spawning renderer, and running the example; to start the renderer it should call itself with a command-line argument added to tell it to act as renderer, in which case it will provide renderer handshake, heartbeat, read_file call, and shutdown)
  - `examples/minimal_renderer/README.md` (to add architecture description and run instructions)
  
  Create an examples/minimal_workspace and examples/minimal_renderer demonstrating:
  - Workspace spawns a renderer with capability to read a file via IO service.
  - Renderer handshakes, sends heartbeat, issues read_file, receives data, and shuts down.
  Provide a README describing the architecture and how to run the example.
- Acceptance criteria:
  - Example runs locally, prints expected logs, and exits cleanly.
- Helpful docs:
  - tokio main patterns; interprocess endpoint addressing.