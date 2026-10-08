- File/copy crate tests: `cargo test -p ctb-io-file -p ctb-io-csc --lib` (hyphenated package names).
- Do not run scripts/format or scripts/format-quick for routine formatting: they auto-stage/commit and destructively clean vendor. Use rustfmt directly on touched files/lines.
- Operator commits during work; blank git diff does not mean edits vanished. Inspect commit history and task baseline.
- rustix 0.38 IFlags is not Copy or PartialEq: use .bits() for repeated masking/comparison and from_bits_retain for syscall values.
- Native Windows stream capture can be cross-compiled with `cargo test -p ctb-io-file --lib --no-run --target x86_64-pc-windows-gnu`; this does not run Windows tests.
- For multiple Rust test-name filters, pass them after `--` to the harness: `cargo test -p ctb-io-file --lib -- filter_one filter_two`; Cargo accepts only one filter before `--`.



- Panorama export investigation: docs/panorama-format.md and `python3 scripts/diagnose-pan-exports.py` document/probe legacy fixtures without writes. `old/test-export-fixtures` prefers target/release/ctoolbox; it does not rebuild.
- Panorama debug CLI is `pan2parsejson`. Procedure payloads can contain tokenized code plus a separate source copy; source-only mutation tests the current AST runtime, not necessarily original Panorama execution.

- Dc data validator CLI only recognizes `--write`/`-w`; other flags are silently ignored. Default validates CSVs embedded directly from src/formats/dcdata/data, not the general asset bundle. For read-only live-file validation use validate_all_data_tables_from_repo (covered by test_math_format_metadata); avoid --write when IDs must remain untouched.

- Use `python3 -B` (or PYTHONDONTWRITEBYTECODE=1) when importing repository helpers during probes: fixture directories are embedded and operator auto-commits can capture __pycache__ artifacts.

- Parallel compression CLI tests need a distinct CTB_TEST_STORAGE_DIR per case: shared storage/cache caused intermittent `Directory not empty (os error 39)` failures. The override is supported by utilities::storage.
- Bash errexit may unwind function-local variables before EXIT traps run. Keep failure-report metadata in worker subshell scope, not function locals; test command errors as well as explicit exits.


- apply_patch cannot delete and add the same path in one patch; separate delete/add operations also lose executable mode. Prefer Update; if replacement is necessary, restore the existing executable bit and validate the direct script entry point.
- Reviews of known-working generators must preserve tested transformations and add audit/provenance safeguards. The FreeBSD/LLVM sysroot synthesis was iteratively validated in a FreeBSD VM; do not remove it based on speculative ABI concerns. GPL-3.0-only and AGPL-3.0-only are prohibited for these sysroot inputs, regardless of narrow exceptions elsewhere.


- Windows/WINE review handoff (2026-10-07): overarching full-platform suite/parity remains incomplete. Current bounded Dc fix passes Linux metadata tests (7) and all Windows file tests under WINE (77, zero ignored). Latest Dc/sparse changes still need native Windows rerun.
- Earlier native Windows 7 checks: file 75 + four runnable file examples; IPC 55 plus three docs; detection 15 docs; standalone shim regression; optimized WebUI 113 and compression 33 all exited successfully. Most Windows CSC tests are cfg-excluded: only seven enabled CSC tests ran, not full Windows functional coverage. Linux CSC 100 passed earlier.
- Genuine bootstrap tests remain active/unverified in Stage 2: WINE execution 2b441fcd-4d64-49ad-82bd-b521523c0089, log /tmp/ctb-wine-bootstrap-continued.log; native VM execution 4ff15cd9-1adc-4435-a76b-8279633d29f3, log /tmp/ctb-native-bootstrap-continued.log. Do not start another VM while native bootstrap runs. At handoff host PID 3698 (WINE main), 19723 (QEMU); recheck state before acting, IDs may change after restart.
- Windows cross-tests use target x86_64-win7-windows-gnu, RUSTC_BOOTSTRAP=1, -Z build-std=std,panic_abort, Clang --target=x86_64-w64-windows-gnu with LLD and existing static ctb_win7_shim; rustdoc needs equivalent linker flags separately but cannot accept -l. Missing rustdoc flags caused the three supplied old low-address execution crashes.
- Native runner: scripts/win-vm test-suite --release/--debug --no-build --crate PREFIX --standalone-tests-dir DIR. Persist current rustdoc executables into a fresh DIR with --no-run --persist-doctests (compile-only is not a runtime pass). Old persisted file examples in ctb_workspace_ipc-doctests remain stale and fail; use newly compiled file examples, not that old directory indiscriminately.
- WINE sparse control FSCTL_SET_SPARSE can return success without the sparse attribute. Test asserts truthful dense capture and strict sparse materialization refusal/no commit there; native Windows must retain sparse classification and its recorded extent-capture warning. Exact Windows sparse extent capture/auditing and requested change-time auditing remain missing.

