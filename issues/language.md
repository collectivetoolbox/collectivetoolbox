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
- [ ] **Freeze and document the minimal trusted bootstrap kernel (`docs/bootstrap-kernel-spec.md`, `src/formats/dcstring/navigator/policy.rs`, `src/formats/dcstring/navigator.rs`)** (Partially Implemented; current document is a draft):
  - The replay guarantee assumes one explicitly specified foundational execution contract, immutable or selected by an explicit version (`BootstrapKernelVersion::V1 = 1`, `BootstrapKernelVersion::CURRENT`, `DcProjectionPolicy::with_kernel_version` / `try_with_kernel_version_number`, and `DcDocumentNavigator::select_bootstrap_kernel_version`). Identical graph bytes under different kernel interpretations are not sufficient. Version selection must be available during bootstrap, before interpretation that depends on it.
  - Wrote [`docs/bootstrap-kernel-spec.md`](../docs/bootstrap-kernel-spec.md) covering the exact host-implemented primitives that graph nodes rely upon:
    1. Physical token codecs (`DcTF`, `.dcmm` header, `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`).
    2. Active triple resolution in `SeedGraphIndex` (`Dc 584 "Has a"`, `Dc 610 "Has value"`, `Dc 591 "Supersedes"`, `Dc 592 "Retracts"`, `Dc 611 "Canonical decomposition"`).
    3. `.Semantic:Pattern` operators (`Dc 599 "Self reference"`, `Dc 600 "Wildcard pattern"`, `Dc 601 "Pattern conjunction"`, `Dc 602 "Pattern alternation"`, `Dc 603 "Pattern repetition"`, `Dc 604 "Negated pattern"`, `Dc 605 "Subpattern match"`, `Dc 606 "Pattern capture"`, `Dc 607 "Pattern replacement"`, `Dc 643 "Range"`, `Dc 644 "Exclusive range endpoint"`, `Dc 642 "Data byte length"`).
    4. Dc-WASM binary lowering (`Dc 647 "WASM unreachable"`..`Dc 1928 "End WASM program"`) and the 32-byte linear-memory mailbox ABI (`DC_WASM_MAILBOX_*` at bytes `0..32`).
- [x] **Enforce intra-step instruction metering (fuel) in `tinywasm` (`vendor/tinywasm/`, `src/formats/dcstring/converters/wasm.rs`, `src/runtime/runtime.rs`)**:
  - `DcWasmSession::step` configures per-step instruction fuel (`DcWasmResourceLimits::max_instructions_per_step`, default `1_000_000`) on `tinywasm::Store` before each `func.call(&mut self.store, ...)`, preventing tight loops (`Dc 650 "WASM loop"` .. `Dc 659 "WASM br"` `0`) from hanging inside a single `step()` call.
  - Added `Trap::InstructionFuelExhausted` inside `tinywasm::Executor::exec_next` and mapped it to `WasmResourceInterruption::InstructionFuelExhausted` and `Privilege::ExcessiveCompute`.
  - Distinguished physical VM metering from logical document progress via `WasmExecutionTrace` and `WasmTraceEvent::ResourceInterruption`: safety-limit interruptions are exposed and recorded as external resource events in the execution trace, while trace replay (`ExecutionContext::replay_wasm_session_trace`) compares logical capability requests and ignores physical instruction count differences across compilers.
- [x] **Enforce linear-memory and output-slice resource budgets (`vendor/tinywasm/`, `src/formats/dcstring/converters/wasm.rs`)**:
  - Capped linear memory at `256` pages (`16 MiB`) per memory instance, single-step output at `4 MiB`, and cumulative output at `16 MiB`. Guest `memory.grow` failure returns WASM `-1`, not a host resource event; host input allocation failures return `MemoryLimitExceeded`. A step rejects an exported memory above a subsequently lowered limit. These are not table-allocation or aggregate host-memory caps.
- [x] **Keep capability authorization strictly in the host (`src/runtime/runtime.rs`)**:
  - Historical graph pinning (`Dc 641 "Global graph system block most recent node ID at time of document creation"`) applies only to language/document semantics and never grants or restores security permissions or capability policies (`ExecutionContext::set_privilege_decision` / `with_privilege_decision`).
  - `ExecutionContext::execute_wasm_session_with_trace` and `ExecutionContext::replay_wasm_session_trace` compare the document's capability requests at each logical step during replay and supply the recorded allow/deny/mock responses while live execution remains governed by the current host policy.

#### Gate 2 Status & Remaining Gaps (2026-11-08)

- **Partially Implemented**: version APIs, VM instruction/page/output limits,
  host policy, and bounded trace replay have unit coverage. The kernel document
  is not yet a complete frozen specification.
- Review fixes: module start functions now receive fuel before instantiation;
  live capabilities default to denial unless explicitly granted; replay does
  not execute/discard an extra step at a recorded interruption and verifies
  the entire trace/result, rejecting trailing events or inconsistent results;
  execution rejects an already over-budget exported memory after cap lowering.
- Corrected the draft's graph-region boundaries, DcTF encoding (no long-token
  escape), binary encapsulation (type byte and big-endian 128-bit lengths,
  without a trailing binary closer), retraction lifecycle, and mailbox offset.
- **Pending Work**: independent conformance fixtures for codecs, lifecycle,
  pattern choice/captures/ranges, WASM numeric/trap/feature behavior and malformed
  input; selection before construction if a second kernel is introduced;
  explicit static/zero-filled guest-memory ownership; table and aggregate-memory
  budgets; recorded construction failures and denied interruptions rather than
  loss of their accumulated trace through `Err`. Gates 3 and 4 remain pending.
- **Validation**: all 196 Dc-string and seven runtime library tests passed on
  Linux musl, with no ignored tests. Production Clippy for both affected crates
  completed with baseline warnings; documentation links and whitespace passed.
- **Verification Limits**: other platforms and the end-to-end event/pixel
  replay guarantee remain unverified.
- **Failed Broader Check**: the prescribed development-binary character-name
  lookup triggered a release build, blocked by six unrelated `E0277` slice/array
  `Option` comparisons in
  [container.rs](../src/formats/detection/container.rs) and
  [droid.rs](../src/formats/detection/droid.rs). Consequently that CLI lookup
  did not run. The historical delimiter names/pairing below were verified from
  the archived CSV and actual converter usage; no detection code was changed.

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
