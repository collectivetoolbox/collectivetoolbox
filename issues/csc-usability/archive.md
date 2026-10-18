## Plan: CSC-Based Archive Command

Status: **Partially Implemented** (2026-10-17).

The top-level `ctb archive create/extract/list/verify` command is registered, with
implementation and argument ownership in `ctb-io-csc`. The initial container
uses journal-style DcMixed sessions and checksummed sections, independent archive
magic/version, and embedded primary data under existing Dc 392.

### Current Progress

- File and stdout creation share a forward-only encoder. Journal v4 now shares
	its checksummed-section framing, with compatibility tests passing.
- `CscArchive` (f633) and `CscArchive1` (f634) are registered, with
	`CTBCSCA\x01` magic and `.cscarchive` detection.
- Primary contents are streamed during writing and retained as lazy file spans
	during reading. Archive verification checks section, file, and attached-stream
	integrity and validates sparse extent maps.
- File-output resume validates committed archive contents and source identities
	and contents before truncating an unclosed final section. Completed archives
	refuse resume. Stdout resume is rejected before output.
- Initial Unix extraction preflights paths, links, and collisions; uses existing
	materialization; and restores directory metadata after children. File payloads,
	hardlinks, symlinks, and small attached streams have source-free round-trip tests.
- Incomplete listing reports only committed members and returns nonzero.
- Scoped musl Clippy, payload projection, archive operations, journal v4
	integration, and archive magic detection have passed. The final library
	regression run passed 420 tests across CSC, file I/O, and DcMixed, with no
	failures or ignored tests; all seven journal v4 integration tests also passed.
- The actual musl executable passed a stdout-creation smoke test: redirected
	output was listed, verified, and extracted after moving the original source
	path away. Binary contents matched, and extraction into existing entries was
	rejected. Final Clippy for CSC and top-level CLI routing passed.
- The read-only musl `validate-data-tables` runner reported no validation
	diagnostics after archive format registration.

### Production Readiness Hardening

Status: **In Progress**. The following work extends the archive MVP to shared
file I/O and ordinary CSC; the MVP test results above do not qualify these
stronger guarantees.

- Shared collision planning now retains native filenames instead of using lossy
	display strings as identity. AppleSingle extension changes and AppleDouble
	companion paths use the same native-name operations in the format, file, and
	CSC layers.
- Linux regressions cover distinct invalid-UTF-8 names, NFC/NFD spellings,
	mixed-case AppleSingle extensions, CSC destination/journal paths, and a
	source-free archive round trip with raw directory and leaf names.
- This hardening slice passed 287 musl library tests and 11 integration tests
	(Apple losslessness, lazy payload projection, and journal v4), with no failures
	or ignored tests. Scoped Clippy passed. A native Windows UTF-16 helper fixture
	was added but has not been executed on Windows.
- Priorities are Linux, native Windows, and portable AppleSingle/AppleDouble
	correctness. FreeBSD qualification follows; native macOS qualification is
	deferred until runtime infrastructure is available.
- Strict reproduction remains the default. Archive creation preserves original
	filesystem/container entries by default; Apple interpretation is explicit.
	Existing entry bytes, metadata, streams, flags, and companions must not be
	modified without explicit overwrite authorization.
- Shared mutation APIs now take `ExistingEntryPolicy`; materialization defaults
	to `Refuse`. Linux no-replace rename and exclusive node creation reject late
	collisions. Directory metadata/streams require explicit permission or live,
	identity-checked ownership from this sandbox's exclusive directory creation.
- CSC uses canonical `--overwrite`; `--always-overwrite` was removed. Resume
	does not inherit authority from intent records. Both move paths require
	explicit `--force` to replace existing entries. Authorized regular replacement
	avoids metadata-only mutation through unrelated hardlink aliases.
- AppleDouble companions now use staged, synced, policy-aware publication, not
	direct final-path writes. Main/companion atomicity and crash recovery are still
	pending. Immutable destination flags are no longer automatically cleared.
- The authorization slice passed all 370 musl tests/doctests across file I/O and
	CSC, with no failures or ignored tests, and scoped Clippy including `ctb-cli`.
	File/CSC tests compile for `x86_64-pc-windows-gnu`; no fresh native Windows
	runtime or VM test was run. Cross-compilation is not runtime qualification.
