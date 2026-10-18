# File and CSC Preservation Safety

The file abstraction and `csc` treat payloads and attached streams as data.
Best-effort metadata mode must not turn payload or stream corruption into success.

## CSC Archives

The experimental `ctb archive` command stores primary file contents under
existing Dc 392 as well as metadata and attached streams. Unlike a CSC journal,
a completed `.cscarchive` is intended to be self-contained. It has independent
magic `CTBCSCA\x01` and format identities, while sharing journal-style DcMixed
records and checksummed-section framing. Creation can stream to stdout without
seeking; only file output supports resume.

Archive writing is a configured journal-owned record engine, not a separate
writer implementation. Journal and archive readers share the modern scanner;
profiles preserve the existing layouts and recovery policies. File/descriptor
durability, archive member naming, and extraction remain workflow adapters.
Source discovery, capture, payload binding, recursive diagnostics, and hardlink
bookkeeping are shared through the journal capture module.

Live source guards retain directory inventories and deliberately interpreted
Apple companion dependencies. Source changes and timestamp-cleanup losses are
checked and retained before record publication or completion. These are live
consistency checks, not filesystem snapshots or a persisted resume inventory.

Archive support is **Partially Implemented**. Capture failures and warnings now
use the shared journal diagnostic representation in checksummed archive sections.
Completion is not a clean-capture guarantee: diagnostic archives return nonzero
and cannot pass lossless verification or normal extraction. Source-only failures
can roll back an uncommitted file-output member; stdout cannot roll back an
emitted member, and sink/persistence failures always abort. Diagnostic-bearing
archives cannot resume until durable failure-resolution semantics exist.

Large-stream memory bounds, durable inventory/recovery, replacement policies,
and platform/security coverage still have gaps. Do not treat checksums as
authentication or use an archive
as the only copy of important data. See [the command reference](cli/archive.md)
and [remaining implementation work](../issues/csc-usability/archive.md).

## Safety Checks

- Regular replacements are staged, checked for exact length and SHA-256, synced,
  and renamed. Sparse extent maps must cover the complete logical file without
  gaps, overlaps, or overflow. Strict mode audits staged data and metadata before
  replacement.
- Failed link or special-node creation leaves the existing destination intact.
- Strict directory metadata and stream failures propagate to the caller.
- Hardlinks are tracked across copy tasks; journal targets are destination-relative.
- Copy roots cannot overlap or refer to the same inode. Dangling symlinks remain
  valid source entries.
- Resume rechecks entries, including previously committed entries. New journal
  records retain filesystem identity for recovering the original directory target.
  Ambiguous older journals fail rather than guessing. Damaged, uncommitted journal
  tails are discarded at the last verified boundary before appending new records.
- Copy-based moves always require strict verification. Skipped entries prohibit
  source cleanup. Cleanup rechecks recorded identities and content, removes only
  copied entries, and uses non-recursive directory removal. Unexpected entries
  therefore prevent removal of their containing directory.
- Regular and AppleSingle audit payload reads use one open handle for probing,
  hashing, and timestamp completion, including completion after read failures.
  Completion refuses changed length or mtime and restores only observed atime,
  never mtime or timestamps on a replacement pathname. An audit is not a
  filesystem snapshot, and other read paths have the limits listed below.
- Untracked-file scanning propagates directory-open, enumeration, and metadata
  errors rather than reporting a partially scanned tree as clean. Missing
  manifest subtrees remain missing entries rather than scan errors.
- Malformed timestamp components are rejected before replacement and Dc
  encoding. Native time conversion is fallible; best-effort metadata application
  retains conversion failures as warnings instead of panicking.
- Journal failure finalization propagates pending-record commit errors rather
  than publishing a descriptor that implies those records were persisted.

## Overwrite Authorization

