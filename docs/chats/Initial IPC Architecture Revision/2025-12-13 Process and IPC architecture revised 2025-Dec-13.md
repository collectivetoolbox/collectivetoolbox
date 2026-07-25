
### Revised Step 12: Thin Network Service Interfaces and IPC Integration
- **Prompt**:
  Edit the following files to refactor the Network service into a thin IPC interface that delegates to existing `network` module that's at the top level of the crate:
  - api.rs (remove detailed networking implementations; replace with trait-based delegation to external `network` module via dependency injection)
  - network.rs (detailed networking implementations should go here; for now add fetch_http and fetch_https methods; later it will primarily use a WebRTC transport to communicate with other instances)
  - protocol.rs (retain StreamControl and threshold constants for transport, but remove protocol-specific logic)
  - multiplex.rs (ensure StreamManager handles only transport coordination, not data processing)
  - router.rs (update routing to pass requests to thin service adapters)

  Refactor Network service:
  - Define minimal traits (e.g., `NetworkService`) with method signatures that match your existing `network` modules' APIs (e.g., `read_file(path: String) -> Result<Vec<u8>, Error>`, `fetch(url: String) -> Result<Vec<u8>, Error>`).
  - Implement service adapters that take trait objects from the existing modules and delegate calls without implementing logic.
  - Add integration tests that mock the external modules and verify IPC routing/serialization works.
  - Leave only networking code in the module; omit other IO code.
- **Acceptance criteria**:
  - IPC module contains no business logic; all network operations delegate to external modules. No lint errors in network.rs or the ipc module. Tests pass with mocked dependencies.
- **Helpful docs**:
  - Dependency injection patterns in Rust; trait objects for modularity.

### Revised Step 13: Thin Renderer Service Interface and Workspace Routing
- **Prompt**:
  Edit the following files to implement a thin renderer service interface that delegates to your existing `renderer` module:
  - `src/workspace/ipc/services/renderer/api.rs (define minimal `RendererService` trait with method signatures matching the existing `renderer` module)
  - router.rs (add routing for renderer methods, delegating to injected `renderer` module instances)

  Implement thin renderer service:
  - Fill in the new `renderer` service module in IPC with trait-based delegation, modeled on `workspace/ipc/services/network`, which is attached for reference.
  - Router forwards renderer requests based on MethodId, enforcing authorization but not implementing rendering.
  - Support "message to parent" via a workspace-scoped method that calls the existing messaging logic.
  - Add tests with mocked renderer modules to verify routing and authorization.
- **Acceptance criteria**:
  - Renderer logic remains in the dedicated module; IPC only handles communication and routing. Authorization tests pass.
- **Helpful docs**:
  - Capability sets and method selectors; least-privilege design.

### Revised Step 14: Cancellation and Quotas/Rate-Limits Enforcement
- **Prompt**:
  Edit the following files to implement request cancellation and basic quotas without embedding service logic:
  - protocol.rs (retain Cancel message handling for transport)
  - multiplex.rs (implement RpcClient.cancel for transport-level cancellation)
  - `src/workspace/ipc/auth/capability.rs` (define QuotaSet and GlobalLimits for rate-limiting metadata)
  - router.rs (enforce quotas/rate-limits on requests, observe cancellation via cooperative checks)

  Implement cancellation and quotas:
  - RpcClient.cancel sends Cancel messages; services observe cancellation via context (e.g., `is_cancelled()`).
  - Enforce bytes/sec limits on IO/Network via token buckets, but delegate actual limits to injected quota checkers from your modules.
  - Add tests that cancel in-flight requests and verify rate-limit errors propagate without service-specific code.
- **Acceptance criteria**:
  - Cancellation stops requests; rate-limits enforced with clear errors. No service logic in IPC.
- **Helpful docs**:
  - Token bucket algorithm; tokio time for cancellation.

### Revised Step 15: Telemetry with Tracing and IDs
- **Prompt**:
  Edit the following files to integrate tracing spans for IPC-level IDs without service details:
  - types.rs (add tracing integration for ProcessId, ConnectionId, RequestId, StreamId)
  - protocol.rs (create spans for Request/Stream handling)
  - router.rs (add spans in dispatch and emit_event)
  - multiplex.rs (add spans in RpcClient operations)

  Integrate tracing:
  - Propagate correlation IDs in logs for IPC operations.
  - Create spans when handling Requests/Streams, logging transport-level events.
  - Add tests that capture logs and assert span fields are present.
- **Acceptance criteria**:
  - Logs include IPC IDs; tests validate fields without service details.
- **Helpful docs**:
  - tracing crate spans and fields.

### STILL TO DO: Parent messaging within process trees (`rg ParentMessenger`)


### Revised Step 16: End-to-End Example and Documentation
- **Prompt**:
  Edit the following files to create examples demonstrating thin IPC integration with existing modules:
  - `examples/minimal_workspace/main.rs` (new file: workspace setup, spawning processes, injecting module dependencies)
  - `examples/minimal_renderer/main.rs` (new file: renderer handshake, heartbeat, calling thin IPC methods that delegate to existing modules)
  - README.md (add architecture description emphasizing IPC as thin layer, with run instructions)

  Create examples:
  - Workspace spawns a renderer, injecting dependencies from your existing modules (e.g., `io`, `network`).
  - Renderer uses IPC to call thin methods (e.g., read_file via `io` module), receives data, and shuts down. Example should have the renderer send a message as a data frame, and also a non-data-frame message requesting a subdocument renderer, which should then get started.
  - README describes IPC as communication-only, with examples running locally.
- **Acceptance criteria**:
  - Example runs cleanly, delegating to existing modules; README clarifies thin IPC design.
- **Helpful docs**:
  - tokio main patterns; interprocess addressing.
