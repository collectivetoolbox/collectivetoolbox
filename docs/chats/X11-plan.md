## Plan: Plain-Rust X11 Transition

Replace the current X11 C-library stack with a first-party pure-Rust backend, while preserving the `x11-dl` integration surface where that reduces downstream patching. The key design decision is: use `x11-dl` as the compatibility target, not the implementation substrate. A true all-Rust result cannot stop at replacing `x11`/`x11-dl`; it must also remove `xcb_ffi`/`tiny-xlib` usage and the direct `libxkbcommon` / `libxkbcommon-x11` FFI in the Linux/X11 path.

**Steps**
1. Phase 1: Freeze the compatibility target and scope boundaries.
2. Recommend `x11-dl` compatibility over `x11` compatibility for the public wrapper surface because upstream `winit` is already structured around `x11-dl`, and the local fork mostly swapped that surface to `x11` rather than changing the higher-level backend design.
3. Keep the goal explicitly split into two layers:
4. Layer A: a pure-Rust X11 client core responsible for connection setup, event delivery, selection/clipboard, IME plumbing, cursor handling, and exposing a pixel-presentable surface.
5. Layer B: a compatibility shim that preserves the `x11-dl` module layout and `Xlib::open()` / `Xcursor::open()` / `Xlib_xcb::open()` / `XInput2::open()` call sites for consumers that still expect that API shape.
6. Exclude from scope any X11 graphics stack beyond what is required to let existing Rust rendering code write pixels into a window.
7. Treat AT-SPI over D-Bus as acceptable for accessibility on Linux/X11.
8. Treat “no C libs at all” as a hard requirement, which means replacing `libxkbcommon` usage too, not just `libX11` / `libXi` / `libXcursor`.

9. Phase 2: Inventory the exact compatibility surface before implementation.
10. Build a symbol and type inventory from the current and upstream X11 backends, grouped into: protocol-backed functions, local-state-backed functions, and functions that should be dropped or deferred.
11. Use the current `winit` X11 backend as the primary consumer inventory and the upstream `winit` backend as the API-shape reference.
12. Do the same for `softbuffer` to separate “needs raw Xlib/XCB handles” from “can operate on a pure Rust connection abstraction.”
13. Produce a small RFC-style table for each function group covering: current caller, required behavior, replacement strategy, and whether the behavior is mandatory for installer parity.
14. Add a rule for provenance comments: when porting logic, callback choreography, or data layout from existing X11 crates or upstream `winit`, add a short source-reference comment pointing to the original file path.

15. Phase 3: Build the pure-Rust transport and protocol core first.
16. Do not attempt to make the current `x11-dl` dynamic loader work statically. Replace the loader internals with compiled Rust implementations that populate the same function-pointer structs with Rust trampolines.
17. Base the transport on a pure-Rust X11 protocol connection rather than `x11rb::xcb_ffi::XCBConnection`; either switch consumers to `x11rb`’s pure Rust connection path or introduce a first-party connection type that implements the minimum request/reply/event behavior the stack needs.
18. Keep the internal connection ABI-width-safe by using explicit X11 protocol integer sizes and serialization boundaries rather than relying on C struct layout or pointer casts; this is what keeps the implementation portable across i686, x86_64, armv7, and aarch64.
19. Implement the minimal windowing feature slice in this order: connection and screen discovery, atom internment, window creation/destruction, event loop wakeups, configure/focus/visibility events, close protocol, cursor updates, clipboard ownership and retrieval, then touch/XInput2.
20. Reuse existing pure-Rust protocol crates where they reduce code, but keep the first-party layer thin and opinionated around the subset your GUI stack needs.