CSC and archive extraction refuse existing destination entries by default.
`csc --overwrite` authorizes replacement; the former `--always-overwrite`
spelling is no longer accepted. Explicit overwrite rewrites regular payloads
rather than reconciling metadata through an unrelated hardlink. Library callers
must choose `ExistingEntryPolicy::Replace` separately from `force_overwrite`;
forcing a rewrite is not permission to modify an existing entry.

This applies to directory/root metadata, streams, flags, and AppleDouble
companions as well as primary bytes. Newly created directories can be finalized
by the same live sandbox instance after opened-identity revalidation. Reopening
a directory or reading a journal intent does not grant that authority. Until
durable publication ownership is recorded, CSC resume needs `--overwrite` when
existing destinations must be repaired or their metadata restored.

`mv` requires explicit `--force` to replace a destination, on both rename and
copy-based paths. Native no-replace publication rejects entries appearing after
preflight, including dangling symlinks. Immutable flags are not automatically
cleared to force replacement. Linux and Windows have native no-replace
implementations; other Unix platforms currently fail closed where the required
regular-file rename operation is unsupported.

AppleDouble companions are staged, timestamped, synced, and published under the
same explicit policy. This is not yet an atomic or crash-recoverable transaction
with the main file. Directory metadata authorization is also not a lock across
subsequent path-based operations. Keep the concurrent-writer restrictions below;
these checks do not establish full race resistance or power-loss safety.

## Metadata Capture and Retention

Capture and reproduction are separate operations. Every captured field is stored
in the journal, including fields the destination cannot reproduce. Copy records
are committed before materializing each entry, so a strict metadata failure does
not discard that entry's original metadata. This is incremental capture, not a
preflight archive of the entire source tree.

New version-4 journals encode entities in checksummed semantic Dc mixed-mode
sections. Legacy binary journals and their `CTBMETA1`/`CTBMETA2` preservation
records remain readable and resumable. The current entity representation retains:

- Complete portable metadata, including birth time when reported by the OS,
  nanosecond timestamps, inspection time, ownership, mode, semantic flags, and
  raw platform flag bits with their originating OS.
- Extensible native records with signed integers, unsigned integers, and opaque
  byte values, without converting them through display strings.
- Complete discovered logical extent maps and each attached stream's original
  name, encoding, classification, descriptor, and payload bytes.

Standalone file JSON and Dc serialization includes the captured environment.
An explicit serialization context may replace a matching environment with a
reference to a definition already stored in the journal header. References
resolve to the shared Arc; missing definitions are errors. Different source
environments remain inline, including those of attached streams, and an explicit
unknown environment remains unknown. Recording or resuming a journal never
replaces source provenance with the machine currently reading it.

Lightweight commands capture the environment lazily when metadata first needs
it. The default capture makes no network requests; it uses cached IP addresses,
clock offset, and signature verification only when already available.
`--full-provenance` opts into lazy full capture, including network discovery.
Parallel workers share the same snapshot for the operation. A full workspace
boot eagerly captures and caches its environment before startup completes;
subsequent quick captures reuse that startup snapshot. An explicit environment
cache refresh replaces the startup snapshot as well.

Readers still accept `CTBMETA1` records. Those older records inherited omitted
environments from the session header. A legacy journal with no environment
definition remains without one when resumed. Older readers do not support the
newer preservation records or version-4 semantic journals.

Dc references use EL syntax rather than a serialization-specific semantic tag:
Dc 540 references a bound identifier framed by Dc 270..271. Dc 269 declares a
concrete value with its existing infix syntax, `<identifier> 269 <value>`; a
shared field uses `<field identifier> 269 540 <bound identifier>`. Dc 541 and
542 enter and exit lexical reference scopes. Inner declarations shadow outer
ones, and exiting restores the outer bindings. Document-local declarations do
not survive into another document. Explicit enclosing-scope APIs are an opt-in,
not ambient or process-global state. Typed declaration/scope helpers are
available in the Dc codec; consumers must read declarations and boundaries in
document order rather than treating a reference as a standalone value.

### Pending Reference Codec Work