- Generated CLI reference refresh and a new executable smoke run remain pending.
	The final disk check left 27 GB free, so no further build/VM was started near
	the repository's 25 GB minimum. Existing generated help may still show the old
	overwrite spelling until normal executable/doc generation runs.
- Persistent archive error/warning sections now reuse shared journal semantic
	conversions and operational incident construction. Listing displays them and
	returns nonzero; completed byte integrity is separate from clean capture.
	Normal verification and extraction reject diagnostic captures before writes.
- Creation supports `--continue-on-error`: retained planning/runtime failures
	permit other source members, with nonzero status. File source-only failures
	discard buffered bytes before rollback; stdout aborts after a partial member.
	Sink, flush/sync, rollback, and diagnostic-persistence failures always abort.
	Hardlink definitions are registered only after successful member commitment.
- Resume refuses committed diagnostics before truncation, and cannot be combined
	with `--continue-on-error`. No durable failure-resolution or intended-inventory
	semantics are claimed. Older experimental readers reject diagnostic sections;
	diagnostic-free v1 reads and journal wire behavior remain compatible.
- Journal `mark_failed` now propagates pending commit/encoding failures instead
	of silently discarding them and updating the failure descriptor. The focused
	invalid-timestamp regression verifies queued metadata and descriptor retention.
- Recovery now rejects truncated or unconsumed trailing bytes after completion;
	completion cannot turn a partial trailing record into a valid listing. The
	focused regression checks every section-marker prefix after the marker.
- This diagnostic slice passed 400 musl tests/doctests with no failures or
	ignored tests, scoped file/CSC/CLI Clippy, and Windows CSC test compilation.
	Native Windows runtime, a new executable smoke run, and generated CLI reference
	refresh remain unverified. The latest disk check left 31 GB free.

#### Pending Work

- Qualify authorization under hostile concurrent mutation and replace remaining
	path-based metadata/stream operations with anchored operations. Live ownership
	checks do not close the subsequent check/use window or establish snapshot
	isolation. Persistent applied-identity recovery authority remains pending.
- Complete shared lazy capture/stream projection through journal batching,
	materialization, and verification, with source inventory/change detection.
- Harden Apple parsing/raw representation retention and staged main/companion
	publication. Persistent local extraction/publication recovery state is now in
	scope; it is not a required archive sidecar and does not modify the archive.
- Complete durable diagnostic resolution/retry, exhaustive torn-record recovery,
  shared durability/error propagation, and adversarial tests. Current retained
  failures are not evidence of a complete original source inventory.
- Qualify actual destination namespace collisions, native Windows containment,
	and crash/power-failure guarantees. Exact filename preservation in these
	passing Linux cases does not establish universal filesystem/platform fidelity.
- Special-file overwrites remain deferred; reject unsupported collisions without
	changing originals. No recursive directory deletion to resolve collisions.

### Remaining Gaps vs Plan

- Persistent diagnostics and source-error continuation are present, but durable
	resolution/retry and intended-source inventory remain pending. Resume rejects
	recorded diagnostics; stdout cannot persist an in-band failure after partial
	member emission and instead aborts with incomplete output.
- Attached-stream capture and projection still buffer stream data; only primary
	contents have the complete lazy bounded-memory path. Very large streams need
	a generic shared capture/projection extension before claiming full payload-
	independent memory bounds.
- `--overwrite` supports regular-file replacement and directory merging only;
	preexisting symlinks, hardlinks represented as link entries, and special entries
	are rejected. Harden current mutation authorization first; special-file
	overwrite expansion is deferred under the production-readiness scope above.
- Extraction is explicitly limited to Unix. Platform fidelity, concurrent
	archive/destination mutation, metadata operations, and full race-resistance
	need further review and adversarial testing. Do not treat preflight checks as
	snapshot isolation or proof of safety against hostile concurrent mutation.
- Some CSC options (full provenance, source-change policy choices, filesystem
	boundaries, and Apple stream restoration choices) are not exposed yet.
- Recovery accepts structurally unclosed final sections but intentionally rejects
	closed sections with invalid digests. Recovery at arbitrary partial-token or
	partial-footer boundaries needs further tests and a precise corruption policy.
- The first implementation is not a production-ready backup/snapshot tool. Keep
	originals until separate validation confirms their recoverability.

The following sections retain the approved target plan; they are not claims
that all planned behavior is already present.