- Dc attribute correction (2026-10-08, operator commit d90071e25): dcstring 76 tests, file 95 tests, CSC 99 tests, live schema validation and `cargo clippy -p ctb-dc-macro -p ctb-formats-dcstring -p ctb-io-file --lib --no-deps` passed. CSC test_full_provenance_csc_and_related_commands hung >8 minutes; interrupted and excluded via harness --skip (no test code changed). Strict dependency-inclusive Clippy -D warnings blocked by existing ctb_asset_bundle too_many_lines/large_stack_arrays warnings. Windows/WINE not rerun for this correction. Existing FileTimestamps/AppleMetadata custom codecs remain outside this bounded correction.

- Shared references/scopes (2026-10-08): dcstring 79, file 96, CSC 101 passed; CSC full-provenance test still excluded, Windows not rerun. Catalog test_math_format_metadata and targeted --lib --no-deps Clippy passed. Final focused grammar and nonempty-payload streaming checks passed; streaming tags/suffix must use payload_bytes() to avoid duplicate DcMst headers. Pending: general EL dispatch, contextual JSON Serde renames/nested traversal, EnvDescription.extra Dc omission. Typed reader declarations are document-local; with_context is explicit outer-scope import.
- apply_patch currently rejects repeated Update File entries for the same path with Duplicate Path; combine all hunks under one file header.

- Operator warning (2026-10-08): build/test one platform at a time and run `cargo clean` before Linux/Windows switches; simultaneous artifacts filled 438G filesystem to 98%. Keep this constraint for future runs.
- Linux x64 musl linker correction: .cargo/config.default.toml and active config now select scripts/x86_64-musl-gcc. Before correction, IPC minimal-workspace linked glibc libm.a and failed; exact example passed with wrapper. readelf confirmed static PIE, no INTERP, no DT_NEEDED (DYNAMIC section alone is normal for static PIE).
- Full-provenance CSC test passed unskipped on Linux musl in 284.81s (2026-10-08); earlier apparent hang was not reproduced. DNS/network retries and blocking-client teardown cause long waits. No TLS/CRLite bypass added.
- Operator explicitly authorized re-ignoring full test_bootstrap_ts_compilation (2026-10-08), not other tests. WINE/native runners retain --include-ignored for others but skip this one in routine suites; native --filter enables explicit execution. Focused Linux release bootstrap tests: 2 passed, 1 ignored. Full bootstrap parity still unverified.
- Current Linux validation (2026-10-08): configured suite including JS/examples/all targets/CLI compression passed with authorized full-bootstrap exclusion. Optimized file tests 97 and CSC 103 passed, zero ignored (provenance 284.74s). Workspace doctests initially failed only renderer examples' missing egui_demo_lib/egui_extras. Replaced with plain egui UI and hidden first-frame ViewportCommand::Close to exercise window startup without endless event loop; renderer 3 and full workspace doctests passed. Shipping application binary was not rebuilt/audited in tests-only run; exact IPC example linkage audit is the only binary audit so far.
- CSC args test module enabled cross-platform (cfg(test) rather than cfg(all(test, unix))) on 2026-10-08; all 8 existing tests passed under WINE, including exact filename bytes and AppleSingle .as/.asf copy paths. Other CSC test modules remain Unix-gated; Windows CSC full parity remains incomplete.