- The typed scope/declaration helpers do not yet provide automatic dispatch in
  the general EL evaluator.
- Contextual JSON currently handles directly annotated shared fields using
  their Rust field names; Serde renaming and automatic traversal of arbitrary
  nested shared records need further work.
- `EnvDescription.extra` remains omitted by its pre-existing Dc attribute.
  JSON retains this overflow map; standalone Dc snapshots are not yet lossless
  for those unknown environment fields.

Linux capture includes Unix stat fields, the requested `statx` fields and support
masks, `FS_IOC_GETFLAGS`, and, where supported, `FS_IOC_FSGETXATTR` fields (including
project ID, extent hints, and raw extended flags) and inode generation. Enumerated
xattrs are read in full, including Linux POSIX ACL and security-label xattrs.
Metadata-only inspection skips the main payload hash, not streams or extent maps.
Unexpected native query errors abort capture instead of manufacturing empty
metadata. An API reporting that a feature is unsupported is distinct from a
permission error or a failed read.

Birth times and native fields without a reproduction implementation are compared
with the destination. Strict mode fails on differences; best-effort mode warns.
Malformed metadata and native read errors fail in either mode. Flag application
includes readback even when the separate copy verification pass is disabled.
Identity and allocation observations (for example inode number, mount ID, extent
count, and inode generation) are retained, not assigned to the destination.
Portable fields have their own audit policies, including the optional ctime check.

Linux has no general birth-time setter here. Consequently, strict copies and
copy-based moves commonly refuse new entries whose birth time differs. They must
not report such a destination as a fully reproduced original.

Journals containing native metadata, birth times, raw flags, attached streams,
or semantic warning/error diagnostics are retained even with
`--delete-manifest-after`, and after copy-based moves. Warning records also
require retention. Copy-based move failures retain the journal and report its
path in the error, including failures during source cleanup. A failure can
follow successful removal of earlier verified entries; the journal is needed
to diagnose or recover that partial move.
The result reports the retained journal path. Keep it with the copy: it holds
original metadata that the destination alone may not preserve. Journals contain
potentially sensitive security metadata and stream contents. They do not contain
the main file payload and are not standalone backup archives. Legacy journals
cannot recover fields that their writers never stored. A checksummed record that
cannot be decoded is an error, not permission to truncate it during resume.

## Filename Identity Requirement

Required by the operator's actual backup corpus; full filesystem/platform
coverage still needs qualification. Shared collision planning, AppleSingle
extension changes, and AppleDouble companion construction retain native names.
Linux regressions cover distinct raw names and NFC/NFD spellings in planning,
CSC copying/journaling, and source-free archive extraction. These tests do not
qualify native Windows execution or all destination namespace collision cases.
Thousands of files have different contents
but names differing only in Unicode normalization after Apple File Protocol
mishaps. These are distinct files, not duplicate spellings to clean up.

Preserve Linux path components as exact raw bytes, including invalid UTF-8;
preserve Windows-native names as exact UTF-16 code units. Never apply Unicode
normalization (NFC, NFD, NFKC, or NFKD) to stored or materialized filenames.
Display strings and normalized/case-folded comparisons must not become identity
keys. This applies throughout capture, archive/journal serialization, indexing,
resume, verification, and materialization. Linux-to-Linux operations must not
change a single filename byte. Cross-platform transcoding, when necessary, must
be exact and reversible; unrepresentable names are materialization errors.

For example, byte names `b"caf\xc3\xa9"` and `b"cafe\xcc\x81"` must retain their
separate identities and different payloads. Test directory components as well
as leaf names, invalid UTF-8, index rebuilds, and interruption/resume.

Archive creation must preserve both entries without regard to the eventual
extraction filesystem. If that filesystem aliases their names, skipping an
unmaterializable entry is acceptable only with a logged/displayed materialization
error and an unsuccessful operation result. Never silently overwrite one entry
with the other, normalize a name, or automatically rename it. Check collisions
with other source entries and existing destination entries before destructive
replacement, including after resume; retain original source/archive names and
payloads. Use actual destination namespace behavior, not an assumption that all
Windows or macOS filesystems normalize names alike.

