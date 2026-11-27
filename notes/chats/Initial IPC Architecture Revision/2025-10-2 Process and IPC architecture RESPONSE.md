Thanks for the detailed summary and code. You’re on the right track with isolating documents into separate OS processes and brokering cross-process calls. Below is an actionable plan that addresses the main flaws and guides a refactor toward a robust, efficient, and cross-platform architecture.

High-level goals to keep
- Keep process isolation per document (“renderer”), supervised by a single “workspace” process.
- Enforce single-writer/single-process ownership for resources that require it (e.g., database).
- Allow renderers to call services via well-defined, capability-gated IPC.
- Ensure a renderer crash doesn’t bring down the workspace or other renderers.
- Provide a high-throughput/low-overhead path for large payloads (e.g., frames).

What’s problematic today
- Transport and protocol
  - JSON over HTTP with polling (ureq + tiny_http) is inefficient, blocking, and hard to extend (no backpressure, no streaming, high overhead for binary frames).
  - Polling with tight loops is wasteful. You want a persistent, duplex channel with request-response and streaming support.
- Concurrency model
  - Mixing Tokio async and blocking (ureq, tiny_http, std::thread::spawn) will cause performance and correctness problems.
  - Global state via OnceCell<Arc<Mutex<…>>> plus std::sync::Mutex inside async contexts is risky; you can deadlock or block the runtime.
- Process management
  - No proper supervisor tree semantics; cleanup relies on panic hooks and best-effort iteration, not OS facilities.
  - ProcessManager uses processes.len() for IDs; removing entries later can lead to ID reuse/bugs.
  - No structured lifecycle (graceful shutdown with escalation; heartbeats; watchdog).
- Security/authorization
  - Per-channel “auth key in header” with dynamic add_channel is simple but doesn’t tie method authorization to capabilities; any holder of a channel can call any method for that service.
  - Server trusts names too much (Vec<Channel> keyed by name). Use a map keyed by unguessable token, and bind allowed methods per token.
- Design boundaries
  - Storage/DB code in IPC module is a smell. If DB is single-process, make it a dedicated service process and move DB code behind that service’s API.
  - “One subprocess per Rust module” will cause a process explosion and awkward service boundaries. Prefer a small set of service processes with well-defined APIs, not per-module processes.
- Data plane for frames
  - JSON-encoding bitmaps is a non-starter for performance.

Recommended architecture

1) Process model and supervision
- Workspace = supervisor + IPC router
  - Owns all listener endpoints.
  - Spawns and tracks child processes and their metadata.
  - Enforces authorization by capability token per child.
  - Handles routing for anything that must be centrally arbitrated (e.g., access to shared services).
- Service processes (stable small set)
  - storage service: sole owner of the DB (only this process touches the DB).
  - io/network service(s): optional, depending on how much you want to isolate.
  - renderer processes: one per document (and per embedded document), sandboxed.
- Renderer subprocesses should not spawn more subprocesses directly
  - Instead, renderers ask the workspace to spawn a child (renderer or worker), passing a capability bundle. This preserves policy enforcement and supervision.
- OS-level supervision
  - Unix: create a new process group/session for each renderer subtree (setpgid) and set PR_SET_PDEATHSIG to ensure children die if parent dies.
  - Windows: attach each subtree to a Job Object with “kill on close” semantics. On shutdown, terminate the job to kill the whole subtree.
  - Keep explicit Child handles and wait on exit to reclaim resources and update ProcessManager.

2) IPC transport and protocol
- Transport
  - Use a persistent, duplex, framed connection per process.
    - Unix: Unix Domain Sockets via tokio::net::UnixStream
    - Windows: Named Pipes via a crate like tokio-named-pipes or interprocess crate (LocalSocket) for a cross-platform abstraction.
  - Avoid per-message HTTP or polling. Maintain a single connection and multiplex logical channels over it.
- Framing & serialization
  - Use length-delimited framing (tokio-util::codec::LengthDelimitedCodec) or an established RPC protocol.
  - Serialize with bincode or MessagePack (rmp-serde) for compactness. JSON can still be supported for debug, but not for hot paths.
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
  - Prefer a compile-time-checked API: define traits/enums representing methods. Use serde with enums or codegen (e.g., tarpc, tonic) to generate client/server stubs.

3) Data plane for large payloads (frames)
- Use shared memory or memory-mapped files for large binary blobs.
  - Unix: memfd + pass FD over Unix socket (SCM_RIGHTS).
  - Windows: CreateFileMapping + DuplicateHandle or named shared memory.
  - Cross-platform fallback: temporary memory-mapped files via memmap2; pass a handle/path/token via control channel; readers map and read.
- Wrap blobs with a lifecycle token so the server can clean up if the producer crashes.
- Control plane (RPC) sends metadata and a token; data plane (shared memory) carries the bytes. This is close to zero-copy and will be orders of magnitude faster than JSON.

4) Capability and permission model
- Capability tokens:
  - Minted by workspace per process or per “channel” (logical namespace).
  - Bind to allowed method set (e.g., renderer A can call IO.readFile, Network.getUrl; cannot call Storage.put).
  - Optionally include per-method quotas and rate limits (e.g., bytes/sec for network, frame rate).
- Enforce at the server boundary (workspace or service process).
- Avoid linking disallowed libraries in renderer process:
  - Practically, Rust static linking can’t “block” a crate at runtime. The robust approach is to keep sensitive libraries only in the service process; the renderer uses only the client stubs.

