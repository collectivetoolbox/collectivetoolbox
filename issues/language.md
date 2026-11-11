# Language Layers (2026-11-11)

The language design is split into three languages; these names are distinct from
the implementation-stage directory numbers below:

- **StageL 3** is the core language, to be implemented directly in WAT. Its types
  are `b`, `letters`, `u8`, `i32`, `u32`, `i64`, `u64`, lists `l/T` of supported
  element types, `void`, and `never`. Named functions, including nested functions,
  are directly callable but are not first-class values. `v/RETURN/name` is
  declaration syntax, not a function-value type.
- **StageR 3** builds on StageL 3 and adds the 128-bit primitives `i128` and
  `u128`, arbitrary-precision `int` and `rat`, `dc`, `view`, `cursor`, `s`
  (DcMstr), `dcs` (DcStr), `option/T`, and `result/T`. These are not StageL 3
  core types. StageR 3 does not add first-class functions.
- **EM3** is the full VSO language built on StageR 3, with speakable and compact
  code profiles. Only EM3 supports first-class functions: function values,
  callbacks passed as values, anonymous functions, and functions returned from
  other functions, including escaping closures.

All three languages are **expression-oriented, like Lisp**. Every normally
completing expression returns a value. A block or function returns the value of
its final expression; earlier expressions are evaluated for their effects and
their values discarded. `void` denotes the unit value, not an absence of a value:
declarations, `new`, `set`, normally completed loops, and empty blocks return
unit. `never` describes an expression that does not return normally. An `if`
returns its selected branch's value; the branch types must agree, with `never`
compatible with the other branch. Dot and normalized semicolon separate
expressions without discarding the final value.

**Tail calls do not grow the call stack**, including mutual tail calls. This is
a language guarantee, not an optional optimization. A call is in tail position
when its result is returned directly, including in the selected branch of a
tail-position `if` or the final expression of a returned block. Non-tail calls
can still grow the stack; tail calls do not imply bounded heap allocation.

**Nested functions capture lexical bindings from where they are defined, not
from where they are called.** Captures refer to shared bindings, so mutations
are visible through the capture. A caller's local variables do not replace the
definition-site environment. StageL 3 and StageR 3 nested functions cannot
escape as values; EM3 closures can, keeping captured bindings alive after the
defining invocation returns. Loop bodies and conditional branches in the core
are expressions, not first-class callbacks.

The intended implementation stages are:

- **Stage 0 (`src/formats/dcdata/data-src/stage_0_impls/`)** uses WAT for the
  foundations, including the implementation of StageL 3.
- **Stage 1 (`src/formats/dcdata/data-src/stage_1_impls/`)** uses StageL 3 for
  the StageR 3 extensions, then StageR 3 for the full EM3 language implementation.
- **Stage 2 (`src/formats/dcdata/data-src/impls/`)** uses EM3 for the remaining
  high-level definitions and the full VSO code/localization language described
  in [the language design discussion](../docs/chats/2026-10-31-language.md).

## Pending Work

These are design requirements, not implemented compiler/runtime guarantees.
The StageL 3 source normalizer does not implement type checking, execution,
lexical capture, or tail-call lowering. Core arithmetic failure behavior must
be specified without relying on StageR 3 `result/T`; StageR 3 checked arithmetic
uses result values. ABI, ownership, definite initialization across nested calls,
stack-safe tail-call lowering with captures, and EM3 closure lowering remain
to be specified and implemented.


> Using i32.clz((255 - b0) << 24) (Dc 750) computes the DcTF byte length for all 128 <= b0 < 255 in a single expression!

If you run across things like this that are reused, it's potentially worth allocating a new Dc to keep it DRY.

# Graph-Defined Dc Language, Execution Kernel, and Compatibility Plan

**Overall Status**: `Partially Implemented`

Reviewed against [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md) and source at `4cf7ad77baa56c8da382f9d606c1ec03781d439c` on 2026-11-08.

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
4. Evaluated streams must preserve incremental expansion-before-framing, kept
  distinct from unevaluated source/CST inspection. Evaluated values contribute
  to the final document string; printing is not a separate effect that removes
  those values from evaluation. `Dc 256 "void lang.say([string] text)"` emits
  its string with a final line feed if needed. An emitted delimiter may close
  a block in that evaluated document even when it was quoted in the source
  argument.

---

## Completed Foundations (Cleaned Up from Earlier Proposals)

