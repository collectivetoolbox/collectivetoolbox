# AGENTS.md

## Dev Environment & Workflow

### Setup, Builds, and Testing
- **Fresh dev container setup**: `scripts/apt-install-build-deps; scripts/apt-install-dev-tools`
- **Build & test scope**: Prefer building or testing individual workspace crates (e.g., `cargo test -p ctb-renderer`); a full build takes 5–10 minutes.
- **Linux target**: Always use the `musl` build in the dev container. Do not install extra host packages for a `glibc` build.
- **Disk space**: Keep at least 25 GB of disk space free (the filesystem slows down sharply below that and can lock up the computer). A full single-target build consumes most available space, so run `cargo clean` when switching targets.
- **Asset bundle test failures**: If `get_test_help_bytes` or other asset-loading tests fail, inspect the failure first. When stale or missing build assets are confirmed, run `cargo clean` and re-test, but never clean while other builds or tests are running.
- **No network ports in tests**: Tests must never listen on ports; use mocks or in-memory/socketpair transports instead.
- **Front-end workflow**: Skip `cargo check`, `cargo test`, and `./test` for front-end-only changes. Use `./lint --quick` to check JS/TS types.
- **Lingering server processes**: Run `killall ctoolbox` (or similar) if needed to stop background server processes.
- **Antigravity chat folders**: Never run temporary Cargo builds in Antigravity chat folders; all new files there are indexed as "Artifacts" and lock up the IDE.
- **Clippy errors** are important to resolve. **Clippy warnings** have proven impossible to turn off for CLI compilation without breaking their display in rust-analyzer; their output is mostly noise and may be disregarded.
- **Sub-agents** - if operator has specifically requested that you use sub-agents for a task, **never** report back to operator unless the task is fully and correctly completed. If the sub-agents did not finish the work, identify remaining tasks and create new sub-agents to finish it. Monitor their work periodically (or use a second lightweight agent to) to identify cases where they try to reinvent duplicative code rather than using, extending, or refactoring existing implementations - code must **strictly** be kept DRY.
- Tests are usually run using cargo. For testing that can't be done on the host, you can use the ci-logs/vm script. Helpful options include "screenshot", "screentext" (screenshot + OCR), and "serve" if you can't read one directly.

### Dependencies & `vendor/`
- **`vendor/ctb-vendored/`**: Auto-updated from crates.io. May be referenced, never edited.
- **`vendor/upstream-for-reference/`**: For reference and generating diffs only; never update, use, or compile these files. Active patched copies live in the enclosing `vendor/` folder.
- **Other `vendor/` folders**: Limited patches are acceptable when necessary.
- **Adding vs. updating dependencies**: Adding new dependencies not already in the repository is discouraged unless specifically requested by your operator.
  - You may propose new dependencies, but must ask before adding them (CI runs offline; the operator vendors changed dependencies before pushing).
  - Version bumps of existing dependencies—especially when `cargo-deny` reports a security advisory—are encouraged and do not require approval.
  - If you want to run a local build offline for testing (either offline or online is acceptable for general development), but the vendored dependencies folder is outdated, stop and report back to the operator. Do **not** revert dependency changes to force it to run anyway.