5) Concurrency and runtime
- Standardize on Tokio across the app.
  - Replace tiny_http/ureq with async equivalents (tokio sockets or a cross-platform IPC abstraction).
  - Replace std::sync::Mutex in async code:
    - Use tokio::sync::Mutex/RwLock when lock will be held across awaits.
    - Or better, architect so locks aren’t held across await points (message passing and internal task ownership).
- Replace busy loops and sleep() with await-based readiness and backpressure.
- Use tracing for structured logs with spans carrying process and request IDs.

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
- IO/Network service(s)
  - Optional separation. If network/file IO should be restricted, put them in separated services and gate access by capability.
  - Otherwise, keep in renderer but use capability gating at the workspace for cross-boundary calls.
- Renderer
  - Single process per document. Don’t spawn per-module worker processes unless you have strong isolation needs; prefer shared services.
- Embedded documents
  - Embedded renderer is just another renderer child, with a capability set that includes “message to parent” and limited outbound calls.
  - Two-way communication via the workspace router so policies can apply.

8) Protocol shape (sketch)
- Request { id: u64, method: enum, args: Value } serialized with bincode or rmp.
- Response { id: u64, ok: bool, result: Value | error: String }.
- Stream { id, kind: “start/next/end”, blob_token? } for frame flows.
- Auth handshake: Hello { token }, HelloOk { capabilities }.

Key refactors in your codebase

- Replace ProcessManager’s ID generation
  - Keep a monotonic counter last_id; don’t derive IDs from processes.len().
  - Track process groups/job handles for subtree management.
- Collapse IPC_API + string method matching into typed enums/traits
  - Define a ServiceMethod enum per service. Deserialize directly into that enum to avoid “stringly-typed” jq/jqq parsing and panics.
- Replace HTTP server and polling
  - Introduce a cross-platform IPC endpoint (interprocess LocalSocket or per-OS socket/pipe) with Tokio.
  - Maintain one connection per child with a multiplexed protocol.
- Move DB code into storage service process
  - Expose typed RPC; the renderer and workspace use client stubs.
  - Remove DB references from other processes entirely.
- Large payload path
  - Introduce a BlobAllocator service: allocate() -> BlobToken, writer maps and writes, notify consumer via RPC; consumer maps and reads; allocator cleans up.
- Authorization model
  - Capability tokens minted by workspace with per-connection allowed_methods; enforced at the service boundary. Drop “add_channel” messages exposed to children.
- Observability
  - Use tracing with spans: span per connection, per request, include pid, doc_id, request_id.

Cross-platform notes
- Unix:
  - Use setpgid and prctl(PR_SET_PDEATHSIG, SIGKILL) for orphan cleanup.
  - UDS + SCM_RIGHTS for sending FDs (memfd).
- Windows:
  - Use Job Objects to manage subtree lifetimes.
  - Named pipes for IPC; shared memory via CreateFileMapping.
- For both:
  - Memory-mapped files (memmap2) are a good lowest-common denominator for blobs.

Migration plan (phased)

Phase 0: Hardening current approach
- Fix ProcessManager ID monotonic counter and avoid unwraps.
- Restrict add_channel to the workspace-only control connection. Don’t accept from arbitrary channels.
- Stop logging secrets (auth keys).
- Replace jq/jqq string parsing with serde + typed structs.

Phase 1: Introduce typed RPC and capability gating (still over current transport)
- Define ServiceMethod enums and a Request/Response envelope.
- Add per-connection capability check before method dispatch.
- Keep HTTP temporarily but eliminate polling by switching to long-lived connection (if sticking with HTTP, at least use HTTP/2 with persistent streams; otherwise go straight to sockets).

Phase 2: Switch transport to persistent sockets/pipes with Tokio
- Introduce a single server listener in workspace; children connect.
- Implement length-delimited frames + rmp-serde or bincode.
- Replace ureq/tiny_http calls and thread polling loops with async tasks.

Phase 3: Storage service extraction
- Move DB code into a dedicated storage process.
- Update storage module to be a client stub only.
- Add request-level authorization and quotas.

Phase 4: High-throughput data plane
- Implement shared memory blobs with a simple allocator and lifecycle.
- Replace bitmap-in-JSON with blob tokens in RPC.

Phase 5: OS-level supervision and robustness
- Implement process groups + PDEATHSIG (Unix) and Job Objects (Windows).
- Add heartbeats, graceful shutdown with escalation, and subtree kill.

Phase 6: Sandbox tightening (optional but recommended)
- Linux: seccomp-bpf profiles to limit syscalls in renderer processes.
- macOS: seatbelt rules where feasible.
- Windows: process mitigation policies; consider AppContainer if acceptable.

Significant flaws identified in the current plan/code
- JSON-over-HTTP polling is inefficient and unsuitable for large payloads and streaming.
- Blocking HTTP client/server in an async runtime; potential deadlocks and poor performance.
- Weak authorization: channel name + shared secret without per-method capability scoping.
- ProcessManager ID management and use of HashMap len as ID; unsafe unwrapping and potential panics.
- Global state and std::sync::Mutex in async contexts.
- Renderer spawning additional subprocesses for modules leads to process sprawl and unclear boundaries.
- DB access placed in IPC for “safety”; the correct solution is a dedicated storage service that owns the DB.
- No robust OS-level subtree kill or heartbeat/health checks; cleanup relies on best-effort iteration.

If you’d like, I can sketch a minimal transport switch (Tokio + LocalSocket with a simple Request/Response envelope and a capability check) or outline a typed ServiceMethod enum and dispatcher to replace the string-based IPC_API and jq parsing.