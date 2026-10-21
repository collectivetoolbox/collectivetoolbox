# CSC and Archive Usability

Status: **Partially Implemented**. Consolidates the former milestone/archive
plans and the file-detection transcript from build `216aeaa258`. This is not
qualification as a production backup or snapshot tool. Keep originals until
their recoverability has been independently verified.

## Remaining Gaps and Verification

Items retained from earlier reports are not fresh reproductions. Old test
counts and cross-compilation results do not qualify current native behavior.

### File Detection and Output

Current regression assertions cover journal-v4 magic, gzip inner-version
identification, compatibility-mode decompression, and descriptor magic. Plain
ASCII ranking above an extension-only `.cscdesc` hint is intentional; an
extension does not establish valid descriptor structure.

- [ ] Reproduce the original compressed-journal matrix with Brotli, gzip, szip,
  zstd, and zlib, both with and without `-z`. Confirm content-backed inner
  journal versions rather than treating filename chains as validated contents.
- [ ] Check Brotli decompression probing: the old transcript evaluated zero
  bytes and reported only extension evidence.
- [ ] Check chain presentation and ranking: avoid presenting every historical
  journal/szip version as an equally supported content match. Distinguish
  extension-derived chains from validated chains.
- [ ] Review rich-output labels for compound formats (`Format: Unassigned`)
  and scores displayed as `125/100` or `255/100`. Determine the intended score
  contract rather than assuming accumulated evidence scores are percentages.

### Filesystem Fidelity and Scale

- [ ] Maintain a captured/restored/failed-closed/failed-open/untested matrix
  for NTFS, btrfs, and ZFS. Include streams, security descriptors/ACLs, reparse
  points, timestamps, attributes/flags, hardlinks, sparse files, short names,
  exact native names, and case/normalization collisions. Include btrfs project
  and extent flags and special file kinds; include ZFS xattr modes and dataset
  `casesensitivity`, `normalization`, and `utf8only` settings.
- [ ] Reconcile the old Windows metadata/reparse fail-open allegations with
  current behavior and [Windows findings](2026-10-08-Windows-csc.md). Inject
  capture failures and verify retained diagnostics and non-success exit status.
  Check strict sandboxed directory creation separately from cross-compilation.
- [ ] Complete bounded-memory capture/projection of large attached streams
  through shared journal batching, materialization, and verification. Primary
  payload streaming alone does not establish payload-independent memory bounds.
- [ ] Run and record hostile-tree copy probes on btrfs and native Windows NTFS:
  NFC/NFD and case-only names, invalid native name sequences, empty/small/large
  files, hardlinks, symlinks/reparse points, sparse files, unreadable entries,
  and ADS. Record exact names/hashes after first, repeated, and interrupted copy;
  measure journal bytes per entry and peak memory.
- [ ] Qualify actual destination namespace collision handling. Existing Linux
  native-name regressions do not establish collision safety on every filesystem.
- [ ] Measure resume redo: at most 500 files and at most the larger of the most
  recent file and 1 GB, or fail with a usable journal. Whole-file restart is
  permitted; silently shortened trees are not. Measure committed-entry recheck
  cost and avoid loading the whole tree merely to resume.
- [ ] Establish a journal/index memory and disk budget for 100 million entries;
  1 KiB overhead per entry is approximately 100 GB. Small fixtures are not
  qualification for that scale.
- [ ] Reconcile safety documentation with demonstrated platform behavior after
  runtime validation, especially [CSC safety](../docs/csc-safety.md).
- [ ] Qualify ZFS separately on BSD or Illumos; FreeBSD execution alone is not
  a ZFS result. Do not substitute Linux ZFS or mocks. VM preparation is a
  separate follow-up, not part of this documentation cleanup.

### Archive Safety and Recovery

- [ ] Replace remaining path-based metadata/stream mutations with anchored
  operations; test hostile concurrent source/archive/destination mutation.
  Live ownership checks and preflight are not snapshot isolation and do not
  eliminate check/use races. Persist applied-identity recovery authority.
