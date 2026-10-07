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

## September 26, 2026 Investigation
- The stopped build's root asset script started at 08:32:25 UTC, but its saved
	`output` still had the previous run's 08:03:04 timestamp. Kaitai's script
	started and wrote its output at 08:30:46; its library completed at 08:31:10.
	These are artifact timestamps, not a live process profile.
- `prepare_runtime_assets` read all of `built/v86_images.rsrc` solely to parse
	its header. The local bundle was 11,964,990,385 bytes (11.14 GiB), matching
	the reported build-script memory spike. It now uses the existing 72-byte
	header reader. An isolated Rust probe against that actual bundle took
	2.2 ms and peaked at 2,092 KiB RSS. Do not read entire resource bundles for
	metadata, even when the bundle itself is cached.
- The existing asset-packer test now checks header reads from a 16 GiB sparse
	bundle and missing, truncated, and invalid headers. Run it with
	`cargo test --offline -p ctb-build-support --lib test_write_resource_bundle_streaming -j 2`.
- Cargo now defaults to four jobs in `.cargo/config.default.toml`, preserving
	incremental compilation and linker settings. Override with
	`CARGO_BUILD_JOBS` or `cargo -j`. CI defaults to two jobs through the
	environment: patching `.cargo/config.toml` was ineffective because the
	jobs line was commented and `./build` overwrites that file.
- Kaitai's generated category-module cleanup must not depend on `read_dir`
	order. Check whether the category directory exists, not whether it has
	already been visited. Isolated Rust checks covered both creation orders,
	stale-module removal, and unchanged-output mtime preservation.
- Cargo directory watches are recursive. Watching Kaitai's `tests` directory
	included `tests/generated` despite filtering its individual files; the
	parent watch is now excluded. A compiled fixture check verified that
	handwritten inputs remain watched and generated outputs have no watched
	ancestor.
- Recent Kaitai libraries were about 140 MB, not progressively ballooning.
	Its broad source hash still includes runtime and handwritten-test inputs;
	this can cause unnecessary regeneration but was not the observed 11 GiB
	allocation. Narrowing it needs separate dependency-coverage tests.
- Cached utility fingerprints show separate flag sets for direct Cargo
	commands and `./build` (the latter adds `target-feature=+crt-static` and
	spells warning flags differently). Switching between them can compile
	separate artifacts. No flag changes were made here to avoid another broad
	invalidation or changes to static-linking behavior.
- The newest available CI log was September 11, not the stopped local build.
	It records a test-profile compilation of 10m 12s. A full build was not
	rerun during this investigation; end-to-end speedup remains unmeasured.

- Build performance investigation and measurements are recorded in repository memories/build_performance.md (2026-09-26).
- Asset preparation previously read a 11.96 GB v86 resource bundle solely for its 72-byte header; use read_existing_asset_bundle_header.
- Canonical Cargo config is .cargo/config.default.toml; ./build overwrites .cargo/config.toml. CI concurrency belongs in CARGO_BUILD_JOBS.
- Focused asset check: cargo test --offline -p ctb-build-support --lib test_write_resource_bundle_streaming -j 2. Avoid full builds when host is swapping.
- Warning: `cargo clean --target x86_64-pc-windows-gnu` removed cached Windows 7 and host build artifacts too in this workspace (125.9 GiB). Do not rely on --target for narrowly scoped cleanup; prefer explicitly identified generated directories.
- Windows 7 VM tests: use Clang targeting x86_64-w64-windows-gnu with LLD, dynamic GCC lib directory, and the compatibility shim as a native static archive before raw imports. MinGW GCC's -fuse-ld=lld still used GNU ld here; late shim objects left unsupported DLL imports, and GNU ld + static shim produced an import-table execute fault. scripts/win-vm now uses the tested configuration and Win+R launch. Verified 142 utilities and 53 IPC tests in Windows VM and WINE (one pre-existing incomplete cancellation test ignored).

- Windows debug application startup overflow: Clap routing::Command::augment_subcommands reserves 0xdd808 bytes (~886 KiB) plus nested argument-builder frames, exceeding the default PE 1 MiB stack. Outer async boxing did not solve it and was removed. build.rs now gives the ctoolbox Windows binary an 8 MiB reserve (GNU --stack / MSVC /STACK); PE header verified and --version/--help passed under WINE and Windows 7 VM. Full GUI startup and MSVC build remain unverified.