### Confirmed Decisions
- User approved proceeding with create, extract, list, verify; resumable creation and persistent diagnostics; initially uncompressed storage. DRY reuse is an explicit priority: one serialization path and source traversal/capture path, shared with journal code wherever behavior is the same, not parallel archive copies of those implementations.
- Include stdout creation in MVP: `ctb archive create - SOURCE...`. One forward-only `Write`-based archive encoder serves files and stdout with identical format bytes; seek/truncate/sync are capabilities of file-output orchestration, not encoder requirements. Stdout carries archive bytes only; progress, diagnostics and summaries go to stderr. Check and propagate final flush and broken-pipe errors; a broken pipe is a failed archive write, never success.
- Stdout cannot resume, truncate a failed member, fsync, or perform output read-back verification. Reject `create - --resume` before emitting bytes and explicitly document these differences. Continue-on-error may skip and record a source failure before beginning its record, but failure/change after a member starts aborts stdout output without a valid footer/completion marker; do not attempt to close a partial record and continue. Report failures on stderr when a valid in-band diagnostic can no longer be written. File output retains rollback/resume semantics. Consumers use the existing verify operation on the saved output.
- Stdout wire format and shared APIs are MVP acceptance criteria, not a deferred redesign. Pipe input for extract/list/verify and stdin sources remain deferred. Avoid temporary payload spooling or buffering whole members to simulate seekable output. Preserve source payload hashing/change detection before committing the section. Keep the stdout encoding path forward-only even when supplied a seekable test sink; no separate stdout serializer.
- User approved `CTBCSCA\x01` (8 bytes), `.cscarchive`, and new format ID allocation from canonical CSV sources. Allocate available IDs only after checking the live database; never assume f633 or a particular long Dc is free.
- Top-level command is `ctb archive`, not `ctb csc archive` or the existing cp-compatible `csc --archive` flag.
- User approved default sandboxed extraction with no overwrite, plus explicit overwrite and trusted-link options. Trusted links must NOT disable path containment for writes or hardlinks.
- User approved narrowly scoped shared DcMixed streaming changes where necessary, preserving journal wire bytes and existing behavior.
- User correctly identified existing Dc 392: semantic.csv defines it as `File data`, syntax `[data]`, description `File contents`. Reuse Dc 392, NOT attached-stream Dc 385. No new semantic Dc is required or approved. Earlier research overlooking Dc 392 is superseded.

### Findings and Architecture To Reuse
- Baseline HEAD: `7e1bcac008de14e10eaf46f8292279c2c2f75ee7`. Operator can commit while work proceeds; inspect committed delta, HEAD diff, and untracked files before closing implementation.
- `/workspaces/ctoolbox/src/io/csc/journal.rs`: `JOURNAL_MAGIC_V4 = CTBCSCJ\x04`; `JournalSessionRecord`, `JournalErrorRecordPayload`, `JournalWarningRecordPayload`, `JournalWriter`, `read_v4_journal_snapshot`. Dc 547/548 session, 543/544 checksummed section, 545 digest, 546 sequence, 551 completion. `commit_batch` buffers a DcMstring; `open_for_resume` truncates to validated committed length. Reader already uses on-disk `DcDocumentNavigator` but retains a metadata index; it does not load the entire archive-equivalent document into RAM.
- `/workspaces/ctoolbox/src/io/file/entity.rs`: `FileEntity` derive for Dc 317/318; regular kind has size, SHA-256, sparse extents. `encode_to_writer_with_payload_and_context` ALREADY streams primary bytes under Dc 392; reuse it rather than inventing a payload wire encoding. Its buffer-based counterpart is not bounded-memory for full files. Current primary encoders omit a payload tag for empty files; a zero-size regular record therefore legitimately has no Dc 392.
- `/workspaces/ctoolbox/src/io/file/streams.rs`: `AttachedStream` includes Dc 385 payload, and `encode_to_writer_with_stream_reader` exists. Existing capture and ordinary decoding can materialize large streams, so do not claim fully bounded payload memory without addressing large attached streams too.
- `/workspaces/ctoolbox/src/io/file/payload.rs`: `PayloadSource` is Read + Seek + Send; exposes size/extents and fallible `finish` for source-change checks and fidelity diagnostics. `DiskPayloadSource` is the source adapter.
- `/workspaces/ctoolbox/src/formats/dcstring/navigator.rs`: `DcDocumentNavigator`, `BinaryPayloadRef` with Inline/Borrowed/FileSpan, `sha256_of_span`; current `materialize` allocates whole payload. Need bounded span reads without materializing. FileSpan offsets are meaningful only with their backing archive handle, not standalone memory references.
- `/workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs`: generic `DcMixedFlattened` support and binary derive handling. Preserve the rule that schemas are derive-driven; any lazy binary projection extension must be generic, not special parsing for archives.
- `/workspaces/ctoolbox/src/io/file/materializer.rs`: `materialize_entity(entity, Some(payload), sandbox, options)`; `/workspaces/ctoolbox/src/io/file/verifier.rs`: `verify_materialized_entity_ext`. Reuse these for restoration/verification, including metadata ordering and loss reporting.
- `/workspaces/ctoolbox/src/io/file/sandboxable_dir.rs`: descriptor-relative containment; `/workspaces/ctoolbox/src/io/file/path_policy.rs`: StrictSandboxed and separate symlink policy. Existing RejectEscapingSymlinks is explicitly best-effort for chained links; do not promise stronger guarantees from that validator. StrictSandboxed must remain mandatory for archive member writes, metadata operations and hardlinks, including trusted-link mode. Validate platform guarantees and fail explicitly where not available.
- `/workspaces/ctoolbox/src/io/csc/path_resolution.rs`: source trailing-slash conventions are reusable, but `resolve_tasks` interprets destination as a filesystem copy target; do not feed the archive output file as the member destination.
- `/workspaces/ctoolbox/src/cli/routing.rs`: command enum, lightweight command list, CSC dispatch pattern. Keep nontrivial logic out of ctb-cli.