- **Graph-Owned Routine Acceptance, Execution Dispatch, and Dc-WASM Routine Header Lowering (2026-11-09)**:
  - The selected System graph's named `Routine` and `wasm_routine` patterns control
    literal acceptance, direct expression execution, and Dc-WASM routine header
    lowering in both [`eval.rs`](../src/formats/dcstring/navigator/eval.rs) and
    [`wasm.rs`](../src/formats/dcstring/converters/wasm.rs). Removed both the
    redundant post-match Rust header validation and fixed-position `Dc 260 "Begin literal value"`
    header assumptions; `dcstring_to_wasm_with_resolver` and
    `parse_wasm_routine_from_cursor` extract leading and post-signature routine
    header tokens from the active `SyntaxRuleResolver`'s `Routine` and
    `wasm_routine` rules and validate routines against the active graph grammar
    using an inert evaluator (`tokens_match_named_construct_with_resolver`) so
    candidate values are never executed recursively. Foundational Dc-WASM opcode
    body lowering remains in the kernel.
  - `test_gate3_routine_header_uses_generated_graph_history` in
    [`e2e_dc_syntax.rs`](../src/formats/dcstring/tests/e2e_dc_syntax.rs)
    generates ledgers from the real CSV grammar, modifies only `Routine`,
    preserves assigned nodes, and roundtrips active/history `.ctbnbdl`
    bundles. It checks unchanged pinned document octets, grammar diagnostics,
    exact completion bytes and pure-session replay, revised-rule execution
    (including non-WASM leading prefix tokens such as `u0052` `'R'` as well as
    post-signature header tokens), direct `DcWasmSession::from_dc_wasm_with_resolver`
    and `DefaultDcExpressionEvaluator` graph rejection without a procedural
    fallback, and historical/future-pin errors.
    Synthetic snapshots use explicit caller resolvers; a separate execution
    case exercises normal pin discovery from real bundled graph history.
  - The existing generated production rule needs no new assignment or
    historical backfill. The oldest tested snapshot retains grammar coverage;
    WASM execution uses a newer bundled historical snapshot containing its
    instruction definitions. Capability/event and raster replay remain pending
    under Gate 3; this is not completion of the wider execution migration.
  - Verification: Linux musl dcstring library tests (211) and syntax integration
    tests (48) pass, with zero failures or ignored tests. Scoped dcstring Clippy
    passes for the library and syntax integration target.

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
- **Historical Snapshot Resolution, Pin Validation, Scoped Inheritance, and Transitive WASM Splicing (`src/formats/dcdata/seed_graph.rs`, `src/formats/dcdata/syntax/resolver.rs`, `src/formats/dcstring/navigator.rs`, `src/formats/dcstring/converters/wasm.rs`)**:
  - `SeedGraphIndex::load_at_most_recent_from_asset_bundle` (`src/formats/dcdata/seed_graph.rs:479-520`) rejects pins outside the System block (`l8589934592..=l12884901887`), rejects pins newer than the local graph (`most_recent_node_id > head_id`), and requires complete historical `.ctbnbdl` bundle coverage through `most_recent_node_id` whenever superseded (`Dc 591 "Supersedes"`) or retracted (`Dc 592 "Retracts"`) statements affect the snapshot.
  - `DcDocumentNavigator` (`src/formats/dcstring/navigator.rs`) propagates pin-resolution errors into diagnostics instead of silently substituting `HEAD`, supports `Dc 641 "Global graph system block most recent node ID at time of document creation"` in raw stream prefixes and document/environment headers (`Dc 235 "Begin document"`..`Dc 236 "End document"`, `Dc 340 "Begin execution environment description"`..`Dc 341 "End execution environment description"`, `Dc 617 "Begin graph node definition"`..`Dc 618 "End graph node definition"`), honors `policy.unpinned_fallback_most_recent_node_id` before root header pin discovery and on non-stub navigation paths (`effective_resolver()`, `peek_token()`, `read_next_token()`) as well as policy-aware `.dcmm` file constructors (`from_dcmm_file_with_kernel_version`, `try_from_dcmm_file_with_kernel_version_number`, `from_dcmm_file_with_policy`), inherits the enclosing document's pin into nested documents unless overridden, rejects pins/references targeting unimplemented blocks (`4294967296..8589934592` and `> 12884901887`), and preserves incremental expansion-before-framing for WASM-backed splice routines while debiting the navigator's shared `SharedWasmTaskQuota`.
  - `expand_graph_wasm_shorthands` and `dcstring_to_wasm_with_resolver` (`src/formats/dcstring/converters/wasm.rs`) expand both `DcShorthand::Long(node_id)` references and transitive Short-Dc WASM helpers resolved via `Dc 611 "Canonical decomposition"`, enforcing snapshot pin bounds, unsupported-block checks, retraction checks, and cycle detection.
