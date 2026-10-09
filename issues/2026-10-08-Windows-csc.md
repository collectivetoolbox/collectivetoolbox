# Windows CSC Coverage and Fidelity Handoff

Date: 2026-10-08

Status: **Partially Implemented**. Passing enabled tests does not establish
Windows CSC feature parity. This handoff covers CSC and its `io/file`
dependencies, not a comprehensive audit of every Windows filesystem feature.

## Remaining Gaps vs Upstream / Pending Work

### 1. Most CSC Functional Tests Are Not Compiled on Windows

Eight test modules still use `#[cfg(all(test, unix))]`:

| Module | Test gate | Coverage absent from Windows runs |
| --- | --- | --- |
| CLI | [cli.rs](../src/io/csc/cli.rs#L236) | CLI integration tests |
| Copy engine | [copy_engine.rs](../src/io/csc/copy_engine.rs#L1534) | Main copy, resume, links, sparse files, metadata, Apple formats, collisions |
| Index engine | [index_engine.rs](../src/io/csc/index_engine.rs#L864) | Index workflows |
| Index metadata | [index_meta.rs](../src/io/csc/index_meta.rs#L437) | Index metadata tests |
| Journal | [journal.rs](../src/io/csc/journal.rs#L1321) | Journal and provenance integration tests |
| Move engine | [move_engine.rs](../src/io/csc/move_engine.rs#L285) | Move workflows |
| Search engine | [search_engine.rs](../src/io/csc/search_engine.rs#L645) | Search workflows |
| Verifier | [verifier.rs](../src/io/csc/verifier.rs#L618) | Main post-copy verification tests |

These are compile-time exclusions, not ignored tests. `--include-ignored`
cannot enable them. Windows CSC currently runs 15 tests versus 103 on Linux;
the counts alone do not identify which tests are portable or meaningful on
each platform.

Next steps:

- Start with an ordinary copy/verify test in the main copy-engine module.
  Separate genuinely Unix-specific imports, fixtures, and assertions from
  portable workflows. Reuse existing test modules and helpers.
- Move the module gate to `#[cfg(test)]` where feasible; gate individual
  Unix-only tests or operations. Do not simply remove the gate and weaken
  assertions until everything compiles.
- Port copy/resume, journal readback, indexing/search, move, and verification
  workflows incrementally. Include destination corruption and failure cases.
- Add Windows-specific ADS, native path, reparse/link, timestamp, and sparse
  fixtures where Unix fixtures do not exercise the equivalent contract.
- Report genuinely Unix-only cases separately from missing Windows coverage.
  Do not simulate POSIX semantics and label that native Windows parity.

### 2. Exact Windows Sparse Extents Are Not Captured or Audited

The capture path retains the native sparse attribute and logical payload, but
explicitly records this warning in
[entity.rs](../src/io/file/entity.rs#L870):

> Windows sparse extent capture is not implemented; native sparse attribute and
> logical payload are retained

The auditor in [verifier.rs](../src/io/file/verifier.rs#L528) rejects a requested
sparse check if either the expected entity or actual destination is sparse:

> Windows sparse extent auditing is not implemented

Consequently, retaining bytes and the sparse attribute is not proof that hole
layout or allocation semantics were preserved. The native attribute must not
be erased merely to make strict verification pass.

Important WINE distinction: `FSCTL_SET_SPARSE` can return success without
actually setting `FILE_ATTRIBUTE_SPARSE_FILE`. The existing
[capture regression](../src/io/file/entity.rs#L1753) exercises this distinction.
When the attribute is absent, capture must truthfully classify the file as
dense. On native Windows, a genuinely sparse file must remain classified as
sparse, with its extent-capture limitation recorded until implemented.

Next steps / acceptance criteria:

- Implement native allocated-range discovery and map it to the existing
  payload/extent representation; investigate `FSCTL_QUERY_ALLOCATED_RANGES`
  rather than inventing a parallel format.
- Preserve sparse layout during materialization, then independently audit it.
  Use native volume capabilities and actual results, not API success alone.
- Test leading, interior, and trailing holes, empty files, allocated zero-filled
  ranges, and unsupported destination filesystems. Specify what allocation
  granularity permits before asserting byte-exact allocation boundaries.
- Verify payload bytes separately from layout fidelity. Strict unsupported
  preservation must not be reported as a successful lossless commit.
- Ensure unavoidable limitations and failures appear in user diagnostics,
  semantic Dc metadata, and the CSC journal, including continue-on-error paths.
- Keep WINE and native Windows expectations capability-specific; a WINE
  limitation must not disable a working native implementation.

### 3. Requested Windows Change-Time Auditing Is Unsupported

[args.rs](../src/io/csc/args.rs#L278) enables change-time checking for
`--check-ctime` or `--strict`. The underlying Windows auditor explicitly refuses
the check when timestamps are present:

> Windows change-time auditing is not implemented

See [verifier.rs](../src/io/file/verifier.rs#L525). This is a known missing
implementation, not permission to silently ignore a requested check. Windows
metadata change time must not be conflated with creation/birth time.

Next steps / acceptance criteria:

- Trace capture, serialization, materialization, and audit of Windows native
  change time. Investigate `GetFileInformationByHandleEx` / `FILE_BASIC_INFO`
  and the existing native metadata APIs before adding new abstractions.
- Check exact 100 ns representation, pre-epoch values, and the distinction
  between creation, access, write, and change times.
- Define truthful behavior where restoration or stable equality cannot be
  guaranteed. A verification failure is preferable to silently dropping the
  requested check; best-effort limitations must be recorded and surfaced.
- Add CSC-level tests for explicit `--check-ctime`, `--strict`, default caveats,
  and journal readback. Validate WINE and native Windows separately.

### 4. CSC Native Hardlink Identity Verification Is Missing

In [CSC verifier.rs](../src/io/csc/verifier.rs#L350), hardlink identity comparison
uses Unix device/inode values. The non-Unix branch explicitly fails:

> Native hardlink identity verification is not implemented on this platform

This is a confirmed verification gap, not a claim that every Windows hardlink
capture or creation path has been audited.

Next steps / acceptance criteria:

- Reuse native volume/file identifiers already captured by
  [metadata/windows.rs](../src/io/file/metadata/windows.rs#L129) where suitable.
  Compare identity, not matching contents or link counts alone.
- Test actual hardlinks, independent files with identical contents, missing
  targets, multiple destination roots, and resume behavior.
- Check the full capture/materialize/journal/verify path before calling Windows
  hardlink support complete.

## Completed Bounded Work and Validation

- The eight existing argument tests now compile cross-platform. These cover
  strict/progress options, exact filename checking, Apple write-mode conflicts,
  best-effort resolution, `.as` / `.asf` output, and continue-on-error parsing.
  They do not replace the gated engine integration tests.
- The portable fidelity tests in
  [copy_engine.rs](../src/io/csc/copy_engine.rs#L1428) cover native-path journal
  roundtrips, preservation of warning/error details and stream names/data, and
  refusal to discard journal failures under continue-on-error. Some construct
  errors directly; that is not full native filesystem integration coverage.
- Fixed pre-epoch fractional `FileTimestamps` Dc decoding for all four timestamp
  fields using Euclidean division/remainder. Modern and pre-epoch roundtrips
  now have regression coverage. This does not implement change-time auditing.
- Fixed five Windows file-crate Clippy findings without lint suppressions.
- Fixed the native runner to start its VM after compilation/ISO packaging.
  Optimized compression and JavaScript test binaries retain the original test
  cases and input sizes.

Latest verified results:

| Environment | Result | Scope / caveat |
| --- | --- | --- |
| Linux musl | CSC 103 + file 97 passed; zero ignored | Fresh final scoped run; provenance passed unskipped in 284.18 s |
| WINE | CSC 15 + file 79 passed; zero ignored | Timestamp-fix run; earlier workspace all-target tests and corrected doctests also passed |
| Native Windows 7 | All 119 packaged binaries passed | Includes nine doctest executables; not evidence that gated CSC tests ran |
| Native Windows 7, final file rebuild | File 79 passed; zero ignored | Includes the final Windows filesystem lint cleanup |
| Windows file-crate Clippy | Passed | Targeted `--lib --no-deps` check |

Only Windows 7 was tested natively. Modern Windows validation remains pending.
Full Windows CSC fidelity is **not** established by these results.

## Reproduction and Working Constraints

Build/test one platform at a time. Run `cargo clean` before switching Linux and
Windows targets: keeping both previously exhausted disk space. Do not clean
while another Cargo run is active. No validation runs remain active at handoff.

Linux scoped tests:

```sh
cargo test --release --target x86_64-unknown-linux-musl \
  -p ctb-io-file -p ctb-io-csc --lib --jobs 4 -- --include-ignored
```

Native Windows scoped tests from the repository root:

```sh
RUSTFLAGS='-Awarnings -Aunfulfilled_lint_expectations' CTB_SKIP_DOCS=1 \
  scripts/win-vm test-suite --debug --crate ctb-io-csc
RUSTFLAGS='-Awarnings -Aunfulfilled_lint_expectations' CTB_SKIP_DOCS=1 \
  scripts/win-vm test-suite --debug --crate ctb-io-file
```

The VM runner adds its own Windows linker/shim flags. Do not pass already
expanded Windows `RUSTFLAGS` into it. Use `--no-build` only when the selected
test binaries are known to match current source; an older passing binary is
not validation of newer edits.

For WINE, use the existing Windows target/runner configuration and the
`x86_64-win7-windows-gnu` target with `-Z build-std=std,panic_abort` and the static
Windows 7 shim. Prefer the repository runner over reconstructing linker flags.
Never make a WINE fallback replace a working native Windows API. Older-Windows
shims must prefer true implementations when available.

Tests must not listen on ports. Do not add dependencies without operator
approval or edit `vendor/ctb-vendored` / `vendor/upstream-for-reference`.
Use `#[crate::ctb_test]`, checked arithmetic, and existing helpers. Do not
suppress failures, remove formats, reduce compression inputs, or weaken
fidelity assertions to get a green suite.

The only newly authorized test deferral is
`tests::test_bootstrap_ts_compilation`, the exceptionally slow full two-stage
TypeScript bootstrap. It is unrelated to CSC; do not extend that authorization
to CSC tests. Routine runners retain other include-ignored coverage.

## Evidence Logs and Separate Follow-Ups

Logs are local session artifacts and may disappear; regenerate rather than
assuming these paths exist in another checkout:

- `/tmp/ctb-linux-file-csc-final.log`
- `/tmp/ctb-wine-file-csc-timestamps-fixed.log`
- `/tmp/ctb-native-full-runtime-retry.log`
- `/tmp/ctb-native-file-final.log`
- `/tmp/ctb-win-file-clippy.log`

WINE post-doctest CLI follow-up (2026-10-08): the tests-only build path previously
called the compression smoke script without an explicit executable, allowing
it to select an existing Linux binary. The Windows branch now builds the
current target CLI and explicitly invokes it through the configured WINE
runner. The actual edited stage passed with a freshly built debug Windows CLI;
all existing compression cases and gzip/bzip2/brotli compatibility checks ran.
Debug/release dispatch and unchanged Linux selection were also checked. Log:
`/tmp/ctb-wine-post-doctest-cli.log`. The entire configured workspace command was
not rerun in this follow-up; release CLI runtime validation remains separate
from the passing debug stage. No Windows CSC fidelity gaps above are resolved
by this CLI-stage result.

Separate from Windows CSC parity: existing Linux application/installer
artifacts were audited as static PIE with no interpreter or `DT_NEEDED`
entries; a fresh shipping-binary audit and Alpine/Guix runtime validation remain
pending. Do not treat those unrelated outstanding checks as CSC test failures.