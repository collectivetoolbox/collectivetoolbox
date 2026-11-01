# Duplicate Node Recovery Audit

## Remaining Gaps

- Full-workspace tests, full repository linting, and non-Linux platforms were not
  run. Focused musl tests and production Clippy are recorded below.
- The original deleted Run 1 ledger is unavailable. Its bodies are verified
  against the committed deterministic reconstruction, not independently observed
  original bytes. The recorded historical replay was inspected but not rerun.
- An edit-by-edit history cannot be recovered from phase commits. The available
  records establish allocation contents and phase boundaries, not every transient
  source edit or filesystem operation.
- The hidden-operational-state finding remains open. Candidate-order inference,
  asymmetric inbox ownership checks, live recovery-count assertions, and UTF-8
  interrupted-write recovery were corrected during the follow-ups below.

## Verdict

Keep the recovered ledger. Do not retract all post-incident nodes or replay the
allocation scripts against the current live repository.

The authoritative recovery is internally consistent with the committed evidence.
The incident ledger prefix is unchanged, every new ID is accounted for, branch
definitions are preserved, and current CSV reconciliation converges without any
new nodes. The active statement multiset equals the incident Run 2 multiset after
substituting the explicitly recovered Run 2 IDs; this comparison does not apply
the Rust index's implicit canonical-branch filtering.

Audit started at `ea9e1e15b423a09032c272a59123e061be9f60be` against incident
`a9046402890cfcccc1b50d2c2e76a4889694814b`. Operator commits continued during the
audit, including committing the initial audit script. No graph files were edited
or new nodes allocated by this audit.

## Verified Evidence

Run `python3 -B scripts/graph-recovery/audit_recovery.py` for the saved read-only
checks. The command uses committed manifests and ledger data, not hidden
initialization markers or high-water directories.

- All 17,833 incident-commit node definitions retain their exact original prefix
  bytes.
- All 57 erased tail IDs have their exact reconstructed Run 1 definitions.
- The fixed-point dependency closure contains 602 ambiguous IDs and leaves 24
  overwritten IDs unambiguous.
- Every raw candidate matches its historical branch body. Every resolved
  candidate matches that branch's dependency substitutions. Every terminal
  ambiguity record names the expected two resolved candidates.
- Step 2a: 2,465 exact allocations, ending at `l8589954889`.
- Step 2b: 2,838 exact allocations, ending at `l8589957727`.
- Step 3: 616 exact allocations, ending at `l8589958343`.
- All 5,919 additions have matching manifest lines and tracked journal items.
  Both boundary ledger hashes of every phase match. There are no additional
  unmanifested IDs in the current ledger.
- All journal record checksums, manifest digests, pending/committed pairs, and
  transaction-chain boundaries match.
- The active/history bundle file inventories and hashes match the final tracked
  journal. Rust tests additionally verify decoded tokens, partitioning, and the
  complete disjoint union of both bundles.
- All 13,731 active Run 2 statement occurrences are preserved, including escaped
  atom cases. Comparison uses a multiset, so duplicate or missing occurrences
  cannot disappear through set deduplication.
- Four saved earlier high-water snapshots each match the 17,833-node prefix.
  The final moved snapshot matches all 23,752 nodes. The remaining external
  high-water ledger also matches all 23,752 nodes.

## Rust Findings

### Resolved: Candidate Order Was Incorrectly Treated As Authority

The original report's explanation was incorrect: knowing which historical run
produced each candidate does not make either alternative authoritative. The
historical ID remains ambiguous. Recovery evidence can justify replacement
references in particular statements, not a general preference between the
alternatives.

Removed the candidate-order filtering and guessed allocation region from
[seed_graph.rs](../../src/formats/dcdata/seed_graph.rs), and removed automatic
earlier-branch retirement from
[graph_ledger.rs](../../src/formats/dcdata/graph_ledger.rs). Activity now follows
explicit revisions rather than position in an ambiguity list. Regression tests
exercise both candidate orders, retain both statements and their references,
and require an explicit retraction to retire a candidate statement.

The committed recovery's explicit retractions remain in place. After removing
the inference, the live CSV planner still proposes zero changes, and the
independent active-statement multiset comparison still matches the incident.

### Resolved: Interrupted UTF-8 Writes Are Recovered As Bytes

[journal.rs](../../src/build_support/ctbnbdl/journal.rs) now separates incomplete
final records as raw bytes before decoding complete journal and chunk lines.
Complete-line UTF-8 corruption remains a hard error. A torn chunk must still be
an exact byte prefix of the item recorded in the pending transaction.

The append-only writer in [ctbnbdl.rs](../../src/build_support/ctbnbdl.rs) also
compares raw bytes, allowing it to finish a split character without rewriting
existing bytes. Its strict equality/prefix checks remain enforced.