- **Portable `.wasm` Bytecode Instrumentation, Logical vs. Physical Growth Separation, and Trusted Supervisor Bootstrap Verification (`src/formats/dcstring/converters/wasm_quota.rs`, `src/formats/dcdata/seed_graph.rs`, `src/runtime/runtime.rs`)**:
  - Restored unmodified upstream `tinywasm 0.8.0` and implemented `instrument_wasm_module_bytes` (`src/formats/dcstring/converters/wasm_quota.rs`) with `deduct-before-effect` compute-quota accounting in appended WASM globals (`$__ctb_deduct`, `$__ctb_deduct_batch`), recording `last_interrupted_op_ordinal` and `last_interrupted_op_required_cost` on `WasmResourceInterruption::ComputeQuotaExhausted` (`Privilege::ExcessiveCompute`).
  - Enforced store-wide and per-instance logical memory page (`256` pages = `16 MiB`) and table element (`65_536`) limits returning WASM `-1` on logical denial while raising `MemoryLimitExceeded` / `TableLimitExceeded` on physical allocation failures below the logical cap, plus separate step output (`4 MiB`), session output (`16 MiB`), capability response payload (`4 MiB`), and trace buffer (`16 MiB`) caps.
  - Implemented `SeedGraphIndex::resolve_trusted_system_supervisor_bootstrap_tokens` and `verify_authenticated_bundled_system_provenance` (`src/formats/dcdata/seed_graph.rs:568-731`) to verify that the unpinned 1-Short-Dc startup bootstrap entry resolves strictly from authenticated application-bundled System statements.

---

## Remaining Gaps / Pending Work (Gate-Driven Plan)

With the compatibility and replay contract defined in **Core Architectural Guarantee**
above, the remaining engineering work is organized into the implementation gates
below.

#### Existing Dc Semantics: Evidence and Regression Requirements

- Existing Dcs may be reused when their historical meaning fits. Names alone
  are not sufficient evidence of arity, position, delimiters, or scope.
- The change in `333fd1f0b` added identifiers and framing for
  `Dc 235 "Begin document"` and `Dc 236 "End document"`, not new names or a new
  semantic assignment. The historical CSV declares their opener/closer pairing
  ([dcs.csv](../old/eite-older/php/dcs.csv)); the converter actually emits them
  around converted content ([ember.php](../old/eite-older/php/ember.php)). This
  supports the pairing, but does not independently establish modern nested pin
  semantics.
- Scope recognition requires a container closer in the selected resolver, so an
  old snapshot without the newer document framing cannot gain that behavior
  through a procedural constant check.
- For future existing-Dc grammar changes, record historical declarations and
  actual usage in the review/commit, explain how proposed semantics agree,
  and test both an older pin and the new graph definition. If evidence conflicts,
  resolve that conflict before changing the decoder. Recognition must come from
  the selected graph, including equivalences, rather than a `HEAD`-only allowlist.
  Add graph changes through the append-only generator; never rewrite old IDs
  or treat accepting an old token under a new grammar as proof of compatibility.

### Gate 2: Remaining Minimal Trusted Kernel & VM Budget Work

**Resource terminology and required behavior**: A **compute quota** is the
document's allowance of logical work under the selected System supervisor's
versioned accounting rules. **Accounting** means deducting each operation's
specified cost from that allowance (**deduct-before-effect**), and **increasing
the compute quota** means granting an additional allowance when authorized.
Runtime-specific instruction counters are not a portable compute quota and must
never determine logical interruption points.