### CLI and Behavioral Contract
- `ctb archive create ARCHIVE SOURCE...`: create_new output; output path explicit, no forced suffix rewrite. Suggested extension .cscarchive. Sources use CSC conventions against a virtual archive root: directory without trailing slash retains basename; with trailing slash includes contents; a file retains basename. Reject ambiguous duplicate member paths, overlapping roots that duplicate members, and archive output located within/inode-aliasing any source tree/file. Capture root metadata for contents-only roots; reject conflicting claims to virtual-root metadata rather than silently dropping them.
- `ctb archive create ARCHIVE --resume`: recover sources and session from archive; reject extra source operands. Revalidate committed source identities/metadata/content before skipping them, and require unfinished source files. Resume does not claim a point-in-time snapshot. If a committed member's source is gone/changed, fail rather than constructing a mixed snapshot. Completed archive refuses append/resume as a new operation.
- `ctb archive extract ARCHIVE DEST`: self-contained; does not access original source paths. Default no overwrite of preexisting entries; distinguish directories created during this extraction from preexisting collisions. Explicit `--overwrite` replaces entries safely without following symlinks or writing through preexisting hardlinks. Explicit `--trust-symlink-targets` preserves raw targets but does not authorize writes outside DEST. Special node restoration requires explicit opt-in; unsupported/suppressed items and metadata failures are reported as loss, not successful fidelity. Use existing explicit best-effort metadata policy for cross-filesystem losses.
- `ctb archive list ARCHIVE`: show member paths/types/sizes and archive completion/error/warning status without loading payloads. An incomplete/errored archive is visibly partial and returns a non-success status; never present committed prefix as a complete archive.
- `ctb archive verify ARCHIVE`: source-independent full structural validation, monotonic sequence checks, section SHA-256, payload size and file SHA-256, stream checksums, extent validity and hardlink consistency. Does not mean signature/authenticity verification and does not require original files or extraction.
- Share option definitions/helpers locally only where actually common; do not refactor all CscArgs merely to add archives. Reuse relevant progress, provenance, source-change and continue-on-error policies. Do not expose irrelevant copy-only switches.
- Creation diagnostics are stored inside archive; no .cscdesc is required to read, extract, verify or resume. Do not create an automatic companion file in v1. Archive itself retains original metadata when target cannot reproduce it.
- By default errors stop creation; continue-on-error records failures and can finalize the operation but always returns non-success with partial status. A completion marker alone is not evidence of a lossless archive; diagnostics must be checked. Extract/verify reject incomplete or failed archives before destination writes. Listing remains available for inspection. Partial salvage/extraction is deferred.

### Implementation Steps and Dependencies