### Collaboration, Git, and Sub-Agents
- **Inspecting recent changes**: Record the starting commit with `git rev-parse HEAD`. Review committed changes with `git diff <start>..HEAD`, staged/unstaged changes with `git diff HEAD`, and new files with `git status --short`. The operator commits and pushes during work, so neither a working-tree diff nor `origin..HEAD` alone is sufficient.
- **Live operator edits**: If edits from the operator appear while you are working on files, do not revert them—re-read the files and align with the direction of those changes.
- **Scope confirmation**: If a seemingly small request would require large-scale changes or significant refactoring, ask to confirm before editing, or use `/plan` mode.
- **Clear communication**: When communicating to the operator, make sure to include `ctoolbox character_description` output for referenced Dcs. Use this output to make sure what you're asking/proposing actually makes sense.
- **Moving a module into a subcrate**: Keep this mechanical: move the files using a tool/script (do not regenerate them), copy and trim `Cargo.toml`, update the package name, replace `mod` with `use`, and register the crate in the root `Cargo.toml`. Ask before doing anything beyond this.
- **Repository memory**: Write important long-term architectural or debugging notes to the `memories/` folder in the repository; these are reviewed and versioned as part of the codebase.
- **Sub-agent models**: Never use models provided by Anthropic (e.g., when spawning sub-agents). `GPT-6.1 Sol` with short context is a good option, or `GPT-6 Luna` for lightweight work.
- **Context window & compaction survival** (this rule is for Antigravity only, not for Copilot): In Antigravity, the harness compacts context abruptly and without warning, and you have **no** visibility into your current token count or context limit. For any multi-file review, audit, planning task, implementation, or refactor spanning more than ~3,000 lines or 6 files, you **must** checkpoint incrementally as you go:
  1. **Create a handoff artifact up front**: Before or immediately after your first batch of reads, create a `handoff-notes.md` artifact in the current conversation folder (not user facing; do not request feedback).
  2. **Cap read batches and append findings every batch**: Read at most ~3,200–4,800 lines (4–6 full `view_file` calls) per step. After each batch—before or alongside issuing the next batch of `view_file` calls—update `handoff-notes.md` with: (a) exact files and line ranges fully inspected, (b) every concrete finding/bug with exact `file:line` references, (c) the remaining queue of files/line ranges, and (d) findings and relevant notes from operator clarifications and AGENTS.md.
  3. **Recover on compaction**: Before resuming after any context reset/compaction, read `handoff-notes.md` before doing anything else so no findings or line-range tracking are lost across compaction.
  4. **Trim or summarize** completed/resolved tasks if it's getting long, to maintain focus on current work.

## Reporting, Status Tracking, and Scope Honesty
- **Lead responses with what is left, broken, or deferred**:
  - Do NOT write cheerful "all done!" recaps. Prefer reporting failures over successes (twenty passing tests matter far less than the one failing test that still needs work). Start every summary with:
    1. **Failing / Skipped Tests or Lints** (if any). Pausing on a difficult test and reporting it is normal; leaving tests skipped, ignored, or failing without highlighting them in the headline summary is unacceptable.
    2. **Known Gaps & What's Left** (unimplemented parts, missing formats/flags, or edge cases remaining for full parity).
    3. **Deferred Items** (what was deliberately omitted and why).
  - Summarize completed work only as secondary context below the gaps.
- **Never label partial work as "Implemented" or "Complete"**:
  - When porting other software, if a feature has even one unhandled upstream case (e.g., missing flag, partial header parsing, or deferred prerequisite), its status MUST be marked as `Partially Implemented` or `In Progress`, NEVER `Implemented` or `Complete`.
  - In tracking documents (such as `issues/*-status.md`), any status update MUST explicitly maintain a `Remaining Gaps vs Upstream` (or `Pending Work`) subsection listing exactly what is still missing. Never erase or gloss over gaps when recording new progress.
- **Differentiate sub-task completion from feature completion**:
  - If asked to implement a bounded subset of a feature (e.g., "implement the ones available so far"), frame your conclusion as partial progress toward the whole, not as the feature being finished.
- **Treat optimistic misrepresentation as a bug**:
  - Calling something "complete" or "working" when known edge cases or formats were skipped is a critical accuracy failure. When in doubt, err on the side of skepticism about your own implementation.
  - *Rationale*: Completed work can be seen directly in the code and tests and crossed off the list. An honest accounting of *what remains* is critical so context is not lost when moving to subsequent stages of implementation.

## Standards and Style Guide