Given identical document input, pinned definitions, supervisor revision,
effective graph-backed workspace configuration, and ordered external events,
the same compute quota must produce a user-facing permission request for
additional compute (`Privilege::ExcessiveCompute` /
`WasmResourceInterruption::ComputeQuotaExhausted`) at **exactly the same
logical execution point regardless of what WASM runtime it runs on** (using
unmodified standard WASM engines without needing a forked WASM runtime). Pause
before the next source WASM operation whose cost exceeds the remaining compute
quota, before that operation has any effect. Runtime speed, optimization, and
backend instruction dispatch counts must not move this boundary. Independent
emergency host interruptions remain distinct external events. See
[the resource contract](../docs/bootstrap-kernel-spec.md#45-portable-resource-semantics-versus-host-watchdogs).

- [ ] **Complete and freeze the minimal trusted bootstrap kernel** (`docs/bootstrap-kernel-spec.md`):
  - [x] Select the kernel version (`BootstrapKernelVersion::V1`) before leading pin discovery and token decoding on every navigation path, including file-backed documents (`src/formats/dcstring/navigator/navigator.rs`).
  - [x] Specified Section 5 ("Independent Conformance Fixtures") in [`docs/bootstrap-kernel-spec.md`](../docs/bootstrap-kernel-spec.md) and added `gate2_bootstrap_kernel_conformance_fixtures` in [`src/formats/dcstring/tests/e2e_dc_syntax.rs`](../src/formats/dcstring/tests/e2e_dc_syntax.rs) covering:
    - Physical token codecs (`.dctf`, `.dcutf`, `.dcmm` v2, `.dcmm1` legacy v1) and malformed-input recovery (`test_gate2_conformance_physical_token_codecs_and_malformed_recovery`).
    - Active graph statement and equivalence resolution across `Dc 584 "Has a"` + `Dc 610 "Has value"`, transitive `Dc 591 "Supersedes"` chains, `Dc 592 "Retracts"`, retracted retractions, and `Dc 397 "Equivalent representations"` (`test_gate2_conformance_active_graph_statement_and_equivalence_resolution`).
    - `.Semantic:Pattern` operators (`Dc 602 "Pattern alternation"` ordered choice with capture rollback on backtrack, `Dc 606 "Pattern capture"`, `Dc 643 "Range"` 3-operand arity with `Dc 644 "Exclusive range endpoint"`, `Dc 603 "Pattern repetition"`, `Dc 605 "Subpattern match"` 0-token cycle detection and `MAX_SYNTAX_EXPANSION_DEPTH = 32` enforcement) and deterministic rejection of malformed/truncated/ungrouped `.Semantic:Pattern` operator sequences (`test_gate2_conformance_semantic_pattern_operators_and_malformed_patterns` and `src/formats/dcdata/support/seed_graph_reconstruct.rs`).
    - Dc-WASM lowering (`Dc 1927 "Begin WebAssembly expression"`..`Dc 1928 "End WebAssembly expression"`) and the 32-byte linear-memory mailbox ABI (`0..32`, including `Dc 641 "Global graph system block most recent node ID at time of document creation"` little-endian `u128` at `16..32` and `YieldFrame` / `RequestCapability` / `Complete` statuses) (`test_gate2_conformance_dc_wasm_lowering_and_mailbox_abi`).
  - [ ] Remaining gap: Keep `docs/bootstrap-kernel-spec.md` `Partially Implemented` and draft until the self-hosted System supervisor resource migration and Gate 3/4 migrations are resolved; then freeze `BootstrapKernelVersion::V1`.
- [ ] **Move resource accounting policy and configuration under the trusted System supervisor document in the graph**:
  - Distinguish logical budget policy from emergency host watchdogs.
  - Hardcode only the workspace bootstrap entry document: one short Dc, with
    no graph pin. On startup, resolve its latest trusted System-block definition
    from the application-bundled trusted base and run that System document as
    the privileged top-level supervisor. Wikidata and other extra resources
    cannot supply bootstrap authority; check authenticated bundled-origin
    statement provenance (including supersession and dependency statements), not
    just System-block numeric membership or unverified provenance claims. No
    particular short Dc is assigned here.
  - Define logical limits, accounting rules, exhaustion policy, and user
    configuration validation/inheritance in the selected System document, not
    only in host Rust defaults. Store user settings as a typed graph document
    referenced by that supervisor, resolve and validate it once at task
    admission into compact local integer counters, and never query the graph per
    instruction. Ordinary documents cannot elevate themselves by declaring
    privilege, choosing a historical pin, or referencing the entry Dc.
    Independent host watchdogs remain process safeguards, not logical budget
    policy.
  - Batch fixed-cost instructions only when safe (when enough compute quota
    remains and no earlier trap or effect inside the block could change the
    observable prefix; near exhaustion, use exact operation-level deduction, and
    never pre-deduct across an operation whose earlier trap or effect would
    alter observable state).
  - Deduct output allowance before publication and specify whether capability
    payloads, responses, and retained trace buffers consume separate allowances.
  - Record the actual trusted supervisor revision, dependency snapshots,
    accounting version, effective configuration bytes, and logical interruption
    checkpoint for replay; do not pin the startup entry or let ordinary
    document pins select an older supervisor. Apply configuration changes at
    explicit ordered boundaries rather than mutable graph reads during
    execution, and specify grant ordering when concurrent child tasks share
    quotas. See
    [the resource contract](../docs/bootstrap-kernel-spec.md#45-portable-resource-semantics-versus-host-watchdogs).
- [ ] **Author the self-hosted System supervisor Dc-WASM document and graph-backed user quota settings schema in `.ctbnodes` / `.ctbnbdl`, retiring transitional host Rust resource scaffolding (`src/formats/dcdata/`, `src/formats/dcstring/converters/wasm.rs`, `src/formats/dcstring/converters/wasm_quota.rs`, `src/runtime/runtime.rs`)**:
  - [x] Allocated the production Short Dcs (`Dc 1936 "Dc document execution kernel"` and `Dc 1937 "Collective Toolbox Workspace"`, with `Dc 1937 "Collective Toolbox Workspace"` declaring `Dc 588 "Is based on"` `Dc 1936 "Dc document execution kernel"`) in `src/formats/dcdata/data-src/categories/semantic.csv` (reserving `.Semantic` block `0x110790..0x11088F` in `src/formats/dcdata/data-src/README.dc-blocks.csv`), emitted their System-block `.WASM` supervisor definitions into `.ctbnodes` / `.ctbnbdl`, and wired `SystemSupervisorAdmission::admit_execution_kernel*` / `admit_workspace_supervisor*` and `ExecutionContext::new_execution_kernel` / `new_workspace_supervisor` (`src/runtime/runtime.rs`).
  - [x] `SystemSupervisorAdmission::admit_from_graph_config_node` / `admit_from_bootstrap_tokens` (`src/runtime/runtime.rs`) resolves and validates graph-backed user quota configuration documents (`encode_graph_quota_config_tokens` / `decode_graph_quota_config_tokens`) through the active System supervisor `.WASM` definition, and `SystemSupervisorAdmission::verify_ordinary_document_cannot_elevate` / `verify_ordinary_dc_document_cannot_elevate` (`ExecutionContext::execute_ordinary_dc_wasm_default_with_trace`) allow pinned ordinary documents and ordinary documents referencing `supervisor_entry_dc` to execute without supervisor authority while denying inline privilege claims (`Dc 527 "Execution environment process role is workspace supervisor"` = `Dc 517 "Boolean true"` or `Dc 635 "Collective Toolbox seed graph data source"` = `Dc 636 "Collective Toolbox Dc CSV seed database"`).
  - [ ] Remaining gap: Move default logical resource limits (`DcWasmResourceLimits`, `DEFAULT_MAX_WASM_*`), operation cost weights, `__ctb_*` bytecode instrumentation (`instrument_wasm_module_bytes`), and task-tree quota management (`SharedWasmTaskQuota`) out of transitional host Rust scaffolding (`wasm.rs` / `wasm_quota.rs`) into the self-hosted System supervisor `.WASM` program so bytecode transformation and cost tables are versioned entirely by the recorded System supervisor revision rather than the host binary. Once migrated, `wasm.rs` / `runtime.rs` should retain only zero-table Dc-WASM lowering (`Dc 1927 "Begin WebAssembly expression"`..`Dc 1928 "End WebAssembly expression"`), the 32-byte linear-memory mailbox ABI (`0..32`), and host capability mediation / emergency process watchdogs.
- [x] **Execute and verify actual WASM resource interruptions during trace replay (`src/runtime/runtime.rs:1241-1346, 1672-1734`)**:
  - In `ExecutionContext::replay_wasm_session_trace`, `session.step()` is executed for every recorded `WasmTraceEvent::ResourceInterruption` (`decision != PermissionDecision::Allow` and `decision == PermissionDecision::Allow`), verifying the exact recorded `last_interrupted_op_ordinal` and `last_interrupted_op_required_cost` before applying `session.increase_compute_quota(...)` or finalizing the trace.
  - In `ExecutionContext::replay_wasm_bytes_trace`, construction-time `from_wasm_bytes_with_limits` resource interruptions (`after_logical_steps == 0`) are instantiated, executed, and verified directly against `recorded_trace.events`.
- [x] **Preserve intra-step continuations and `SharedWasmTaskQuota` child state across mid-step pauses, and record `start`-section non-resource traps (`src/formats/dcstring/converters/wasm.rs`, `src/runtime/runtime.rs`)**:
  - Replaced `DcWasmSession::restore_pre_step_state_for_resume` whole-step re-execution with a live worker-backed resumable continuation (`SessionWorkerHandle` and `__ctb_suspend_on_quota`) that suspends in place at `$__ctb_deduct` / `$__ctb_deduct_batch` and preserves `SharedWasmTaskQuota` child-helper state across mid-step pauses.
  - Non-resource traps during module `start` (`WasmStartSectionTrap`) are captured as `WasmTraceEvent::NonResourceTrap { after_logical_steps: 0, .. }` in `WasmExecutionTrace` and verified on replay.
- [x] **Align Dc-WASM opcode support across `wasm.csv`, `wasm_quota.rs`, and the runtime, and expand cross-runtime conformance (`src/formats/dcstring/converters/wasm_quota.rs`, `scripts/test-wasm-compat`, `docs/bootstrap-kernel-spec.md`)**:
  - Explicitly gated the `0xFB` GC opcode page (`Dc 903 "WASM struct.new"`..`Dc 1158 "WASM Reserved 0xFB 0xFF 0x01"`), exception-handling instructions (`Dc 655 "WASM throw"`, `Dc 657 "WASM throw_ref"`, `Dc 678 "WASM try_table"`), and `0xFD` Relaxed SIMD (`Dc 1671 "WASM i8x16.relaxed_swizzle"`..`Dc 1926 "WASM Reserved 0xFD 0xFF 0x03"`) in `BootstrapKernelVersion::V1` (`BootstrapKernelVersion::ensure_wasm_opcode_supported`).
  - Expanded [`scripts/test-wasm-compat`](../scripts/test-wasm-compat) to cover quota boundaries, division traps, table growth + `call_indirect`, numeric edge cases (`i64.rotl`, `i32.trunc_sat_f64_s`), and continuation resumption across `tinywasm` and V8/Node `WebAssembly`.

#### Resource Logic in Host Rust

The resource code is **not dead code yet**: the System supervisor document has
not been authored. [`DcWasmSession`](../src/formats/dcstring/converters/wasm.rs)
and [`instrument_wasm_module_bytes`](../src/formats/dcstring/converters/wasm_quota.rs)
remain active through [`DefaultDcExpressionEvaluator`](../src/formats/dcstring/navigator/eval.rs)
and [`ExecutionContext`](../src/runtime/runtime.rs).

Under the supervisor design above and the [resource contract](../docs/bootstrap-kernel-spec.md#45-portable-resource-semantics-versus-host-watchdogs),
the following host Rust logic is transitional:
- **Default budgets and configuration**: `DcWasmResourceLimits` and
  `DEFAULT_MAX_WASM_*` in [wasm.rs](../src/formats/dcstring/converters/wasm.rs)
  should give way to the supervisor and its graph-backed user settings schema.
- **Logical cost model and bytecode instrumentation**: Operation cost weights,
  batching rules, and the `__ctb_*` instrumentation and continuation transform
  in [wasm_quota.rs](../src/formats/dcstring/converters/wasm_quota.rs) and
  [wasm.rs](../src/formats/dcstring/converters/wasm.rs) must be versioned by
  the supervisor revision rather than the host binary.
- **Task-tree quota state and continuation replay**: `SharedWasmTaskQuota` and
  `DcWasmSession::restore_pre_step_state_for_resume` are transitional; the
  supervisor owns task admission, child-helper debiting, and resumption after a
  `Privilege::ExcessiveCompute` grant.

Once logical resource management is migrated, the host still needs zero-table
Dc-WASM lowering, the 32-byte mailbox and `step()` runner, capability mediation,
and emergency process watchdogs. Physical allocation failures and watchdog
interruptions remain external host events rather than logical budget policy.

### Gate 3: Prove One End-to-End Vertical Migration with Real Graph History

The bounded Routine migration is recorded under Completed Foundations. The
remaining compatibility suite below is still pending, including capability
ordering, asynchronous events, and canonical raster output.

- [ ] **Build the end-to-end historical-behavior and trace-replay regression suite**:
  - Using real generated graph history (`Dc 591 "Supersedes"` / `Dc 592 "Retracts"` across active and historical `.ctbnbdl` bundles):
    1. [x] Author a document pinned to `Dc 641 "Global graph system block most recent node ID at time of document creation"` at revision $R_1$ (`test_gate3_routine_header_uses_generated_graph_history` in `src/formats/dcstring/tests/e2e_dc_syntax.rs` and `test_gate3_async_event_ordering_prefix_invariance_and_vertical_migration` in `src/runtime/runtime.rs`).
    2. [x] Append a modified definition at revision $R_2 > R_1$ via the ledger generator.
    3. [x] Preserve the original document octets and record its observable trace and external responses/events. Verify that it resolves the $R_1$ definition and replays the identical trace after the graph update and the Rust behavior migration. Compare requests before supplying recorded responses and compare output bytes and outcomes, not internal parser/VM representations. A document pinned to $R_2$ must use the updated $R_2$ definition.
    4. [x] Verify that a document pinned to an unavailable future revision $R_3 > R_2$ produces an explicit diagnostic/error rather than silently running against $R_2$.
    5. [x] Add focused replay cases for denied/mocked capabilities and asynchronous event ordering at defined logical delivery points, using mocks or in-memory transports (`test_gate3_async_event_ordering_prefix_invariance_and_vertical_migration` in `src/runtime/runtime.rs`). Different external responses may change output, but must not change requests or outputs that precede their delivery.
    6. [ ] Remaining gap: Add canonical raster-output cases with explicit font and format-chain inputs, including subpixel rendering and vertical/horizontal pixel alignment. Compare exact pixel bytes across implementation revisions and supported hardware; changing rendering configuration changes inputs, not the compatibility guarantee.
    7. [x] Exercise internal semantic constructs through observable trace outputs across pinned historical (`l8589958834`) and `HEAD` snapshots (`test_gate3_internal_semantic_constructs_observable_trace_replay` in `src/formats/dcstring/tests/e2e_dc_syntax.rs` and lexical scope boundary enforcement in `src/formats/dcstring/dc_mixed_serde.rs`): framing boundaries for `Dc 590 "Is a closing delimiter of syntactic block for"`, `Dc 255 "Ignore following Dc"`, and `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`; decoded values for `Dc 6 "Begin number"`..`Dc 7 "End number"`, `Dc 260 "Begin literal value"`..`Dc 261 "End literal value"`, and `Dc 270 "Begin identifier"`..`Dc 271 "End identifier"`; lexical bindings for `Dc 541 "Enter identifier scope"`..`Dc 542 "Exit identifier scope"`, `Dc 269 "Assignment operator"`, `Dc 268 "void lang.assign([identifier] identifier, [value] value)"`, `Dc 540 "Reference bound value"`, `Dc 276 "Reference named object"`, `Dc 638 "For each"`, and `Dc 639 "This element"`; output from `Dc 256 "void lang.say([string] text)"`, genuine spliced streams, and `Dc 397 "Equivalent representations"` / `Dc 561 "Partially equivalent representations"`; and forgiving-recovery diagnostics.

### Gate 4: Incremental Expansion & Cleanup (Post-Proof)
- [ ] **Eliminate implicit Rust rule synthesis in `DatasetRuleResolver::from_seed_graph` (`src/formats/dcdata/syntax/resolver.rs`)**:
  - Emit explicit `Dc 590 "Is a closing delimiter of syntactic block for"` statements into `.ctbnodes` so container closers do not rely on `detect_container_closer` heuristics (`src/formats/dcdata/syntax/resolver.rs:1716-1734`).
  - English-name scraping (`extract_el_type_name`) is no longer present in
    runtime definition loading. Continue moving literal constraints to explicit
    graph rules/subtypes, preserving historical behavior; the `string` header
    restriction remains procedural.
- [ ] **Migrate the historical output routine without changing older pins**:
  - The analogous `string` header check remains necessary: its historical named
    pattern accepts a generic literal, so a HEAD-only restriction cannot justify
    deleting that check for older pins.
  - `Dc 256 "void lang.say([string] text)"` contributes its string to the final
    evaluated document, adding a line feed only when needed. Printed and
    returned document values are equivalent; emitted closers can close outer
    blocks. Stream and value evaluation share one implementation, and exact
    historical hello-world fixture checks preserve this behavior.
  - The real historical snapshot `l8589958834` has no graph WASM definition for
    this routine. Appending a definition at HEAD cannot backfill an older pin.
    The historical host implementation therefore remains pending migration,
    without HEAD substitution. A graph-WASM definition in the selected snapshot
    takes precedence, including through equivalent routine identifiers.
  - Source inspection remains distinct from evaluated output. There is no
    separate prohibition on using emitted values in evaluated grammar or
    closing a block through emitted output.
- [ ] **Migrate remaining procedural constraint & value evaluators (`src/formats/dcstring/navigator/eval.rs`)**:
  - Incrementally migrate subtype constraints (`number_satisfies_subtype`, `type_tokens_satisfy`, `eval_list_constraints`, `eval_data_construct`, `eval_typed_data_construct`), lexical bindings (`Dc 269 "Assignment operator"`, `Dc 540 "Reference bound value"`, `Dc 638 "For each"`), and representation sets (`Dc 397 "Equivalent representations"`, `Dc 561 "Partially equivalent representations"`) into pinned graph definitions, retiring each duplicate Rust representation only after its vertical compatibility test passes.
- [ ] **Enforce graph WASM source-provenance linting (`issues/issues.md`)**:
  - Add a dataset/graph linter ensuring that only foundational primitive Dcs use direct `.WASM` definitions in the graph, and all higher-level routines in the graph are accompanied by their human-readable/editable high-level language source so opaque unattributed WASM cannot enter the graph.

---

## Deferred Items (Not Required for Compatibility Closure)

### 1. StageL 3, StageR 3, and EM3 Textual Languages
- **Status**: `Deferred` until Gates 1–3 are established.
- **Pending Work**: Implement the WAT-based StageL 3 core, the StageR 3 type
  extensions, and then EM3, following the [language layers](#language-layers-2026-11-11).
  Expression values, lexical capture, and stack-safe tail calls are required
  throughout; first-class functions are EM3-only. The older assembler/VSO tier
  labels below do not replace this implementation hierarchy.
- **Preserved Design Notes from [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md)**:
  - **`Dcsh`+ assembler (earlier Tier 0 proposal)**: Minimal human-writable assembler extending `src/formats/dcstring/converters/dcal.rs` with `@ident` resolution, inline literals, and automatic closer insertion via `Dc 590 "Is a closing delimiter of syntactic block for"`.
  - **EM3 (VSO / Polish Prefix Semantic & Programming Language in `src/formats/dct_el/`; earlier Tier 1 proposal)**:
    - General-purpose English-like VSO / Polish prefix language usable for both code and UI localization (transpiling to semantic Dcs and projecting into natural languages or WASM).
    - Two lexical profiles mapping to the same token stream:
      - **Speakable profile**: `Doing see I you.`, `Is to dipre meet I you par good.`, `Let var foo add 3 4.`, `Let var int32 var bar var foo.`, `Set var bar mul var bar dipre daun.`, `Let verb i32 sum i32 p1 i32 p2 par add p1 p2.`
      - **Compact code profile**: `doing see I you`, `is to is meet I you } good`, `let /foo add 3 4`, `let i32/bar foo`, `set /bar mul /bar /bar`, `let v/i32/sum i32/p1 i32/p2 } add p1 p2;`
    - **EM3 function / verb literal conventions**:
      - Named routine: `let v/i32/sum i32/p1 i32/p2 } add p1 p2;`
      - Anonymous verb literal: `v/i32/ p1 p2 } ...`
      - Untyped/any return (`Dc 275 "Built-in type identifier: * (Object or any subtype)"`): `v//...`
      - Unit-returning routines: explicit `v/void/log`; routines that never return normally: `v/never/log` (neither uses `v//log`).
    - **Repetition Dcs (`dipre`, `daun`, `dato`)**: Compact 1-token repetition of the word before previous (`dipre`) or previous word (`daun`, `dato`) to provide elegant variation when spoken.
    - **Possibly new anaphoric / backreference Dcs**: Compact 1-token backreferences to the previous clause or previous identifier to avoid repeating long identifiers or assigning intermediate node IDs.

### 2. Wholesale CST (`DcSyntaxKind`) Rewrite
- **Status**: `Deferred`.
- Replacing `DcSyntaxKind` variants in `src/formats/dcstring/navigator/arena.rs` or removing compiled lookup caches in `DatasetRuleResolver` is not required for backwards compatibility; compiled Rust tables and AST caches are acceptable wherever they faithfully and deterministically reflect the document's pinned graph snapshot.