- [ ] Harden Apple parsing and original raw/container/companion retention;
  complete staged main/companion publication and persistent local recovery.
- [ ] Complete durable diagnostic resolution/retry and intended-source inventory.
  Retained errors do not prove that all intended members were enumerated.
- [ ] Expand torn-record, partial-token/footer, completion, rollback, descriptor,
  flush/sync, broken-pipe, and diagnostic-persistence failure tests. Closed
  checksum corruption must not become successful ordinary recovery.
- [ ] Qualify native Windows archive containment/extraction and shared mutation
  authorization. Earlier archive extraction was Unix-only; Windows test
  compilation does not establish runtime support. Follow with FreeBSD runtime
  qualification; native macOS awaits infrastructure.
- [ ] Audit archive exposure of applicable CSC options: full provenance,
  source-change policy, filesystem boundaries, and Apple stream restoration.
- [ ] Reconcile `--recover` behavior with the older preflight/reject-diagnostics
  contract. Test recoverable committed members from incomplete/errored input,
  explicit loss reporting, and non-success partial status; do not silently
  present salvage as a complete archive.
- [ ] Test redirected stdout source/output identity overlap before emitting
  bytes, sharing file-output checks. Document that shell-side truncation occurs
  before the program can intervene and cannot be undone.
- [ ] Refresh generated CLI references/help and run a fresh executable smoke
  test after the shared-journal and overwrite changes, including stdout create,
  source removal, list/verify/extract, refusal of existing destinations without
  authorization, and extension-independent archive detection.
- [ ] Qualify crash/power-failure guarantees and large primary/attached payloads
  with bounded readers, short reads/writes, injected errors, and measured memory.

## Contracts to Preserve

- `ctb archive create/extract/list/verify` is separate from `csc -a`. Archive
  and journal use one canonical writer/reader and shared capture path, not
  separate serialization grammars. Preserve legacy journal compatibility.
- Archives are self-contained: no original source or `.cscdesc` sidecar is
  needed for read/extract/verify/resume. Reuse semantic file-data Dc 392 and
  derive-driven payload projection; no archive-specific metadata parser.
- One forward-only encoder handles file and stdout creation. Stdout contains
  archive bytes only; diagnostics/progress go to stderr. Reject stdout resume
  before output. A mid-member failure aborts without a valid completion marker;
  propagate final flush and broken-pipe errors.
- File resume validates committed content and source identities before truncating
  an uncommitted tail. Refuse completed-archive append/resume, mixed live-source
  snapshots, and silently accepted corruption. Completion alone is not clean
  capture; diagnostics and source inventory matter.
- Preserve exact native filenames and raw symlink targets under explicit policy;
  normalization is not identity. Reject escaping/absolute/drive/UNC member paths,
  duplicate/conflicting roots, unsafe hardlinks, and output self-inclusion.
- Default extraction refuses existing entries. `--overwrite` authorizes regular
  replacement/directory merging, not arbitrary deletion. Resume intent is not
  overwrite authority. Trusted symlink targets never waive write containment.
  Restore directory metadata after children and restrictive flags last.
- Explicit best-effort losses and continue-on-error captures remain visible and
  non-successful. Unsupported special entries must not mutate originals.
- Windows 7 NTFS results do not qualify every later Windows feature. Inode IDs,
  ctime, and physical layout are source facts, not extraction identity promises.

## Deferred

- Special-file overwrite expansion and recursive deletion to resolve collisions.
- Compression inside the archive format, encryption, signatures/authentication,
  multipart, completed-archive append/update, deduplication, stdin sources,
  pipe input for list/verify/extract, and extraction resume.
- Efficient sparse payload storage: v1 stores logical dense bytes while keeping
  sparse extent metadata; document the space cost rather than claiming loss.
- btrfs subvolume/ZFS dataset send/receive, the Dc runtime, renderer, graph trust,
  and graph editing. Existing archive approval supersedes the old milestone's
  instruction not to start an archive format.