The general inbox regression test covers every internal byte cut in representative
two-, three-, and four-byte UTF-8 characters. It exercises real pending journal
and chunk recovery, rejects invalid UTF-8 in complete records without modifying
the corrupt file, preserves the ledger prefix, verifies committed provenance,
and checks that a second run allocates nothing. Neither the fix nor this test
depends on the recovery audit scripts. No graph corruption was found or repaired
in the live repository.

### Resolved: Inbox Ownership Protection Is Symmetric

[inbox.rs](../../src/formats/dcdata/graph_ledger/inbox.rs) now requires target
ownership to match input provenance for supersessions, retractions, property
values, and preferred qualifiers. The existing ownership test checks both
directions of cross-owner rejection and same-owner acceptance for each operation.
This is general inbox behavior, not a recovery-specific restriction.

### Medium: Historical Replay Rewrites The Live Worktree

[step0_extract_transcript_data.py](../../scripts/graph-recovery/step0_extract_transcript_data.py#L214)
checks out historical Rust files into the working tree, then restores them from
HEAD. This can overwrite local edits and depends on surrounding current source
and build configuration. It also assumes the unrecovered ledger count and a
particular external transcript path, so it is not a safe rerun command now.

Keep its committed historical version as process evidence. For future replay,
extract pinned sources into an isolated workspace, explicitly build musl, and
accept preserved transcript evidence as an input. Never check out old graph data
over the authoritative recovered ledger.

This is a warning about hypothetical reuse, not a request to rerun recovery.
Deleting the completed workflow's working files after the independent audits is
compatible with retaining its historical versions in Git.

### Medium: Operational State Remains Hidden

[ctbnbdl.rs](../../src/build_support/ctbnbdl.rs#L415) maintains redundant hidden
high-water ledgers. [journal.rs](../../src/build_support/ctbnbdl/journal.rs#L1971)
also writes an initialization marker from its reader gate and discards marker
write failures.

These files are not the only allocation record: the complete journal and sources
are tracked, and all available shadows agree. Keep the journal authoritative;
remove or explicitly classify redundant shadows as non-authoritative caches.
Keep the lock inode stable while processes are running. Moving a canary can
separate new readers/writers from locks still held on the old inode.

### Resolved: Recovery Counts Are Pinned To The Historical Prefix

The recovery invariants now inspect only the immutable ledger prefix ending at
`l8589958343`, excluding later allocations and revisions. Its active view reuses
the production bundle partition helper rather than duplicating selection logic.
The live zero-drift test now uses the existing dry-run API, so it cannot allocate
nodes while reporting a failing convergence check.

## What To Keep

Keep exact immutable-ledger checks, the tracked journal and manifests, durable
transaction/resume support, reader/writer locking, bundle union verification,
provenance validation, and protection against reusing retired value nodes. The
rename and source-ingestion support are useful ongoing features, not changes to
the identities of historical nodes.

Do not revert the Rust recovery commit wholesale: current readers require the
journal, and the code supplies the mechanism for safely appending future changes.
The one-off candidate-selection assumptions and UTF-8 interrupted-write defect
have been corrected. Address the remaining hidden operational state without
changing already-assigned IDs.

Blanket retraction would also cancel recovery revision statements, potentially
reactivating the incident's ambiguous definitions. It would not recreate missing
Git edit history. No ledger correction is indicated by this audit.

## Validation And Changes

- Follow-up musl graph/seed/inbox tests: 28 passed, zero failed or ignored.
  These include live Dc/format CSV readback parity, syntax parity, symmetric
  ownership, ordering-independent ambiguity, and historical-prefix invariants.
- Existing CSV edit tests additionally verify the resulting active property and
  relationship state, and zero-change second passes after those edits.
- Multibyte interrupted-write tests verify byte-safe journal and chunk recovery,
  strict complete-line corruption rejection, prefix preservation, and idempotency.
- Standalone musl journal/bundle tests: 5 passed, zero failed or ignored.
- Live filesystem CSV dry-run: 23,752 nodes, zero appends, zero supersessions,
  zero retractions, zero chunk writes, and no validation diagnostics.
- Affected production Clippy initially failed on inbox body indexing; checked
  access was substituted and the same command then passed with existing warnings.
- The focused inbox parser behavior test was rerun after that change.
- The reused Python helper's incorrect escape and retraction-subject tokens were
  corrected. Focused helper regressions and the complete read-only audit passed.
- Those Step 0 helper corrections affect the read-only audit that imports its
  parser; they do not retroactively change the recovery artifacts or require
  rerunning allocation.
- The independent audit command and this report were added. Graph data was not
  changed. The hidden-state finding remains an unimplemented recommendation.
- Follow-up production Clippy completed successfully with existing warnings.

The Step 3 reconciliation was already committed in `7abf6f67c`, and its script
performs a second convergence pass. The new dry-run independently confirms that
no additional CSV conversion append is needed at the audited state.

The normal `./build` workflow runs CSV reconciliation through its lint stage.
`build-quick` invokes `./build --quick`, which disables that stage; plain
`cargo build` does not itself run CSV-to-graph reconciliation. Source CSV edits
therefore need the normal validation workflow or an explicit validator run.