## Stream Names

Stream names are tagged raw bytes or raw Windows UTF-16 code units. The journal
preserves non-UTF-8 byte names and unpaired UTF-16 surrogates on either host;
decoding a stream name does not require constructing a host filesystem path.
Lossy string conversion is for diagnostics only.

Native recreation converts names only at the filesystem boundary. Exact Unicode
transcoding is allowed; unrepresentable names and names colliding after conversion
are rejected before stream writes. Verification compares destination-native names
without changing the original journal name. Recreating a foreign stream also
requires the target's namespace to accept it: storing an NTFS name in a journal
does not make that name a valid Linux xattr. Stream write or payload failures
remain fatal even in best-effort metadata mode.

Dc serialization preserves UTF-16 names as exact code-unit lists and invalid
UTF-8 names as binary payloads. It never reconstructs native names from lossy
display strings. Windows stream batches validate names and payload descriptors
before writing any stream; invalid names cannot overwrite the primary data fork.

Standard WINE can reject native stream names containing unpaired surrogates.
That is a surfaced materialization failure, not a successful empty-stream copy.
Unsupported stream enumeration becomes a captured fidelity warning rather than
an empty successful enumeration. Read and write diagnostics retain native error
causes and the exact encoded stream name when known. csc stores materialization
errors in the captured entity's metadata and in its error journal, retaining the
original stream descriptor and payload. Continuing after an item error still
reports and journals the failure; journal persistence failures abort the copy.
These limitations do not restrict native Windows operations that can preserve
the same name successfully.

## Platform Status

Windows named data stream enumeration uses the native wide-character API and
retains its UTF-16 names and payloads. Native stream recreation and available
Windows security metadata are captured and applied; unsupported preservation
operations remain surfaced failures. Default materialization and auditing of
dense regular files are exercised under WINE and native Windows 7, including
payload corruption detection and timestamp comparison. Windows capture does not
invent POSIX mode, UID, or GID values: those fields remain absent, while native
attributes and security metadata retain the Windows representation.

Windows sparse capture uses allocated-range queries. Sparse materialization
uses the native control result and retained attribute, not WINE identity.
Auditing does not forgive missing holes just because WINE is detected: a volume
advertising sparse support can still produce a sparse mismatch, even when a
best-effort materialization returned a warning and correct dense payload.
Windows change time is captured and can be audited when requested; restoration
and newer-Windows runtime qualification remain pending.

### Remaining Windows Gaps

- Sparse fidelity and allocation-granularity coverage on different filesystems
  remain incomplete. POSIX ownership and permission modes cannot be verified
  as native Windows metadata.
- Payload-source access-time suppression/restoration and Windows change-time
  restoration remain unqualified.
- Descriptor-relative sandboxing and full identity, security, and reparse-case
  parity remain unverified. Dense-file tests do not establish lossless copying
  for every entity type or cross-device move.
- Newer native Windows versions have not been exercised by the available
  Windows 7 VM. Cross-compilation alone is not a Windows runtime test.

Universal native metadata capture supports BSD/macOS ACLs outside xattrs:
Darwin extended ACLs (macOS) and FreeBSD NFSv4/POSIX.1e ACLs are captured with
dual binary (acl_copy_ext) and canonical text representations, preserving
granular permissions, inheritance flags, and trivial ACL distinctions using
symlink-safe link APIs. The extensible journal retains their values, and
restoration applies them with strict validation. Do not treat current coverage
as proof that every metadata class on every source filesystem has been
enumerated.

## Remaining Limits

- These operations do not take a filesystem snapshot or exclude concurrent
  writers. A writer can still race between final verification and unlink, or
  replace path components used by path-based metadata operations. Quiesce writers
  and protect source and destination directories for destructive moves.