21. Phase 4: Replace the Xlib-dependent `winit` X11 path.
22. Start from upstream `winit`’s `x11-dl`-based structure, not the current `x11`-based fork, and reintroduce upstream-style imports wherever possible so future rebases are narrower.
23. Refactor the backend so `x11-dl` compatibility objects are wrappers over the pure-Rust core instead of loaders for C symbols.
24. Replace the `XOpenDisplay` / `XGetXCBConnection` / `XDefaultScreen` startup path with a direct pure-Rust connection bootstrap.
25. Remove the dependency on `Xlib_xcb` interop by making the X11 backend own its protocol connection directly.
26. Keep the current IME, event, and cursor modules as behavioral references, but treat them as candidates for targeted rewrites because they currently assume Xlib-managed objects and callbacks.
27. Preserve the current X11 module structure where practical so rebasing against upstream remains a file-by-file exercise instead of a full backend fork.

28. Phase 5: Replace the non-Xlib C dependencies that block the all-Rust target.
29. Patch `softbuffer` so its X11 backend no longer depends on `tiny-xlib` or `x11rb`’s dlopen/XCB-FFI path; the backend should accept either a pure XCB/raw handle path or a small first-party display/window abstraction.
30. Replace the Linux/X11 keyboard stack that currently links directly to `xkbcommon` and `xkbcommon-x11` with a Rust keymap/state/compose pipeline.
31. Split keyboard support into sub-workstreams: XKB keymap acquisition from the X server, key state/modifier tracking, keysym-to-text conversion, and compose/dead-key handling.
32. Keep IME support separate from raw XKB translation: XKB covers keymap and compose mechanics; IME still requires XIM- or portal-equivalent behavior on X11.
33. Expect clipboard and IME to be the two highest-risk protocol areas after keyboard, because both require asynchronous ownership/state machinery rather than simple request/reply wrappers.

34. Phase 6: Decide the long-term compatibility boundary.
35. If the `x11-dl` compatibility shim can satisfy `winit` and any remaining consumers with a contained function subset, keep it as a local patched crate and move vendored dependencies back toward upstream feature wiring.
36. If the shim grows into a large Xlib reimplementation surface with little practical reuse, stop widening it and instead let the first-party X11 backend become the long-term internal API, with only a minimal `x11-dl` façade for crates that cannot yet be refactored.
37. In either case, do not invest further in the `x11` crate path; keep it only as a temporary historical reference while the migration proceeds.

38. Phase 7: Consolidate retained runtime resources.
39. Keep the XKB data tree; it remains necessary even after dropping the X11 C libraries because keyboard layout/compose resolution still needs the same data source.
40. Move the canonical XKB asset tree out of `built/assets/x11/xkb` into a stable source-controlled asset location shared across installer and future GUI code, with `assets/resources/linux/xkb` as the recommended canonical destination.
41. Update the build pipeline so `built/assets/...` remains a generated output, not the source of truth for XKB data.
42. Leave fonts, locales, and generic icons where they already live; they are not X11-C-library artifacts and do not need to move for this transition.
43. Do not plan on carrying repo-local cursor theme assets unless a later runtime gap proves they are needed; the current repo does not have a distinct X11 cursor resource tree to preserve.

