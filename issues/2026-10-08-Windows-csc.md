# Windows CSC Coverage and Fidelity Handoff

Date: 2026-10-08

Status: **Partially Implemented**. Passing enabled tests does not establish
Windows CSC feature parity. This handoff covers CSC and its `io/file`
dependencies, not a comprehensive audit of every Windows filesystem feature.

## Review Follow-Up (2026-10-10)

### Payload Completion Audit

Status: **Partially Implemented**. `PayloadSource::finish` now provides fallible
completion and retained fidelity warnings. Disk readers check size and mtime
before restoration, use the open handle rather than the original pathname, and
restore only atime with an omitted mtime. This prevents replacing-path and
concurrent-mtime clobbering in this reader. Cleanup failures and the unavoidable
Unix ctime change caused by atime restoration are returned as fidelity warnings.
Drop is only a logging safety net, not the completion contract.

Regular and AppleSingle materialization complete the payload before committing
the destination. Strict mode rejects completion warnings; best-effort receipts
retain them for existing CSC semantic metadata and journal propagation.
AppleSingle also rejects short, enlarged, and checksum-mismatched payloads
instead of committing incomplete or incorrect data forks. CSC's preliminary
checksum completes its source and surfaces/records cleanup warnings. Source
change warnings now persist in semantic metadata, and nearby warning journal
writes propagate failures rather than silently discarding them.

Validation: Linux file 112 and CSC 113 passed, zero ignored, before the last
CSC warning-persistence edit; four focused completion/AppleSingle regressions
passed. Affected Linux production Clippy passed before that last edit. FreeBSD
file 111 passed, zero ignored. FreeBSD CSC validation is running; WINE and
native Windows reruns are pending. These are bounded tests, not full-workspace
parity evidence.

#### Remaining Gaps / Pending Work

- The last CSC warning-persistence edit still requires executable validation.
- Aborted read paths and direct detection/compression readers can still rely on
  drop-time logging instead of explicit, semantically retained completion.
- Completion checks size and mtime, not every mutation, inode/path replacement,
  stream change, or native change-time observation. Comprehensive source
  stability and change-policy behavior remain pending. The adjacent entity
  capture path still writes mtime after a metadata check, leaving a race.
- Best-effort dense-reader extent diagnostics, lower-level fidelity propagation,
  Windows change-time restoration, modern Windows validation, and full suites
  on all four platforms remain pending.
- Existing root-permission and ZFS fixture early returns remain bypasses;
  zero ignored tests does not establish the requested no-bypass coverage.

### Capture Audit Continuation

Status: **Partially Implemented**. Unix extent discovery now returns errors for
unsupported or failed queries and invalid/nonadvancing offsets instead of
returning a log-only dense fallback. Native query failures remain I/O errors so
CSC can retain numeric OS errors in journal records. Capture retains logical bytes with an
explicit semantic warning that sparse layout fidelity is unknown; strict
auditing cannot interpret an uncertain extent map as verified fidelity.
Deterministic regressions cover failed and valid seek sequences, and the capture
warning is checked through Dc roundtrip. The journal regression now requires
multiple native-fidelity warnings in both warning records and committed entities.

Capture also records failed source timestamp restoration and failed post-stream
symlink timestamp checks. Regular-file capture avoids rewriting unchanged
timestamps, uses the open file handle for restoration, and refuses to overwrite
mtime when a concurrent source change is detected on the restoration path.
Metadata-only capture has a ctime-preservation regression.

CSC's preliminary checksum path now records payload-open and hash failures at
their existing journal stages instead of silently retaining a zero-digest
sentinel and proceeding to hardlink/skip decisions. A Linux regression uses
`/proc/self/mem` to require a real checksum-read failure before any regular-file
entity or destination is committed under both error policies, without returning
early for root.

