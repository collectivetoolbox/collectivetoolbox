# Minimal Workspace Example

This example, minimal-workspace, operates like a process-isolated web browser 
for documents (not primarily using web technologies, but similarly active). It 
serves as a proof-of-concept to resolve architectural issues or missing features 
in the IPC module before implementing the full workspace.

## Primary Components

1. **Workspace runtime loop** - The main coordinator process
2. **Runtime loops for individual documents** - Isolated processes for document 
   handling
3. **Leaf services:**
   - Singleton services owned by the workspace (network, storage)
   - Per-runtime services owned by the runtimes (renderer)

The "nested runtimes" represent security isolation boundaries between processes, 
similar to iframes in web documents. However, this separation is not strongly 
enforced yet.

## Key Architecture Points

### ChildIpcContext

The `ChildIpcContext` trait (defined in `ctb_utilities::ipc::service_traits`) 
provides an abstraction for child processes to communicate with the workspace:

- `request_spawn_runtime()` - Request spawning a sub-runtime process
- `request_spawn_renderer()` - Request spawning a renderer process
- `send_to_parent()` - Send text messages to the parent process

The `PeerChildIpcContext` (in `ctb_workspace_ipc::connection`) implements this 
trait using an `IpcPeer` connection, allowing runtime subprocesses to request 
spawns through the workspace.

### Service Traits

- `RuntimeClientTrait` - Abstract client for runtime service IPC calls
- `RendererClientTrait` - Abstract client for renderer service IPC calls
- `RenderSettings` / `RenderMode` / `RenderTarget` - Abstract DTOs for render 
  settings that don't depend on the renderer crate

## Intended Execution Flow

1. Workspace creates an IPC socket and starts listening.
2. Workspace spawns a network service singleton process, injecting dependencies 
   from the existing network module. That completes workspace boot.
3. Workspace spawns a runtime subprocess (R1).
4. Workspace sends an IPC request for data from the network module using the 
   network service's `echo` method. (note 1)
5. Workspace sends the above data to the runtime subprocess using shared memory 
   via the data plane IPC module in a call to `test_simple_nested_document`.
   (note 2)
6. Runtime R1 uses its `ChildIpcContext` to request a renderer and nested 
   runtime (R2) from the workspace.
7. The workspace accepts the request and starts the requested processes.
8. The runtime R1 calls `render_from_string(...)` on its renderer.
9. The runtime R1 calls `test_prepend(...)` on its sub-runtime R2.
10. R2 should pass its own response as a normal, non-shared-memory message back
    to R1.
11. R1 formats the messages into the finished "frame" and sends it back to the 
    workspace as a shared-memory data plane message.
12. Workspace prints out the rendered frame to demonstrate it works.
13. The runtime attempts to send a message directly to the network subprocess.
    - This message should be denied (capability restriction).
    - The workspace logs an ERROR about the denial.
14. The runtime R1 sends a "shut down requested" message to the workspace.
15. The workspace accepts and initiates graceful shutdown. The IPC process 
    manager will cleanly terminate the process tree, verify all processes 
    exited, and if any linger more than 30 seconds, kill them.

## Success Conditions

- All processes exited cleanly before 30 seconds.
- "Runtime input document: Hello from network module. Rendered: Hello from 
  network module. With subdocument: Prepend example 12345: Hello from network 
  module." was printed to stdout.
- An ERROR level tracing message was logged by the IPC router when it 
  intercepted and denied the unauthorized network request.
- The example process exited 0.

## Running the Example

Run as workspace (parent):
```bash
cargo run --example minimal-workspace
```

Run as runtime subprocess:
```bash
cargo run --example minimal-workspace -- --runtime
```

Run as nested subruntime subprocess:
```bash
cargo run --example minimal-workspace -- --subruntime
```

Run as network subprocess:
```bash
cargo run --example minimal-workspace -- --network
```

## Notes

(1) For normal document starts, shared memory might not be used, but this confirms it's possible to send shared memory from the workspace to a subprocess.

(2) The nested runtime and renderer subprocesses should be owned by the requesting runtime (R1). A renderer should not be able to request a subprocess be created owned by another process, only itself.