#### Phase 1: Format and Streaming Foundation
1. Register `CscArchive` family and `CscArchive1` version following existing CscJournal/CscJournal4 conventions. Version implies family and is based on DcMixed f484, not CscJournal. Add .cscarchive and CTBCSCA\x01 detection to CSV/magic sources. Keep canonical magic defined once in CSC. Existing generated FormatId/Dc metadata is regenerated through repository codegen, never hand-edited. Unknown versions must be rejected by archive reader without being mistaken for journals. This step can run in parallel with step 2.
2. Add a reusable, derive-annotated payload-aware file projection in ctb-io, using flattened FileEntity fields and an optional lazy binary Dc 392 field; preserve the exact FileEntity outer record shape. Use existing `DcMixedFlattened` machinery rather than duplicating identity/metadata schema. Extend generic binary projection to retain owned lazy file spans where needed; add bounded reader/copy support for BinaryPayloadRef and a CSC archive-backed PayloadSource with strict read/seek bounds. Large attached streams also need a generic payload-aware projection/reader path, not forced Vec materialization. Preserve existing FileEntity/AttachedStream APIs for current callers; factor common schema/encoders rather than adding struct-specific Dc parsing.
3. Add a narrowly shared CSC checksummed-section writer/scanner helper, extracted from journal.rs only as necessary. Stream section bytes through incremental SHA-256 directly to a forward-only `Write` sink; emit the digest in the existing footer with no backpatching, seeking, or output reread. Track byte offsets with checked counters. One encoder handles file and stdout output; only the file sink orchestration owns sync, truncation and resume. Never build full payload batches in DcMstring. Reuse existing direct file payload encoders, extending stream handling generically where needed. Keep journal APIs, strict/forgiving policies, legacy conversion, bytes, and existing descriptor behavior unchanged. For archive scanning, separate strict read/verify from resume-only recovery and expose committed member spans/metadata without payload buffers. Depends on step 2; step 1 needed for integration.

Phase 1 gate: demonstrate current journal regression tests and payload-span round trip before building operation orchestration.

#### Phase 2: Archive Operations in CSC
4. Add `archive` module with ArchiveWriter/ArchiveReader and versioned magic, reusing journal session/diagnostics/commit conventions and the shared helpers. One member per committed section initially limits rollback scope; directory/metadata-only members use the same commit protocol. Write payload and metadata, validate source `finish` and observed size/digest, then emit the commit footer; sync only for file output. If source validation fails on file output, truncate only the current uncommitted section, persist the diagnostic, and stop or continue per policy. For stdout, abort on any failure after a record begins without emitting a valid footer/completion marker; retain pre-record error diagnostics where safely representable. Share all record encoding and source inspection logic across both sinks. Dense logical primary bytes plus original sparse extent map are the v1 representation; no new sparse wire framing, dedup, or content addressing. Preserve hardlinks via member-relative references and emit data only for the defining file. Validate sizes, extents, missing/duplicate payloads, and distinguish empty regular files from absent content in nonempty files. Depends on steps 1-3.
5. Implement creation/resume traversal using CSC copy_engine's existing source inspection, hardlink tracking, provenance, errors and source-change logic through narrowly extracted helpers where needed. Do not call the copy pipeline with a fake disk destination and do not duplicate it wholesale. Persist typed source paths and stable mapping into member-relative identities. Absolute source provenance is never an extraction destination. Resume validates committed sections before truncation, restores session/shared environment/sequence and diagnostics, then revalidates committed sources. Only interrupted final/uncommitted data may be discarded automatically; interior checksum corruption, unsupported versions, or corruption in a completed archive is a hard error, not recovery success. Flush errors/finalization errors propagate. Depends on step 4.
6. Implement list and verify on ArchiveReader; member index is O(member metadata), payload buffers bounded independently of file size. Listing may scan/hash sections but does not materialize payloads. Verify hashes actual archived bytes and checks all streams, completion and diagnostics. Reject unknown structural records under strict policies rather than silently projecting them away. Depends on step 4; can run alongside step 5.
7. Implement extraction from archive-backed PayloadSource through materialize_entity and existing verification APIs. Preflight the entire completed archive with verify and path/link/collision planning before modifying DEST. Resolve target platform path encoding loss/collisions explicitly; no lossy Unicode normalization or name sanitization. Reject absolute/escaping member names and hardlink targets, including cross-platform drive/UNC forms, duplicate paths and file/directory conflicts. Never create hardlinks outside validated member identities; handle forward references deterministically. Restore directory metadata after children and restrictive flags last. Preserve exact raw symlink targets only under chosen policy; never follow archive-created or preexisting links for writes. Validate archive bytes during extraction as well so preflight is not an excuse to trust a subsequently modified archive. Special-device/FIFO payload reads are never implicit. Depends on steps 4 and 6.