Bounded validation: final Linux file 108 and CSC 113 passed with zero ignored,
and affected production Clippy passed. Final-source FreeBSD file 107 and the
updated journal regression passed; its earlier full CSC 110 run predates
checksum error propagation, then all 45 copy-engine tests passed from final
source. A subsequent Unix errno-type correction passed both extent regressions
in FreeBSD and the final Linux suites above. Native Windows 7 and WINE file 91
passed. WINE CSC 93 passed after
checksum error propagation. Native Windows 7 CSC 93 passed before checksum
error propagation, then all 33 copy-engine tests passed from the updated binary.
Windows production Clippy completed with existing warnings, not a warning-clean
result. FreeBSD production Clippy is blocked by four
pre-existing undocumented unsafe blocks in `sys_flags.rs` (1297, 1323, 1360,
1435). All listed runtime counts have zero ignored tests, but the existing
early-return fixture bypasses below remain. These results are not full-workspace
coverage. Both VMs are stopped and no validations remain active. Logs are under
`/tmp/ctb-audit-*`.

#### Remaining Gaps / Pending Work

- Native Windows and FreeBSD full CSC-suite snapshots predate the final checksum
  change; all affected copy-engine tests were rerun afterward. Windows snapshots
  differ from final source only by the subsequent Linux-only failure regression
  and removal of an unused timestamp import and a Unix-only errno correction,
  not Windows production behavior. FreeBSD full file/CSC-suite snapshots predate
  that errno correction; its two affected extent regressions passed afterward.
- Full-workspace runtime coverage on all four platforms remains pending.
- Existing CSC permission fixtures return early as root, in addition to the
  ZFS sparse-negative early return documented below. Zero ignored tests does not
  prove no-bypass coverage; nonprivileged execution or deterministic fixtures
  are required.
- Lower-level payload-reader timestamp restoration in `DiskPayloadSource::drop`
  still discards errors and can overwrite timestamps; it needs fallible lifetime
  completion and diagnostic propagation rather than a destructor-only fix.
- Unsupported extent discovery now fails loudly in direct payload-reader paths;
  a best-effort dense-reader API carrying retained fidelity diagnostics is pending.
- Source stability is not comprehensively enforced on every capture path;
  the noatime, metadata-only, stream, and Windows paths need independent review.
- The other fidelity gaps below remain pending.

Status: **Partially Implemented**. The recent-commit review identified and
corrected these bounded deficiencies:

- Invalid sparse-layout fallback diagnostics now survive in materialization
  receipts instead of being available only in logs.
- Sparse materialization uses the native control result and retained attribute,
  not an unconditional WINE rejection. Unsupported native filesystems are not
  mislabeled as WINE limitations.
- CSC prints actual recorded warning codes/messages, retains warning-bearing
  journals when deletion is requested, and distinguishes failed files from error
  records. Historical error records no longer produce a clean-success summary.
- Unix auditing propagates failure to restore timestamps changed by its reads.
- Semantic journal OS errors preserve signed 32-bit values; ambiguous `Some(0)`
  is explicitly rejected rather than silently decoded as `None`.
- The FreeBSD VM runner builds its standard library from source and includes
  ignored tests, with only the authorized routine TypeScript bootstrap exclusion.
  Explicit filters permit that bootstrap test to run.
- The Windows COM compatibility shim attempts to load available COMBase before
  selecting an already-loaded Ole32. Its existing standalone regression now
  verifies that lookup ordering on WINE and native Windows 7.

Validation so far: Linux file 104, CSC 112, and Dc-string 89 tests pass, with
zero ignored; affected Linux production Clippy passes. FreeBSD file 103 and
CSC 110 tests pass, zero ignored; the final CLI-status follow-up also passes
in the FreeBSD VM. Native Windows 7 and WINE file 89 tests pass, zero ignored;
the standalone shim and CSC 93 tests pass on both, zero ignored. Both VMs are
stopped. VM protocol tests pass all ten cases; FreeBSD sysroot tests pass all
35 cases. Windows-target Clippy completes but emits
existing repository warnings; it is not a warning-clean result. The final
CLI-status follow-up postdates the Linux CSC run.

