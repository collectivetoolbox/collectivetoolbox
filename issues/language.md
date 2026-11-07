# Graph-Defined Dc Language, Execution Kernel, and Compatibility Plan

**Overall Status**: `Partially Implemented`

Reviewed against [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md) and source at `662c3e8e0563c8f0e4d89a3e1e987124b10e2323` on 2026-11-07.

## Core Architectural Guarantee

**An existing document must resolve the exact same transitive definitions and retain the exact same framing, validation, and execution behavior after future code or graph updates.**

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
- **Initial Historical Graph Loading & Root Pin Detection (`src/formats/dcdata/seed_graph.rs`, `src/formats/dcstring/navigator.rs`)**:
  - `SeedGraphIndex::load_at_most_recent_from_asset_bundle` (`src/formats/dcdata/seed_graph.rs:324-344`) and `DcDocumentNavigator::detect_leading_document_most_recent` (`src/formats/dcstring/navigator.rs:289-317`) provide initial plumbing for `Dc 641 "Global graph system block most recent node ID at time of document creation"` (subject to the High-severity defects in Gate 2 below).

---

## Remaining Gaps / Pending Work (Gate-Driven Plan)

The previously proposed "four drastic phases" (wholesale `DcSyntaxKind` CST deletion and static two-pass framing ahead of splicing) are replaced by the following five verification and migration gates.

### Gate 1: Define the Compatibility Contract
- [ ] **Specify stable cross-revision invariants**:
  - Document the exact invariants guaranteed for a document pinned via `Dc 641 "Global graph system block most recent node ID at time of document creation"`:
    1. Structural framing boundaries (`Dc 590 "Is a closing delimiter of syntactic block for"`, prefix arity, `Dc 255 "Ignore following Dc"` escaping, and `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"` encapsulation).
    2. Decoded primitive values (`Dc 6 "Begin number"`..`Dc 7 "End number"`, `Dc 260 "Begin literal value"`..`Dc 261 "End literal value"`, `Dc 270 "Begin identifier"`..`Dc 271 "End identifier"`).
    3. Lexical scope and variable bindings (`Dc 541 "Enter identifier scope"`..`Dc 542 "Exit identifier scope"`, `Dc 269 "Assignment operator"`, `Dc 268 "void lang.assign([identifier] identifier, [value] value)"`, `Dc 540 "Reference bound value"`, `Dc 276 "Reference named object"`, `Dc 638 "For each"`, `Dc 639 "This element"`).
    4. Spliced token stream output (`Dc 256 "void lang.say([string] text)"` and `Dc 397 "Equivalent representations"` / `Dc 561 "Partially equivalent representations"`).
    5. Diagnostics, forgiving recovery container boundaries, and capability/effect requests (`WasmStepOutcome::RequestCapability`).
- [ ] **Separate Unevaluated Source Mode from Evaluated Spliced Mode**:
  - Preserve incremental expansion-before-framing when evaluating spliced streams (`splicing = true`), where `Dc 256 "void lang.say([string] text)"` or WASM splice routines can emit openers/closers that alter container boundaries.
  - Keep unevaluated source inspection (`splicing = false`, used by source editing, formatting, and raw CST inspection) distinct so `Dc 256 "void lang.say([string] text)"` is treated as an ordinary routine call frame without mutating surrounding containment.
- [ ] **Define policy for unpinned legacy documents**:
  - Because an unpinned document's creation revision cannot be recovered automatically, specify the explicit fallback policy (e.g., resolve against a designated baseline snapshot or `HEAD` with an explicit diagnostic/provenance note).

### Gate 2: Make Snapshot Resolution Trustworthy (Fix Silent Substitution)
- [ ] **Propagate pin-resolution errors instead of silently falling back to `HEAD` (`src/formats/dcstring/navigator.rs:280-287`)**:
  - In `DcDocumentNavigator::bind_document_most_recent_node_id`, stop discarding `Err` from `syntax_resolver_at_most_recent(node_id)`. Record an explicit syntax/diagnostic error under the active `DcProjectionPolicy` while still allowing raw token/source inspection and forgiving recovery.
- [ ] **Reject pins newer than the local graph (`src/formats/dcdata/seed_graph.rs:333-338`)**:
  - In `SeedGraphIndex::load_at_most_recent_from_asset_bundle`, only return `Arc::clone(shared)` when `most_recent_node_id == head_id`. When `most_recent_node_id > head_id` (and exceeds the highest available historical/active bundle ID), return a descriptive error stating that the document requires a newer System graph (`l{most_recent_node_id} > l{head_id}`) rather than silently substituting `HEAD`.
- [ ] **Require complete historical graph bundles when resolving historical pins (`src/formats/dcdata/seed_graph.rs:346-364`)**:
  - When `most_recent_node_id < head_id` and active nodes have superseded/retracted prior statements, verify that `system_graph_history.rsrc` (`data/graph/system/*.ctbnbdl`) is present and covers the pinned revision without gaps, returning an explicit error if history is absent or incomplete.
- [ ] **Enforce transitive snapshot isolation for all helpers (`src/formats/dcstring/converters/wasm.rs:1913-1944`, `src/formats/dcdata/syntax/resolver.rs`)**:
  - Ensure that every referenced subpattern (`Dc 605 "Subpattern match"`), equivalence (`Dc 611 "Canonical decomposition"`), subtype (`Dc 587 "Is a subtype of"` / `Dc 586 "Is an instance of"`), and WASM helper or value node (`run_graph_wasm_tokens_on_dc_slice` in `src/formats/dcstring/converters/wasm.rs:1923-1934`) resolves strictly against the document's pinned `SeedGraphIndex` (filtering out any node with `id > most_recent_node_id` and applying only `Dc 591 "Supersedes"` / `Dc 592 "Retracts"` up to `most_recent_node_id`).