#### Phase 3: Routing, Documentation, Integration
8. Add archive args/subcommand enum in CSC and run_archive dispatch; register top-level lightweight Archive command in ctb-cli routing without changing CscArgs or `csc -a`. Parsing can be developed in parallel with Phase 2 after contract is fixed; runtime dispatch depends on operations.
9. Add detection tests covering exact version, family, extension-only evidence, wrong-extension magic, and archive vs journal disambiguation. Check normal file-info/detection integration uses generated registry without adding hand-maintained format lists. Preserve journal conversion support untouched; archive-specific generic convert input support is out of scope, and should return a clear unsupported-input error rather than misidentify the archive.
10. Document CLI examples, on-disk layout, recovery/completion/error contract, root mapping, safe/trusted links, overwrite/special-file policy, sparse dense-storage choice, metadata caveats, permissions, and lack of authentication. Update csc-safety.md with a clear journal-versus-self-contained-archive distinction and preserve CSC's existing caveats. Do not advertise production-grade snapshots or universal platform fidelity. Depends on stable operations/flags.

### Critical Files
Existing paths to modify/reuse:
- /workspaces/ctoolbox/src/cli/routing.rs — Command, dispatch, lightweight command registration.
- /workspaces/ctoolbox/src/io/csc/csc.rs — archive module export.
- /workspaces/ctoolbox/src/io/csc/journal.rs — narrowly shared commit/scanner internals, compatibility gate.
- /workspaces/ctoolbox/src/io/csc/copy_engine.rs — source capture/traversal reuse only as required.
- /workspaces/ctoolbox/src/io/csc/path_resolution.rs — source-to-member mapping conventions.
- /workspaces/ctoolbox/src/io/file/entity.rs — existing payload streaming and reusable derived projection.
- /workspaces/ctoolbox/src/io/file/streams.rs — attached stream encoding/lazy projection.
- /workspaces/ctoolbox/src/io/file/payload.rs — PayloadSource contract.
- /workspaces/ctoolbox/src/io/file/materializer.rs and verifier.rs — restoration/verification, only change for demonstrated archive requirements.
- /workspaces/ctoolbox/src/io/file/sandboxable_dir.rs and path_policy.rs — containment guarantees and link policy reuse.
- /workspaces/ctoolbox/src/formats/dcstring/navigator.rs and dc_mixed_serde.rs — generic bounded binary reads/projection.
- /workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/ — generic derive extension only if existing flatten/binary hooks are insufficient.
- /workspaces/ctoolbox/src/formats/dcdata/data/categories/formats/database.csv — family/version format registrations beside journals.
- /workspaces/ctoolbox/src/formats/dcdata/data/magic/ctoolbox.magic — signature/extension.
- /workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv — reference Dc 392; no semantic change required.
- /workspaces/ctoolbox/src/io/csc/tests/e2e_journal_v4.rs — existing compatibility gate.
- /workspaces/ctoolbox/src/formats/detection/detection.rs — focused detection tests following journal examples.
- /workspaces/ctoolbox/docs/csc-safety.md — safety/format distinction.
New files only as needed: /workspaces/ctoolbox/src/io/csc/archive.rs and its focused args/cli/container submodules following normal module layout; /workspaces/ctoolbox/src/io/csc/tests/e2e_archive.rs for integrated round-trip/adversarial tests; /workspaces/ctoolbox/docs/cli/archive.md for command reference. Keep small helper tests beside their owning modules.