### Remaining Gaps / Pending Work

- Full-workspace runtime coverage on all four platforms is not established by
  these affected-crate checks. Modern Windows and other filesystem variants
  remain unvalidated.
- The existing CSC sparse-verification fixture returns early when zero-filled
  destination blocks remain sparse (notably compressed ZFS), bypassing its
  negative sparse assertions. FreeBSD's passing count does not prove this
  scenario was exercised; a deterministic allocation-query fixture is pending.
- Unix extent discovery's log-only fallbacks are addressed by the capture audit
  continuation above. Lower-level stream, flag, security, and directory-sync
  diagnostic propagation remains incomplete.
- Sparse auditing still assumes missing sparse support from WINE identity in
  best-effort mode, and granularity handling assumes platform-wide allocation
  limits rather than independently measured destination capabilities.
- Windows change-time restoration and the other previously documented gaps
  remain unresolved. Existing sections below contain historical results and
  must not be interpreted as validation of this review's current source.

## Remaining Gaps vs Upstream / Pending Work

### 1. Most CSC Functional Tests Are Now Compiled and Passing on Windows and WINE (Completed 2026-10-09)

The five test modules previously blocked behind `#[cfg(all(test, unix))]` have been un-gated and ported to Windows and WINE:

| Module | Test gate | Status |
| --- | --- | --- |
| Copy engine | [copy_engine.rs](../src/io/csc/copy_engine.rs#L1534) | Un-gated to `#[cfg(test)]`. Portable workflows enabled; genuinely Unix-specific fixtures (POSIX FIFOs, raw device nodes, symlink dangling edge cases, noatime) individually gated. |
| Index engine | [index_engine.rs](../src/io/csc/index_engine.rs#L864) | Un-gated to `#[cfg(test)]`. Symlink indexing assertions gated with `#[cfg(unix)]`. |
| Index metadata | [index_meta.rs](../src/io/csc/index_meta.rs#L437) | Un-gated to `#[cfg(test)]`. |
| Move engine | [move_engine.rs](../src/io/csc/move_engine.rs#L285) | Un-gated to `#[cfg(test)]`. |
| Search engine | [search_engine.rs](../src/io/csc/search_engine.rs#L645) | Un-gated to `#[cfg(test)]`. Path separators normalized for Windows path glob/substring searching. |

Enabled Windows CSC test coverage increased from **40 tests to 87 tests**.
All 87 tests pass on both WINE and native Windows 7 VM (0 failures, 0 ignored). Linux musl runs all 106 tests (0 failures, 0 ignored).

Key issues identified and resolved during this port:
- **Canonical Stream Name Reconciliation**: Introduced `StreamName::canonicalize_native()` in [streams.rs](../src/io/file/streams.rs). On Windows NTFS, alternate data streams are reported with `:name:$DATA` syntax, whereas AppleDouble and in-memory streams use bare stream names (`com.apple.ResourceFork`). `canonicalize_native()` reconciles leading `:` and trailing `:$DATA` suffixes on Windows while bridging UTF-16 and byte encodings. Updated [verifier.rs](../src/io/file/verifier.rs) and [materializer.rs](../src/io/file/materializer.rs) to use canonical stream names, resolving 5 post-copy verification failures on Apple formats under WINE and native NTFS.
- **Extended-Length Path Support (MAX_PATH / 260 chars)**: Win32 metadata queries in [metadata/windows.rs](../src/io/file/metadata/windows.rs) and stream operations in [streams/windows.rs](../src/io/file/streams/windows.rs) now use an enhanced `path_to_wide()` that converts paths to verbatim `\\?\` (and `\\?\UNC\`) format with normalized backslashes. This resolved `test_long_path_names_and_deep_nesting` failure on native Windows 7 where paths exceeding 260 characters previously failed to open handles and omitted change times.
- **Cross-Platform Identity Queries**: Replaced platform-specific `ino()`/`dev()` assertions with cross-platform `query_file_identity` in `test_hardlinks_preserved`, `test_hardlinks_across_destination_roots`, and `test_hardlinks_preserved_across_resume`. Replaced POSIX atime/mtime assertions with cross-platform `FileTime` checks in `test_best_effort_metadata_timestamp_tolerance`.

### 2. Windows Sparse Restoration and Auditing (Partially Implemented, 2026-10-09)

Status: **Partially Implemented**, not lossless. Native allocated ranges are
discovered via `FSCTL_QUERY_ALLOCATED_RANGES`, materialization punch-restores
holes via `FSCTL_SET_ZERO_DATA`, and the auditor in `verifier.rs` validates
hole layout and sparse attributes across platforms.

#### Implementation Details (Completed 2026-10-09)
- **Restoration Sequence**: Integrated in [materializer.rs](../src/io/file/materializer.rs#L820):
  1. Issues `FSCTL_SET_SPARSE` on `temp_file`. Verifies attribute retention.
  2. Seeks and writes data extents, accumulating payload SHA-256 digest.
  3. **Windows Cache Manager Flush**: Calls `temp_file.sync_all()` *before*
     issuing zeroing controls. This guarantees that dirty memory pages in the
     Windows Cache Manager do not subsequently flush back over zeroed extents
     and reallocate clusters upon handle close or commit.
  4. Punches holes via `FSCTL_SET_ZERO_DATA` for each `Extent::Hole`.
  5. Extends file length via `temp_file.set_len(initial_size)` to establish EOF
     and preserve trailing holes.
- **Auditing**: Replaced the previous stub in [verifier.rs](../src/io/file/verifier.rs#L994)
  with native sparse discovery via `get_file_extents`. Verifies both sparse
  attribute and physical hole layout.
- **WINE Capability Isolation**: WINE stubs `FSCTL_SET_SPARSE` without setting
  `FILE_ATTRIBUTE_SPARSE_FILE`, and fails `FSCTL_SET_ZERO_DATA`. In strict lossless
  mode (`options.strict_lossless`), materialization bails because sparse layout
  cannot be honored. In best-effort mode, it falls back to dense writing with a
  receipt warning, and auditing tolerates the missing sparse capability with
  `ignored.sparseness += 1`.
- **Validation**: Added `test_sparse_materialization_and_auditing` in
  [materializer.rs](../src/io/file/materializer.rs#L1925), verifying leading,
  interior, and trailing 64 KB holes across all platforms:
  - Linux musl: 102/102 tests pass (0 failures).
  - WINE: 87/87 tests pass (0 failures).
  - Native Windows 7 VM: 87/87 tests pass (0 failures).
  - All 90 CSC tests pass on WINE and native Windows 7 VM (0 failures).

#### Remaining Gaps vs Upstream / Pending Work
- **NTFS 64 KB Sparse Allocation Granularity**: On NTFS with default 4 KB
  clusters, the physical compression / sparse unit is exactly 16 clusters
  (64 KB / 65,536 bytes). `FSCTL_SET_ZERO_DATA` on sub-64KB holes writes logical
  zeros but NTFS leaves physical clusters allocated. When capturing entities
  with sub-64KB holes, a warning is recorded in metadata. In strict lossless mode,
  sub-64KB holes cause verification failure; in best-effort mode, `ignored.sparseness`
  is incremented. Full sub-64KB hole parity is an inherent NTFS physical limitation.
- **ReFS Support**: ReFS has distinct sparse allocation characteristics (often
  64 KB cluster sizes, differing sparse unit rules) that have not been tested
  against a live ReFS volume.
- **CLI and Journal Diagnostics**: Ensure sub-64KB granularity truncation and
  WINE sparse limitations are prominently rendered in user-facing CLI summary
  diagnostics and continue-on-error journal entries.

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
Windows/filesystem coverage, and the remaining runtime diagnostic gaps listed
in item 6 below.
WINE also reports restored old mtime as birthtime in a strict captured-metadata
fixture. The dedicated post-close mtime regression does not request birthtime
restoration; it does not establish WINE creation-time fidelity.

### 4. Native Hardlink Work Is Bounded, Not Full Parity

CSC verification now compares native volume/file identities on Windows.
Regression coverage includes cross-root hardlinks, relocation, an equal-byte
independent-file negative case, and new hardlinks during interrupted resume.
Journal paths are relative even when Windows returns verbatim canonical paths;
resume opens with write-data access before truncating an uncommitted tail.

Hardlink verification now propagates non-NotFound metadata errors with the
affected link/target path instead of reporting them as missing. A portable
overlong-component regression checks both error paths and preserves the
underlying I/O error; existing missing-link/target tests remain unchanged.
WINE reports an overlong component as NotFound rather than a non-NotFound
error. That fixture explicitly checks WINE's missing-link/target reporting;
non-NotFound propagation is validated on Linux and native Windows.

Pending work: the main copy-engine test module remains Unix-gated and capture
still falls back to zero identity after a failed Windows identity query.
These gaps prevent claiming comprehensive native hardlink parity.

### 5. Full-Provenance Test Latency Versus Live Capture

The integration test previously made live Internet calls for copy, verify,
index, and move. Public IPv4, IPv6, and clock-offset discovery are sequential;
only successful results are cached, so offline failures are repeatedly queried.
Although IP queries request a five-second timeout, CRLite cache preparation
precedes that timeout. Its bootstrap client has no explicit request deadline
and retries failed downloads. The five-second timeout is not a bound on full
capture. This explains why the test could take minutes offline.

The workflow test now uses the existing named-test CRLite download override to
produce deterministic offline errors. IP-query workers inherit the test name
and isolated storage, so the mock applies without changing TLS/revocation
policy or using a real user's cache. A counter requires full capture to attempt
discovery through that transport. All copy, verify, index, move, and provenance
readback assertions remain active. The focused Linux workflow now passes in
0.16 seconds; the current complete CSC suite passes in 0.44 seconds.
The complete WINE CSC suite now passes in 1.13 seconds, compared with 525.59
seconds before test isolation. The native Windows 7 VM suite still takes
35.48 seconds; its remaining VM/runtime overhead has not been profiled.

Pending work: successful live IP/clock discovery coverage and an end-to-end
production deadline including CRLite preparation and blocking client teardown.
No claim of seconds-bounded production capture follows from the offline test.

### 6. Runtime Materialization Diagnostics Are Partially Retained

Bounded follow-up (2026-10-09): `check_metadata_replication` now returns its
best-effort birthtime and native-metadata discrepancy warnings instead of
making them available only in logs. `apply_entity_metadata` also returns
ownership, permission, and timestamp application failures that best-effort
mode permits. Windows read-only permission errors are no longer discarded:
strict mode fails and best-effort mode returns a diagnostic. Native APIs and
strict fidelity requirements have not been replaced with WINE fallbacks.

`MaterializeReceipt.warnings` carries those diagnostics to file API callers.
CSC retains them in the entity's semantic `FileMetadata.warnings` and records
`FileMaterializationWarning` journal records for regular files, symlinks,
special nodes, and deferred directory metadata. Deferred directories now
refresh their journal entity after metadata application. The Unix identical
file optimization also performs the best-effort replication check.

The existing opaque-native-metadata regression is now portable. It exercises
real materialization with an unreproducible byte-valued attribute, unchanged
payload, strict rejection, returned diagnostics, metadata operation failures,
and `FileMetadata` Dc roundtrip retention. A new portable CSC regression
checks actual materialization warnings and native metadata through journal
readback. Its two passes exercise a new copy and a repeated copy; only Unix
currently uses the identical-file optimization.

Remaining Gaps / Pending Work:

- Lower-level best-effort diagnostics emitted by stream, flag, ACL, Windows
  security/reparse, and directory-sync helpers are not all structured return
  values. The replication check captures supported final-state discrepancies,
  but this does not prove that every permitted operation failure is retained.
- Direct file API callers must retain receipt warnings in their semantic
  metadata or journal. Returning a receipt does not automatically mutate the
  source entity, and not every downstream caller has been audited.
- The non-Unix identical-file optimization is explicitly disabled. Native
  link-count and sparse checks are prerequisites before enabling it safely.
- The CSC regression exercises real file materialization and journal helpers,
  not injected failures through every engine dispatch or continue-on-error
  workflow. Main engine coverage remains gated as described in item 1.

### 7. Semantic Path Fidelity Refactor (In Progress)

The original omission was reproduced without clearing captured raw fields.
Following operator approval, independently mutable `raw_relative_path` and
`raw_filename` fields were removed. Native path bytes and filenames now derive
from `relative_path`; `enclosing_path` remains a separate semantic path.
The existing Dc derive and tags 337/357 remain in use. No identity-specific Dc
serializer or encoding-specific Dc identifier was added.

The shared `PathBuf` codec now emits typed f229 filepath lists containing f228
filename representations and Dc 513 delimiters. Equal UTF-8 text and filename
bytes use Dc 397; non-UTF-8 bytes remain binary. Windows WTF-8 decoding retains
unpaired surrogates and rejects bytes that would require replacement. Unix
backslashes inside filenames are preserved. The legacy path codec remains
readable, but contradictory equivalents now fail. The journal's existing
byte layout is retained; conflicting stored path/filename values are rejected.
Stream names remain authoritative in `AttachedStream.name`.

New `CTBMETA3` preservation records additionally retain the identity through
its generic derived Dc representation. This preserves absent versus empty
paths, enclosing paths, and Archive/Remote/Fifo provenance. The primary path
must agree with the legacy byte slot. Older readers reject the new marker;
older records remain readable with a logged and retained fidelity warning
when semantic identity is missing. Already-lost information is not recovered.

Whole captured-entity Dc roundtrips now pass on Linux, WINE, and native
Windows 7 without clearing metadata or identity bytes in the new capture
regression. Linux passes dcstring 89, CSC 107, and file 100 tests, zero ignored,
plus affected production Clippy checks. Native Windows 7 passes CSC 88,
dcstring 89, and file 85 tests, zero ignored. After adding strict rejection of
removed JSON fields, WINE passed file 86 tests, zero ignored. After the semantic
journal upgrade and legacy-warning regressions, the complete current WINE
CSC suite passes 90 tests, zero ignored (7.66 s). The Linux and earlier native
counts above predate that upgrade. A Windows path regression
also passes after replacing new bare format integers with typed `FormatId`
variants. Affected Windows production Clippy checks pass.

Configured WINE workspace validation stopped at repository preflight lints,
before running tests. Remaining findings include undocumented fallback uses
in the Dc macro, missing headers in shared/source Dc modules and a VM Python
test, and tempdir/test-boilerplate findings in other modules. The new identity
test module's boilerplate violation was corrected. Direct release workspace
examples, all-target tests, and doctests ran separately under WINE with no
reported failing test targets. The only harness exclusion was the authorized
TypeScript bootstrap test; Windows-inapplicable packages were excluded from
the build. The broad CSC binary passed 89 tests, including semantic identity
roundtrip, but predates the final legacy-warning regression. The current
debug CSC suite passed all 90 as noted above. Current codec tests also passed
all 89 with WINE exception tracing enabled.

Two unhandled WINE page faults appeared during IPC and software-renderer
doctests even though their harnesses reported success. These are unresolved
runtime diagnostics, not a clean runtime validation. A trivial WINE startup
and the current codec suite did not reproduce them. The native Windows 7
workspace run passed all 110 packaged binaries, including CSC 90 with both
semantic-identity journal regressions, dcstring 89, and file 86, zero ignored
in those suites. It predates the concurrent sparse-materialization edits.
The rebuilt file binary subsequently passed all 87 tests under both WINE
and native Windows 7, zero ignored (0.97 s and 26.80 s). Native doctest
preservation was blocked by two new sparse Win32 calls lacking explicit
unsafe boundaries; statement-scoped FFI expectations fixed compilation.
No preflight failure is reported as a configured-workflow pass.

The initial doctest preservation pass also omitted proc-macro/rlib targets
and ignored doctest bodies. Its four-executable inventory was incomplete and
must not establish native coverage. Corrected inclusion exposed compile
failures in the scoped IPC macro example and historical resource-bundle
examples. These examples are now runnable, with focused WINE results of one
and three passed tests respectively, zero ignored. Corrected preservation
then found two invalid installer examples, now runnable with real imports,
a headless egui frame, and the supported locale variant; both pass under
WINE, zero ignored. Explicit renderer doctests also pass all three under
WINE without reproducing the earlier page faults. Those earlier diagnostics
remain unexplained rather than being treated as a proven fix.

All 13 preserved doctest executables subsequently passed on native Windows 7,
alongside the 89-test codec binary (14/14 binaries passed). This includes the
renderer and formerly ignored examples. Proc-macro doctest commands also
reported success but produced no standalone executables; their reported
compilation results are not separate native runtime execution evidence.
Fresh Windows release CLI tools passed the compression compatibility stage
and all seven JavaScript tests under WINE. Both VMs are stopped. After checking
for active builds and cleaning Windows artifacts, fresh Linux musl suites
passed dcstring 89, CSC 109, and file 102 tests, zero ignored. Current affected
production Clippy checks also pass. A broader Linux examples/all-target/
doctest/JavaScript/compression CLI run is active, with only the authorized
TypeScript bootstrap exclusion in the all-target harness.

The first broader Linux all-target run failed only the installer sidebar
fixture, which searched for the English label `Home` while concurrent locale
tests changed translated labels. The fixture now verifies the native home
path instead; all 146 installer tests pass, zero ignored. The workspace
all-target retry and its following doctest/CLI stages are active. This last
test-only assertion change has not been rerun on Windows; the recorded native
workspace pass predates it.

Remaining Gaps / Pending Work:

- Legacy JSON metadata containing the removed duplicate fields is now rejected
  explicitly rather than silently discarded. An automatic lossless migration
  for those exports remains unimplemented.
- Foreign path bytes unrepresentable as a local `PathBuf` remain available in
  the original Dc source but cannot be materialized through that local type.
  General cross-platform semantic path projection still needs coverage.
- The codec preserves semantic separators, not their original Windows lexical
  spelling. Modern Windows and unusual native filesystem/name coverage remain
  pending; Windows unpaired-surrogate codec tests do not establish filesystem
  creation/materialization capability everywhere.
- Whole-workspace validation is in progress, not a verified pass. The native
  runtime run passed 110 binaries and its fresh file follow-up passed 87
  tests; the native doctest follow-up passed 14 binaries. WINE's earlier
  reported-success doctest page faults remain unexplained. Fresh Linux
  affected-crate tests and production Clippy pass after the required
  platform-switch clean; the broader Linux run is active. Full configured
  suites remain blocked by the recorded preflight findings.
  Existing sparse, change-time restoration, and diagnostic gaps remain open.

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

Historical and current verified results:

| Environment | Result | Scope / caveat |
| --- | --- | --- |
| Linux musl, complete CSC + file suites | CSC 106 + file 97 passed; zero ignored | All functional and diagnostic tests passing (0.45 s and 0.06 s) |
| Linux musl, current semantic identity follow-up | Dcstring 89 + CSC 109 + file 102 passed; zero ignored | Latest journal and concurrent file changes included; affected production Clippy passes; broader run active |
| Linux musl, installer follow-up | 146 passed; zero ignored | Repaired locale-dependent sidebar assertion; workspace retry active; assertion not rerun on Windows |
| WINE, complete un-gated CSC suite | CSC 87 passed; zero ignored | All 5 test modules enabled and passing (6.26 s) |
| Native Windows 7 VM, complete un-gated CSC suite | CSC 87 passed; zero ignored | VM runner; all functional, index, search, and long path tests passing (333.10 s) |
| Native Windows 7, semantic identity follow-up | 110 packaged binaries passed; CSC 90 + dcstring 89 + file 86, zero ignored in those suites | Includes latest semantic journal regressions; predates concurrent sparse edits |
| Native Windows 7, current doctest follow-up | 13 preserved doctest executables + codec binary passed (14/14) | Includes renderer and repaired examples; not modern Windows validation |
| WINE, current release CLI stages | JavaScript 7/7 and compression compatibility checks passed | Freshly built Windows tools; not a configured preflight pass |
| Native Windows 7 / WINE, rebuilt file follow-up | File 87 passed in each; zero ignored | Includes concurrent sparse changes; native 26.80 s / WINE 0.97 s; not modern Windows validation |
| Linux / Windows affected-crate Clippy | Passed | File and CSC production `--lib --no-deps` checks |
| Linux musl, scoped diagnostic follow-up | CSC 106 + file 97 passed; zero ignored | Runtime warning regressions included; 0.45 s and 0.03 s |
| WINE, scoped diagnostic follow-up | CSC 40 + file 81 passed; zero ignored | Prior baseline before un-gating 5 test modules |
| Native Windows 7, scoped diagnostic follow-up | CSC 40 + file 81 passed; zero ignored | Prior baseline before un-gating 5 test modules |

Only Windows 7 was tested natively. Modern Windows validation remains pending.
Full Windows CSC fidelity is **not** established by these results.
The whole-workspace suites were not rerun for the runtime diagnostic changes;
the historical full-suite results above are not a fresh workspace validation.

The first native diagnostic regression run failed because the new fixture
assumed Unix's identical-file optimization also existed on Windows. The
fixture now explicitly asserts the actual platform contract while retaining
all warning, native-metadata, payload, and serialization checks; the full
native and WINE affected suites then passed. No tests were ignored or skipped.

The staged Windows payload writer is now synced and closed before metadata
restoration, preventing its eventual close from resetting restored mtime.
The native full-provenance workflow reproduced the failure before this change
and passes after it. The existing Windows payload regression also verifies an
old mtime with exact 100 ns precision after handles close, on native Windows
and WINE. Creation-time open/write errors now propagate rather than being
discarded. Offline-provenance and hardlink error changes pass current Linux,
native Windows, and WINE validation with the fixture distinction above.

## Reproduction and Working Constraints

Build/test one platform at a time. Run `cargo clean` before switching Linux and
Windows targets: keeping both previously exhausted disk space. Do not clean
while another Cargo run is active. Native workspace, file, and doctest
follow-up runs have finished. Fresh Linux musl affected-crate tests and
production Clippy pass. Broader Linux workspace validation is active after
cleaning completed Windows artifacts; do not clean or switch back to Windows
while it runs.

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
- `/tmp/ctb-diagnostics-linux-suites.log`
- `/tmp/ctb-diagnostics-linux-clippy.log`
- `/tmp/ctb-diagnostics-native-file-final.log`
- `/tmp/ctb-diagnostics-native-csc.log`
- `/tmp/ctb-diagnostics-wine-file.log`
- `/tmp/ctb-diagnostics-wine-csc.log`
- `/tmp/ctb-diagnostics-windows-clippy.log`

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