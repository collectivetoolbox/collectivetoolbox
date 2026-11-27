# Dc-Native Personal MVP Roadmap

Planning baseline: 2026-10-02. Status: **Proposed / not implemented by this plan**.
Target: personal use on Linux and Windows in roughly one to two months, at
15-20 focused hours per week. The eight-week budget is 120-160 hours, not a
prediction that every acceptance gate will fit. Do not budget additional hours
on top of this to rescue scope.

## Remaining Gaps and Release Boundary

- A usable Dc-native application has not been established by this assessment.
  The renderer entry point passes strings through; the runtime has some Dc
  evaluation machinery, but its start entry point only logs document size.
- The file-info plan still lists native semantic graph documents and lossless
  archive representation as pending. Detection checkmarks do not establish an
  end-to-end document or preservation workflow.
- The current preservation journal is not a standalone archive. The safety
  notes describe Windows blockers, concurrency hazards, and incomplete
  guarantees, but newer Windows metadata code partially supersedes those notes.
  Native qualification, fail-closed error handling, and bounded stream capture
  remain risks. No runtime tests were run for this planning assessment.
- The intended workload is unusually demanding: over 100 million approximately
  220-byte files, plus roughly 16 TB of more conventionally sized files. A small
  successful demo is not evidence of readiness for that backup.
- Distributed global history, team replication, signing/publication workflows,
  and old-version retrieval are deferred, not abandoned or treated as solved.
- Private collaboration, recoverable encrypted backups, and metadata-private
  sync require a separate security design and independent review. MLS or a
  cryptography dependency alone does not establish those guarantees; see the
  [security architecture note](security-architecture.md).

There are two independently reportable outcomes: a **Dc workspace preview** and
a **preservation-qualified tool for a named filesystem/test matrix**. Neither
status implies the other. Until the preservation gate passes, retain originals
and an independent recovery path; do not use this as the only backup or perform
destructive moves on irreplaceable data.

## Product Decision

Build one personal **Dc ontology and file/archive workspace**, alongside the
CLI tools that make it practically useful. The central demonstration is:

1. Build the bundled Dc definitions into real, persistent global graph nodes.
2. Open a node, follow a relation, inspect its original content and effective
   assertions, and see the history behind the displayed result.
3. Create a resumable, verified archive of a real file tree; reopen it without
   access to the source and inspect its entries as Dc documents.
4. Select an entry in the same workspace, follow its format/metadata references
   into the ontology, and restore it through the same preservation engine used
   by `csc`.
5. Restart the application, reopen the workspace/archive, and resume interrupted
   work without losing identity, content, or the ability to explain a failure.

This is breadth-first across the application's actual boundaries, not a shallow
implementation of every planned feature. A new detector, codec, or widget is not
a milestone unless it removes a blocker in this demonstration.

## Confirmed Intent

These decisions were clarified with the operator on 2026-10-02 and take
precedence over assumptions in older documents.

- Every graph node is a document, character, and semantic entity. Dc is the
  implementation-independent representation, not metadata decorating a separate
  HTML application or a conventional graph database.
- Existing global Dc identities are permanent. Global content is append-only;
  changes to assertions or document revisions are represented by new nodes.
- Local/team graphs are mostly append-only, but deleted nodes and revisions
  may have their contents actually discarded. IDs remain allocated forever in
  that graph. Derived statements from deleted content must leave the index.
- Deleting a revision resolves the remaining valid assertions, usually revealing
  the previous revision. Deleting a document removes all revisions the user may
  delete. A local deletion marker can hide a global node but cannot delete it;
  removing the marker can reveal it again.
- Effective state depends on trust in identities, documents, and eventually
  graph-query rules. Trust levels and other user configuration are themselves
  nodes in the primary local graph, except general client PC settings, which are shared between all users. Workspace
  scopes separate personal/work identities, configuration, and signing keys;
  one login or subscription does not collapse those boundaries. Additional
  identities are intended to have separate password protection.
- Statement provenance and per-scope trust filtering are separate from the
  eventual shared statement index. Local trusted revisions may override global
  definitions. Safe-mode login must be able to ignore those overrides and use
  shipped behavior. Distrusting content is reversible; deleting its only
  retained payload is not.
- The existing CSV builder should emit the node contents at build time and
  validate that already-published node contents have not changed. Ship core Dcs
  and current revisions; eventually distribute historical nodes through the
  global graph. Distributed storage is not an MVP dependency.
