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
- The ongoing Rust behavior has the policy and durability issues below. They do
  not invalidate the committed recovery, but should be addressed before relying
  on this machinery for general-purpose future allocations.

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

### High: Canonical Choice Is Inferred From List Position

[seed_graph.rs](../../src/formats/dcdata/seed_graph.rs#L280) treats the last
candidate of every ambiguity list as canonical. It also guesses an allocation
region by subtracting 100 from a candidate ID at
[seed_graph.rs](../../src/formats/dcdata/seed_graph.rs#L340).
[graph_ledger.rs](../../src/formats/dcdata/graph_ledger.rs#L1235) then retracts
earlier-branch statements before reconciling the CSV dataset.

Run 2 ordering is corroborated by this recovery's evidence, so the recorded
retractions are correct. That does not justify a global rule that the last
alternative is authoritative, or that nearby IDs belong to recovery. Scope this
selection to an explicit verified recovery manifest or separately approved
policy. The generic index should not silently discard other valid alternatives.

### High: A Torn UTF-8 Tail Can Block Crash Recovery

[journal.rs](../../src/build_support/ctbnbdl/journal.rs#L933) and
[journal.rs](../../src/build_support/ctbnbdl/journal.rs#L1379) read the entire
journal/chunk as a UTF-8 string before identifying its final unterminated line.
A real interrupted byte write can split a multibyte comment character, causing
decoding to fail before the supported torn-tail recovery logic runs.

Read bytes first, validate complete lines strictly, and preserve the incomplete
suffix as bytes. Add multibyte torn-journal and torn-chunk tests. The existing
ASCII fault-injection tests do not exercise this failure. No such corruption
exists in the committed recovery.

### Medium: Ownership Protection Is Asymmetric

[inbox.rs](../../src/formats/dcdata/graph_ledger/inbox.rs#L563) and
[inbox.rs](../../src/formats/dcdata/graph_ledger/inbox.rs#L584) prevent manual
inbox revisions from modifying CSV-owned nodes, but do not perform the reciprocal
check for CSV-declared inbox revisions targeting manual nodes. The regular CSV
builder has additional protections, but direct inbox allocation bypasses that
builder. Test and enforce both directions, or explicitly authorize exceptional
cross-owner operations. All nodes in this recovery are CSV-owned.

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

### Medium: Live Tests Are Pinned To The Recovery's Final Counts

[seed_graph.rs](../../src/formats/dcdata/seed_graph.rs#L2882) asserts exact current
active/full counts and inactivity totals. Legitimate future CSV additions will
break these tests even if the recovery remains correct. Make exact-count recovery
tests operate on a pinned historical in-memory fixture, while live tests validate
invariants rather than freezing the entire graph.

[graph_ledger.rs](../../src/formats/dcdata/graph_ledger.rs#L3686) calls the writing
update API before asserting zero drift. Use the existing dry-run option so a
failing verification test cannot allocate nodes merely to discover drift.

## What To Keep

Keep exact immutable-ledger checks, the tracked journal and manifests, durable
transaction/resume support, reader/writer locking, bundle union verification,
provenance validation, and protection against reusing retired value nodes. The
rename and source-ingestion support are useful ongoing features, not changes to
the identities of historical nodes.

Do not revert the Rust recovery commit wholesale: current readers require the
journal, and the code supplies the mechanism for safely appending future changes.
Narrow the one-off canonical-selection assumptions and fix the durability gaps
without changing already-assigned IDs.

Blanket retraction would also cancel recovery revision statements, potentially
reactivating the incident's ambiguous definitions. It would not recreate missing
Git edit history. No ledger correction is indicated by this audit.

## Validation And Changes

- Focused musl graph/seed/inbox tests: 26 passed, zero failed or ignored.
- Standalone musl journal/bundle tests: 5 passed, zero failed or ignored.
- Live filesystem CSV dry-run: 23,752 nodes, zero appends, zero supersessions,
  zero retractions, zero chunk writes, and no validation diagnostics.
- Affected production Clippy initially failed on inbox body indexing; checked
  access was substituted and the same command then passed with existing warnings.
- The focused inbox parser behavior test was rerun after that change.
- The reused Python helper's incorrect escape and retraction-subject tokens were
  corrected. Focused helper regressions and the complete read-only audit passed.
- The independent audit command and this report were added. Graph data was not
  changed. The broader policy/durability findings above are recommendations,
  not implemented fixes.

The Step 3 reconciliation was already committed in `7abf6f67c`, and its script
performs a second convergence pass. The new dry-run independently confirms that
no additional CSV conversion append is needed at the audited state.