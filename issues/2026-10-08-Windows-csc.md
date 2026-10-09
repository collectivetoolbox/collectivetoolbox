# Windows CSC Coverage and Fidelity Handoff

Date: 2026-10-08

Status: **Partially Implemented**. Passing enabled tests does not establish
Windows CSC feature parity. This handoff covers CSC and its `io/file`
dependencies, not a comprehensive audit of every Windows filesystem feature.

## Remaining Gaps vs Upstream / Pending Work

### 1. Most CSC Functional Tests Are Not Compiled on Windows

Five test modules still use `#[cfg(all(test, unix))]`:

| Module | Test gate | Coverage absent from Windows runs |
| --- | --- | --- |
| Copy engine | [copy_engine.rs](../src/io/csc/copy_engine.rs#L1534) | Main copy, resume, links, sparse files, metadata, Apple formats, collisions |
| Index engine | [index_engine.rs](../src/io/csc/index_engine.rs#L864) | Index workflows |
| Index metadata | [index_meta.rs](../src/io/csc/index_meta.rs#L437) | Index metadata tests |
| Move engine | [move_engine.rs](../src/io/csc/move_engine.rs#L285) | Move workflows |
| Search engine | [search_engine.rs](../src/io/csc/search_engine.rs#L645) | Search workflows |

These are compile-time exclusions, not ignored tests. `--include-ignored`
cannot enable them. Windows CSC now runs 38 enabled tests; the previous Linux
baseline ran 103 tests before these changes. Journal, verifier, and CLI modules
are portable, with genuinely POSIX fixtures individually gated;
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

### 2. Windows Sparse Restoration and Auditing Are Unimplemented

The capture path now queries native allocated ranges and retains logical
payload independently. It explicitly records this warning in
[entity.rs](../src/io/file/entity.rs#L870):

> Windows sparse extent materialization and auditing are not implemented;
> captured allocated ranges and logical payload are retained

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
sparse, with its restoration/auditing limitation recorded until implemented.

Next steps / acceptance criteria:

- Validate native allocated-range discovery with multi-range and pagination
  fixtures; the existing `FSCTL_QUERY_ALLOCATED_RANGES` implementation uses the
  existing payload/extent representation.
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

### Native Sparse Investigation (2026-10-09)

Sparse restoration/auditing remains **Partially Implemented**, not lossless.
Native allocated-range capture now uses `FSCTL_QUERY_ALLOCATED_RANGES`, with
pagination and checked offsets, mapping ranges to the existing semantic
`Extent` representation. The all-hole 4096-byte NTFS capture regression passes
on Windows 7. Failed range queries retain logical bytes and record a metadata
warning. Native sparse entities explicitly warn that restoration/auditing is
not implemented; strict sparse auditing still refuses unsupported fidelity.

Blocked restoration experiment, for an agent with Windows documentation access:

- Windows 7 NTFS, VM runner, live read/write handle, `FSCTL_SET_SPARSE` succeeded
  and the sparse attribute was retained. An initial 4096-byte all-hole file
  captured as `Hole { offset: 0, length: 4096 }`.
- Extending to 327680 bytes and writing 4096 zero bytes at offset 65536 and
  `tail` at offset 196608 allocated gaps through the last written range.
  Marking sparse before writes and seeking alone did not preserve holes.
- `FSCTL_SET_ZERO_DATA` on `[0,65536)`, `[131072,196608)`, and
  `[262144,327680)` returned success. After syncing and closing, native queries
  reported `Data(0,131072), Hole(131072,65536), Data(196608,65536),
  Hole(262144,65536)`: the leading range remained allocated.
- The fixture required leading/interior/trailing holes and an allocated
  zero-filled range. It failed three repair attempts; assertions were not
  weakened to count that as fidelity. The unverified restoration/audit
  experiment and failing fixture were removed at the operator's direction.
- Investigate NTFS retained/resident initial allocation, valid-data-length
  transitions, zero-range granularity/alignment, and handle lifetime/order.
  Confirm with independent native tools and documentation before choosing a
  restoration algorithm. Do not infer deallocation from ioctl success alone.

Local diagnostic log: `/tmp/ctb-native-sparse-roundtrip.log`; regenerate the
experiment rather than relying on that transient artifact. The experiment
used `scripts/win-vm test-suite --debug --crate ctb-io-file --filter
test_windows_sparse_capture_and_materialization`.

Remaining gaps: validated multi-range capture fixtures, allocation pagination
coverage, native sparse restoration and auditing, capability-specific WINE
materialization diagnostics, and modern Windows/filesystem coverage. This
bounded capture work does not close the sparse parity item.

### 3. Windows Change-Time Restoration Remains Unimplemented

Capture and requested auditing now use `FILE_BASIC_INFO.ChangeTime` rather
than substituting mtime. Conversion retains exact 100 ns precision, including
fractional dates before the Unix epoch. Native ticks and semantic timestamps
survive Dc serialization and journal readback. `--check-ctime` and `--strict`
detect a changed expected ctime; defaults retain the ignored-ctime caveat.

Restoration is still unimplemented. Captured entities and journals explicitly
warn that original native change time is retained but not restored. Requested
checks are not silently disabled. Native Windows 7 and WINE regressions cover
capture, conversion, mismatch reporting, flags, caveats, and journal readback.

Pending work: restoration with independently verified native equality, modern
Windows/filesystem coverage, and structured journal diagnostics for dynamic
best-effort metadata replication failures that currently appear only in logs.
WINE also reports restored old mtime as birthtime in a strict captured-metadata
fixture. The dedicated post-close mtime regression does not request birthtime
restoration; it does not establish WINE creation-time fidelity.

### 4. Native Hardlink Work Is Bounded, Not Full Parity

CSC verification now compares native volume/file identities on Windows.
Regression coverage includes cross-root hardlinks, relocation, an equal-byte
independent-file negative case, and new hardlinks during interrupted resume.
Journal paths are relative even when Windows returns verbatim canonical paths;
resume opens with write-data access before truncating an uncommitted tail.

Pending work: the main copy-engine test module remains Unix-gated, capture
still falls back to zero identity after a failed Windows identity query, and
hardlink verification still classifies non-NotFound metadata errors as missing.
These gaps prevent claiming comprehensive native hardlink parity.

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
  now have regression coverage; Windows change-time auditing was subsequently
  added as described above, but restoration remains unimplemented.
- Fixed five Windows file-crate Clippy findings without lint suppressions.
- Fixed the native runner to start its VM after compilation/ISO packaging.
  Optimized compression and JavaScript test binaries retain the original test
  cases and input sizes.

Latest verified results:

| Environment | Result | Scope / caveat |
| --- | --- | --- |
| Linux musl | CSC 103 + file 97 passed; zero ignored | Fresh final scoped run; provenance passed unskipped in 284.18 s |
| WINE, current scoped executables | CSC 38 + file 80 passed; zero ignored | Full provenance passed in 525.59 s; final exact mtime assertion also passed separately |
| Native Windows 7, current scoped rebuild | CSC 38 + file 80 passed; zero ignored | Includes portable CLI, journal/provenance, ctime, hardlink/resume regressions |
| Windows affected-crate Clippy | Passed | Current `--lib --no-deps` check for file and CSC |
| Native Windows 7 | All 119 packaged binaries passed | Includes nine doctest executables; not evidence that gated CSC tests ran |
| Native Windows 7, final file rebuild | File 79 passed; zero ignored | Includes the final Windows filesystem lint cleanup |
| Windows file-crate Clippy | Passed | Targeted `--lib --no-deps` check |

Only Windows 7 was tested natively. Modern Windows validation remains pending.
Full Windows CSC fidelity is **not** established by these results.

The staged Windows payload writer is now synced and closed before metadata
restoration, preventing its eventual close from resetting restored mtime.
The native full-provenance workflow reproduced the failure before this change
and passes after it. The existing Windows payload regression also verifies an
old mtime with exact 100 ns precision after handles close, on native Windows
and WINE. Creation-time open/write errors now propagate rather than being
discarded. Current-source Linux validation is still pending.

## Reproduction and Working Constraints

Build/test one platform at a time. Run `cargo clean` before switching Linux and
Windows targets: keeping both previously exhausted disk space. Do not clean
while another Cargo run is active. Windows/WINE checks have finished; the
current-source Linux scoped run is active after the required cleanup.

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