- The first UI may use a browser, but the applications and runtime must not
  depend on page loads, routing, MVC, or request/response lifecycle semantics.
  The desired interaction is a persistent desktop-style workspace, adaptable
  to touch later, not a collection of web pages.
- `ctoolbox csc` remains the rsync-like synchronization tool. A new
  `ctoolbox archive` command will create/extract archives, sharing the csc crate
  and relevant options. An archive is a Dc mixed-mode document representing
  file tree, metadata (including log of errors/warnings), with identifying magic bytes.
- The first archive command is local and unencrypted, with no server or sync
  dependency. Users would protect its plaintext data and metadata using suitable storage.
- Future private/team collaboration treats servers and unauthorized peers as
  untrusted, including for document metadata and usage-pattern privacy. Work
  recovery authority must not expose personal data. These are stronger goals
  than ordinary end-to-end encryption and remain unqualified; the security note
  records the operator's requirements and unresolved feasibility tradeoffs.
- Capture is independent of the eventual extraction filesystem. Successful
  archive creation means the supported source information was fully retained;
  I/O or permission errors must be logged, displayed, and produce exit code 1.
  Extraction must report/fail or stop on information it cannot recreate, not
  silently downgrade to a successful lossy extraction.
- Archive creation followed by extraction on the same machine must have the
  same preservation semantics as synchronizing the same source directly.
- NTFS, btrfs, and ZFS are first-priority filesystems. Persistent on-disk file
  kinds and their data are critical, not optional because they are uncommon.
- The source includes thousands of files with different payloads whose names
  differ only in Unicode normalization. Filenames must remain exact native sequences, not normalized text:
  Linux names are bags of bytes. Preserve each distinct name and payload in
  archives, journals, indexes, resume, and Linux materialization. A destination
  unable to distinguish names may skip colliding entries with a reported
  materialization error, never silently overwrite or rename them.
- For the initial backup, the operator accepts an approximately unchanged source
  and a final repeat sync of the actively changing directory, like the current
  rsync workflow. This is not a promise of snapshot consistency. Source inode,
  ctime, and allocation observations are preserved facts, not requirements to
  manufacture identical destination identity or physical placement.

## Assessment of the Existing Plans

### File-info: useful implementation backlog, wrong critical path