- A tree copy is not a transaction. Earlier entries may have been replaced when
  a later entry fails. Metadata-only reuse can update an existing destination
  in place; it does not provide rollback. Flags applied after rename can also
  fail after replacement has committed.
- Crash durability depends on filesystem and storage behavior. Successful fsync
  is not proof against hardware faults; no power-failure injection was performed.
- POSIX ctime is captured but is not generally assignable. Sparse verification
  checks hole presence, not identical physical allocation. * see below
- Access-time restoration itself can change Unix ctime. Audit completion does
  not yet return a semantically retained diagnostic for that side effect.
  Source capture now retains observed regular-file restoration ctime changes
  as warnings. Symlink capture and audit use no-follow, atime-only restoration
  with identity/mtime checks, but their pathname check and syscall are not
  atomic, and symlink restoration ctime changes still need retained diagnostics.
  AppleDouble timestamp failures are surfaced errors; a companion and its main
  payload are not committed as one atomic transaction. Stream audit reads also
  remain outside regular-payload handle completion.

## Validation

Run `cargo test -p ctb-io-file -p ctb-io-csc --lib` for regression coverage.
Compile Windows file tests with
`cargo test -p ctb-io-file --lib --no-run --target x86_64-pc-windows-gnu`.


## * Is Requiring Identical Holes or Allocation a Good Revision?

#### A. Identical Physical Allocation: **No**
Physical allocation refers to the underlying on-disk blocks (e.g. disk LBAs, physical cluster addresses, or allocation group extents queried via `FIEMAP`/`FIBMAP`).
- Two distinct files on disk (or across two filesystems/drives) will virtually **never** have identical physical block addresses.
- Physical allocation is internal to the underlying block layer and filesystem driver.

#### B. Strict 1:1 Logical Hole Extents (`actual_extents == expected_extents`): **Generally No (with caveats)**
Enforcing exact matching of extent boundaries (`actual_extents == expected_extents`) as a hard verification check creates significant issues across real-world filesystems:

1. **Filesystem Block Size / Cluster Size Differences**:
   - Ext4 defaults to 4 KiB blocks; XFS or ZFS may use 16 KiB, 64 KiB, or variable record sizes; FAT32/exFAT/NTFS use varying cluster sizes.
   - A 4 KiB hole created on ext4 cannot physically exist as a hole on a filesystem formatted with 64 KiB blocks—the destination filesystem must either allocate the block and write zeroes or merge it into adjacent extents.
2. **Unaligned Hole Boundaries**:
   - Filesystems can only punch holes on filesystem block boundaries. If an application created a hole that does not start and end on block boundaries, the unaligned boundary bytes must be backed by allocated data blocks containing zeroes.
3. **Extent Coalescing and Delayed Allocation (`delalloc`)**:
   - Modern filesystems (Btrfs, XFS, Ext4) dynamically merge adjacent extents or hold unwritten/delayed allocations in cache. The extent map reported immediately after writes or `sync` can differ in extent counts and splits from the source without any change to the logical file.
4. **Target Filesystem Without Sparse Support**:
   - If copying to a target filesystem that does not support sparse files (e.g., FAT32, ISO9660, or certain SMB/NFS mounts), seeking past a hole will materialize as physical zeroes upon sync. The SHA-256 content verification will pass (all bytes match), but strict extent verification would fail.

---

### Conclusion & Possible Incremental Improvements

The existing architecture already tracks and reproduces holes during read and materialization. Strict extent equality during verification would lead to false-positive verification errors whenever copying across different block sizes or filesystem types.

If you want to improve sparse verification without causing false positives:
- **Hole containment check (relaxed verification)**: Verify that any hole on the destination is contained within the expected hole ranges of the source, rather than expecting identical extent boundaries.
- **Space-saving sanity check**: Check that `stat.st_blocks` on the destination is smaller than the logical size if the source was sparse and the target filesystem is known to support sparse files.

(I think it's probably fine to accept just a check that it's sparse.)