- [ ] **Specify pin placement and nested-document inheritance (`src/formats/dcstring/navigator.rs:289-317`)**:
  - Support and validate `Dc 641 "Global graph system block most recent node ID at time of document creation"` both at the raw stream start and inside document/environment headers (`Dc 235 "Begin document"`..`Dc 236 "End document"`, `Dc 340 "Begin execution environment description"`..`Dc 341 "End execution environment description"`, `Dc 617 "Begin graph node definition"`..`Dc 618 "End graph node definition"`).
  - Define how nested documents inherit the enclosing document's pin unless they declare their own scoped `Dc 641 "Global graph system block most recent node ID at time of document creation"`.

### Gate 3: Document the Minimal Trusted Kernel & Enforce VM Budgets
- [ ] **Freeze and document the minimal trusted bootstrap kernel**:
  - Write a specification covering the exact host-implemented primitives that graph nodes rely upon:
    1. Physical token codecs (`DcTF`, `.dcmm` header, `Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`).
    2. Active triple resolution in `SeedGraphIndex` (`Dc 584 "Has a"`, `Dc 610 "Has value"`, `Dc 591 "Supersedes"`, `Dc 592 "Retracts"`, `Dc 611 "Canonical decomposition"`).
    3. `.Semantic:Pattern` operators (`Dc 599 "Self reference"`, `Dc 600 "Wildcard pattern"`, `Dc 601 "Pattern conjunction"`, `Dc 602 "Pattern alternation"`, `Dc 603 "Pattern repetition"`, `Dc 604 "Negated pattern"`, `Dc 605 "Subpattern match"`, `Dc 606 "Pattern capture"`, `Dc 607 "Pattern replacement"`, `Dc 643 "Range"`, `Dc 644 "Exclusive range endpoint"`, `Dc 642 "Data byte length"`).
    4. Dc-WASM binary lowering (`Dc 647 "WASM unreachable"`..`Dc 1928 "End WASM program"`) and the 32-byte linear-memory mailbox ABI (`DC_WASM_MAILBOX_*` at bytes `0..32`).
- [ ] **Enforce intra-step instruction metering (fuel) in `tinywasm` (`src/formats/dcstring/converters/wasm.rs:1721-1798`)**:
  - `DcWasmSession::step` currently calls `func.call(&mut self.store, ...)` synchronously with no instruction budget, allowing a tight loop (`Dc 650 "WASM loop"` .. `Dc 659 "WASM br"` `0`) to hang the host inside a single `step()` call.
  - Add per-call instruction fuel / metering inside the `tinywasm` interpreter loop so execution traps or yields `Privilege::ExcessiveCompute` deterministically after a bounded number of WASM instructions.
- [ ] **Enforce linear-memory and output-slice resource budgets (`src/formats/dcstring/converters/wasm.rs:1665-1685`, `1801-1830`)**:
  - Cap maximum linear memory pages (both during `provide_input` and WASM `memory.grow` instructions), single-step output slice length (`out_len`), and cumulative yielded output bytes across a session.
- [ ] **Keep capability authorization strictly in the host (`src/runtime/runtime.rs:266-370`)**:
  - Ensure historical graph pinning applies only to language/document semantics and never grants or restores obsolete security permissions or capability policies.

### Gate 4: Prove One End-to-End Vertical Migration with Real Graph History
- [ ] **Migrate one concrete Rust-defined behavior into the system graph and remove its procedural fallback**:
  - Select a small self-contained behavior currently hardcoded in Rust (for example, `Dc 256 "void lang.say([string] text)"` payload extraction in `DefaultDcExpressionEvaluator::eval_stream_splice` at `src/formats/dcstring/navigator/eval.rs:260-300`, or a specific format/value rule), emit its definition through `src/formats/dcdata/support/graph_ledger.rs` into `.ctbnodes` / `.ctbnbdl`, and delete the corresponding procedural Rust fallback.
- [ ] **Build the end-to-end historical-behavior regression test**:
  - Using real generated graph history (`Dc 591 "Supersedes"` / `Dc 592 "Retracts"` across active and historical `.ctbnbdl` bundles):
    1. Author a document pinned to `Dc 641 "Global graph system block most recent node ID at time of document creation"` at revision $R_1$.
    2. Append a modified definition at revision $R_2 > R_1$ via the ledger generator.
    3. Verify that the document pinned to $R_1$ resolves the $R_1$ definition and produces identical framing, values, and output, while a document pinned to $R_2$ uses the updated $R_2$ definition.
    4. Verify that a document pinned to an unavailable future revision $R_3 > R_2$ produces an explicit diagnostic/error rather than silently running against $R_2$.

### Gate 5: Incremental Expansion & Cleanup (Post-Proof)
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
- **Status**: `Deferred` until Gates 1–4 are established.
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
    - **Anaphoric / backreference Dcs (`dipre`, `daun`)**: Compact 1-token backreferences to the previous clause/statement (`dipre`) or previous operand (`daun`) to avoid repeating long identifiers or minting intermediate node IDs.

### 2. Wholesale CST (`DcSyntaxKind`) Rewrite
- **Status**: `Deferred`.
- Replacing `DcSyntaxKind` variants in `src/formats/dcstring/navigator/arena.rs` or removing compiled lookup caches in `DatasetRuleResolver` is not required for backwards compatibility; compiled Rust tables and AST caches are acceptable wherever they faithfully and deterministically reflect the document's pinned graph snapshot.
