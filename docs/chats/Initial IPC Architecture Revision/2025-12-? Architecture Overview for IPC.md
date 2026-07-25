Architecture Overview:

- The application typically runs from a single binary. That process I call the "workspace".
- The workspace then allows the user to open documents, as "renderer" processes (document runtime).
- Each "renderer" process, and the root workspace process, should be able to run certain code: file format conversions, i/o, network access, etc. depending on the permissions the document loaded into the renderer has been granted.
  - My thinking is that those features should be handled by the renderer creating additional subprocesses, roughly one subprocess for each Rust module that it needs to run code from (there aren't strict, clear boundaries between the modules, though - for instance, the UI might call some APIs from storage to load icons from disk - possibly this represents a design flaw).
- Renderers can start other renderers, representing documents embedded within other documents.
- Renderers, and the workspace, should be able to make function calls to the subprocesses they spawn, as well as to the workspace (and embedded documents should be able to pass limited messages back to their embedder).
- If a process panics or is killed, it should take responsibility for killing its own subprocesses (perhaps recursively).
- If a renderer process panics, it should only bring down that document or subdocument, not the whole workspace or other documents.
- Basically I'm trying to imitate how web browsers isolate documents from each other so they can't crash the browser or other documents.
- IPC uses a thin design with abstract services; the real implementations are in other crates outside of the IPC crate.

The current goal is to create:

### End-to-End Example and Documentation

  Edit the following files to create examples demonstrating thin IPC integration with existing modules, which should compile to a single binary which can be run either with no command-line flags or with one of three that it will use internally, described as follows:
  - `examples/minimal_workspace/main.rs` (new file: workspace setup, spawning processes, injecting module dependencies)
  - `examples/minimal_workspace/renderer.rs` (new file for a renderer or sub-renderer process instance, containing the code path that should be run when `--renderer` or `--sub-renderer` passed: renderer handshake, heartbeat, calling thin IPC methods that delegate to existing modules)
  - `examples/minimal_workspace/network.rs` (new file for a network service instance, shared across the workspace and all renderers, containing the code path that should be run when `--network` passed: listens for IPC messages and responds with a fixed example string when requested)

  If the necessary features aren't implemented suitably in the ipc crate, it may need other changes.

  Create examples:
  - Workspace spawns a network service process and a renderer process, injecting dependencies from the existing modules (e.g., `io`, `network`).
  - Renderer uses IPC to call thin methods (e.g., read_file via `network` module), and test as follows:
    - Receive data to render from workspace
    - Sends a message back to the workspace with the data as a data frame; workspace then prints the "rendered" (simply echoed for now) frame out to demonstrate it works
    - Renderer sends a normal, non-data-frame message to the workspace to start a subprocess (owned by the renderer - a renderer should not be able to request a subprocess be created owned by another process, only itself).
    - Workspace accepts the request and starts a nested sub-renderer.
    - Sub-renderer sends a message, like "Hello from sub-renderer", supposedly as a "rendered" string, to the enclosing renderer, which then passes it to the workspace, again to be printed to demonstrate the example is working.
    - Renderer attempts to send a message directly to the network subprocess. It should be denied as the renderer IPC service should not be given the relevant capability permission to interact with the network IPC service, and the renderer should print something to indicate the denial.
    - Renderer sends a "shut down requested" message to the workspace.
    - Workspace accepts the request and shuts down, cleanly terminating the process tree.
- **Acceptance criteria**:
  - Example runs cleanly, delegating to existing modules.

Please only edit files within the `src/workspace/ipc` crate. Ignore the outdated `src/workspace/ipc_old` crate.