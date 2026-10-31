Original problem: Some node IDs were allocated twice by deleting them after assignment and then checking out the pre-assignment state of the graph from Git. This causes those nodes to be ambiguous, which is meant to never happen by making the graph strictly append-only. As that was not enforced in any technical way, it left room for this type of blunder. To prevent this reoccurring, I've added scripts which *should* enforce through technical means that the .ctbnodes files are strictly append-only. Since the .ctbnbdl files are, I believe, generated from the .ctbnodes files, that should also ensure that the union of the active and history .ctbnbdl files should also be append-only going forward. To address the now-ambiguous node IDs as best as possible, I'm looking into how to model this situation precisely in the graph.






Thanks, I think we're back and it's set up now! Great.

Next, let's look at recovering from the duplicated node IDs. So, I *think* the best way to approach this will need a few steps:
1. Come up with a new system for inserting graph nodes that *aren't* derived from the CSVs. I think:
  1a. Rename src/formats/dcdata/fixtures/ to src/formats/dcdata/data-src/.
  1b. Add new folders: src/formats/dcdata/data-src/nodes/{inbox,allocated}.
  1c. Add a build step that looks at all files in the 'inbox' folder, determines their filetype from a list of permitted types (maybe f635 .dcsh right now would be the best pick?), checks if any existing node has the same data and is not self-referential (I think? I want to avoid adding duplicate nodes, but in some cases duplicate nodes are *necessary* for one reason or another, for instance all the ordinary Dc IDs are 0-byte nodes, and if a node has been replaced and then its data needs to be referenced again, I guess it would need to be inserted again - how that should interact with atomic/data nodes, I'm not sure), and if they pass checks, allocates them a new node ID and appends them to the current .ctbnodes file, and moves them into the 'allocated' folder, prefxing their filename with their assigned node ID.
2. For every node ID that was assigned twice:
  2a. Insert new nodes for each body given to each overwritten node. E.g. if 123 was given body of string "abc" and overwritten with "is a, foo, bar" insert four new nodes: a new "abc", a new "is a, foo, bar" node (the latter being an intentional duplicate of 123), a node stating 348 "Ambiguous values" with a [list] of the other two nodes, and a node stating 591 "Supersedes" pointing to the original overwritten node and to the new "Ambiguous values" node, so that the original statement is correctly documented as ambiguous between the two values it was assigned.
  2b. *Somehow* figure out how to *disambiguate which ambiguous node values* were used for each of the above duplicated statements - e.g. if the "is a, foo, bar" referred to a node 234 for "foo" which was one of these ambiguous nodes, we would need to add, I *think*, a subsequent 591 "Supersedes" statement replacing the "is a, foo, bar" with "is a, foo, bar" but with the "foo" reference changed from the now-ambiguous 234 to whatever it was *meant* to point to.
3. Finally, re-run the graph CSV derivation, letting it automatically insert any needed new 591 "Supersedes" and 592 "Retracts" statements to leave the graph coherenent to the intended data.

Does that sound like the correct recovery path? How can we go about this with really careful auditing and precision to make sure it's recovered correctly and doesn't make even more of a mess? I think the source data where you found the original and overwritten graph values should be copied using a checked-in/committed script (*not* regenerating them but scripting what to copy out of the session transcript or wherever you found it using an auditable, committed script), and every step should also be implemented as a script, with all scripts and data committed to Git with every edit and step, so that the full process can be reproduced and audited after the fact.
 [/plan](slashCommand;plan)




# Append-Only Graph Ledger Recovery & Manual Node Allocation Plan

## Goal Description

In session `23f9eaf6-7cd6-416e-8470-852b9878ea88`, two separate graph derivation runs were executed against the commit `3345d2dff` baseline (`17,207` nodes, `l8589934592`..`l8589951798`):
1. **Run 1 (`task-680`, launched at `step_index == 679` [`19:27:09Z`] / process exited at `19:28:04Z` [notification queued and delivered at `step_index == 687` at `19:28:48Z`])**: Appended `683` nodes (`l8589951799`..`l8589952481`, `17,890` total nodes), which included 19 invented file extensions (`57` nodes = `19 × 3`) and an earlier `Dc 573 "Description"` string for `f95` (`Dce`).
2. **Run 2 (`task-693`, launched at `step_index == 692` [`19:29:15Z`] / process exited at `19:30:01Z` [notification delivered at `step_index == 698` at `19:30:58Z`], committed in `a90464028`)**: After resetting `assets/data/graph/`, appended `626` nodes (`l8589951799`..`l8589952424`, `17,833` total nodes), overwriting `l8589951799`..`l8589952424` and erasing `l8589952425`..`l8589952481`.

This plan implements a permanent, auditable, strictly append-only recovery and manual node ingestion pipeline in the following strict sequence:
1. **Step 0 (Evidence Preservation, Wall-Clock Mutation-Window Audit, & Precomputed Step 2a/2b Manifests First — Before Any Code Refactoring)**:
   - Audit **every** step (`step_index == 1..699`) and wall-clock execution interval (`Created At`, `Completed At`, `[Message] timestamp`) in `23f9eaf6/transcript_full.jsonl` from commit `3345d2dff` through `a90464028`, verifying that `task-680`'s process execution window (`19:27:09Z`..`19:28:04Z`) finished `39 seconds` before `step_index == 685` modified `document.csv` on disk (`19:28:43Z`..`19:28:46Z`).
   - Strictly separate **directly observed evidence** (`step_index == 684` `Target` and `Replacement` payloads from physical JSONL line `685`, `step_index == 687` & `698` task stdout logs, commit `3345d2dff` git tree, and commit `a90464028` surviving `.ctbnodes` bytes) from **reconstructed Run 1 output** (`reconstructed_run1_nodes_8589951799_8589952481.ctbnodes`), documenting explicitly that because Run 1's `.ctbnodes` files were never read into the transcript before being overwritten, Run 1's node bodies are reconstructed via deterministic replay of the unmodified `a90464028` / `3345d2dff` generator and corroborated by the `19:28:04Z` summary metrics (`17,890 total / 683 appended / 1 superseded / 0 retracted`).
   - Precompute and commit the deterministic manifests for **Step 2a** (`step2a_manifest.json`) and **Step 2b** (`step2b_manifest.json`).
2. **Step 1 (Shared Durable Transactional Ledger Writer, Initial Journal Checkpoint Bootstrap, Single-Inode Watcher Lock, & `fixtures/` -> `data-src/` Pipeline)**:
   - Rename `src/formats/dcdata/fixtures/` to `src/formats/dcdata/data-src/` and create `src/formats/dcdata/data-src/nodes/{inbox,allocated}`.
   - Establish the initial durable journal checkpoint (`batch_id: "bootstrap-a90464028"`) in `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl` under `LedgerWriteGuard` (`flock(LOCK_EX)`) by validating the `17,833`-node `.ctbnodes` ledger and `.ctbnbdl` bundle partition against the pinned Step 0 evidence **before** enabling gated readers in `build.rs` / `dc_codegen` / `validate_data_tables`.
   - Route **all** ledger writers (`process_inbox_nodes` and `update_system_graph_ledger`) through a single shared durable transactional writer (`execute_durable_ledger_transaction`), locking the **single permanent lock inode** `.git/ctb-append-only-canary` (`LedgerWriteGuard` wrapping `flock(LOCK_EX)`, mounted at `/canary` in the watcher container) and writing to the **unified journal** (`src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl`, mounted read-only at `/journal-dir/ledger-journal.jsonl` in the watcher container).
   - Separate internal writer-guarded operations (`&LedgerWriteGuard` + in-memory `&[LedgerNode]` snapshot) from external guarded readers (`LedgerReadGuard` wrapping `flock(LOCK_SH)` + `ensure_ledger_journal_clean`).
3. **Step 2 (Restore Erased Tail & Disambiguate Overwritten Node IDs via `nodes/inbox/` with `--require-manifest`)**:
   - **Step 2a**: Stage and allocate the `57` restored Run 1 tail nodes (`l8589952425`..`l8589952481`) + `603` `Dc 348 "Ambiguous values"` / `Dc 591 "Supersedes"` quadruplets (`2,412` nodes at `l8589952482`..`l8589954893`), verified inside the transaction via `--require-manifest step2a_manifest.json`.
   - **Step 2b**: Stage and allocate the branch-resolved statements (`l<r1_resolved(N)>`, `l<r2_resolved(N)>`) + `Dc 591 "Supersedes"` links + updated `Dc 348 "Ambiguous values"` records (`l<ambig_resolved(N)>`), verified inside the transaction via `--require-manifest step2b_manifest.json`.
4. **Step 3 (Generate & Execute Ownership-Scoped CSV Reconciliation Manifest)**:
   - Using the tested Step 1c Rust reconciliation planner (`validate_data_tables --plan-reconciliation-manifest step3_manifest.json`), generate and review `step3_manifest.json` against the committed Step 2b ledger, then execute `validate_data_tables --write --require-manifest step3_manifest.json` under the shared lock to reconcile CSV-owned statements against `DesiredGraphState` via `Dc 591 "Supersedes"` and `Dc 592 "Retracts"`.

---

## Confirmed Design Decisions & Reviewer Fixes (Rounds 1–8)

1. **Step 0 Wall-Clock Execution Window Audit & Explicit Epistemic Boundary (Fixes R1-F3, R2-F6, R4-F1, & R5-F1)**:
   - Step 0 audits both `step_index` and the exact wall-clock timestamps recorded in `23f9eaf6/transcript_full.jsonl`:
     - `step_index == 679` (`19:27:09Z`): Launched `./scripts/validate-data-tables` (`task-680`), which backgrounded at `step_index == 680` (`19:27:33Z`) and **completed execution at `19:28:04Z`** (recorded in `[Message] timestamp=2026-10-31T19:28:04Z` inside `step_index == 687`).
     - `step_index == 685` (`edit_file` on `document.csv`): Executed from `Created At: 2026-10-31T19:28:43Z` to `Completed At: 2026-10-31T19:28:46Z` — **39 seconds after `task-680` had already exited at `19:28:04Z`**.
   - **Explicit Epistemic Boundary**:
     - Because `task-680` ran without per-syscall file-access tracing and its `.ctbnodes` output was overwritten at `step_index == 688..693` before being read into the transcript, Run 1's `.ctbnodes` bytes were not directly captured in `transcript_full.jsonl`.
     - Step 0 labels `reconstructed_run1_nodes_8589951799_8589952481.ctbnodes` explicitly as **deterministic reconstruction corroborated by the `19:28:04Z` task summary metrics (`17,890 total / 683 appended / 1 superseded / 0 retracted / 7 chunk files updated`) and 100% exact Run 2 replay**, noting the absence of direct historical `.ctbnodes` byte capture as an explicit, documented boundary.

2. **Initial Journal Checkpoint Bootstrap & Separately Buildable Recovery Command in `ctb_build_support` (Fixes R8-F1 & R9-F1)**:
   - **Why Explicit Bootstrap & a `ctb_build_support` Recovery Command Are Required**:
     - [build.rs](file:///workspaces/ctoolbox/src/formats/dcdata/build.rs#L35-L38) in `ctb-formats-dcdata` executes `ctb_build_support::dc_codegen::generate_dc_file` and `generate_role_file` before `ctb-formats-dcdata` (and its binary `validate_data_tables`) can compile. Because `dc_codegen` enforces `ensure_ledger_journal_clean` (which rejects a missing journal, an open `BATCH_PENDING`, or a torn journal line), `validate_data_tables` cannot be compiled or rebuilt while a transaction (or initial bootstrap) is pending in `ledger-journal.jsonl`.
     - Therefore, **both** the initial bootstrap helper (`bootstrap_initial_ledger_journal`) and the pending-journal recovery engine (`recover_pending_ledger_journal`) live directly in `ctb_build_support::ctbnbdl` (which has no `build.rs` dependency on `dc_codegen` and already owns `.ctbnodes` parsing and `.ctbnbdl` bundle compilation) and are exposed as standalone commands on `ctb-lint`:
       - `cargo run --manifest-path src/build_support/Cargo.toml --bin ctb-lint -- bootstrap-ledger-journal`
       - `cargo run --manifest-path src/build_support/Cargo.toml --bin ctb-lint -- recover-ledger-journal [--require-manifest <path>]`
   - **Bootstrap Protocol (`bootstrap_initial_ledger_journal` in `ctb_build_support::ctbnbdl`)**:
     1. Acquires `LedgerWriteGuard` (`flock(LOCK_EX)` on `.git/ctb-append-only-canary`).
     2. **Crash Recovery Between Bootstrap's `BATCH_PENDING` and `BATCH_COMMITTED` (`R9-F1`)**:
        - If `ledger-journal.jsonl` already exists and has a single open `BATCH_PENDING` record with `kind == "bootstrap_checkpoint"` (or a torn trailing `BATCH_PENDING` / `BATCH_COMMITTED` line from an interrupted bootstrap), `bootstrap_initial_ledger_journal` delegates directly to `recover_pending_ledger_journal` under the same `LedgerWriteGuard` to repair any torn suffix, re-verify the pinned Step 0 `.ctbnodes` and `.ctbnbdl` baseline, write and `fsync` `BATCH_COMMITTED` and `.git/ctb-graph-baseline/journal-initialized`, and complete bootstrap idempotently.
     3. **Refuses to bootstrap** if any of the following hold:
        - `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl` already exists with a committed checkpoint (or a non-bootstrap record).
        - The local post-initialization marker `.git/ctb-graph-baseline/journal-initialized` already exists on disk (meaning the journal was previously initialized and later deleted/tampered with -> **hard error: missing journal after initialization**).
        - Any `.dcsh` file exists in `src/formats/dcdata/data-src/nodes/inbox/` or `src/formats/dcdata/data-src/nodes/allocated/`.
        - The on-disk `.ctbnodes` ledger does not match the pinned Step 0 evidence (`src/formats/dcdata/data-src/recovery-audit/step0_recovery_summary.json` and `observed_run2_nodes_8589951799_8589952424.ctbnodes`: exactly `17,833` nodes, `l8589934592`..`l8589952424`, exact `baseline_ledger_sha256`, and byte-for-byte agreement with `.git/ctb-graph-baseline/`).
     4. Validates the existing active (`assets/data/graph/85899_34592.ctbnbdl`) and historical (`assets/data/graph/system/85899_34942.ctbnbdl`) bundles against the `17,833` `.ctbnodes` records using `verify_compiled_bundles_match_ledger`.
     5. Without appending any graph nodes (`items: []`), writes and `fsync`s the initial durable checkpoint pair (`BATCH_PENDING` + `BATCH_COMMITTED` with `batch_id: "bootstrap-a90464028"`, `kind: "bootstrap_checkpoint"`, `baseline_last_id: 8589952424`, `final_last_id: 8589952424`, `baseline_ledger_sha256`, `final_ledger_sha256`, `active_bundles_sha256`, `history_bundles_sha256`, `record_sha256`) to `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl`, `fsync`s `src/formats/dcdata/data-src/nodes/allocated/`, and writes + `fsync`s `.git/ctb-graph-baseline/journal-initialized`.
     6. Furthermore, whenever `ensure_ledger_journal_clean` validates an existing `ledger-journal.jsonl` on a fresh checkout (where `ledger-journal.jsonl` is already tracked in Git), it ensures `.git/ctb-graph-baseline/journal-initialized` is stamped on disk so that deleting `ledger-journal.jsonl` afterward is always caught and rejected as post-initialization journal removal.
     7. In Step 1c, `bootstrap-ledger-journal` is executed **before** enabling `LedgerReadGuard` + `ensure_ledger_journal_clean` on gated readers, and the initialized `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl` is committed to Git in Step 1c.
   - **Standalone Recovery Command (`recover_pending_ledger_journal` in `ctb_build_support::ctbnbdl`, Fix R9-F1)**:
     - Because every `BATCH_PENDING` record already stores the complete planned `items` (`node_id`, `raw_line`, `target_chunk`, `inbox_name`, `allocated_name`, `inbox_sha256`), `baseline_last_id`, `baseline_ledger_sha256`, `final_last_id`, `final_ledger_sha256`, and `manifest_sha256`, `recover_pending_ledger_journal` in `ctb_build_support::ctbnbdl` can execute the full Resume Path (repairing torn journal/chunk lines, finishing primary `.ctbnodes` and `.git/ctb-graph-baseline/` shadow writes, moving remaining `nodes/inbox/` files to `nodes/allocated/`, `fsync`ing files and directories, rebuilding `.ctbnbdl` bundles via `compile_nodes_slice_to_ctbnbdl_with_write_guard`, and writing `BATCH_COMMITTED`) **purely inside `ctb_build_support` without compiling `ctb-formats-dcdata`**.
     - `execute_durable_ledger_transaction` in `ctb-formats-dcdata` calls this same `ctb_build_support::ctbnbdl::recover_pending_ledger_journal` function at the start of every writer invocation, so standalone recovery via `ctb-lint recover-ledger-journal` and in-process writer recovery share a single DRY implementation.

3. **Separate Fresh-Start vs. Resume Paths for `--require-manifest` & Zero Pre-Verification CSV Mutations (Fixes R5-F2, R6-F2, & R7-F3)**:
   - `execute_durable_ledger_transaction` accepts `--require-manifest <manifest.json>` and operates under `LedgerWriteGuard` (`flock(LOCK_EX)` on `.git/ctb-append-only-canary`) with three explicit state paths based on `ledger-journal.jsonl` (after repairing any unterminated trailing line):
     1. **Fresh-Start Path (No open `BATCH_PENDING` in `ledger-journal.jsonl`)**:
        - Verifies current on-disk ledger equals `manifest.baseline_last_id` and `manifest.baseline_ledger_sha256`.
        - Verifies all input files (`nodes/inbox/*.dcsh` for inbox runs; all CSV tables and `BATCH_COMMITTED` `NodeProvenanceMap` for CSV reconciliation runs) match `manifest.pinned_input_sha256s`.
        - Runs in-memory prevalidation/planning and verifies that the planned `items`, `final_last_id`, and `final_ledger_sha256` match `manifest` byte-for-byte.
        - Records `manifest_sha256: Some(sha256(manifest_bytes))` inside `BATCH_PENDING`, writes + `fsync`s `BATCH_PENDING`, and proceeds into execution.
     2. **Resume Path (An intact `BATCH_PENDING` record is open in `ledger-journal.jsonl`)**:
        - Verifies `BATCH_PENDING.manifest_sha256 == Some(sha256(manifest_bytes))` and `BATCH_PENDING.items == manifest.planned_items`.
        - Verifies the current on-disk `.ctbnodes` ledger equals the exact `manifest.baseline_ledger_sha256` prefix (`<= baseline_last_id`) followed by an **exact verified byte prefix** of `BATCH_PENDING.items`.
        - For each `kind == "inbox_allocation"` item in `BATCH_PENDING.items`, verifies its `.dcsh` file exists with `sha256 == item.inbox_sha256` and matching `# @provenance` at **either** `nodes/inbox/<item.inbox_name>` **or** `nodes/allocated/<item.allocated_name>`.
        - Resumes execution directly from `BATCH_PENDING.items` without replanning or allocating new IDs.
     3. **Idempotent Already-Committed Path**:
        - If the latest `BATCH_COMMITTED` record already has `manifest_sha256 == Some(sha256(manifest_bytes))`, verifies `final_ledger_sha256`, `allocated` files, and `.ctbnbdl` bundle hashes/partition, and returns `Ok` idempotently.
   - **Zero Pre-Verification CSV Writes in `--write --require-manifest` (`R7-F3`)**:
     - In [validate_data_tables.rs](file:///workspaces/ctoolbox/src/formats/dcdata/bin/validate_data_tables.rs), when `--write --require-manifest <path>` is invoked, it first checks in memory (`load_dc_category_csvs_with_assigned_ids` / `load_format_category_csvs_with_assigned_ids`) that **zero** category CSVs have unassigned or miscalculated IDs and validates all tables in memory, then runs `execute_durable_ledger_transaction` (which checks `--require-manifest` against the untouched on-disk CSVs before writing `BATCH_PENDING`).
     - Only **after** `execute_durable_ledger_transaction` commits `BATCH_COMMITTED` and releases `LedgerWriteGuard` does `validate_data_tables` write `generate_merged_csvs` and the downstream `.generated.rs` / `prepare_assets` outputs.

4. **Pending-Aware Provenance Validation Ordering vs. `BATCH_COMMITTED`-Only Validation (Fixes R5-Clarification & R6-F1)**:
   - A **"committed journal entry"** means a record in `ledger-journal.jsonl` whose `BATCH_PENDING` transaction has a corresponding complete, checksum-verified **`BATCH_COMMITTED`** record in `ledger-journal.jsonl` (not a Git commit).
   - Under `LedgerWriteGuard`, `execute_durable_ledger_transaction` inspects `ledger-journal.jsonl` and runs `BATCH_PENDING` recovery **before** calling committed-only `load_and_validate_node_provenance`. During `BATCH_PENDING` recovery, nodes `<= baseline_last_id` are validated against prior `BATCH_COMMITTED` records while nodes `> baseline_last_id` are validated against the verified prefix of `BATCH_PENDING.items` (accepting each `.dcsh` file at either `nodes/inbox/` or `nodes/allocated/`). Only after `BATCH_COMMITTED` is written does strict `BATCH_COMMITTED`-only provenance validation run.

5. **Separating Writer-Internal Operations (`LedgerWriteGuard`) from External Guarded Readers (`LedgerReadGuard`) & Full Bundle `(id, tokens)` Verification (Fixes R4-F2, R5-F3, R6-C1, R7-F1, R7-F2, & R8-Detail)**:
   - **Explicit Guard Architecture**:
     1. **`LedgerWriteGuard` (`flock(LOCK_EX)` on `.git/ctb-append-only-canary`)**:
        - Held by `execute_durable_ledger_transaction`, `bootstrap_initial_ledger_journal`, and Python `lock_pass` in `scripts/lock-graph-append-only`.
        - Inside `execute_durable_ledger_transaction`, initial ledger loading calls `load_ledger_nodes_from_dir_with_write_guard(&write_guard, &graph_dir)` (using the already-held `write_guard` without opening a second lock fd), and Phase 2 `.ctbnbdl` compilation calls `compile_nodes_slice_to_ctbnbdl_with_write_guard(&write_guard, &updated_nodes, &graph_dir, framing)` **directly from the validated in-memory `updated_nodes: &[LedgerNode]` snapshot** (never re-reading from disk via an external reader gate or calling `ensure_ledger_journal_clean` while `BATCH_PENDING` is open).
     2. **`LedgerReadGuard` (`flock(LOCK_SH)` on `.git/ctb-append-only-canary`) + `ensure_ledger_journal_clean`**:
        - Acquired **only** by external repository readers and packagers outside an active writer transaction: standalone `validate_all_data_tables_from_repo`, `compile_ctbnodes_dir_to_ctbnbdl`, `dc_codegen` (`build.rs`), `asset_packer::prepare_assets`, and the downstream `.generated.rs` codegen steps in `validate_data_tables.rs` (after `LedgerWriteGuard` is dropped).
   - **Full Bundle Inventory, Union, Uniqueness, `(node.id, &node.tokens)` Equality, & Partition Verification (`R7-F2` & `R8-Detail`)**:
     - `BATCH_COMMITTED` records `active_bundles_sha256: BTreeMap<String, String>` and `history_bundles_sha256: BTreeMap<String, String>`.
     - `ensure_ledger_journal_clean(repo_root, &read_guard)` verifies:
       1. `ledger-journal.jsonl` exists on disk (never trusting a missing journal), has at least one `BATCH_COMMITTED` entry, and has no open `BATCH_PENDING` or torn trailing line, and stamps `.git/ctb-graph-baseline/journal-initialized`.
       2. The on-disk `.ctbnbdl` files in `assets/data/graph/` and `assets/data/graph/system/` match `BATCH_COMMITTED.active_bundles_sha256` and `BATCH_COMMITTED.history_bundles_sha256` byte-for-byte.
       3. `verify_compiled_bundles_match_ledger` decodes all active and historical `.ctbnbdl` bundles and compares **projected `(node.id, &node.tokens)` pairs** (explicitly excluding `node.comment`, since `.ctbnbdl` binary bundles do not preserve `.ctbnodes` `#` comments):
          - Active and historical node ID sets are strictly ascending and **disjoint** (`active_ids.is_disjoint(&history_ids)`).
          - Their merged sorted `(node.id, &node.tokens)` union equals `committed_nodes.iter().map(|n| (n.id, &n.tokens))`.
          - The `(active_decoded, history_decoded)` `(node.id, &node.tokens)` split equals `partition_active_and_inactive_nodes_with(&committed_nodes, framing)`.

6. **Shared Effective-State Resolver for Dynamic Duplicate Checking & Cross-Ownership Protection (Fixes R1-Q3, R3-F3, R5-F4, & R5-F5)**:
   - Prevalidation uses `compute_effective_inactive_ids` (shared with `SeedGraphIndex`) so revision cancellations accurately update `active_nodes_by_tokens`, and requires `# @allow-duplicate-of: l<id1>[, l<id2>, ...]` to equal `*active_matches`.
   - `# @provenance: manual` nodes cannot attach `Dc 610 "Has value"` or `Dc 570 "Preferred"` qualifiers to non-`Manual` `has_a_id`s or supersede/retract non-`Manual` nodes, and `LedgerBuilder::apply_desired_state` aborts with an explicit cross-ownership conflict error if any active `Manual` node references a CSV-owned node scheduled for withdrawal or collides on a single-valued `(subj, prop)` in `DesiredGraphState`.

---

## Proposed Execution Sequence

```mermaid
flowchart TD
    S0["Step 0 (First): Wall-clock transcript audit (19:27:09Z..19:28:04Z),\npinned historical replay, separate observed vs reconstructed,\nprecompute step2a_manifest.json & step2b_manifest.json"] --> S1A["Step 1a/1b: git mv fixtures -> data-src,\ncreate data-src/nodes/{inbox,allocated}"]
    S1A --> S1C1["Step 1c-bootstrap: Run bootstrap_initial_ledger_journal under LOCK_EX\nto validate ledger + bundles against Step 0 evidence &\ncreate initial ledger-journal.jsonl checkpoint before enabling reader gates"]
    S1C1 --> S1C2["Step 1c-writer: Enable LedgerReadGuard + ensure_ledger_journal_clean,\nunified durable writer, fresh/resume --require-manifest,\npending-first provenance, & run all unit/fault-injection tests"]
    S1C2 --> S2A["Step 2a: Restore l8589952425..l8589952481 (57 nodes)\n& allocate 603 Dc 348 / Dc 591 quadruplets (2,412 nodes)\nvia validate_data_tables --process-inbox --require-manifest step2a_manifest.json"]
    S2A --> S2B["Step 2b: Allocate branch-resolved statements\n& supersede Dc 348 records via Dc 591 Supersedes\nvia validate_data_tables --process-inbox --require-manifest step2b_manifest.json"]
    S2B --> S3["Step 3: Plan step3_manifest.json via validate_data_tables --plan-reconciliation-manifest,\nthen execute validate_data_tables --write --require-manifest step3_manifest.json\n& verify all live + history invariants"]
```

---

### Step 0 (Executed First): Wall-Clock Audit, Historical Replay, & Precomputed Step 2a/2b Manifests

#### [NEW] `scripts/graph-recovery/step0_extract_transcript_data.py`
- Runs **before** any modifications to `src/formats/dcdata/` or `src/build_support/`:
  1. Audits all steps (`step_index == 1..699`) and wall-clock timestamps (`Created At`, `Completed At`, `[Message] timestamp`) in `/root/.gemini/antigravity/brain/23f9eaf6-7cd6-416e-8470-852b9878ea88/.system_generated/logs/transcript_full.jsonl` and `git diff 3345d2dff..a90464028`, writing `invocation_and_mutation_window_audit.json`.
  2. Saves observed evidence (`observed_step684_record.json`, `observed_step687_and_698_logs.json`, `observed_run1_document_csv_slice.csv`, `observed_run2_document_csv_slice.csv`, `observed_run2_nodes_8589951799_8589952424.ctbnodes`).
  3. Replays the unmodified historical generator on `(commit 3345d2dff + Run 2 document.csv)` and asserts 100% token-for-token and comment-for-comment identity with `observed_run2_nodes_8589951799_8589952424.ctbnodes`.
  4. Replays the unmodified historical generator on `(commit 3345d2dff + Run 1 document.csv)` to produce `reconstructed_run1_nodes_8589951799_8589952481.ctbnodes` and verifies the `19:28:04Z` summary metrics (`17,890 total / 683 appended / 1 superseded / 0 retracted / 7 chunk files updated`).
  5. Computes the transitive dependency-ambiguity closure (`23` unambiguous, `603` ambiguous, `57` erased tail) and writes `collision_manifest.json`, `step2a_manifest.json`, and `step2b_manifest.json`.
  6. Commits Step 0.

---

### Component 1: Initial Journal Checkpoint Bootstrap, Unified Durable Writer, & `data-src` Pipeline (`Steps 1a–1c`)

#### [MODIFY] `src/formats/dcdata/fixtures/` -> `src/formats/dcdata/data-src/` (`Steps 1a & 1b`)
- Move via `git mv src/formats/dcdata/fixtures src/formats/dcdata/data-src` and create `src/formats/dcdata/data-src/nodes/{inbox,allocated}/.keep`.
- Update `src/formats/dcdata/fixtures` path references to `src/formats/dcdata/data-src` across [dcdata.rs](file:///workspaces/ctoolbox/src/formats/dcdata/dcdata.rs), [graph_ledger.rs](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs), [updater.rs](file:///workspaces/ctoolbox/src/formats/dcdata/updater.rs), [validation.rs](file:///workspaces/ctoolbox/src/formats/dcdata/validation.rs), [format.rs](file:///workspaces/ctoolbox/src/formats/dcdata/format.rs), [syntax.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax.rs), and [modify-columns.py](file:///workspaces/ctoolbox/scripts/modify-columns.py), and commit Steps 1a & 1b.

#### [MODIFY] [lock-graph-append-only](file:///workspaces/ctoolbox/scripts/lock-graph-append-only), [ctbnbdl.rs](file:///workspaces/ctoolbox/src/build_support/ctbnbdl.rs), [dc_codegen.rs](file:///workspaces/ctoolbox/src/build_support/dc_codegen.rs), [graph_ledger.rs](file:///workspaces/ctoolbox/src/formats/dcdata/graph_ledger.rs), [seed_graph.rs](file:///workspaces/ctoolbox/src/formats/dcdata/seed_graph.rs), & [validate_data_tables.rs](file:///workspaces/ctoolbox/src/formats/dcdata/bin/validate_data_tables.rs) (`Step 1c`)
1. **Bootstrap Initial Journal Checkpoint Before Enabling Reader Gates (`R8-F1`)**:
   - Implement `bootstrap_initial_ledger_journal` in [ctbnbdl.rs](file:///workspaces/ctoolbox/src/build_support/ctbnbdl.rs) and expose `ctb-lint bootstrap-ledger-journal`.
   - Execute `cargo run --manifest-path src/build_support/Cargo.toml --bin ctb-lint -- bootstrap-ledger-journal` **before** enabling gated readers in `dc_codegen` ([build.rs](file:///workspaces/ctoolbox/src/formats/dcdata/build.rs)), validating the `17,833`-node `.ctbnodes` ledger and `.ctbnbdl` bundle partition against the pinned Step 0 evidence and writing the durable initial `BATCH_PENDING` + `BATCH_COMMITTED` (`batch_id: "bootstrap-a90464028"`) checkpoint to `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl` under `LedgerWriteGuard` (`LOCK_EX`).
2. **`LedgerWriteGuard` vs. `LedgerReadGuard`, Full `(node.id, &node.tokens)` Bundle Verification, & Watcher Mounts (`R7-F1`, `R7-F2`, & `R8-Detail`)**:
   - Writers and Python `lock_pass` acquire `LedgerWriteGuard` (`flock(LOCK_EX)` on `.git/ctb-append-only-canary`); internal writer steps (`load_ledger_nodes_from_dir_with_write_guard` and `compile_nodes_slice_to_ctbnbdl_with_write_guard`) take `&LedgerWriteGuard` and operate directly on the validated in-memory `&[LedgerNode]` slice without opening a second lock descriptor or calling `ensure_ledger_journal_clean` while `BATCH_PENDING` is open.
   - External readers, `dc_codegen` (`build.rs`), and `asset_packer::prepare_assets` acquire `LedgerReadGuard` (`flock(LOCK_SH)` on `.git/ctb-append-only-canary`) and call `ensure_ledger_journal_clean` (verifying `ledger-journal.jsonl` exists and ends with `BATCH_COMMITTED`, verifying `BATCH_COMMITTED` bundle SHA-256s, and verifying via `verify_compiled_bundles_match_ledger` that the decoded active + history `.ctbnbdl` bundles have disjoint node IDs, equal `committed_nodes` in `(node.id, &node.tokens)` pairs, and match `partition_active_and_inactive_nodes_with(&committed_nodes, framing)` in `(node.id, &node.tokens)` pairs).
   - Update `run_host_init()` and `run_unlock()` in `scripts/lock-graph-append-only` to mount `-v "${REPO_ROOT}/src/formats/dcdata/data-src/nodes/allocated:/journal-dir:ro"`.
3. **Unified Durable Transactional Writer with Fresh/Resume `--require-manifest`, Pending-First Recovery, & Zero Pre-Verification CSV Writes**:
   - Runs pending-journal recovery **before** committed-only `load_and_validate_node_provenance`.
   - Implements the Fresh-Start, Resume (`BATCH_PENDING` bound via `manifest_sha256`), and Already-Committed (`BATCH_COMMITTED`) paths for `--require-manifest <path>`.
   - In [validate_data_tables.rs](file:///workspaces/ctoolbox/src/formats/dcdata/bin/validate_data_tables.rs), dispatches `--process-inbox`, `--plan-reconciliation-manifest <path>`, and `--write [--require-manifest <path>]` at the top of `main()`, ensuring `--write --require-manifest` verifies normalized CSVs in memory and completes the manifest-enforced ledger transaction **before** writing `generate_merged_csvs` or any `.generated.rs` files.
4. **Effective-State Duplicate Checking & Provenance Ownership Contract**:
   - Reuses `compute_effective_inactive_ids` for dynamic duplicate checking across revision cancellations; enforces `BATCH_COMMITTED` `NodeProvenanceMap` validation after recovery; enforces cross-ownership boundary checks for `Manual` nodes; and reconciles CSV-owned property groups component-wise (preserving shared IDs such as `l8589951812`).
   - Commits Step 1c (including the initialized `src/formats/dcdata/data-src/nodes/allocated/ledger-journal.jsonl`).

---

### Component 2: Step-by-Step Recovery Execution (`Steps 2a, 2b, 3`)

#### [NEW] `scripts/graph-recovery/step2a_stage_ambiguous_nodes.py`
- Stages the precomputed Step 2a `.dcsh` files (`57` restored Run 1 tail nodes at `l8589952425`..`l8589952481` + `603` quadruplets = `2,412` nodes at `l8589952482`..`l8589954893`) into `src/formats/dcdata/data-src/nodes/inbox/`.
- Executes `validate_data_tables --process-inbox --require-manifest src/formats/dcdata/data-src/recovery-audit/step2a_manifest.json` and commits Step 2a.

#### [NEW] `scripts/graph-recovery/step2b_stage_disambiguations.py`
- Stages the precomputed Step 2b `.dcsh` files (branch-resolved statements `l<r1_resolved(N)>`, `l<r2_resolved(N)>`, `Dc 591 "Supersedes"` links, updated `Dc 348 "Ambiguous values"` records `l<ambig_resolved(N)>`, and disambiguated tail statements) into `src/formats/dcdata/data-src/nodes/inbox/`.
- Executes `validate_data_tables --process-inbox --require-manifest src/formats/dcdata/data-src/recovery-audit/step2b_manifest.json` and commits Step 2b.

#### [NEW] `scripts/graph-recovery/step3_reconcile_csv_graph.sh`
- Runs `validate_data_tables --plan-reconciliation-manifest src/formats/dcdata/data-src/recovery-audit/step3_manifest.json` using the tested Rust reconciliation planner to produce and inspect the exact Step 3 reconciliation manifest (`Dc 591 "Supersedes"` and `Dc 592 "Retracts"` statements).
- Executes `validate_data_tables --write --require-manifest src/formats/dcdata/data-src/recovery-audit/step3_manifest.json` under the shared lock to apply Step 3, rebuild `.ctbnbdl` bundles, and update generated files.
- Runs `./scripts/validate-data-tables` a second time to verify `0` nodes are appended, then commits Step 3.

---

## Verification Plan

### Automated Invariant & Fault-Injection Tests
1. **Bootstrap, Unified Durable Writer, Watcher, & Fault-Injection Tests (`cargo test -p ctb-formats-dcdata --lib graph_ledger` & `cargo test -p ctb-build-support`)**:
   - **Initial journal bootstrap & missing-journal rejection tests (`R8-F1`)**:
     - **First build from an uninitialized checkout at the pinned Step 0 baseline**: In a checkout fixture with no `ledger-journal.jsonl` and no `.git/ctb-graph-baseline/journal-initialized` marker whose `.ctbnodes` and `.ctbnbdl` match the pinned Step 0 evidence, verify that running `bootstrap_initial_ledger_journal` succeeds under `LedgerWriteGuard` (`LOCK_EX`), creates `ledger-journal.jsonl` (`batch_id: "bootstrap-a90464028"`, `items: []`) and `.git/ctb-graph-baseline/journal-initialized` without appending nodes, and that gated readers/builds (`dc_codegen` / `ensure_ledger_journal_clean`) then succeed.
     - **Rejection of arbitrary uninitialized ledger**: In an uninitialized checkout whose `.ctbnodes` or `.ctbnbdl` deviates from the pinned Step 0 evidence (or already has `.dcsh` files in `nodes/inbox/` or `nodes/allocated/`), verify both `bootstrap_initial_ledger_journal` and `ensure_ledger_journal_clean` reject.
     - **Rejection of missing journal after initialization**: After `bootstrap_initial_ledger_journal` (or after `ensure_ledger_journal_clean` has stamped `.git/ctb-graph-baseline/journal-initialized`, or after any allocation transaction), delete `ledger-journal.jsonl` and verify that **both** `ensure_ledger_journal_clean` (gated build/reader) and `bootstrap_initial_ledger_journal` reject the missing journal as tampering/corruption.
   - **Prevalidation, revision cancellation, & `--require-manifest` guards**:
     - Syntax error, self/forward reference, unpermitted active duplicate, incomplete `# @allow-duplicate-of` set, self-supersession (`591 l<a> l<a>`), supersession cycle, or duplicate `Dc 591 "Supersedes"` target aborts before `BATCH_PENDING`.
     - **Revision cancellation duplicate test (`R5-F5`)**: Retracting a prior `591`/`592` revision statement reactivates its original target in `active_nodes_by_tokens`.
     - **`--write --require-manifest` mismatch test (`R5-F2` & `R7-F3`)**: Invoking `--write --require-manifest` with a mismatched manifest aborts before `BATCH_PENDING` and leaves **all category CSVs, `.generated.csv` files, `.generated.rs` files, `ledger-journal.jsonl`, `.ctbnodes`, and `.ctbnbdl` files 100% byte-for-byte unchanged**.
   - **Complete transaction with reader/writer synchronization & `(node.id, &node.tokens)` bundle check (`R7-F1`, `R7-F2`, & `R8-Detail`)**:
     - Execute a complete `execute_durable_ledger_transaction` (including Phase 2 `compile_nodes_slice_to_ctbnbdl_with_write_guard` on commented `.ctbnodes` nodes while `LedgerWriteGuard` is held and `BATCH_PENDING` is open), followed by an external `LedgerReadGuard` read calling `ensure_ledger_journal_clean` (verifying `BATCH_COMMITTED` bundle SHA-256s, active/history disjointness, complete union `(node.id, &node.tokens)` equality against `.ctbnodes` ignoring `.ctbnodes` comments, and partition equality against `partition_active_and_inactive_nodes_with`). Also verify that corrupting a single token or partitioning a node into the wrong bundle is caught by `ensure_ledger_journal_clean` even when the highest node ID is unchanged.
   - **Crash, resume, & durability boundaries**:
     1. **Retrying `--require-manifest` after partial append + file move (`R6-F1` & `R6-F2`)**: In a multi-item inbox batch invoked with `--process-inbox --require-manifest`, simulate a crash after item 1 is appended to `.ctbnodes` and moved from `nodes/inbox/` to `nodes/allocated/` (while items 2..N are still in `nodes/inbox/` and `BATCH_PENDING` is open). Re-invoke the **exact same `--process-inbox --require-manifest` command** and verify that pending-aware provenance validation, the `--require-manifest` Resume Path, completion of items 2..N, `BATCH_COMMITTED` write, and post-recovery `BATCH_COMMITTED`-only provenance validation all succeed.
     2. Torn trailing `BATCH_PENDING` line (both on initial batch and after an intervening `csv_reconciliation` `BATCH_COMMITTED` record in `ledger-journal.jsonl`) -> repaired cleanly on retry; complete (`\n`-terminated) record with bad `record_sha256` -> rejected as hard corruption.
     3. **100th-line torn write + kernel watcher interaction**: Write 99 complete lines + a torn 100th line (without `\n`), run `lock-graph-append-only` (`lock_pass`), verify the chunk remains `FS_APPEND_FL` (`+a`), then run journal recovery to complete the 100th line's suffix via `O_APPEND` and verify a subsequent `lock_pass` seals it with `+i`.
     4. Crash after write/rename but **before `fsync`** -> verify resume unconditionally invokes `sync_all()` on all touched primary/shadow chunks, `allocated` files, and directories, rebuilds `.ctbnbdl` bundles, and writes `BATCH_COMMITTED`.
     5. Torn trailing `BATCH_COMMITTED` line after completed items -> verify resume validates intact `BATCH_PENDING` and completed items, repairs torn suffix, and writes complete `BATCH_COMMITTED`.
     6. **External reader / packager clean-journal gate (`R6-C1`)**: While `BATCH_PENDING` is open in `ledger-journal.jsonl`, verify external `ensure_ledger_journal_clean` rejects reader/codegen/packaging calls until recovery finishes and writes `BATCH_COMMITTED`.
     7. `--dry-run` in clean, `BATCH_PENDING`, and torn-journal states -> zero file mutations.
   - **Provenance & Cross-Ownership Protection Tests (`R5-F4` & `R5-Clarification`)**:
     - Verify a self-contained `# @provenance: manual` statement is preserved untouched across `update_system_graph_ledger` runs.
     - Verify attaching a `# @provenance: manual` `Dc 610 "Has value"` or `Dc 570 "Preferred"` node to a CSV-owned `has_a_id` is rejected by prevalidation, and that if a CSV-owned node referenced by a `Manual` node is scheduled for withdrawal, `apply_desired_state` aborts with an explicit cross-ownership conflict error.
     - Verify missing or conflicting provenance between `nodes/allocated/*.dcsh` and `BATCH_COMMITTED` entries in `ledger-journal.jsonl` fails validation.
2. **Recovery & Live vs. Historical Graph Invariant Tests (`cargo test -p ctb-formats-dcdata --lib seed_graph`)**:
   - **Unchanged Prefix Invariant**: `l8589934592`..`l8589952424` (`17,833` nodes) match commit `a90464028` byte-for-byte.
   - **Exact Tail Restoration Invariant**: `l8589952425`..`l8589952481` (`57` nodes) match `reconstructed_run1_nodes_8589951799_8589952481.ctbnodes` token-for-token.
   - **Preserved Unambiguous Nodes (`23` nodes)**:
     - All `23` nodes (`l8589951799`..`l8589951801`, `l8589951803`..`l8589951812`, `l8589951814`..`l8589951823`) remain **ledger-effective** (`!full_index.inactive_ids.contains(&id)`), including `f95`'s shared `Dc 584 "Has a"` node `l8589951812` and revision statement `l8589951819` (`591 l8589951818 l8589941912`).
     - The `22` non-revision unambiguous nodes are in `assets/data/graph/85899_34592.ctbnbdl`, while `l8589951819` is in `assets/data/graph/system/85899_34942.ctbnbdl` with its supersession of `l8589941912` by `l8589951818` active.
   - **Complete `603`-Node Ambiguity Closure & Post-Reconciliation Branch Inspectability Invariant**:
     - Every node in the `603`-node ambiguity closure (`l8589951802`, `l8589951813`, `l8589951824`..`l8589952424`) resolves via `full_index.get_ambiguous_candidates(id)` to `[l<r1_resolved(N)>, l<r2_resolved(N)>]`, neither candidate references any of the `603` ambiguous IDs, and both `full_index.get_node(r1_resolved(N))` and `full_index.get_node(r2_resolved(N))` remain intact and inspectable in the historical index after Step 3 reconciliation.
   - **Live Runtime Bundle Purity, Uniqueness, & Idempotency Invariant**:
     - Zero nodes from the `603`-node ambiguous set are present in or referenced by `assets/data/graph/85899_34592.ctbnbdl`.
     - `find_duplicate_or_conflicting_active_statements()` returns empty.
     - `reconstruct_dc_defns()` and `reconstruct_format_defns()` round-trip the CSV dataset with zero drift, active + history `.ctbnbdl` bundles partition 100% of `.ctbnodes`, and a second `update_system_graph_ledger` run appends `0` nodes.
3. **Repository Lints & Kernel Guard Canary**:
   - `cargo run --manifest-path src/build_support/Cargo.toml --bin ctb-lint -- all --quick`
   - `./scripts/lock-graph-append-only --test-canary`




Question 1: If those 23 were unambiguous, it sounds fine to leave them as-is even if they were deleted and recreeated in a bit-for-bit-, meaning-for-meaning-identical way.
Question 2: It sounds like those 57 nodes should be restored to their original definitions, yes.
Question 3: I think that sounds good.
