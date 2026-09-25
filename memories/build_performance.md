# Build Performance Guidelines

## Preserve File Modification Times (mtime) in Codegen & Build Scripts
- All build scripts and code generators (`src/build_support/*_codegen.rs`) MUST avoid unconditionally overwriting `.generated` or output files.
- Always check if the file already exists and compare the current file content with the newly generated content (`write_if_changed`). If identical, skip writing so that the file's modification timestamp (`mtime`) is preserved.
- Unconditionally touching or overwriting `.generated` files invalidates Cargo's dependency graph fingerprints, triggering costly cascading recompilations and workspace-wide documentation rebuilding.

## Avoid Workspace-Wide `rerun-if-changed` in Root/Core Crates
- Crates that are depended upon by almost the entire workspace (such as `ctb-utilities`) must be extremely conservative with `cargo:rerun-if-changed`.
- Emitting `rerun-if-changed` for all `.rs` files across other crates (e.g. in IPC scanning) causes `ctb-utilities` to be marked dirty on virtually any source edit.
- If a core crate's build script also emits dynamic environment variables (such as timestamps via `vergen`), changing `rustc-env` forces recompilation of the core crate, which cascades to every dependent crate in the workspace.

## Isolate Build and Version Metadata in Dedicated Leaf Crates
- Dynamic or volatile build metadata (such as git commit SHA or timestamps from `vergen`) MUST NOT be generated or embedded in `ctb-utilities` or other foundational crates.
- Dynamic build metadata is isolated in `ctb-build-info`, which is only depended upon by high-level crates that actually present or check version info (`ctb-workspace`, `ctb-io-webui`).
- `build_info()` uses the git commit timestamp (`VERGEN_GIT_COMMIT_TIMESTAMP`) as its `build_date` to remain stable and reproducible across incremental builds of the same commit.