### Error Handling & Invariants
- **Judge edge-case error handling carefully (the #1 agent mistake)**: Every case needs judgment, not a mechanical fix; passing tests on ordinary inputs can miss rare failures.
  1. **Never silently swallow or discard legitimate edge-case errors** instead of propagating them to callers. Prefer a returned error over a crash, and a crash over silently incorrect results; recoverable errors must return `Result`.
  2. Less critical: **Do not propagate unreachable error states in otherwise-infallible functions**, which creates clunky APIs.
- **Fail early and loudly; never panic in production**:
  - Fallible operations must return `anyhow::Result<T>` (`Result<T, anyhow::Error>`) and propagate errors via `?` or `let Some(val) = val else { return Err(...); };`.
  - Use `anyhow::ensure!` (re-exported as `ensure!` by the prelude) for fallible validation: it returns `Err`, not a panic.
  - Never introduce panicking `unwrap`, `expect`, indexing, or unchecked arithmetic in new production code.
  - Reserve `assert!()`, `unreachable!()`, and `.expect()` (with a Clippy allow/expect reason) strictly for provably infallible operations (such as bitmasks `x & 0x3F` or range-checked bounds) or genuinely unrecoverable startup/installer scenarios—never in a function that returns `Result`.
  - Never use `unwrap_or(0)` or similar fallbacks for infallible operations, as doing so obscures intent and hides broken assumptions.
  - `unwrap_or` (and similar fallbacks) is acceptable only when the fallback behavior is genuine domain logic clearly documented in the function contract, and **requires a comment** explaining *why* it is an acceptable fallback that will not mask a true error. Never "cheat" the `unwrap_or` linter by rewriting an undocumented `.unwrap_or(...)` into an `if let Some` / `else` block.
- **Control flow**: Prefer `if let Some(...) = ...` over callback chains like `.map(...).unwrap_or_else(...)`, unless callbacks are substantially less verbose.

### Rust Idioms, Lints, and Testing
- **No `as` casts**: `as` is strictly disallowed by the linter. Use `try_from` / `from`, or `utilities::math` for fallible exact and approximate float/int conversions.
- **Checked arithmetic & safe indexing**: Clippy denies infix math operators (`+`, `-`, `*`, `/`, `%`) and panicking slice/index operations. Use checked or saturating arithmetic methods and safe indexing (`.get(...)`).
- **Addressing lints**: Fix the underlying cause of lint errors rather than suppressing them. Never place `#[allow(...)]` or `#![allow(...)]` at the top level of a crate or file; attach overrides to the specific method/item where relevant.
- **Lint bypass comments**: Comments and `reason` strings on `expect`, `unwrap_or`, or `allow` must explain *why* the lint's concern does not apply in that specific case—never merely restate *what* the code does.
- **Standard module preludes**:
  - Main crate files (other than `ctb_utilities`) start with:
    ```rust
    #[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
    pub(crate) use ctb_utilities::*;
    ```
  - Submodules start with:
    ```rust
    #[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
    use crate::utilities::*;
    ```
  - Do not remove these preludes. They import `anyhow::{self, Result, Context, bail}`, `tracing`, and the workspace logging macros (`log!`, `log_fmt!`, `warn!`, `warn_fmt!`, etc.). Always use those macros rather than calling `tracing` directly.
- **Tests**:
  - Always use `#[crate::ctb_test]` or `#[crate::ctb_test("tokio")]`—never `#[test]` or `#[tokio::test]` (except when patching vendor code).
  - Any important or nontrivial code must have test coverage.
  - Never add `#[ignore]` or conditional ignore attributes to bypass failing, broken, or inconvenient tests; fix them or report the blocker. New intentional exclusions require operator approval. Preserve the approved exception: `test_bootstrap_ts_compilation`, the very slow full-from-source TypeScript bootstrap test, remains ignored in routine suites.
  - Disregard Clippy warnings about panics/unwrapping in tests when avoiding them would make the test more complex; the standard test boilerplate allows them.
- **Docblocks & comments**:
  - Wrap docblocks at 80 columns (`//!` for modules, `///` for functions/items).
  - Do not wrap proper nouns or tool names (e.g., MathML, StageL) in backticks just to satisfy Clippy; add them to `clippy.toml` instead. Use backticks only for actual code and identifiers.
  - Write self-explanatory code; add comments only where logic is non-obvious.
  - Never alter or remove existing comments, docblocks, `TODO`s, or `FIXME`s unless specifically invalidated or resolved by your edits.

### Code Design, Security, and Licensing
- **General design**: Avoid statefulness where possible. Prefer sync code; use async only when needed. Use the newtype pattern whenever it reduces confusion.
- **Reuse & refactoring**: Reuse existing utilities (especially `ctb_utilities`, such as `utilities::https` for HTTP requests). Refactor or extend existing APIs rather than adding duplicate helpers. When refactoring, do not leave messy or outdated APIs out of concern for callers—clean up the API and update all callers.
- **Canonical constants**: Keep all constants DRY; do not add aliases or copy them into other modules.
- **Module structure & cleanup**: Keep modules concise and narrowly scoped; split growing modules into submodules. If you try multiple approaches during troubleshooting, remove all failed attempts and dead code before finishing.
- **Single canonical naming**: When designing a new format or CLI interface, pick the single best name or syntax rather than supporting multiple aliases (e.g., do not support both `"case:"` and `"case-sensitive:"` tags, `"u"` and `"U"` prefixes for Unicode characters, or duplicate CLI commands).
- **CLI & configuration**: Avoid environment variables unless genuinely necessary; prefer CLI flags. Parse CLI and subprocess arguments into `Invocation`. The `ctb-cli` crate (`src/cli/`) must contain zero nontrivial logic beyond parsing and dispatching to the target crate. Crates supporting CLI commands will typically have a `cli` module with the argument parsing.
- **Production readiness**: Address root issues and missing features rather than adding workarounds:
  - Always use file-descriptor passing for shared memory on Linux, never temp files.
  - Never add in-process fallbacks for IPC calls.
- **IPC**: Annotate IPC methods with `#[ipc_method]` and DTO structs with `#[ipc_dto]`. Call autogenerated bindings via `ipc!(service).method()` (async) or `ipcb!(service).method()` (blocking) without extra imports beyond the prelude (`service` is e.g. `network` or `storage`, or omitted for workspace methods).
- **Security & HTTP**: Never pass secrets as CLI arguments; zeroize secrets and minimize their lifetime and cloning in memory. Avoid returning HTTP 5xx errors if possible, as CDNs may discard their response bodies.
- **Shell scripts**: Never use `rm -f`, `cp -f`, or similar force flags; check and handle failure cases explicitly.
- **Licensing & attribution**:
  - Never delete license blocks or copyright headers. If you reuse algorithms or code from another file, include a comment noting the source.
  - All dependencies (including build-time dependencies, linked libraries, and sysroots) MUST be under AGPL-3+-compatible licenses.
  - Pre-compiled binaries are prohibited, with rare exceptions: the `v86` VM and `tsc` compiler precompiled in the CI Docker image, and blobs not yet filtered out from `vendor/ctb-vendored`.
- **No Node.js dependencies**: Never install Node.js packages in the application; a `node_modules` directory must never exist. External dev-only tools (`tsc` for fast linter runs, Playwright for browser testing) are permitted outside the repository, but must never be application dependencies or required for correct linting.

### Front-End, HTML, and CSS
- **Design system**: Follow and update [docs/design-system.md](docs/design-system.md) when adding UI components.
- **Semantic HTML**: Use proper semantic tags (`<h2>`, `<h3>`, `<button>`) and ensure high-contrast, accessible text styling. Never use faux-headers (e.g., `<span>` with utility classes) or redundant classes (use `<button>`, not `<button class="button">`).
- **Stylesheets over utility classes**: Never add non-layout Tailwind/EncreCSS utility styles (font size, weight, letter spacing, or colors like `text-xs`, `font-bold`, `text-gray-500`, `tracking-wider`) to templates; rely on the project-wide stylesheet. Most templates should only need widget positioning CSS unless introducing a new themeable widget component.
- **Component styles**: Do not create ad-hoc component styling; add missing shared concepts (e.g., `.btn-secondary`, `.btn-secondary.selected`) to the appropriate component CSS file.
- **Assets & units**: Use icon assets instead of inline SVG where available. Avoid inline JavaScript or CSS in templates. Prefer `rem` units over `px` or `em`. Avoid `!important` in CSS; fix selector precedence instead.

### Document Characters (Dc), `FormatId`, and Codegen
- **Generated files**: Must have `.generated` in the filename so CI recreates them. Any hardcoded list of file formats or similar metadata not generated from the CSV database in `src/formats/dcdata/data/` is likely a mistake.
- **Dc constants**: Must only be defined in `ctb_formats_dcdata::dc` (`src/formats/dcdata/dc.generated.rs`, generated from `@ident(...)` annotations in the category CSVs)—never defined, aliased, or copied in other files. They must be `DcChar`, never bare integers. Do not add named Dc constants merely for struct/enum annotations; use inline shorthand in attributes (`123` = short Dc 123; `l123` = long Dc 123; `u1a3` = U+01A3; `f123` = format Dc 123) and only add named constants when referenced in procedural code.
- **Dc semantic classification**: Never classify Dcs with numeric allowlists or direct constant comparisons when equivalence or syntax semantics matter. Generate the classification from the Dc database and use the shared resolver API, including `<equiv>` relationships; adding an equivalent Dc must not require editing procedural decoders. The Dc constants are useful for writing documents, but not for reading them.
- **No "short format ID"**: Format IDs must never be exposed or processed as bare integers (which are indistinguishable from short Dc IDs). Always use the strongly typed `FormatId` enum or `"f<N>"` shorthand (`FormatId::shorthand()`, `FormatId::from_shorthand("f34")`). Never add integer conversion APIs like `short_id()` or `from_short_id()` to `FormatId`.
- **Generalized Dc serialization**: Dc serialization/deserialization must be implemented generically via derive attributes, never special-cased or ad-hoc for individual structs. If the attributes are insufficient, extend them in generally applicable, non-struct-specific ways.
- **Ontology & approval**:
  - Proposals for new Dcs require operator approval.
  - Dcs represent semantic/ontological entities, never Rust fields or serialization structures. Multiple encodings of the same value belong under its existing semantic identifier using Dc 397. Equivalent representations must denote the same value; distinct values or provenance roles must not be presented as equivalent.
  - Path values must follow the declared typed filepath syntax rather than substituting an undelimited string or opaque byte buffer.
  - Dc IDs and semantic meaning are stable and cannot be changed or reassigned. Dc names and syntax are not stable at this time.
- **Strings**: Reach for `DcMstr` / `DcMstring` by default for almost all Dc string use cases. Avoid converting to Rust `String`.

## Architecture & Workspace Layout
- **Multi-process architecture**: The main `ctb-workspace` (`src/workspace/`) process manages application state and spawns subprocesses (`ctb-renderer` in `src/renderer/`, `ctb-io-webui` in `src/io/webui/`, `ctb-storage` in `src/storage/`, etc.) over socket-based IPC (`ctb_utilities`).
- **Core stack**:
  - Async runtime: `tokio`
  - Web UI: `axum` serving HTML/JS embedded from `assets/` via `include_dir` (copied to `built/assets/` during build; opened via `webbrowser`)
  - Storage: Turso database (`src/storage/db.rs`)
  - Networking between client and server: `utilities/https`
  - Networking between clients; global graph sync: TBD
  - Secrets: `zeroize`
- **Crate & module layout**:
  - All workspace sub-crates live under `src/` (nested multiple levels deep) and must be listed in the root `Cargo.toml`.
  - Cargo package names use `ctb-*` (kebab-case) and Rust crate identifiers use `ctb_*` (snake_case). Each sub-crate's entry `.rs` file uses the crate name minus the `ctb_` prefix (e.g., `ctb_renderer` is in `src/renderer/renderer.rs`, `ctb_storage` is in `src/storage/storage.rs`).
  - Modules use `<module>.rs` alongside a `<module>/` directory for submodules (e.g., `src/storage/user.rs` and `src/storage/user/*.rs`).
- **Linux static binary & display invariants**:
  - On Linux `x86` and `x86_64`, the application and installer compile to a single statically linked `musl` binary. Minimize host environment assumptions—bundle assets and link statically rather than expecting system libraries or services.
  - Dynamic linking, `dlopen`, and system libraries on Linux are non-options.
  - Using X11/XWayland when Wayland is available is a non-option: a single statically linked `musl` binary that runs natively on Wayland (on Wayland or Wayland+X11 systems) and X11 (on X11-only systems) is the required target.

