## Review: `workspace/ipc` and `utilities/ipc`

### 🔴 Duplicate Code to Address

**1. `MockProcessService` is defined twice:**
- process.rs (inside `#[cfg(test)]`)
- api.rs (unconditional)

The one in api.rs is always compiled. The one in process.rs is test-only. They're identical. **Recommendation:** Remove the duplicate from process.rs and re-export or use the one from api.rs in tests.

---

### 🟡 Unused / Partially-Used Infrastructure

**2. `IPC_API` static in ipc.rs**
This `LazyLock<Vec<&str>>` listing services appears unused within the new IPC infrastructure. The registry-based dispatch (`#[ipc_method]` + `inventory`) has replaced string-based method lookup. **Recommendation:** If this is only used for documentation or the old IPC system, consider removing or moving to a doc comment.

**3. `StreamManager` trait and `StreamControl` infrastructure**
The streaming infrastructure (client.rs, `StreamControl` in protocol.rs) appears to be scaffolded but not used beyond tests and the peer dispatch loop where streaming messages are essentially ignored:
```rust
Message::Stream(_) => {} // peer.rs:144
```
This is a forward-looking feature, but if it's not being used yet, consider marking it clearly with a `// TODO: wire up streaming` comment.

**4. `data_plane/` folder is empty**
The data_plane/ folder exists but is empty. The data_plane.rs file exists alongside it with actual code. This is fine structurally if you plan to add more modules, but the empty folder could be removed for now.

**5. `platform/` folder is empty**
Similar to above - platform/ is empty while `process_manager/unix.rs` and windows.rs handle platform specifics. Consider removing the empty folder.

---

### 🟡 DRY Opportunities

**6. Repeated postcard encode/decode + error mapping pattern**
The codebase has ~45 instances of `postcard::to_stdvec` / `postcard::from_bytes` with similar error mapping. While the router has `decode_request` and `handle_service_call` helpers, client-side code in peer_clients.rs and process.rs repeats similar patterns.

Consider a small helper module like:
```rust
pub fn encode<T: Serialize>(val: &T, context: &str) -> Result<Vec<u8>> { ... }
pub fn decode<T: DeserializeOwned>(bytes: &[u8], context: &str) -> Result<T> { ... }
```

**7. `dispatch` vs `dispatch_with_session` in router**
router.rs and router.rs share significant logic. The main difference is `dispatch_with_session` provides an FD-aware context. Consider refactoring to have `dispatch` call `dispatch_with_session` with a no-op context, rather than duplicating the authorization and dispatch flow.

**8. Similar `PeerChildIpcContext` and `DeferredChildIpcContext` implementations**
The `ChildIpcContext` impl for `DeferredChildIpcContext` in connection.rs simply delegates each method to `self.ctx()?.method(...)`. This is boilerplate-heavy. A macro could generate these delegations, though the current approach is readable.

---

### 🟡 Outdated / Legacy Code (Outside Scope but Worth Noting)

**9. ipc.rs contains old `Channel` struct**
The `Channel` struct with `port`, `name`, and `authentication_key` in ipc.rs is only used by the old IPC system (`ipc_old/`, workspace.rs, process.rs). Once you fully migrate, this can be removed along with `IpcEndpoint`, `channel_from_args_and_key`, etc.

---

### ✅ Things That Look Good

- **Capability system**: Clean separation between `CapabilitySet`, `CapabilityToken`, `TokenValidator`.
- **Router design**: The `IpcRouter` + `Router` trait is well-factored.
- **Session abstraction**: `FramedSession<T>` properly layers over `FramedConnection`.
- **FD passing**: Properly implements `SCM_RIGHTS` with async recv.
- **Error handling**: Consistent use of `anyhow`, no panics.
- **Registry-based dispatch**: The `#[ipc_method]` + inventory pattern is clean.

---

### 🔵 Minor Notes

**10. FIXME in transport.rs**
There's a comment about not being able to figure out how to wait for EOF properly. This is low-priority but worth revisiting if tests become flaky.

**11. Test-only `MockProcessManager` in process.rs**
The test module imports `MockProcessService` from the same module, but there's also a `MockProcessManager` in process_manager.rs. These are different types (service vs manager), so no duplication - just similar naming.

---

### Summary of Recommended Actions

| Priority | Item | Action |
|----------|------|--------|
| High | Duplicate `MockProcessService` | Remove from process.rs, use the one in api.rs |
| Medium | `IPC_API` static | Remove if unused, or add `#[allow(dead_code)]` with comment |
| Medium | Empty `data_plane/`, `platform/` folders | Delete empty folders |
| Low | DRY postcard helpers | Extract helper functions if refactoring |
| Low | `dispatch` duplication | Refactor to avoid duplicated auth/dispatch logic |
| Deferred | ipc.rs Channel | Remove after old IPC is fully replaced |
