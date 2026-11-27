User: I'm trying to build out an example (minimal-workspace) that operates essentially along the lines of a process-isolated web browser (it's not for web documents, but is meant to operate similarly). It's meant to be a proof of concept example as a stepping stone to iron out any architectural issues or missing features in the IPC module and architecture before implementing the real workspace.

As the minimal-workspace example comes into shape, though, I'm realizing I have a bunch of conceptual oversights, design flaws, and other issues to iron out:

1.  I've built the example workspace around "renderer" processes that are nestable. But I've combined the concepts of a runtime, if that's the right word (event loop and logic handling) with the actual rendering (take the current state of this document and create a text/raster/whatever representation of it for presentation to the user).
2. The IPC model is based on service crates that live outside of the IPC crate, with thin IPC interfaces to them within the IPC crate. However, since the "real" implementations don't depend on the IPC crate, it's not obvious how they can initiate IPC calls when they need to.
3. The "nested renderers" (which should probably be "nested runtimes, each with their own renderer process or thread") are meant to be security isolation boundaries between processes: they represent nested documents, so basically iframes. However, I haven't really taken that separation into account at all and I'm not sure what it would entail.
4. The "data plane" shared memory IPC is using a temp file, rather than real file descriptor passing end-to-end using SCM_RIGHTS so Unix can use `memfd`.
5. The thin IPC service layers are extremely verbose and boilerplate-y, so it needs tens of lines across several files to add a new method to any service.

Please develop a step-by-step workable plan I can iterate on to get this project closer to my end goal. Please include steps written in a way that are suitable to be used for LLM prompting. Thank you!




## Plan: Multi-Process IPC Architecture Refactoring

Refactor the minimal-workspace example to separate runtime from rendering concerns, implement proper security isolation for nested processes, complete file descriptor passing for shared memory, reduce service boilerplate through code generation, and enable service modules to initiate IPC without tight coupling.

### Steps

1. **Rename and restructure renderer → runtime separation**:  Create a new stub crate at `ctoolbox/src/runtime`, and move the `start` method from the exsiting `renderer` crate into it, updating references. Create a new `RuntimeService` in ctoolbox/src/workspace/services/runtime/api.rs. Update most existing references to `RendererService` to `RuntimeService` in ctoolbox/src/workspace/services/renderer/api.rs. Update ctoolbox/examples/minimal-workspace/renderer.rs to separate document runtime logic (event loop, IPC orchestration) from actual rendering methods (create visual output). Move rendering-specific methods to a new `RendererService` that the runtime can call, backed by the existing `renderer` crate. Update all references throughout the IPC module and minimal-workspace example.

2. **Implement SCM_RIGHTS FD passing in transport layer**: Extend ctoolbox/src/workspace/ipc/transport/framed.rs to support ancillary data (file descriptors) alongside message frames. Add `send_with_fds()` and `recv_with_fds()` methods using the existing Unix FD operations from unix.rs. Update shared_memory.rs `BlobAllocator` to use `create_memfd()` and pass FDs via SCM_RIGHTS instead of creating temp files. Wire FD transfer through session layer in ctoolbox/src/workspace/ipc/session.rs so blob allocation includes FD handoff.

3. **Design dependency injection interface for service modules**: Create a new `IpcContext` trait in the IPC module that provides abstract methods like `spawn_child_runtime()`, `send_to_parent()`, `allocate_shared_memory()` without exposing concrete IPC types. Implement `IpcContext` for `WorkspaceRuntime` in ctoolbox/src/workspace/workspace_runner/workspace_runtime.rs. Update external service modules (renderer.rs) to accept `&dyn IpcContext` parameters instead of `&WorkspaceRuntime`, breaking the circular dependency. Services can then initiate IPC operations through the interface without depending on the IPC crate.

4. **Implement nested runtime isolation model**: Design a capability inheritance system where nested runtimes receive a restricted subset of parent capabilities (similar to iframe sandbox attributes). Add `isolation_policy` field to `SpawnChildRequest` in ctoolbox/src/workspace/services/process/api.rs with options like `Inherit`, `Restricted(Vec<ServiceName>)`, `Isolated`. Update ctoolbox/src/workspace/workspace_runner/workspace.rs to enforce parent-child ownership validation server-side. Add direct parent-child IPC channels (bypassing workspace) for performance, while keeping workspace as capability authority. Document security boundaries and isolation guarantees.

5. **Reduce service boilerplate with proc-macro codegen**: Create a new `ctb_ipc_macros` crate with a `#[ipc_service]` proc-macro that generates dispatcher match arms, client wrapper methods, and router integration from service trait definitions. The macro should parse `#[async_trait] trait XService` definitions in files like ctoolbox/src/workspace/services/network/api.rs and auto-generate the dispatcher in `dispatch_network()`, client methods in `NetworkClient`, and request/response DTO boilerplate. Apply to all services in ctoolbox/src/workspace/services/ directory, reducing the ~18 code blocks per 3-method service to just the trait definition and implementation.

6. **Update minimal-workspace example with all improvements**: Refactor ctoolbox/examples/minimal-workspace/ to demonstrate: (a) Runtime spawning a sub-runtime with restricted capabilities showing isolation enforcement, (b) Shared memory transfer using memfd + SCM_RIGHTS, (c) Service modules using `IpcContext` interface to spawn children and communicate, (d) Clear separation between runtime (subprocess orchestration) and rendering (visual output generation). Update ctoolbox/examples/minimal-workspace/execution-flow.md documentation to reflect architectural improvements.

### Further Considerations

1. **Architecture validation approach**: Should the refactoring proceed sequentially (step 1 → 2 → 3...) allowing each change to be tested in isolation, or should steps 1-3 be prototyped in parallel on separate branches since they have minimal dependencies? Sequential is safer but slower; parallel finds integration issues earlier.

2. **Proc-macro scope and flexibility**: Should the `#[ipc_service]` macro (step 5) generate all boilerplate automatically with minimal configuration, or should it offer customization points for special cases like streaming responses, bidirectional channels, or custom serialization? Full automation is simpler but may need escape hatches.

3. **Security isolation depth**: For nested runtime isolation (step 4), should the implementation include OS-level sandboxing (seccomp-bpf filters, separate namespaces), or focus purely on capability-based access control at the IPC layer for now? OS sandboxing is more secure but significantly more complex and platform-specific.
