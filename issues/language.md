# Graph-Defined Dc Language, Execution Kernel, and Compatibility Plan

**Overall Status**: `Partially Implemented`

Reviewed against [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md) and source at `662c3e8e0563c8f0e4d89a3e1e987124b10e2323` on 2026-11-07.

## Core Architectural Guarantee

**Given identical document input octets, identical resolved graph snapshots, and an identical external event/response sequence, conforming implementations must produce a bit-for-bit identical observable execution trace after future code or graph updates.**

This is an execution/replay contract, not a requirement to preserve internal
parser trees, decoded-value representations, bindings tables, or VM states.
Those mechanisms may change provided observable document behavior does not.

The complete inputs are:
1. The original document octets, before parsing them as Dcs.
2. The resolved graph selected by the document's pins, including all transitive
  dependencies. The System pin selects the System block; lower definition
  regions, excluding the reserved Data block, follow the foundational kernel
  contract. The reserved Data block beginning at `4294967296` and blocks above
  System follow the same per-block pinning rule: references must declare pins
  for those blocks too. Support for these additional blocks remains pending.
3. All external responses and events: clock readings, filesystem and network
  results, keyboard/mouse input, authorization decisions, cancellation, and
  externally imposed resource interruptions. Their ordering and logical
  delivery points are inputs too.

The observable trace includes the ordered capability and I/O requests/calls,
converted document bytes, emitted frame bytes, document-visible diagnostics,
and completion, failure, or suspension outcomes. For interactive or nonterminating
documents, equality applies to every corresponding finite trace prefix, not only
to a final converted document.

"The same execution states" means the same logical document execution points,
not identical Rust states, CPU instructions, WASM instruction counts, or wall
clock times. During replay, the next request must be identical; delivering the
recorded response must produce identical subsequent outputs and the next
request. Asynchronous events need specified logical delivery points and ordering.

Graph pinning fixes the document's requests, not the host's authorization policy.
A current host may deny a request that an older host allowed; that is a different
external response, not a compatibility violation. Host-only logging and diagnostic
wording are outside the trace unless explicitly exposed to the document.

Canonical rendering is deterministic software execution and includes the final
rasterized pixel output in the observable trace. Fonts, rendering algorithms,
and all other defining information must come from the pinned graph or explicit
inputs, never implicit host rendering defaults. The output format chain specifies
the frame conversion configuration, including subpixel rendering and vertical or
horizontal pixel alignment. With identical inputs, graph pins, and configuration,
canonical pixel output must be bit-for-bit identical regardless of hardware.
This guarantee concerns generated pixels, not the physical appearance of a display.

GPU rendering is not planned for the current implementation. A distant-future
optional GPU optimization could provide a non-authoritative performance optimization, but must
never define or replace the canonical rendering of the document. Any differing pixels when using GPU rendering would be outside the canonical output contract. Functionality must never depend on GPU.

Append-only System-block storage (`l8589934592..=l12884901887`) is necessary but not sufficient on its own:
1. Snapshot resolution for `Dc 641 "Global graph system block most recent node ID at time of document creation"` must never silently substitute `HEAD` when history is missing, when a pin is newer than the local graph, or when resolver loading fails.
2. Foundational kernel semantics (transport/framing codecs, `.Semantic:Pattern` interpretation for `Dc 599 "Self reference"`..`Dc 607 "Pattern replacement"`, `Dc 611 "Canonical decomposition"` equivalence resolution, Dc-WASM lowering, and the 32-byte linear-memory mailbox ABI) must be explicitly specified and frozen or versioned.
3. Document-controlled WASM execution must be bounded by independent instruction metering, linear-memory caps, and output byte limits inside each `step()` call—not merely an outer loop counter around synchronous `tinywasm` calls.
4. Evaluated streams with token splicing (`Dc 256 "void lang.say([string] text)"` and WASM-backed splice routines) must preserve incremental expansion-before-framing, kept distinct from unevaluated source/CST inspection.

---

## Completed Foundations (Cleaned Up from Earlier Proposals)

