There are *two* “loops” in play—

1) the IPC router/event loop (accept connections, route messages, service spawn/proxy requests), and  
2) the “application” loop (whatever your product does over time: UI, background jobs, state sync, etc).

Right now your runner effectively does (a) setup, (b) calls the workspace’s `main()`, and only *after that returns* does it start the IPC loop. That makes `main()` feel like “boot”, and it creates the surprising gotcha that a long-running loop inside `main()` prevents IPC from functioning (and can deadlock spawn/proxy).

## What’s the usual approach?
In most systems, the *event loop* is the thing that runs “forever”, and “app code” is expressed as:
- callbacks/handlers invoked by the loop, and/or
- tasks/futures scheduled onto the loop, and/or
- a top-level `run()` future that is awaited *while the loop is already active*.

In other words: **the event loop owns the lifetime**, and app logic runs “inside” it.

Tokio specifically is: you start a runtime, then you spawn tasks and/or await a “main future”; timers/IO wake tasks; there isn’t a single user-written `while {}` loop unless you’re doing something very specific.

## How browsers do it (high-level)
Browsers have an event loop per “agent” (roughly: a JS execution context) that pulls tasks from queues:
- input events, timers, network callbacks, etc (task queue)
- promise continuations (microtask queue)
- rendering/compositing is coordinated with its own schedule (often vsync-driven), not just JS tasks

JS feels sequential because your code runs to completion between events; the loop is managed for you and your code is “scheduled” via callbacks/promises.

Your instincts (“do this, then this, and in the background wait for X”) map very well to async Rust + tasks. The key is: don’t make *your* `main()` be “the loop”; make it *schedule* work onto the real loop.

## Recommendation: change the runner so IPC loop starts first
The most intuitive fix to the runner API is: **start the IPC event loop before calling the workspace’s long-lived function**, so there’s no “main can’t be a loop” gotcha.

A typical shape:

1. bind listener  
2. start singleton services  
3. start IPC routing loop in a background task (or as a concurrently-polled future)  
4. call workspace `run()` (long-lived)  
5. `select!` between:
   - workspace run finishes
   - shutdown requested (ctrl-c, UI asks to quit, parent message)
   - IPC loop fails (fatal error)
6. shutdown tree gracefully

This also eliminates the deadlock warning in minimal-workspace: you *can* await things that generate spawn/proxy requests because the IPC loop is already pumping them.

### Naming: don’t call it `main()`
Given this architecture, calling the workspace hook `main()` is misleading. Common alternatives:
- `run()` (best general-purpose)
- `app()` / `run_app()`
- `startup()` for the one-shot boot hook, plus `run()` for the long-lived part

So you end up with something like:
- `boot(rt)` = one-shot initialization (start UIs, warm caches)
- `run(rt)` = long-lived orchestration (spawn tasks, await shutdown)

## About input events and “will IPC be laggy?”
IPC over a local Unix socket with compact encoding (postcard) is usually fast enough for *commands* and *state updates*. Where it gets tricky is **high-frequency streams**:
- mouse move, scroll deltas, per-keystroke IME composition, animation ticks, etc.

For those, the usual approach is:
- keep the tight UI event loop *in the UI process* (GUI, TUI, web UI)
- send *semantic events* or *coalesced updates* over IPC (e.g., “cursor moved to X,Y”, not every raw motion)
- for rendering pipelines, send “frame models” or “diffs”, not per-input interrupts

So: yes, passing “every event” through IPC can be wasteful, but you don’t need to. Treat IPC as the boundary between subsystems, not as a wire for raw device interrupts.

## A mental model that fits your multi-interface goal
Given “desktop graph database groupware + web UI + GUI + interactive CLI”:

- Workspace process = authoritative state + orchestration + persistence + sync
- Each UI (web, GUI, TUI) = a client:
  - subscribes to state / queries snapshots
  - sends user intents (commands) to the workspace
- Renderer/runtime/io/etc = specialized services (possibly separate processes)

That’s essentially “browser-like”: the UI loop is local, and it talks to a “backend” over a messaging boundary.

## Concrete runner API tweak (minimal, intuitive)
If you want a change that removes surprises without overdesign:
- Rename `Workspace::main` → `Workspace::run`
- Change `WorkspaceRunner::run()` to start its internal IPC/event loop before awaiting `workspace.run(rt).await`
- Ensure the runner shuts down when `workspace.run()` returns (or when shutdown is requested)

That preserves your current trait design, but makes the semantics match the name: `run()` is allowed to be long-lived.

If you want, I can propose a small patch to the runner that does exactly this (start IPC loop first, then call the workspace hook), using `tokio::select!` for clean shutdown and without introducing new concepts.