**Relevant files**
- `~/ctoolbox/vendor/winit/Cargo.toml` — current local feature wiring that swapped upstream `x11-dl` usage to `x11`.
- `~/ctoolbox/vendor/winit/Cargo.toml.orig` — upstream dependency shape showing `x11-dl`, `x11rb`, and `xkbcommon-dl` as the intended integration surface.
- `~/ctoolbox/vendor/winit/src/platform_impl/linux/x11/ffi.rs` — current patched FFI import surface.
- `~/ctoolbox/vendor/upstream-for-reference/winit/src/platform_impl/linux/x11/ffi.rs` — upstream `x11-dl`-based FFI surface to restore toward.
- `~/ctoolbox/vendor/upstream-for-reference/winit/src/platform_impl/linux/x11/xdisplay.rs` — best reference for the current `Xlib::open()` / `Xcursor::open()` / `Xlib_xcb::open()` / `XInput2::open()` call pattern.
- `~/ctoolbox/vendor/winit/src/platform_impl/linux/x11/mod.rs` — central X11 event-loop/backend structure to preserve semantically while removing C-library assumptions.
- `~/ctoolbox/vendor/winit/src/platform_impl/linux/x11/ime/` — XIM behavior reference; high-risk rewrite zone.
- `~/ctoolbox/vendor/winit/src/platform_impl/linux/common/xkb/ffi.rs` — direct `xkbcommon` / `xkbcommon-x11` FFI that must be removed for an all-Rust solution.
- `~/ctoolbox/vendor/softbuffer/Cargo.toml` — current X11 feature split including `tiny-xlib` and `x11-dlopen`.
- `~/ctoolbox/vendor/softbuffer/src/backends/x11.rs` — X11 pixel transport path that already relies mostly on `x11rb`, but still bridges through Xlib for some handle cases.
- `~/ctoolbox/vendor/x11-dl/src/link.rs` — current dlopen-based loader design to replace with compiled-in Rust trampoline tables.
- `~/ctoolbox/vendor/x11/src/link.rs` — current static-extern alternative; useful as a contrast, but not the recommended long-term compatibility surface.
- `~/ctoolbox/vendor/x11/build.rs` — current vendored C-library build machinery that should become removable at the end of the migration.
- `~/ctoolbox/src/storage/minimal/xkb.rs` — current embedded-XKB extraction path and the immediate consumer of the retained XKB asset tree.
- `~/ctoolbox/built/assets/x11/xkb` — current generated XKB asset location to replace as the source of truth.

**Verification**
1. Produce a symbol inventory showing every `x11-dl` / `x11` / Xlib / XInput2 / Xcursor / Xlib_xcb call site still needed by `winit` and `softbuffer`, and mark each as implemented, deferred, or removed.
2. Ensure the X11 build graph no longer depends on `vendor/x11/build.rs`, `libX11`, `libXi`, `libXcursor`, `tiny-xlib`, `xcb_ffi`, `libxkbcommon`, or `libxkbcommon-x11` for the installer target.
3. Verify the installer and any other X11-enabled targets compile for at least GNU and musl on x86_64 and i686, plus armv7 and aarch64 where already supported by the workspace.
4. Run manual X11 smoke tests covering: window create/show/close, resize, focus changes, keyboard text entry, dead keys/compose, IME preedit/commit, pointer and wheel input, touch where XI2 is available, clipboard copy/paste including large payloads, and accessibility tree visibility via AT-SPI tooling.
5. Rebase-check the `winit` and `softbuffer` forks against their upstream references after the migration scaffolding lands; success means the remaining local patch set is materially smaller and centered on the pure-Rust backend rather than on forcing `x11` in place of `x11-dl`.

**Decisions**
- Recommended compatibility foundation: `x11-dl`, not `x11`.
- Recommended implementation foundation: a new pure-Rust X11/XKB backend layer, not a direct extension of either existing wrapper crate’s internals.
- Accessibility is in scope through AT-SPI over D-Bus, not through an X11-native accessibility protocol.
- Rendering stays out of scope beyond providing a window/surface path for existing Rust rendering code.
- Canonical retained data resource: the XKB tree currently generated under `~/ctoolbox/built/assets/x11/xkb`, recommended to move to `assets/resources/linux/xkb` as the hand-maintained source.

**Further Considerations**
1. The hardest feasibility question is not `x11` versus `x11-dl`; it is whether to preserve Xlib semantics for `winit` long enough to minimize patch delta, or to rewrite the backend more directly around a pure-Rust X11 connection and accept a temporary larger fork.
2. If you need a strict first milestone, make it “C-free window creation + events + clipboard on X11 with upstream-style `x11-dl` wiring restored,” then land keyboard/XKB and IME as the second milestone; that sequence de-risks the transport layer before the hardest text-input work.
3. If future full-project portability matters more than strict `winit` compatibility, consider letting the first-party X11 backend become the long-term abstraction boundary and treating the `x11-dl` shim as a migration aid rather than as a permanent public API.