### Verification
Implementation verification results are recorded in Current Progress above.
The following list describes the full target gates; unreported cases remain
pending rather than implicitly covered by the initial passing subset.
1. First behavior gate: create archive containing >4 KiB binary data with NUL/all byte values and an empty file; remove original source tree; list, verify, extract; assert bytes and supported metadata. Include attached-stream bytes to prove self-containment beyond primary content.
2. Stream safety gate: large generated primary file and attached stream, bounded-size reader instrumentation, bounded seeks and short reads, no whole-file Vec allocation; corrupted/truncated binary length/checksum rejected. Explicitly exercise FileSpan projection above 4 KiB and cursor restoration. Memory bound is payload-independent plus member/metadata index; do not label full operation O(1). Test the same encoder with a Write-only sink that does not implement Seek, handles short writes and injected errors, and compare deterministic output with file-sink bytes under identical metadata/session inputs. CLI gate: `ctb archive create - SOURCE... > saved.cscarchive`, then verify/extract after removing sources. Assert stdout contains only format bytes, progress and summary remain on stderr, broken pipe/final flush failures return nonzero, stdout resume is rejected before any bytes, and a mid-member source failure cannot produce a successfully verifiable archive. Use in-memory sinks or OS pipes/socketpairs, never listening ports.
3. Recovery gate: interrupt in payload, section footer and completion; resume commits exactly once, discards only uncommitted tail, never reports damaged prefix as complete. Inject write/read/flush failures; source growth/shrink/change/replacement/missing files on resume; completed archive append rejected; diagnostic preservation and non-success partial status tested.
4. Fidelity gate: directories/root metadata, trailing-slash source mapping, multiple sources/collisions, symlinks/dangling targets, hardlink ordering, sparse/empty/all-hole files, xattrs/ACLs/resource forks/ADS, non-UTF8 and cross-platform names. Platform-capability cases tested where supported and clearly report any environment skips. Strict restoration errors on unsupported fidelity; explicit best-effort warns.
5. Security gate: absolute/parent/drive/UNC names, symlink-poisoned ancestors, preexisting symlink and hardlink overwrite, hardlink escape, duplicate entries, chained links, special node opt-in, archive output self-inclusion, archive mutation between preflight/extraction. Assert nothing outside DEST changes.
6. Compatibility: `cargo test -p ctb-io-csc --target x86_64-unknown-linux-musl --test e2e_journal_v4`; unchanged journal fixtures and legacy journal tests; csc -a parsing unchanged.
7. Focused implementation commands, according to touched scope: `cargo test -p ctb-io-csc --target x86_64-unknown-linux-musl archive`; `cargo test -p ctb-io --target x86_64-unknown-linux-musl --lib`; `cargo test -p ctb-formats-dcstring --target x86_64-unknown-linux-musl`; `cargo test -p ctb-formats-detection --target x86_64-unknown-linux-musl csc_archive`; `cargo test -p ctb-formats-dcdata --target x86_64-unknown-linux-musl`; `cargo test -p ctb-cli --target x86_64-unknown-linux-musl archive`. All new tests use #[crate::ctb_test], no listening ports. Run narrow tests immediately after each owning change before broader gates.
8. Run scoped musl Clippy on changed crates and resolve errors; regenerate format metadata through normal repository build/codegen and run table validation. Before builds check >=25 GB free and no concurrent build conflicts; do not install glibc packages or clean while another build runs. If vendored dependencies block testing, stop and report rather than reverting them. No new dependencies planned.
9. Manual CLI smoke through repository-built binary: help, create/resume, delete sources, list/verify/extract, verify existing destination requires explicit overwrite, confirm `ctb file`/detection identifies .cscarchive independently of extension. Report failing/skipped checks first and keep remaining gaps explicit.

### Exclusions and Remaining Gaps
- Implementation is partial; the Remaining Gaps vs Plan section above tracks
	the work still required. Planned gates not explicitly reported as passing
	remain pending.
- Compression, encryption, cryptographic signatures/authentication, multipart archives, append/update of completed archives, deduplication, general salvage/partial extraction, stdin/pipe input for extract/list/verify, stdin sources and extraction-resume are deferred. Stdout archive creation IS included in MVP. Reading/extraction initially uses seekable archive files; stdout resume/durability/read-back verification is not supported.
- For stdout redirected to a regular file, inspect output descriptor identity where supported and reject source/output overlap or self-inclusion before traversal writes archive bytes. Shell redirection can already have truncated a file before the process starts; document that the command cannot undo shell-side damage. Add a self-inclusion regression for redirected stdout, sharing identity checks with file-output validation.
- Implementation is proceeding with the approved DRY priority and stdout MVP
	requirement; forward-only stdout creation is present in the initial code.
- v1 stores sparse files as logical dense bytes while retaining extent metadata for restoration; efficient sparse storage is deferred and must be documented as a space cost, not lost sparse metadata.
- No unrelated CSC journal migration, broad copy-engine rewrite, new dependencies, new semantic Dcs, or frontend work.
- Full metadata fidelity remains constrained by privilege, OS/filesystem support, and concurrent source/target modification, just as for CSC. Unsupported or skipped cases must remain visible in diagnostics and final reporting.