The [authored problem statement](file-info.md#L188) already identifies Dc
documents, graph relations, and lossless archives as near-term goals. The large
Phase 5 checklist makes detection parity look like their prerequisite when it
is not. An unknown format can still be preserved byte-for-byte, represented as
a Dc document, and displayed with an explicit unknown-format result.

Keep Phases 1-4 as reusable groundwork, subject to integration validation.
Freeze Phase 5 except for correctness, security, or performance faults affecting
the selected workflows. Do not interpret its reported 88-case upstream result
as proof of universal detection parity; the same plan still lists gaps, and no
fresh verification was performed here.

Bring a bounded slice of Phase 7 forward immediately, interleave Phase 8 with
it, and implement only the Phase 9 lookups the explorer needs. Phase 6's complete
catalog and Phase 5's remaining language/base detection work can wait. Keep the
existing file-info checklist as a domain backlog, not the application's master
schedule, and do not erase unresolved boxes.

### Issue list: an inbox, not an execution queue

[issues.md](../issues/issues.md) mixes data-loss risks, everyday UX bugs,
compatibility ambitions, speculative features, and already-resolved tasks.
Without a release boundary, all of these compete equally with the core model.
Triage them by the user workflow they block, not by their proximity in the file
or how satisfying they would be to finish.

| Disposition | Existing issues/examples | Admission rule |
| --- | --- | --- |
| Now: preservation blockers | Windows tests/capture, flags, streams, FIFO/file kinds, journal, archive verification | Required by the NTFS/btrfs/ZFS preservation contract |
| Now: architectural workflow | Graph relations, native Dc definitions, actual runtime/renderer connection | Required by the ontology/archive demonstration |
| As encountered | Error severity, local authentication/access, lag/focus/navigation, startup or installation failures | Prevents using or safely diagnosing the selected workflow |
| Later | Detection priors, extra languages, FNV parameter coverage, additional codecs, MIME consolidation | Not needed to preserve or open the chosen data |
| Later | Minification, packaging proliferation, installer cosmetics, bundled browser work, IPC cleanup | No concrete blocker to the chosen Linux/Windows entry path |
| Outside this MVP | Relay/hosting, collaboration, remote accounts, distributed history, monetization | Preserve architectural room; do not build services now |

The graphics issue proposing Linux `dlopen` shims is not needed for the current goals. Do not revive it as an MVP requirement; it's a further-along idea. Likewise, the
older Redb suggestion is not a reason to replace the current storage backend.

The [2019 prioritization](../old/2019may7n3-Report.md#L339) puts data loss,
security, correctness, and accessibility ahead of new features. Keep that order
within the selected workflows; do not use it to turn every defect in every
experimental subsystem into an MVP blocker. Its UI vision also argues for one
reusable document/selection model, not separate permanent concepts for a file
picker, an entity picker, and a future query-result picker.

### What to reuse, not restart

- [Global graph storage](../src/storage/models/graph_impl.rs) already has
  system-block allocation and packaged-node publication. Audit and extend its
  transaction/identity guarantees rather than starting a second graph store.
- [Dc code generation](../src/build_support/dc_codegen.rs) already reads category
  CSVs. Extend the shared build/validation pipeline with native-node output;
  generated Rust names/constants may remain conveniences, not semantic authority.
- [Graph layout](../src/storage/minimal/data/global-graph-layout.csv) already
  separates Unicode, ordinary Dcs, formats, reserved data, and system documents.
- [Runtime](../src/runtime/runtime.rs), [renderer](../src/renderer/renderer.rs),
  and IPC provide pieces, not evidence that the full document path works.
- [csc](../src/io/csc/csc.rs) and the shared file abstraction provide copying,
  journaling, verification, and metadata work to extend. Avoid a second archive
  extractor that applies metadata differently from direct synchronization.

Specific first-week probes, not a full code audit:

- [Windows stream capture](../src/io/file/streams/windows.rs#L63) uses a whole
  file read for each attached stream and retains the results. A large ADS can
  therefore defeat a streaming main-payload implementation. Test bounded-memory
  source handles/chunks across capture, journal, archive, and restoration.
- [Windows metadata capture](../src/io/file/metadata/windows.rs#L102) exists,
  including security/reparse work, but handle-information failure can return
  success without that information; reparse-query failure can be a warning.
  Test failures at the API boundary and propagate failures to capture status.
  Do not assume missing data means an unsupported property.
- [Non-Unix strict traversal](../src/io/file/sandboxable_dir.rs#L271) explicitly
  rejects strict sandboxing. Confirm the exact Windows entry paths and close
  the protection gap before qualifying extraction.

## Architecture to Settle Early

The aim is a few durable contracts, not a complete ontology or a generic new
framework. Record the exact encoding/API choices with fixtures before making
them permanent. The decisions below are requirements/proposals, not claims
that the current implementation already satisfies them.

### 1. Identity, statements, trust, and effective state

Use graph-qualified node references; names and labels never become identity.
Preserve the distinction between global IDs, local references, short Dcs, and
typed format IDs. Reference a particular revision when reproducibility matters;
resolve a canonical/current view only when explicitly requested.

Relation instances need their own identities because qualifiers and withdrawals
can refer to a particular assertion, not merely an identical triple. Start with
the predicates needed for definition labels, types, relationships, provenance,
revision/withdrawal, and archive entities. Reuse existing definitions and allocate
new ones deliberately. Do not turn category membership, prose, or CSV ordering
into inferred semantic authority.

The effective view is a deterministic, rebuildable projection of available
statements under an explicit scope and trust policy. Do not store one universal
"latest revision" as the authoritative state. Statement provenance must retain
its source node/revision and asserted identity/signature evidence, distinct from
the active user's trust decision. Caches need a scope/policy generation as well
as a content generation; distrust or changed policy invalidates effective views.

Start with one usable scope and the current user's trusted assertions, plus
trusted bundled definitions and an explicit distrust operation. Prove the API
with a second isolated test scope so identity and configuration are not global
singletons. Full trust-query rules, manager approvals, key-management UX, and
shared indexes are later work. Do not manufacture signature verification when
only local provenance has been established, or make trust confer I/O privileges.
An option on the login screen will allow to enter recovery/safe mode even when graph-stored
configuration is broken.

Global immutability and local deletion are different policies. Reserve graph IDs
after deletion; remove derived statements and invalidate dependent projections.
Resolve surviving trusted revisions after deleting a revision. Distinguish
delete-revision, delete-document, distrust, and local-hide-global-node operations.
Never cascade deletion into a referenced global node or another graph's content.
If payload storage is deduplicated, garbage collection must account for remaining
references; indexes/caches must not retain a deleted payload as hidden history.

Demonstrate one local presentation/label override of a global node and safe-mode
recovery. This proves the boundary without implementing arbitrary user-modified
runtime code. A full local graph editing UI is not required for this milestone.

### 2. Build-produced native documents

Extend the CSV validation/build path to emit real Dc-encoded node bodies plus a
versioned inventory mapping stable identity to exact content digest and origin.
The inventory is a build/publication aid, not a replacement semantic database.
Known historical bodies must be immutable; changed assertions create new system
nodes according to the existing allocation layout. No automatic renumbering on
sort order changes, and no duplicate new nodes on an unchanged rebuild.

Keep source CSVs as authoring/bootstrap input. Runtime graph/definition lookup
must consume the generated documents and their indices, not quietly reparse CSV
as a second authority. Keep a minimal decoder/bootstrap vocabulary sufficient to
read the vocabulary itself; do not require graph access to decode a graph record.

Emit inspectable native artifacts during the build and install/bundle the core
definitions and required current-revision closure for offline use. Choose packing
based on existing storage facilities; do not make one filesystem file per node
an architectural requirement. Until network history exists, keep historical
artifacts in a durable local/release archive rather than discarding them.

Preserve a resolver boundary distinguishing bundled, locally available, deleted,
and not-yet-fetched history and documents. Future signed distribution can fill missing content
without changing identity or document encoding. Do not claim erasure coding,
network availability, or historical replay of dependencies is solved by this.

The first semantic migration may cover a small representative set. Before
cutover, account for every field in the current dataset: semantically encode it,
preserve it explicitly pending interpretation, or mark migration incomplete.
The goal is not to finish all future format catalogs first.

### 3. Persistent runtime and output-independent interaction

Use this ownership boundary:

```text
persisted Dc documents + graph policy
    -> runtime instance (state, events, capability requests)
    -> semantic Dc snapshot/update (stable document/element identities)
    -> renderer (lowering, equivalence, decoration, projection)
    -> I/O adapter (browser first; native/text targets later)

input adapter -> semantic event -> runtime -> next snapshot/update
```

Displaying a Dc document executes it, as with PostScript. The runtime owns that
execution, document state, focus/selection semantics, commands, and event ordering.
The renderer lowers/projects the runtime's resulting snapshot rather than
independently re-executing the source document or owning its application state.
Browser DOM, HTML, URLs, and CSS are outputs/adapter concerns, never the canonical
document. A URL may identify a document without defining its lifetime.

Begin with retained state and a bounded event/update protocol. Specify snapshot
generation, input target identity, resize, cancellation, and stale-frame handling.
Coalesce replaceable display updates under backpressure; do not drop commands.
The first implementation may send complete bounded snapshots; efficient diffs
can follow without changing the application model. Huge file trees must use
bounded views, not huge snapshots.

Use a modest desktop-style arrangement: a document/entry list, an active document
view, an inspector/history view, and persistent job progress. Reuse selection and
document-reference semantics for nodes and files. Preserve independent state for
two open documents, keyboard focus, resize, and long-running work. Defer docking,
a visual forms designer, general rich-text editing, and a wholesale shell rewrite.
The explorer's view/application description must itself be a Dc document, not
just a Handlebars page querying graph data. Reusable native runtime primitives
are acceptable; the full application need not be self-hosted Dc code yet.

Keep semantic geometry/dimensional information until projection. Only implement
the 2D/text subset needed now; do not force every document into browser layout or
require a general n-dimensional renderer before the first explorer works.

Follow the [renderer/runtime intent](renderer-flow.md): permissive execution and
recoverable malformed structures are distinct from strict artifact validation.
Normal display attempts execution even with unknown or malformed constructs;
it is not a non-executing preview. Only a bounded amount of pure computation is
available without user authorization. Reading local nodes, persistent storage,
file/network I/O, large memory use, high CPU use, and high-resolution timers
require explicit capabilities, denied unless authorized by the user. Parse/runtime
errors strengthen the warning around capability grants, not prohibit execution.
MVP denial can be functional while auto/manual mocking stays deferred and visible
as such. Preserve unknown/malformed content and support recovery/continuation.

Raw byte/token inspection, parsing, and deserialization are distinct from running
the inspected document for display: they must not implicitly execute its quoted
payloads. If indexing requires evaluation, it must use the capability-gated
runtime rather than gain ambient privileges from the indexer.

### 4. One preservation model, separate operations

Treat live filesystem capture and archive entry reads as sources for the same
bounded transfer/materialization machinery. Archive writing is a destination
able to retain all supported captured information. Share metadata interpretation,
stream handling, verification, and reporting; retain distinct operation policies.

Keep long-running jobs in the workspace/csc services, independent of view and
browser lifetime. Closing a view must not kill a backup. UI and CLI submit the
same operations and observe the same durable job/journal identity; reconnecting
observes the existing job rather than submitting a duplicate operation.

| Operation | Success means | Not a success |
| --- | --- | --- |
| `csc` synchronization | Requested entries satisfy the declared destination policy, with durable resumable progress | Silently skipped payloads or concealed metadata failures |
| `archive` creation | Self-contained verified Dc archive retains supported source data, metadata, names, and log | Journal-only output, external source dependencies, unreadable entries hidden by exit 0 |
| `archive` extraction | Requested entries materialize under the declared preservation contract | Silently discarding fields the destination cannot reproduce |

Define a versioned mixed-mode framing contract: magic/version, typed Dc metadata,
bounded payload chunks, lengths/digests, entry identity, commit records, and an
unambiguous finalized/completeness state. Indexes must be rebuildable from the
archive; they cannot be the only copy of essential metadata. Unknown record
handling must not produce false claims of full restoration.

Filename identity must follow the
[exact-name preservation requirement](csc-safety.md#filename-identity-requirement).
Unicode normalization, case folding, and display strings must not replace raw
identity keys in capture, archive lookup, journaling, resume, or verification.
Destination collision checks are separate from source identity: use the actual
filesystem's name-equivalence behavior without rewriting stored names. Do not
assume all Windows or macOS filesystems have identical normalization behavior.

Retain raw path components and encoding, stream names/content, metadata precision,
unknown native fields, hardlink identity, sparse logical content, and source
provenance. Preserve unsupported-to-recreate information without pretending it
was applied. Inode numbers, ctime, and physical allocation are source observations;
do not demand identical destination values. Birth time, permissions, streams,
and other restorable properties need explicit destination capability policies:
an inability to reproduce an intended property remains a reported failure under
the default extraction contract, not a silent downgrade to an observation.

Reuse the native file/metadata Dc schema for journal records and archive entries,
while allowing separate journal and archive framing. They will both be Dc mixed mode documents consisting primarily of a tree of files, but with contents absent in a journal. Define what happens to
existing interrupted journals before changing their writers: provide a validated
reader/migration path or refuse with an actionable message and retain the old
file. Never reinterpret old bytes as the new schema. Version the archive and
keep golden compatibility fixtures before calling its format archival/stable.

Do not conflate archiving a filesystem tree with byte-identical reconstruction
of arbitrary decoded tar/zip containers. Initially preserve existing archives as
opaque original files; full lossless structural re-encoding can wait.

Compression is optional and independent of correctness. Use an existing proven
codec only if it does not compromise bounded memory, seeking, or resume. No new
compressor or dependency is required to establish the archive contract. Compression is not needed for MVP. Journals should not use compression. Compression should not be part of the archive format, but a layer added outside.

### 5. Scale, restart, and security are functional requirements

- Bound RAM, open handles, queue sizes, and per-entry metadata buffering. Stream
  main payloads and attached streams. Store work queues and large hardlink maps
  on disk rather than growing them with tree size.
- Keep discovery state durable. Resume must not require restarting all work or
  loading 100 million entries into memory. State whether restart revalidates
  committed entries, how much it reads, and how changes are detected.
- Benchmark large flat directories as well as deep trees; a bounded traversal
  stack does not imply bounded per-directory buffering. Avoid full-tree format
  detection/decompression during backup. Detection is optional inspection work,
  never a prerequisite for preserving an opaque file.
- Require stable/quiesced input or a supported snapshot for meaningful consistent
  backups. Interrupted/restarted work on a changed live tree must detect/report
  the change, not silently claim a point-in-time snapshot.
- For the initial approximately-static workload, record that consistency policy
  in the operation and test a final repeat sync. It may reconcile changes but
  does not retroactively turn the earlier archive into an atomic snapshot.
- Specify entry/chunk commit boundaries, torn-write handling, finalization,
  verification, restart cost, and index rebuild before declaring archive v1
  stable. Distinguish process-kill testing from actual power-loss qualification.
- Fail closed on path traversal, destination symlink/reparse attacks, name
  collisions, truncation, checksum failures, metadata read errors, and unsafe
  special-file handling. Merely listing a file does not authorize reading a FIFO
  as a byte stream or accessing a device.
- Do not capture unrelated host environment/network identifiers by default.
  Security descriptors and streams themselves may be sensitive; make artifact
  access controls and privacy part of the capture contract.

### 6. Security boundaries without a collaboration prerequisite

Apply the [security architecture note](security-architecture.md) before freezing
identity or archive formats: separate identities from rotating keys, exact Dc
bodies from security envelopes, semantic trust from read authorization, and
recoverable document history from expiring communication secrets. Private indexes
must respect readership boundaries; a shared team graph is not one universal
decryption domain. Shared PC settings must not become a private-key store.

Include a 1-2 hour initial threat-model/decision record within M0's existing
budget. It is not an estimate for solving metadata privacy or recovery. Keep
archive encryption and MLS outside this local MVP. Before private cloud/team
use, require explicit recovery/revocation policies, a vetted protocol/profile
selection, metadata-privacy feasibility work, and independent security review.
If privacy goals cannot be met by the proposed infrastructure, report the gap
and seek a product decision rather than silently weakening them.

## Delivery Sequence and Budget

These are timeboxes for deciding the next scope tradeoff, not implementation
estimates or permission to waive acceptance criteria. Work on one milestone at
a time; interleave the two tracks by finishing small vertical slices. Aim for a
visible demonstration every week, including useful CLI demonstrations.

| Slice | Approximate window | Budget | Demonstrable outcome / exit gate |
| --- | --- | --- | --- |
| M0: contracts and risk probes | Week 1 | 6-8 h | Record identity/deletion, initial security boundaries, and preservation decisions; run a small NTFS and btrfs/ZFS fixture capture; measure current copy/resume memory and journal growth; identify blockers rather than assuming Windows works |
| M1: real Dc nodes | Weeks 1-2 | 16-22 h | Builder emits representative real global nodes; open/export/reload by identity, append one changed assertion, rebuild a scope/trust-aware index; old bytes and IDs unchanged; unchanged build produces no new nodes; test local revision deletion |
| M2: standalone archive | Weeks 2-4 | 24-32 h | Archive a representative real tree, interrupt/resume, verify without source access, extract, compare with direct csc results; clear exit 1 on capture errors; persist native Dc metadata rather than another temporary schema |
| M3: Dc-native explorer | Weeks 4-5 | 16-24 h | Persistent runtime renders actual graph nodes and archive entries through renderer/IPC; follow links, inspect history and select entries without page reload; bounded updates and keyboard interaction |
| M4: joined daily workflow and scale | Weeks 5-7 | 24-32 h | Initiate/monitor/cancel/resume existing copy/archive jobs from Dc workspace; demonstrate local override and safe mode; continue target filesystem fixes and increasing-scale trials; finish current-dataset migration before definition-lookup cutover |
| M5: qualification and release | Weeks 7-8 | 12-18 h | Linux/Windows use, recovery/corruption tests, replay/deletion tests, bounded-resource evidence, migration notes, and exact tested support matrix |

Total planned work is 98-136 hours, leaving 22-24 hours of the respective
eight-week budgets for integration and surprises. The windows overlap only to
allow moving the next small slice forward, not to encourage concurrent feature
development. Start scale/Windows probes in M0 and repeat throughout M2/M4; do
not discover platform feasibility in the last week.

At four weeks, a useful partial delivery is native node artifacts plus a
self-contained archive/copy preview on an explicitly qualified subset. It is
not the Linux-and-Windows MVP if either platform or critical source file kinds
remain unsupported. If M2 consumes its timebox, reduce explorer features before
reducing preservation safety. If NTFS safety blocks progress, name it as the
blocker and ship only an honestly limited preview.

The four-week outcome is a checkpoint, not a second smaller promise of full
qualification. A 100M-entry/16-TB backup may take longer than an implementation
timebox just to execute and verify. Calendar time for unattended trials and a
second usable copy of the source must be planned separately from coding hours.

### First work queue

1. Record the remaining contracts below and build the smallest adversarial
   fixture corpus from the operator's actual filesystem classes.
2. Add a build-generated node fixture for an existing Dc, a format, and two
   qualified relation instances; prove exact persistence and immutable updates.
3. Open those nodes through the existing graph API and a diagnostic text view.
   Do not finish the ontology before making its first node visible.
4. Capture the same small tree through direct `csc` and the proposed archive
   source/destination interfaces; establish a shared comparison oracle.
5. Make that archive independently readable and resumable, then expose the
   node/archive view through the persistent runtime-renderer path.

## Acceptance Evidence

### Graph and runtime

- Golden native Dc records, identity/content immutability checks, deterministic
  rebuilds, incremental import idempotency, and intentional-update fixtures.
- Relation instances with qualifiers; two otherwise-identical triples must not
  collapse; withdrawal/revision resolution is deterministic and inspectable.
- Delete a local revision and rebuild: no derived statement survives from that
  payload, the ID remains reserved, and the remaining trusted revision resolves.
- Change trust without deleting bytes; the result changes only in the intended
  scope and can be restored. A second scope cannot inherit the first's private
  configuration or authority. Verify global overrides and safe-mode bypass.
- Keep graph-stored user configuration through restart. Show distrust, deletion,
  locally hidden global content, and unavailable history as distinct states.
- Install/open offline using bundled current dependencies; distinguish absent
  remote history from an empty history or a nonexistent node.
- Display a document by executing it, including bounded pure computation that
  produces visible output. Unknown/malformed constructs must not require a
  strict-validation pass before attempting execution and recovery/continuation.
- Test denial without user authorization for local-node reads, persistent
  storage, file/network I/O, large memory use, high CPU use, and high-resolution
  timers. Cover compute interruption, event ordering, and adapter reconnection.
- Separately test raw inspection/parsing/deserialization and quoted literals:
  these must not implicitly execute the inspected or quoted payload as a program.
- Change selection/state repeatedly without navigation or runtime restart;
  replay semantic input against the same runtime without a browser dependency.

### Preservation and scale

- Test NTFS on Windows and btrfs/ZFS on their actual deployment hosts. Mock tests
  and Windows cross-compilation are useful but do not replace native runtime
  evidence. Record filesystem/OS versions, privileges, and unsupported features.
- Include huge and empty files, very high entry counts, raw/unrepresentable
  names, duplicate/case-colliding names, streams, ACLs, flags, hardlinks, sparse
  files, symlinks/reparse points, and persistent special kinds actually present.
- Make Unicode-normalization collisions an M0 fixture and release gate: include
  NFC/NFD-equivalent filename and directory-component pairs with different
  payloads, plus invalid UTF-8 names. Assert byte-exact names and separate
  payloads through Linux sync, archive round-trip, index rebuild, and interrupted
  resume. On a destination that aliases names, including pre-existing entries,
  assert reported materialization errors and no collision-driven overwrites or
  automatic renaming. Archive creation must still retain both entries exactly.
- Compare capture -> archive -> extraction with capture -> direct sync under
  identical policies. Verify payload/stream bytes and the full metadata contract,
  not just visible filenames and sizes.
- Inject permission/read/write errors, disk-full conditions, corrupt records,
  truncated payloads, and interruption around commits. Test resumed output
  against uninterrupted output. Never promote an incomplete archive to complete.
- Measure entry throughput, payload throughput, peak RAM/handles, journal/index
  bytes per entry, time to resume, revalidation I/O, and time to rebuild indexes.
  Use rising entry counts and real source samples, not just a giant single file.
- Before full-volume qualification, run at least one million-entry trial with
  repeated interruptions. It can falsify linear-memory/restart assumptions, but
  it does not substitute for measurements at the actual workload size.
- Extrapolate duration and disk requirements before the full workload trial.
  Approximately 100 million extra bytes per byte of per-entry overhead makes
  metadata/index size a first-class concern: even 1 KiB/entry is about 102 GB.
  Record measured estimates as estimates, not successful full-volume tests.
- Agree an acceptable restart delay and memory/disk budget in M0. The tool must
  resume useful work inside that budget across the operator's shutdown cadence.

The existing narrow regression command from [csc safety](csc-safety.md) is
`cargo test -p ctb-io-file -p ctb-io-csc --lib`. Extend nearby test suites for new
contracts. No tests may listen on network ports. Use in-memory/mock transports
for automated protocol tests and explicit manual runtime checks where needed.

## Unresolved Decisions, Not Hidden Assumptions

1. **Trust and deletion encoding:** define the initial trust/provenance records,
  deterministic precedence/tie behavior, and minimum non-content ID reservation
  after deletion. Revision deletion resolves surviving trusted statements; that
  behavior is settled, not an open product question. Local deletion must not be
  advertised as secure erasure from existing backups. Arbitrary trust rules and
  team-wide deletion propagation remain deferred.
2. **Immutable build history:** where is the authoritative ledger of allocated
   IDs and historical digests kept, how are concurrent development branches
   reconciled, and which artifacts are retained before distribution exists?
   Do not make build-order-dependent ID assignment the permanent solution. Global graph nodes will be version controlled in Git, and *must* remain unchanged after they are first committed, with new data added as statements in subsequent system-block nodes.
3. **Preservation policy:** the distinction between restorable properties and
  source observations is agreed. Produce the field-by-field NTFS/btrfs/ZFS
  matrix, especially birth-time limitations and privileged fields, and map
  failures to explicit operation outcomes without weakening no-silent-loss.
4. **Operating envelope:** encrypted/compressed
   NTFS features in use, actual file-kind inventory, acceptable shutdown/restart
  cost, working disk space, and an affordable native Windows test path. The
  initial approximately-static source assumption is agreed; snapshots can be
  added later without implying that concurrent-writer safety exists now.
  5. **Private collaboration security:** exact permitted metadata leakage, malicious
    server/peer observation and collusion assumptions, recovery authority,
    password/device recovery, key rotation, and post-quantum protocol selection.
    Boundaries belong in M0/M1; resolving the full design and obtaining independent
    review are gates before private cloud/collaboration, not archive prerequisites.

Resolve only decisions needed by the next milestone. Distributed consensus,
global data placement/reconstruction, full permission mocking, a query language,
and a comprehensive ontology are not decisions required to write the first
stable native node or preservation fixture.

## Keeping the Project on Track

- Maintain this roadmap as the execution plan; leave [issues](../issues/issues.md) as the intake
  inbox and detailed domain documents as reference backlogs.
- Keep at most one active implementation ticket. A ticket should fit one or two
  focused sessions and state the demo, acceptance test, and explicit non-goals.
- Spend 30 minutes weekly demonstrating the integrated build, recording failures,
  and choosing the next three tickets. Track usable workflows and unresolved
  safety blockers, not the number of format checkboxes completed.
- A new idea goes into the inbox unless it blocks the current acceptance test,
  fixes a safety failure in scope, or is the smaller route to the same outcome.
- Stop a component when its milestone contract is met. Improvements discovered
  while implementing it become follow-up tickets, not automatic scope additions.
- Reserve research/fun time inside the 15-20 hours, not as extra hours. Unused
  contingency may fund it; research findings do not silently become MVP gates.
- At each timebox boundary report: failing/skipped tests, remaining gaps,
  evidence gathered, and the next decision. Never mark a feature complete when
  required platforms, formats, deletion behavior, or scale tests remain pending.

## Sources and Confidence

Prioritize the operator's clarification above, then explicit authored design
intent, then implementation observations. A recent generated checklist is not
more authoritative than an older, still-valid statement of purpose.

| Source | Recency/provenance | Use in this plan |
| --- | --- | --- |
| [conversation](old/2026-10-01-implementation-plan.md)  | File last changed 2026-10-02 | Discussion and planning of this roadmap |
| [file-info authored brief](file-info.md#L188) through its identity constraint | File last changed 2026-10-01; brief distinguished from the generated checklist | Identity, native statements, archive and format motivations |
| [issues](../issues/issues.md) | Last changed 2026-09-30 in inspected history | Current inbox, not a verified defect inventory |
| [renderer-flow](renderer-flow.md) | Last changed 2026-10-01 | Current runtime, permissions, and n-dimensional rendering intent |
| [outline](outline.md), [projects](../issues/projects.md) | Outline last changed 2026-07-24; mixed-age planning notes | Canonical identity/history and local/shared graph workflows |
| [2019 report](../old/2019may7n3-Report.md) | Authored date 2019-05-07; not tracked at this path in current Git index | Document-centric vision, safety priorities, semantic selections, and lessons from earlier implementation attempts |
| [csc safety](csc-safety.md) | Tracked engineering notes; last-change date not established in this assessment | Preservation caveats and candidate tests, not independent certification |
| [security architecture](security-architecture.md) | Operator clarification and primary standards/library sources checked 2026-10-02 | Security boundaries, recovery/privacy tradeoffs, and pre-collaboration gates |
| Linked source entry points | Read on 2026-10-02; no test execution | Reuse opportunities and specific integration gaps |

### Pending Work Beyond This MVP

Distributed/team graph protocols and deletion propagation; global signing,
replication and historical retrieval; comprehensive format/detection parity;
additional filesystems/platforms; damaged-media recovery; archive-format
structural round-tripping; richer authoring, data matching/ETL/BI, queries and
reports; complete capability mocking UI; mobile/touch refinement; native/raster,
printing and higher-dimensional output; self-hosted visual tooling; hosting and
monetization; complete trust query/rule evaluation and multi-identity password/key
management UX. Preserve the relevant boundaries now, implement these only when
their own user workflows become the next target.

Private cloud/collaboration also requires the security note's open recovery,
key lifecycle, PQ migration, metadata privacy, and independent-review work. The
local plaintext archive MVP must not be described as providing those guarantees.