- **WASM Instruction & Program Dcs Allocated (`src/formats/dcdata/data-src/categories/wasm.csv`)**:
  - Core and extension WebAssembly opcodes across five 256-entry opcode pages (`Dc 647 "WASM unreachable"`..`Dc 1926 "WASM Reserved 0xFD 0xFF 0x03"`) and program delimiters (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`) are allocated in `.WASM` script blocks with immediate/block syntax rules.
- **Zero-Table Dc-WASM Assembler & Pure-Compute `DcWasmSession` (`src/formats/dcstring/converters/wasm.rs`)**:
  - `dcmstring_to_wasm_with_resolver` lowers `Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"` and `Dc 280 "Built-in type identifier: Routine"` literals into standard `.wasm` bytecode.
  - `DcWasmSession` (`src/formats/dcstring/converters/wasm.rs:1485-1839`) implements the 32-byte linear-memory mailbox ABI (`0..32`) and `step() -> StatusCode` protocol (`0 = Complete`, `1 = YieldFrame`, `2 = RequestCapability`) over `tinywasm`, wired into `DefaultDcExpressionEvaluator` (`src/formats/dcstring/navigator/eval.rs:111-144`, `248-259`) and `ExecutionContext::execute_wasm_session` (`src/runtime/runtime.rs:314-370`).
- **Direct `.Semantic:Pattern` Token Evaluation & Offline AST Separation (`src/formats/dcstring/navigator/eval.rs`, `src/formats/dcdata/support/`)**:
  - `DcDocumentNavigator::eval_pattern_tokens` (`src/formats/dcstring/navigator/eval.rs:445-451`) evaluates `&[DcShorthand]` pattern slices directly from `SeedGraphIndex`.
  - `syntax_ast.rs` and `syntax_parser.rs` live under `src/formats/dcdata/support/` (though runtime `DatasetRuleResolver` still reconstructs `SyntaxPattern` alongside raw token slices).
- **`FramingShape` Precomputation, New-Dc Framing Validation, and `Copy` Splicing Checkpoints**:
  - `FramingShape` (`PrefixAtomArity` and `PrefixArity`) fast-paths are wired into `scan_sibling_stub_with_resolver` (`src/formats/dcstring/navigator.rs:1334-1369`), and dataset validation (`src/formats/dcdata/support/validation.rs`) requires deterministic framing for newly allocated Short Dcs above `Dc 634 "Dc format variation nickname"`.
  - `DcDocumentNavigator` replaced heap-cloned `VecDeque<DcChar>` in checkpoints with `spliced_buf: Vec<DcChar>` and `Copy` indices (`spliced_head`, `spliced_end` in `NavigatorCheckpoint`, `src/formats/dcstring/navigator/eval.rs:307-316`) plus precomputed `splice_trigger_tokens` (`src/formats/dcdata/syntax/resolver.rs:1375-1384`).
- **Initial Historical Graph Loading, Root Pin Detection, and Incremental Splicing (`src/formats/dcdata/seed_graph.rs`, `src/formats/dcstring/navigator.rs`)**:
  - `SeedGraphIndex::load_at_most_recent_from_asset_bundle` (`src/formats/dcdata/seed_graph.rs:324-344`) and `DcDocumentNavigator::detect_leading_document_most_recent` (`src/formats/dcstring/navigator.rs:289-317`) provide initial plumbing for `Dc 641 "Global graph system block most recent node ID at time of document creation"` (subject to the High-severity defects in Gate 1 below).
  - `DcDocumentNavigator` (`src/formats/dcstring/navigator.rs:355-408`, `1201-1211`) already preserves incremental expansion-before-framing for evaluated spliced streams (`Dc 256 "void lang.say([string] text)"` and WASM splice routines) while keeping raw source-span editing (`src/formats/dcstring/source_edit.rs`) on unevaluated frame stubs.

---

## Remaining Gaps / Pending Work (Gate-Driven Plan)

With the compatibility and replay contract defined in **Core Architectural Guarantee** above (and the earlier "four drastic phases" / static two-pass framing proposals retired), the remaining engineering work is organized into four actionable implementation gates.

### Gate 1: Make Snapshot & Block Pin Resolution Trustworthy (Fix Silent Substitution)
- [x] **Propagate pin-resolution errors instead of silently falling back to `HEAD` (`src/formats/dcstring/navigator.rs:430-496`)**:
  - In `DcDocumentNavigator::bind_document_most_recent_node_id_with_policy`, stop discarding `Err` from `syntax_resolver_at_most_recent(node_id)`. Record an explicit syntax/diagnostic error under the active `DcProjectionPolicy` while still allowing raw token/source inspection and forgiving recovery.
- [x] **Reject pins newer than the local graph (`src/formats/dcdata/seed_graph.rs:333-358`)**:
  - In `SeedGraphIndex::load_at_most_recent_from_asset_bundle`, only return `Arc::clone(shared)` when `most_recent_node_id == head_id`. When `most_recent_node_id > head_id` (and exceeds the highest available historical/active bundle ID), return a descriptive error stating that the document requires a newer System graph (`l{most_recent_node_id} > l{head_id}`) rather than silently substituting `HEAD`.
- [x] **Require complete historical graph bundles when resolving historical pins (`src/formats/dcdata/seed_graph.rs:360-445`)**:
  - When `most_recent_node_id < head_id` and active nodes have superseded/retracted prior statements, verify that `system_graph_history.rsrc` (`data/graph/system/*.ctbnbdl`) is present and covers the pinned revision without gaps, returning an explicit error if history is absent or incomplete.
- [x] **Enforce transitive snapshot isolation for all helpers (`src/formats/dcstring/converters/wasm.rs:1913-1975`, `src/formats/dcdata/support/seed_graph_reconstruct.rs`, `src/formats/dcdata/syntax/resolver.rs`)**:
  - Ensure that every referenced subpattern (`Dc 605 "Subpattern match"`), equivalence (`Dc 611 "Canonical decomposition"`), subtype (`Dc 587 "Is a subtype of"` / `Dc 586 "Is an instance of"`), and WASM helper or value node (`run_graph_wasm_tokens_on_dc_slice` in `src/formats/dcstring/converters/wasm.rs`) resolves strictly against the document's pinned `SeedGraphIndex` (filtering out any node with `id > most_recent_node_id` and applying only `Dc 591 "Supersedes"` / `Dc 592 "Retracts"` up to `most_recent_node_id`).
- [x] **Support header pin placement, nested-document inheritance, and explicit rejection of unsupported block pins (`src/formats/dcstring/navigator.rs:331-775`)**:
  - Support and validate `Dc 641 "Global graph system block most recent node ID at time of document creation"` both at the raw stream start and inside document/environment headers (`Dc 235 "Begin document"`..`Dc 236 "End document"`, `Dc 340 "Begin execution environment description"`..`Dc 341 "End execution environment description"`, `Dc 617 "Begin graph node definition"`..`Dc 618 "End graph node definition"`).
  - Propagate the enclosing document's pin to nested documents unless they declare their own scoped `Dc 641 "Global graph system block most recent node ID at time of document creation"`.
  - Reject references or pins targeting not-yet-implemented graph blocks (the reserved Data block beginning at `4294967296` and blocks above System) with an explicit unsupported-block diagnostic/error rather than ignoring them or substituting `HEAD`.
  - Define and enforce the explicit fallback policy for legacy documents that omit `Dc 641 "Global graph system block most recent node ID at time of document creation"` (`DcPinProvenance::UnpinnedLegacyFallback`, `DcProjectionPolicy::with_missing_system_graph_pin`, and `DcProjectionPolicy::with_unpinned_fallback_most_recent_node_id`).

#### Gate 1 Review (2026-11-08)

- Corrected missing pin-operand diagnostics, environment header discovery that
  selected pins from literals or nested document scopes, and legacy baseline
  selection that used HEAD to frame the first construct before installing the
  designated snapshot.
- Replaced recursive WASM helper expansion with an explicit work stack and
  retained snapshot/supersession/cycle checks. Added long-chain coverage and
  explicit rejection of dangling escapes.
- Historical completeness is checked only through the requested pin. Missing
  newer history must not reject an available older prefix; available graph
  bounds account for both supplied node lists as well as bundle metadata.
- Linux musl validation: all 192 Dc-string library tests and the focused
  historical snapshot/resolver test passed, with no ignored tests. Production
  Clippy for the two affected crates completed with baseline warnings.

**Remaining Gaps / Verification Limits**: Other platforms and the full Dc-data
suite were not run in this review. Kernel/VM budget work in Gate 2 and the real
cross-revision execution, event, and pixel replay suite in Gate 3 remain pending;
these parser/resolver tests do not establish the full compatibility guarantee.

### Gate 2: Document the Minimal Trusted Kernel & Enforce VM Budgets

**Resource terminology and required behavior**: A **compute quota** is the
document's allowance of logical work under the selected System supervisor's
versioned accounting rules. **Accounting** means deducting each operation's
specified cost from that allowance. Existing code calls its backend instruction
allowance **fuel**; that runtime-specific safety watchdog is not the portable
compute quota. Technical fuel names below describe existing APIs, not the
planned user-facing terminology.

Given identical document input, pinned definitions, supervisor revision,
effective graph-backed workspace configuration, and ordered external events,
the same compute quota must produce a user-facing permission request for
additional compute at exactly the same logical execution point on every WASM
runtime. Pause before the next operation whose cost exceeds the remaining
quota, before that operation has any effect. Runtime speed, optimization and
instruction dispatch counts must not move this boundary. Independent emergency
host interruptions remain distinct external events.

- [ ] **Freeze and document the minimal trusted bootstrap kernel (`docs/bootstrap-kernel-spec.md`, `src/formats/dcstring/navigator/policy.rs`, `src/formats/dcstring/navigator.rs`)** (Partially Implemented; specification remains draft until Gates 3–4 freeze the kernel boundary):
  - The replay guarantee assumes one explicitly specified foundational execution contract, immutable or selected by an explicit version (`BootstrapKernelVersion::V1 = 1`, `BootstrapKernelVersion::CURRENT`, `DcProjectionPolicy::with_kernel_version` / `try_with_kernel_version_number`, and upfront `DcDocumentNavigator::try_from_mstr_with_kernel_version_number` / `from_mstr_with_kernel_version` / `try_from_dc_str_with_kernel_version_number` / `from_dc_str_with_kernel_version` / `from_mstr_with_policy` / `from_dc_str_with_policy` before leading `Dc 641` pin detection).
  - Wrote [`docs/bootstrap-kernel-spec.md`](../docs/bootstrap-kernel-spec.md) and conformance tests (`test_gate2_bootstrap_kernel_v1_conformance_suite`, `test_gate2_zero_filled_static_data_segment_not_overwritten_by_mailbox_input`, `test_gate2_table_element_cap_and_multi_memory_aggregate_cap`) covering the host-implemented primitives that graph nodes rely upon:
    1. Physical token codecs (`DcTF` 1..7-byte and `0xFF` 8..23-byte tiers, `.dcmm` header, `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`).
    2. Active triple resolution in `SeedGraphIndex` (`Dc 584 "Has a"`, `Dc 610 "Has value"`, `Dc 591 "Supersedes"`, `Dc 592 "Retracts"`, `Dc 611 "Canonical decomposition"`, including retracted retractions and retracted supersessions).
    3. `.Semantic:Pattern` operators (`Dc 599 "Self reference"`, `Dc 600 "Wildcard pattern"`, `Dc 601 "Pattern conjunction"`, `Dc 602 "Pattern alternation"`, `Dc 603 "Pattern repetition"`, `Dc 604 "Negated pattern"`, `Dc 605 "Subpattern match"`, `Dc 606 "Pattern capture"`, `Dc 607 "Pattern replacement"`, `Dc 643 "Range"`, `Dc 644 "Exclusive range endpoint"`, `Dc 642 "Data byte length"`).
    4. Dc-WASM binary lowering (`Dc 647 "WASM unreachable"`..`Dc 1928 "End WASM program"`) and the 32-byte linear-memory mailbox ABI (`DC_WASM_MAILBOX_*` at bytes `0..32`, tracking `MemoryInstance::static_data_end()` so zero-filled static data segments are never overwritten).
- [x] **Enforce a backend intra-step safety watchdog in `tinywasm` (`vendor/tinywasm/`, `src/formats/dcstring/converters/wasm.rs`, `src/runtime/runtime.rs`)** (not portable semantic metering):
  - `DcWasmSession` configures per-step instruction fuel (`DcWasmResourceLimits::max_instructions_per_step`, default `1_000_000`) on `tinywasm::Store` during module instantiation, the WebAssembly `start` section (`run_wasm_start_section`), and each exported `step()` call, preventing tight loops (`Dc 650 "WASM loop"` .. `Dc 659 "WASM br"` `0`) from hanging.
  - Added `Trap::InstructionFuelExhausted` inside `tinywasm::Executor::exec_next` and mapped it to `WasmResourceInterruption::InstructionFuelExhausted` and `Privilege::ExcessiveCompute`.
  - Distinguished physical VM metering from logical document progress via `WasmExecutionTrace` and `WasmTraceEvent::ResourceInterruption`: safety-limit interruptions (including `PermissionDecision::Deny` and construction-time resource interruptions via `execute_wasm_bytes_with_trace` / `replay_wasm_bytes_trace`) preserve all accumulated `frames` and `events` in `WasmExecutionTrace`, while trace replay (`ExecutionContext::replay_wasm_session_trace`) compares logical capability requests and ignores physical instruction count differences across compilers.
- [x] **Enforce linear-memory, table-element, and output-slice resource budgets (`vendor/tinywasm/`, `src/formats/dcstring/converters/wasm.rs`)**:
  - Capped store-wide and per-instance linear memory at `256` pages (`16 MiB` total across `Store::total_memory_pages()`), store-wide and per-table elements at `65_536` (`DEFAULT_MAX_WASM_TABLE_ELEMENTS` / `WasmResourceInterruption::TableLimitExceeded`), single-step output at `4 MiB`, and cumulative output at `16 MiB`. Guest `memory.grow` and `table.grow` failures return WASM `-1`; host input/instantiation allocation failures return `MemoryLimitExceeded` or `TableLimitExceeded`. A step rejects an exported memory or table already above a subsequently lowered limit.
- [x] **Keep capability authorization strictly in the host (`src/runtime/runtime.rs`)**:
  - Historical graph pinning (`Dc 641 "Global graph system block most recent node ID at time of document creation"`) applies only to language/document semantics and never grants or restores security permissions or capability policies (`ExecutionContext::set_privilege_decision` / `with_privilege_decision`).
  - `ExecutionContext::execute_wasm_session_with_trace` and `ExecutionContext::replay_wasm_session_trace` compare the document's capability requests at each logical step during replay and supply the recorded allow/deny/mock responses while live execution remains governed by the current host policy.
- [ ] **Define and implement portable resource accounting under the trusted System supervisor**:
  - Distinguish logical budget policy from emergency host watchdogs. Current fuel
    counts optimized/lowered `tinywasm` interpreter dispatches, not a versioned
    sequence of source WASM operations; replaying a recorded interruption does
    not make fresh executions on independent runtimes equivalent.
  - Hardcode only the workspace bootstrap entry document: one short Dc, with
    no graph pin. On startup, resolve its latest trusted System-block definition
    from the application-bundled trusted base and run that System document as
    the privileged top-level supervisor. Wikidata and other extra resources
    cannot supply bootstrap authority; check statement provenance, not just
    System-block numeric membership. No particular short Dc is assigned here.
  - Define logical limits, accounting, exhaustion policy and user configuration
    in the selected System document, not in duplicate host policy. The host
    provides minimal bootstrap/execution primitives, isolation and external
    capability mediation. Ordinary documents cannot elevate themselves by
    declaring privilege, choosing a historical pin or referencing the entry Dc.
    Independent host watchdogs are process safeguards, not logical budget policy.
  - Choose a supervisor-owned interpreter or a validated, non-bypassable
    standard-WASM instrumentation path. A supervisor merely running beside
    uninstrumented native WASM cannot count/preempt its instructions. Instrument
    the standard WASM bytecode produced by transpiling the pinned Dc definitions,
    before any runtime-specific lowering or optimization; do not instrument the
    runtime implementation or use its instruction counts as logical costs.
    Specify start/call/branch/bulk-operation costs, deduct-before-effect,
    overflow, quota replenishment and exact permission-request delivery points.
    Preserve a continuation so an authorized additional allowance resumes the
    paused operation exactly once without repeating earlier effects.
  - Resolve and validate workspace graph configuration through the selected
    System document once at task admission, then use local integer counters
    rather than graph lookups per operation. Nested evaluations and newly
    created sessions must share or debit the supervising task's allowance;
    yields and capability requests must not silently replenish it. Batch
    accounting only when it preserves the exact operation-level exhaustion,
    trap and effect boundary; use exact accounting near quota exhaustion.
  - Make memory/table quotas explicit supervisor policy and configuration,
    including host mailbox/scratch allocations. A physical allocation failure below a
    logical quota must be an external interruption, not a guest-visible `-1`
    that changes document branching. Define output charging/delivery similarly.
  - Record the actual trusted supervisor revision, dependencies and effective
    configuration for replay; do not pin the startup entry or let ordinary
    document pins select an older supervisor. Compare the same pinned child
    source, recorded supervisor revision/configuration and ordered external
    events under independent engines. Verify that fresh executions request
    additional compute before the same logical operation, with identical prior
    effects, and resume identically after the same authorization response.
    Include exact-quota boundaries, start sections, loops, branches, calls,
    bulk operations, batching and repeated helper evaluations. Same-engine tests and replay
    against NOP-padded recompilations are not sufficient. See
    [the resource contract](../docs/bootstrap-kernel-spec.md#45-portable-resource-semantics-versus-host-watchdogs).

#### Gate 2 Status & Remaining Gaps vs Upstream / Plan (2026-11-08)

- **Partially Implemented**:
  - Upfront bootstrap kernel version constructors (`try_from_mstr_with_kernel_version_number`, `from_mstr_with_kernel_version`, `try_from_dc_str_with_kernel_version_number`, `from_dc_str_with_kernel_version`), exact static/zero-filled data segment high-water mark (`MemoryInstance::static_data_end()`), store-wide memory page and table element caps (`max_table_elements`), trace preservation and replay on denied/construction resource interruptions (`execute_wasm_bytes_with_trace`, `replay_wasm_bytes_trace`), and conformance tests for `DcTF`, `.dcmm` binary encapsulation, `SeedGraphIndex` retraction/supersession lifecycles, and WASM numeric traps are implemented and tested.
  - Resolved the 6 `E0277` slice-to-array `Option` comparison errors in `ctb-formats-detection` (`src/formats/detection/container.rs` and `src/formats/detection/droid.rs`) so `./scripts/run-dev-ctb` builds cleanly.
- **Remaining Gaps / Pending Work**:
  - Portable resource accounting and the privileged supervisor path above are
    not implemented. The fork remains a host backend, not a language definition.
  - `docs/bootstrap-kernel-spec.md` remains `Partially Implemented` / draft until Gates 3 and 4 eliminate remaining Rust procedural syntax-rule synthesis and complete self-hosted `.WASM` definitions.
  - Instruction fuel meters VM instructions rather than initial module parsing/validation or per-byte work inside bulk memory/table instructions (`memory.copy`, `memory.fill`, `table.copy`).
  - Non-resource traps during module `start` or `step()` return `Err` rather than a structured trap event in `WasmExecutionTrace`.
  - Cross-platform verification and end-to-end UI event/pixel replay remain pending.

#### Revised Gate 2 Review (2026-11-08)

- Fixed double execution of the WASM start section and implicit invocation of
  an exported `_start` without a start section. Explicit instantiation now
  precedes exactly one core-WASM start-section call under the watchdog.
- Preflight initial memory/table declarations using the existing validated
  parser. Report their actual total requested counts before allocation, instead
  of interpreting backend error text and inventing `limit + 1` counts.
- **Pending Work**: portable supervisor/metering implementation and independent
  resource-boundary conformance remain unresolved. The new start-section fixture
  also passes V8, but that narrow check does not establish resource portability.
- **Validation**: all 200 Dc-string and eight runtime library tests passed on
  Linux musl, with no ignored tests. Production Clippy completed with baseline
  warnings. Documentation links and whitespace passed. No additional runtime
  fork changes were needed for these fixes; other platforms and portable budget
  enforcement remain unverified.

#### Existing Dc Semantics: Evidence and Regression Requirements

- Existing Dcs may be reused when their historical meaning fits. Names alone
  are not sufficient evidence of arity, position, delimiters, or scope.
- The change in `333fd1f0b` added identifiers and framing for
  Dc 235 "Begin document" and Dc 236 "End document", not new names or a new
  semantic assignment. The historical CSV declares their opener/closer pairing
  ([dcs.csv](../old/eite-older/php/dcs.csv)); the converter actually emits them
  around converted content ([ember.php](../old/eite-older/php/ember.php)). This
  supports the pairing, but does not independently establish modern nested pin
  semantics.
- Scope recognition now requires a container closer in the selected resolver,
  so an old snapshot without the newer document framing cannot gain that
  behavior through a procedural constant check. A real historical-snapshot
  regression accompanies the existing nested-scope/pin tests.
- For future existing-Dc grammar changes, record historical declarations and
  actual usage in the review/commit, explain how proposed semantics agree,
  and test both an older pin and the new graph definition. If evidence conflicts,
  resolve that conflict before changing the decoder. Recognition must come from
  the selected graph, including equivalences, rather than a HEAD-only allowlist.
  Add graph changes through the append-only generator; never rewrite old IDs
  or treat accepting an old token under a new grammar as proof of compatibility.

### Gate 3: Prove One End-to-End Vertical Migration with Real Graph History
- [ ] **Migrate one concrete Rust-defined behavior into the system graph and remove its procedural fallback**:
  - Select a small self-contained behavior currently hardcoded in Rust (for example, `Dc 256 "void lang.say([string] text)"` payload extraction in `DefaultDcExpressionEvaluator::eval_stream_splice` at `src/formats/dcstring/navigator/eval.rs:260-300`, or a specific format/value rule), emit its definition through `src/formats/dcdata/support/graph_ledger.rs` into `.ctbnodes` / `.ctbnbdl`, and delete the corresponding procedural Rust fallback.
- [ ] **Build the end-to-end historical-behavior and trace-replay regression suite**:
  - Using real generated graph history (`Dc 591 "Supersedes"` / `Dc 592 "Retracts"` across active and historical `.ctbnbdl` bundles):
    1. Author a document pinned to `Dc 641 "Global graph system block most recent node ID at time of document creation"` at revision $R_1$.
    2. Append a modified definition at revision $R_2 > R_1$ via the ledger generator.
    3. Preserve the original document octets and record its observable trace and external responses/events. Verify that it resolves the $R_1$ definition and replays the identical trace after the graph update and the Rust behavior migration. Compare requests before supplying recorded responses and compare output bytes and outcomes, not internal parser/VM representations. A document pinned to $R_2$ must use the updated $R_2$ definition.
    4. Verify that a document pinned to an unavailable future revision $R_3 > R_2$ produces an explicit diagnostic/error rather than silently running against $R_2$.
    5. Add focused replay cases for denied/mocked capabilities and asynchronous event ordering at defined logical delivery points, using mocks or in-memory transports. Different external responses may change output, but must not change requests or outputs that precede their delivery.
    6. Add canonical raster-output cases with explicit font and format-chain inputs, including subpixel rendering and vertical/horizontal pixel alignment. Compare exact pixel bytes across implementation revisions and supported hardware; changing rendering configuration changes inputs, not the compatibility guarantee.
    7. Exercise internal semantic constructs through observable trace outputs (framing boundaries for `Dc 590 "Is a closing delimiter of syntactic block for"`, `Dc 255 "Ignore following Dc"`, and `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`; decoded values for `Dc 6 "Begin number"`..`Dc 7 "End number"`, `Dc 260 "Begin literal value"`..`Dc 261 "End literal value"`, and `Dc 270 "Begin identifier"`..`Dc 271 "End identifier"`; lexical bindings for `Dc 541 "Enter identifier scope"`..`Dc 542 "Exit identifier scope"`, `Dc 269 "Assignment operator"`, `Dc 268 "void lang.assign([identifier] identifier, [value] value)"`, `Dc 540 "Reference bound value"`, `Dc 276 "Reference named object"`, `Dc 638 "For each"`, and `Dc 639 "This element"`; spliced streams for `Dc 256 "void lang.say([string] text)"` and `Dc 397 "Equivalent representations"` / `Dc 561 "Partially equivalent representations"`; and forgiving-recovery diagnostics).

### Gate 4: Incremental Expansion & Cleanup (Post-Proof)
- [ ] **Eliminate implicit Rust rule synthesis in `DatasetRuleResolver::from_seed_graph` (`src/formats/dcdata/syntax/resolver.rs`)**:
  - Emit explicit `Dc 590 "Is a closing delimiter of syntactic block for"` statements into `.ctbnodes` so container closers do not rely on `detect_container_closer` heuristics (`src/formats/dcdata/syntax/resolver.rs:1716-1734`).
  - Emit explicit graph rules/subtypes for `.EL Types` literal constructs so runtime loading does not scrape English canonical names (`extract_el_type_name`).
- [ ] **Migrate remaining procedural constraint & value evaluators (`src/formats/dcstring/navigator/eval.rs`)**:
  - Incrementally migrate subtype constraints (`number_satisfies_subtype`, `type_tokens_satisfy`, `eval_list_constraints`, `eval_data_construct`, `eval_typed_data_construct`), lexical bindings (`Dc 269 "Assignment operator"`, `Dc 540 "Reference bound value"`, `Dc 638 "For each"`), and representation sets (`Dc 397 "Equivalent representations"`, `Dc 561 "Partially equivalent representations"`) into pinned graph definitions, retiring each duplicate Rust representation only after its vertical compatibility test passes.
- [ ] **Enforce graph WASM source-provenance linting (`issues/issues.md`)**:
  - Add a dataset/graph linter ensuring that only foundational primitive Dcs use direct `.WASM` definitions in the graph, and all higher-level routines in the graph are accompanied by their human-readable/editable high-level language source so opaque unattributed WASM cannot enter the graph.

---

## Deferred Items (Not Required for Compatibility Closure)

### 1. Tier 0 & Tier 1 Textual Languages (`Dcsh`+ and VSO / Polish Prefix Language)
- **Status**: `Deferred` until Gates 1–3 are established.
- **Preserved Design Notes from [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md)**:
  - **Tier 0 (`Dcsh`+ assembler)**: Minimal human-writable assembler extending `src/formats/dcstring/converters/dcal.rs` with `@ident` resolution, inline literals, and automatic closer insertion via `Dc 590 "Is a closing delimiter of syntactic block for"`.
  - **Tier 1 (VSO / Polish Prefix Semantic & Programming Language in `src/formats/dct_el/`)**:
    - General-purpose English-like VSO / Polish prefix language usable for both code and UI localization (transpiling to semantic Dcs and projecting into natural languages or WASM).
    - Two lexical profiles mapping to the same token stream:
      - **Speakable profile**: `Doing see I you.`, `Is to dipre meet I you par good.`, `Let var foo add 3 4.`, `Let var int32 var bar var foo.`, `Set var bar mul var bar dipre daun.`, `Let verb i32 sum i32 p1 i32 p2 par add p1 p2.`
      - **Compact code profile**: `doing see I you`, `is to is meet I you } good`, `let /foo add 3 4`, `let i32/bar foo`, `set /bar mul /bar /bar`, `let v/i32/sum i32/p1 i32/p2 } add p1 p2;`
    - **Function / verb literal conventions**:
      - Named routine: `let v/i32/sum i32/p1 i32/p2 } add p1 p2;`
      - Anonymous verb literal: `v/i32/ p1 p2 } ...`
      - Untyped/any return (`Dc 275 "Built-in type identifier: * (Object or any subtype)"`): `v//...`
      - Void / no-return routines: explicit `v/void/log` or `v/never/log` (rather than `v//log`).
    - **Repetition Dcs (`dipre`, `daun`, `dato`)**: Compact 1-token repetition of the word before previous (`dipre`) or previous word (`daun`, `dato`) to provide elegant variation when spoken.
    - **Possibly new anaphoric / backreference Dcs**: Compact 1-token backreferences to the previous clause or previous identifier to avoid repeating long identifiers or assigning intermediate node IDs.

### 2. Wholesale CST (`DcSyntaxKind`) Rewrite
- **Status**: `Deferred`.
- Replacing `DcSyntaxKind` variants in `src/formats/dcstring/navigator/arena.rs` or removing compiled lookup caches in `DatasetRuleResolver` is not required for backwards compatibility; compiled Rust tables and AST caches are acceptable wherever they faithfully and deterministically reflect the document's pinned graph snapshot.
I’ll read Gate 2 and the nearby execution and workspace configuration code, then assess how graph-backed budgets could stay runtime-independent without adding costly lookups during execution.


**Remaining Gaps**
Gate 2’s direction is sound, but portable accounting, graph-backed configuration, and supervisor execution remain unimplemented. This was a read-only review; no tests or performance benchmarks were run.

**Findings**
1. **High: session budgets can be bypassed through repeated evaluations.** `wasm.rs:2427` create fresh sessions with default limits, including calls from `eval.rs:115`. A document can repeatedly obtain new compute and output allowances. Budgets must belong to the supervised document/task tree, not individual VM sessions.

2. **High: resumable exhaustion needs a new execution contract.** `wasm.rs:2267` is a synchronous function call; the exposed API has no portable continuation. Injecting a trap can stop execution, but cannot safely resume its lost call stack and locals. Budget extensions require either explicit continuations or a supervisor-owned interpreter.

3. **High: physical allocation failures are not safely separated from logical quotas.** `executor.rs:527` returns `-1` for denied growth, while `memory.rs:177` and `table.rs:161` use infallible vector allocation. Allocator exhaustion can abort the process instead of producing a recorded external interruption.

4. **Medium: existing traces cannot identify an intra-step logical boundary.** `runtime.rs:281` record only completed mailbox steps. That supports terminal-prefix replay, but not exact interrupted-operation verification or continuation.

5. **Medium: bootstrap trust needs stronger evidence than graph declarations.** The `seed_graph.rs:164` records declared provenance. Bootstrap authority must come from authenticated bundled-origin information, including supersession and dependency statements, not merely a statement claiming trusted provenance.

**Recommended Design**
Use **supervisor-selected, validated standard-WASM instrumentation** for the performance path. A supervisor-owned interpreter remains the simpler correctness reference.

- **Graph-backed policy:** The workspace System document defines the accounting model, defaults, configuration validation, inheritance, and exhaustion behavior. Store user settings as a typed graph document referenced by that supervisor. Resolve and validate it once at task admission into compact execution data; never query the graph per instruction. Property names and semantic identifiers still need design and approval.

- **Separate accounting from scheduling:** Use lifetime compute/output counters, live memory/table quotas, and a scheduling quantum. Yielding or requesting a capability must not replenish lifetime budgets. Child tasks and graph helpers debit the parent allowance; creating another session cannot create resources.

- **Portable charging:** Instrument the pinned source WASM before backend optimization. Specify costs for executed operations, calls, branches, start sections, bulk-operation lengths, and allocation initialization. Exclude injected bookkeeping from charges. Define checked fixed-width arithmetic, charge-before-effect, and precedence between budget exhaustion and ordinary traps.

- **Fast execution:** Batch fixed-cost instructions where safe. When enough budget remains, execute the batch with one check; near exhaustion, use exact operation-level charging. Do not precharge an entire block if an earlier trap or effect would change the observable prefix. Dynamic bulk operations need separate checks.

- **Non-bypassable suspension:** Transform execution into explicit continuations that can return to the supervisor at accounting checkpoints. Protect accounting state from guest memory writes and validate every entry, call target, growth operation, and start-section path. The current import-free mailbox ABI cannot invoke the supervisor halfway through an ordinary synchronous call without such a transformation.

- **Memory and output:** Account for guest-visible mailbox/scratch allocations deterministically. Logical growth denial may return `-1`; physical failure within an approved quota must externally interrupt. Charge output before publication, specifying whether capability payloads, responses, and retained trace buffers consume separate allowances. Physical backend overhead remains watchdog territory.

- **Configuration and replay:** Record the actual supervisor revision, dependency snapshots, accounting version, effective configuration bytes, and logical interruption checkpoint. Apply configuration changes at explicit ordered boundaries, not through mutable graph reads during execution. Concurrent children need specified grant ordering if they share quotas.

**Verification**
Gate closure should require independent-engine comparisons at exact budget boundaries, including start sections, loops, traps, bulk operations, growth, and repeated graph-helper calls. Add adversarial tests for accounting-state tampering and continuation resumption, plus mocked physical allocation failures.

Benchmark cached startup, steady-state instrumented execution, near-exhaustion fallback, and bulk operations separately. The performance target should be **local integer accounting with infrequent supervisor transitions**, not runtime-specific fuel equivalence.