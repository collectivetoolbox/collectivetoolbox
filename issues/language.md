# Language Layers (2026-11-11)

**Overall Language Stack Status**: `Partially Implemented` (format registrations including `f1424` `Boolean` / `f1425` `Void` / `f1426` `Never` and `.Formats:transformation` entries `f1680` `StageL3ToWasm` / `f1681` `StageR3ToStageL3`, `.EM`, `.EM StageL`, and `.EM StageR` Token Dc repertoire including pure anonymous routine value nodes `Dc 2240 "Begin routine"`..`Dc 2241 "End routine"`, `Dc 599 "Self reference"`, and decoupled `Dc 1942 "Routine test suite"` / `Dc 573 "Description"` / `Dc 574 "Programmatic identifier"` / `Dc 598 "Is associated with"` graph metadata across all active routines, `.Semantic` snapshot scoping `Dc 1938 "Resolve to pin"`, Option B `DcWasmSession` pure host capability ABI (`Dc 1939 "Minimal query of system graph"`, `Dc 1940 "Execute pure WASM program"`, `Dc 2616 "Request host capability"`), StageR 3 graph implementation of `Dc 2608 "Convert Dc string to WASM"` in `impls/std/convert_dc_string_to_wasm.sr3` with host Rust fallback retained strictly for pre-migration historical `Dc 641 "Global graph system block most recent node ID at time of document creation"` pins, Guix-style native Rust WASM substitutes (`ctb-wasm-substitutes` / `ctb-wasm-substitutes-gen`, including Merkle-deduplicated shared functions, auto-discovered runnable System-block snapshots, and `mem0` fast-path access) with deterministic V2 checkpoint-ordinal compute-quota metering, format-implementation boundary linter restricting direct format implementations to `.Formats:transformation`, literal-aware 32-char `.sli3`/`.sri3` normalizer, StageL 3 & StageR 3 parser/validator/Token-Dc codec/`: @dc` ledger auto-allocator/evaluator, StageL 3 Rust & host Dc-WASM compilers, host `SeedGraphIndex` snapshot Token-Dc linker & multi-step converter chain runner, all 39 Stage 0 `.wat` built-in primitives plus self-hosted `stage_0_impls/f1680_stagel3_to_wasm.wat` (`f1680`), unified parameter-type-dispatched Stage 1 `stage_1_impls/std/{assert,bytes,math,u128,i128,dctf,bigint,rat,dcs,dcmst}.sl3` standard library modules with carrier bridging + transitive `may_fault` totality gating, ordered multi-stage namespace overlaying across `stage_0_impls/` < `stage_1_impls/` < `stage_2_impls/` < `impls/` with initial StageR 3 overlays `stage_2_impls/std/{math,dcs}.sr3`, self-hosted `stage_1_impls/std/stager3_to_stagel3.sl3` (`f1681`), and `tinywasm` constant-stack tail calls implemented; `Dc 2616` permission Dcs, checksum-indexed `Dc 1939` reverse lookup, remaining Unicode-ism audit backlog, `view`/`cursor` + `emitExtent` streaming primitives, ASCII `.sl3`/`.sr3` syntax converters, native Rust/JS StageR 3 backends, CLI Dc document execution/rendering wiring, and EM3 pending).

Canonical language specifications now live in:
- **StageL 3 (`f656` `.sl3` / `f659` `.sli3`)**: [`src/formats/stagel/docs/sl3.md`](../src/formats/stagel/docs/sl3.md) — core expression-oriented language to be implemented directly in `stage_0_impls/*.wat` (`b`, `u8`, `i32`, `u32`, `i64`, `u64`, `l/T`, local builder `buf/T`, `n`-ary `tup: ... ::`, positional `struct`, `void`, `never`; peer `let`/`const`/`pub` declarators with `set` and definite-assignment checking; named nested verbs with shared lexical capture; mandatory constant-stack tail calls; no first-class functions).
- **StageR 3 (`f657` `.sr3` / `f660` `.sri3`) & EM3 Boundary (`f658` `.em3` / `f661` `.emc`)**: [`src/formats/stagel/docs/sr3.md`](../src/formats/stagel/docs/sr3.md) — adds `i128`, `u128`, `int`, `rat`, `dc`, `view`, `cursor`, `s` (`DcMst`), `dcs`, `option/T`, `result/T`, monomorphic nominal user-defined value types (`brand` / `type`, named-field `struct`, `enum`) with shallow exhaustive `match`, `try*` checked operations, bare `"..."` Dcts literals, and zero-copy window/extent streaming. User-defined generics, methods/subtyping, and interior field mutation are excluded; first-class functions and escaping closures remain **EM3-only**.

The bootstrap implementation directory stages (ordered `stage_0_impls/` < `stage_1_impls/` < `stage_2_impls/` < `impls/`, where higher stages may overlay existing lower-stage namespaces without `pub` export collisions or upward imports) are:
- **Stage 0 (`src/formats/dcdata/data-src/stage_0_impls/`)**: `.wat` foundations (`1936_dc_document_execution_kernel.wat`, the StageL-Dc-to-WASM `.Formats:transformation` compiler `f1680_stagel3_to_wasm.wat`, and non-trivial StageL built-in `.WASM` primitives).
- **Stage 1 (`src/formats/dcdata/data-src/stage_1_impls/`)**: `.dcsh` and `.sl3` seed implementations (emitted into the System graph as **StageL Token Dcs**), including the StageR-Dc-to-StageL-Dc `.Formats:transformation` compiler (`f1681` `stager3_to_stagel3`) and core `std/*` runtime modules.
- **Stage 2 (`src/formats/dcdata/data-src/stage_2_impls/`)**: `.dcsh`, `.sl3`, and `.sr3` standard-library modules and higher-level StageR 3 overlays (`stage_2_impls/std/{math,dcs}.sr3` supplementing Stage 1 `std/math` and `std/dcs` namespaces).
- **Impls (`src/formats/dcdata/data-src/impls/`)**: `.dcsh`, `.sl3`, and `.sr3` higher-level application, Document Character, UI, and EM3 seed implementations (emitted into the System graph as **StageL / StageR / EM3 Token Dcs**), including EM3 (`f658`) and higher-level VSO/localization definitions described in [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md).

## Transpilation, Concurrency, and Performance Architecture

### Tokenized Dc Graph Representation & Staged Transpilation Flow

- **Strict `.WASM` and format-implementation boundary in the graph**: **Only** foundational Stage 0 `.wat` foundations (`Dc 1936 "Dc document execution kernel"`, Stage 0 `.Formats:transformation` compilers such as `f1680_stagel3_to_wasm.wat`, standalone `@converts(...)` routines, and **StageL built-in primitives**) are emitted as `.WASM` Dcs (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`, optionally paired with `Dc 397 "Equivalent representations"`) in the graph. Direct implementations may never be attached to non-transformation Format Dcs (only `.Formats:transformation` entries or verb Dcs).
- **StageL / StageR Token Dcs as the canonical System-block representation**: Everything implemented *using* StageL 3 or StageR 3 uses dedicated semantic **StageL / StageR Token Dcs** (e.g. Dcs for `add`, `truncRem`, control flow, and direct Dc references to called functions/constants). Untokenized ASCII `.sl3` and `.sr3` files in `src/formats/dcdata/data-src/` act as seed sources—analogous to `categories/*.csv`—from which the generator emits Token Dcs into the System block, keeping *meanings* in the graph and minimizing storage footprint.
- **Lossless source reconstruction & AppleScript-style Dc-native editing**: Textual `.sl3` / `.sr3` source is losslessly reconstructable from its StageL / StageR Token Dcs just like EM3 (they are true alternate representations). Once a Dc-native editor is available, tokenized Dc documents become the preferred editing medium—working like Apple's AppleScript Script Editor, where authors type ordinary text while tokens are identified and saved directly as Dcs.

1. **StageL 3 (`f656`)**: Stored as StageL Token Dcs; compiled to `.WASM` for sandboxed execution via `Dc 1938 "Resolve to pin"` + `Dc 2608 "Convert Dc string to WASM"` dispatching to `f1680` `StageL3ToWasm` (`stage_0_impls/f1680_stagel3_to_wasm.wat`, `@chain(2192 > 1927)`) under the shared execution quota.
2. **StageR 3 (`f657`)**: Lowered **using StageL 3** via `f1681` `StageR3ToStageL3` (`stage_1_impls/std/stager3_to_stagel3.sl3`, `@chain(2221 > 2192)`) and stored as StageR/StageL Token Dcs. StageR 3 initially lowers through StageL 3 -> WAT -> `.WASM` for execution, and later transpiles directly into **Rust**, **JavaScript**, and other target languages. (Individual StageR 3 primitives may also lower directly to WAT where useful without duplicating their semantic specification.)
3. **EM3 (`f658` / `f661`)**: Always has a working baseline path—resolve the StageR 3 functions that implement an EM3 document's constructs and transpile/compile those—while also supporting **higher-level construct refinements** so backends can transpile rich semantic constructs directly into idiomatic target-language code instead of flattening everything first.

Preserve resolved semantic Dc constructs until a target conversion rule is selected:

1. Resolve the construct and its dependencies against the selected graph
   snapshot. Retain its identity, operands, types, lexical bindings, source
   location, and effects in a common semantic representation.
2. Prefer an applicable target-specific refinement that preserves exact
   semantics (evaluation order, allocation/fault behavior, and captures).
3. Otherwise fall back to the construct's lower-level implementation (StageR 3
   for EM3 constructs; StageL 3 / WAT for StageR 3 constructs) and recursively
   transpile that implementation.
4. Report a missing implementation or decomposition cycle explicitly. A
   supported operation that fails during execution must **never** silently retry
   via a fallback, which could duplicate effects or conceal an error.

"Dc-by-Dc" conversion operates on **resolved constructs with their operands
and context**, not independent character substitution. Binding, closure
conversion, mutual recursion, imports, and data layout require enclosing-scope
or module analysis. Rules must use the shared graph resolver, including
equivalences, rather than a backend's private numeric allowlist.

Target-rule eligibility and priority must be deterministic and versioned with
the compilation configuration. Both refined and fallback paths must preserve
values, faults, effect ordering, tail-call bounds, and the observable
execution/replay contract, including logical resource accounting where that
contract applies. Cross-path equivalence tests are a required gate before
enabling a specialized rule.

### Concurrency & Document Isolation Model

- **Single-threaded StageL 3 and StageR 3**: Neither StageL 3 nor StageR 3 provides intra-document multithreading, shared-memory atomics, or lock primitives at this time.
- **Process-level multi-document concurrency**: Parallelism is achieved outside StageL/StageR by running multiple Dc documents concurrently as separate isolated processes/sandboxes. Where high-throughput data sharing is needed, isolated document processes can share explicit host-managed memory regions or immutable graph/file extents (using file-descriptor-backed shared memory on Linux) rather than copying payloads over sockets. To preserve deterministic replay and avoid torn-read hazards in Rust/StageR fast paths, shared regions should be immutable/sealed snapshots (`memfd` + write-sealing) or exchanged at explicit capability yield points rather than unsynchronized concurrent mutation.
- **Future EM3 intra-document concurrency**: Intra-document multithreading or structured concurrency within a single EM3 document is deferred to later EM3 design.

### Graph DRYness, Namespaces (`ns` / `use`), `const` / `pub`, and `: @dc` Auto-Allocation

Because the global graph is **append-only**, keeping graph definitions strictly DRY is essential to prevent storage bloat as the graph grows:
- **Allocate Dcs for individual reusable functions & constants**: Every top-level verb (where bare `v/...` is an implicit file-private `const`) and top-level constant (`const TYPE/name EXPR.`, or `pub TYPE/name EXPR.` / `pub v/...` when exported for reuse across files) in a namespace module (`ns path/to/mod.`, imported via `use path/to/mod: ... ::.`) receives its own dedicated System-region `l<id>` Dc, while nested `v/...` helpers that capture local frames stay inside their enclosing verb's Dc.
  > Example: Using `i32.clz((255 - b0) << 24)` (`Dc 750 "WASM i32.clz"`) computes the DcTF byte length for all `128 <= b0 < 255` in a single expression—reusable helpers of this kind are good candidates for dedicated function Dcs.
- **Auto-assigned `: @dc <shorthand>` source comments (analogous to blank CSV `Dc` cells)**: Standalone implementations of already-assigned Dcs keep `<dcsh>_<slug>` in their filename (`1936_...wat`, `f657_...sl3`), whereas multi-item namespace files get newly assigned `l<id>` numbers written back into `: @dc l<id>` directive comments immediately above each top-level verb or `const`/`pub` binding. `: @dc ...` anchors the item's permanent graph ID across edits/renames while being stripped from emitted Token Dc bodies so adding it never causes spurious graph revisions. See [sl3.md](../src/formats/stagel/docs/sl3.md#4-namespaces-declarations-visibility-const--pub-and--dc-allocation) for exact syntax.
- **Link/inline across function Dcs at compile time**: Giving individual functions their own Dcs keeps the stored graph compact **and** gives precompilation/memoization a canonical identity, while compilers should link and direct-call/inline resolved helper Dcs within a compiled module rather than paying a full Dc-WASM linear-memory mailbox round-trip on every fine-grained helper call.

### Precompilation, Deterministic Caching, and Composite Document Execution

Executing every character, word, widget decomposition, and shaping rule purely through interpreted WASM on demand would be prohibitively slow (already visible in test-suite latency). Because graph evaluation is deterministic given explicit inputs (document octets, `Dc 641 "Global graph system block most recent node ID at time of document creation"` pins, rendering configuration, and external responses), multiple acceleration layers can coexist without changing observable semantics:

1. **Ahead-of-time Rust transpilation built into `ctoolbox`**: Hot/common Dcs (core codecs, standard library functions, UI/layout primitives, shaping routines) transpile to Rust and compile into optimized native machine code shipped inside the `ctoolbox` binary. Precompiled entries must be keyed to the **resolved implementation node ID** (not an unpinned Short Dc ID) so historical snapshot pins execute the exact revision active at their pin, and generated Rust must preserve deterministic compute-quota/recursion-depth checks and non-panicking fault returns.
2. **Cached compiled modules & resolver tables**: Compiled WASM modules, lowered StageR/StageL artifacts, and resolved syntax/property tables should be cached across runs and tests by `(snapshot_pin, node_id)`.
3. **Cached & precomputed execution/rendering states**: Composite Dc documents—spanning UI Dcs (`window`, `menu bar` built from `button` Dcs), word/character Dcs, shaped glyph Dcs, and rendered target outputs—can cache or ship precomputed intermediate states keyed by their explicit inputs. See [`docs/renderer-flow.md`](../docs/renderer-flow.md) for the composite document, character/word/glyph shaping, and output rendering pipeline.

## Pending Work (StageL 3, StageR 3, and EM3)

- [ ] **`DcWasmSession` `SeedGraphIndex` Capability ABI & StageR 3 Implementation of `Dc 2608 "Convert Dc string to WASM"` (`Partially Implemented`)**:
  - **Implemented**: Added the Option B host capability ABI on `DcWasmSession` (`Dc 1939 "Minimal query of system graph"`, `Dc 1940 "Execute pure WASM program"`, `Dc 2616 "Request host capability"`), implemented `Dc 2608 "Convert Dc string to WASM"` in StageR 3 (`src/formats/dcdata/data-src/impls/std/convert_dc_string_to_wasm.sr3`), wired `eval_convert_dc_string_to_wasm` to execute the pinned graph StageR 3 definition while retaining the host Rust fallback strictly for pre-migration historical `Dc 641 "Global graph system block most recent node ID at time of document creation"` pins lacking an active StageR 3 decomposition on `Dc 2608`, and implemented Guix-style native Rust WASM substitutes (`ctb-wasm-substitutes` / `ctb-wasm-substitutes-gen`, with Merkle-deduplicated shared functions, auto-discovered System-block snapshots, and `mem0` fast-path access) with deterministic V2 checkpoint-ordinal compute-quota metering across active and historical `Dc 641` snapshots.
  - **Remaining Gaps / Deferred**: Fine-grained permission Dcs for `Dc 2616 "Request host capability"`, checksum-indexed reverse lookup alternative for `Dc 1939 "Minimal query of system graph"`, remaining Unicode-ism audit backlog (`issues/issues.md`), and optional profiling-gated `#[inline(never)]`-threshold / `vm.deduct` micro-tuning in `ctb-wasm-substitutes` (`issues/issues.md`).
- [ ] **In-Graph StageL 3 Routine `test`-Suite Runner & Snapshot-Keyed Rust AOT Precompilation**:
  - Routine test blocks (`Dc 2206 "Begin routine test block"`..`Dc 2207 "End routine test block"`, attached via `Dc 1942 "Routine test suite"` in the System graph and skipped on historical/bundled `Dc 2203 "Begin routine declaration"`..`Dc 2204 "End routine declaration"` frames by `stage_0_impls/f1680_stagel3_to_wasm.wat`) currently execute on the host through `run_dc_wasm_module_tests` in `src/formats/stagel/stagel3/wasm_codegen.rs` and `StageLWorkspaceVm::run_all_tests` in `src/formats/dcdata/stagel3/eval.rs`. Add an optional self-hosted `.wat` / Stage 1 test-harness mode if in-graph `test`-suite execution is needed, and design snapshot-keyed Rust AOT dispatch/memoization for hot StageL/StageR routines.
- [ ] **StageR 3 Streaming (`view` / `cursor` + `emitExtent`), ASCII Syntax Converters, and Native Backends (Rust / JS)**:
  - Self-hosted Stage 1 `StageR 3 -> StageL 3` lowering (`stage_1_impls/std/stager3_to_stagel3.sl3`, `pub dc/f1681`, `@chain(2221 > 2192)`), Stage 1 runtime libraries (`u128`, `i128`, `dctf`, `bigint`, `rat`, `dcs`, `dcmst`), and initial Stage 2 namespace overlays (`stage_2_impls/std/{math,dcs}.sr3`) are implemented for Token-Dc streams; ASCII `.sl3` / `.sr3` text-to-Dc syntax converters in StageL/StageR and fuller standard-library/UI coverage in `stage_2_impls/` and `impls/` remain pending.
  - Implement `view` (`Dc 2677 "Built-in type identifier: Extent view"`) and `cursor` (`Dc 2678 "Built-in type identifier: Stream cursor"`) sliding-window host paging (`MapWindow` yield) and zero-copy output extent emission (`Dc 376 "Extent byte offset"` / `Dc 377 "Extent byte length"`, `emitExtent`) for multi-gigabyte files and deduplicated graph value-node slices.
  - Implement native `StageR 3 -> Rust` and `StageR 3 -> JS` transpilers (preserving nominal `brand`/`type`/named-field `struct`/`enum` layouts, `BigInt` precision in JS, and physical allocation exhaustion distinct from `error:overflow`), plus snapshot-keyed Rust AOT dispatch (with loop/call compute-quota metering and fault equivalence) so common graph Dcs ship precompiled inside `ctoolbox`.
  - Specify ELAD decoding and text-mapping helpers (`dcaFromElad`, `dctCodeToText`) with explicit `option/s` and `result/...` error contracts before porting legacy StageL v1/v2 codecs (`format-ascii.stagel`, `format-utf8.stagel`).
- [ ] **EM3 (`f658` / `f661`) First-Class Closures, Speakable/Compact Profiles, and Cross-Backend Conformance**:
  - Finalize EM3 function-value declaration syntax, `{ fn_id: u32, env_ptr: u32 }` closure lowering with per-signature `Dc 661 "WASM br_table"` + `Dc 665 "WASM return_call"` dispatchers (plus trampoline fallback on Rust/JS targets without guaranteed mutual tail calls), and cycle-safe arena/tracing lifetime management for escaping closures.
  - Specify quoted syntax data, hygienic macro expansion (if enabled), and logical non-tail recursion limits separately from host stacks.
  - Gate target-specific optimization rules on cross-backend equivalence tests (`StageR -> WAT`, `StageR -> Rust`, and `StageR -> JS`) covering mutated captures, mutual tail calls, arithmetic boundary faults, and mixed Dc/binary strings.

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

- **Self-Hosted Stage 0 StageL-Dc-to-WASM Converter Routine, `n`-Ary Tuples & Positional `struct`s, and Pinned Shared-Quota `Dc 2608` Dispatch (2026-11-14)**:
  - Implemented `src/formats/dcdata/data-src/stage_0_impls/stagel3_to_wasm.wat` (`@dc l8589979107`, `@converts(2192 1927)`) as a standalone System-block `Dc 280 "Built-in type identifier: Routine"` entity declaring `[Dc 302 "Type conversion" Dc 2192 "Begin module" Dc 1927 "Begin WASM program"]`, compiling StageL 3 Token-Dc module bundles directly to Dc-WASM (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`) in `.wat` with full static type/cycle/`buf`-escape validation, coinductive structural `eq`/`ne`, LIFO shadow-frame mutable lexical captures popped before non-child `Dc 665 "WASM return_call"` tail calls, and linked Stage 0 `.WASM` operator primitives + Stage 1 transitive Token-Dc dependencies (`bundle_dc_module_for_wasm_converter` in `src/formats/dcdata/stagel3/seed_linker.rs`).
  - Added `.Semantic` snapshot scoping `Dc 1938 "Resolve to pin"` (`[pin_description]`) and `.EL Builtins` `Dc 2608 "Convert Dc string to WASM"` (`src/formats/dcdata/data-src/stage_0_impls/2608_convert_dc_string_to_wasm.wat`), wired into `Dc 1936 "Dc document execution kernel"` (`ExecutionContext` / `SystemSupervisorAdmission` in `src/runtime/runtime.rs` and `eval_convert_dc_string_to_wasm` in `src/formats/dcstring/navigator/eval.rs`) so both compilation and target `.WASM` execution run under the pinned snapshot and share a single `SharedWasmTaskQuota`.
  - Added anonymous `n`-ary tuples (`Dc 2672 "Built-in type identifier: Tuple"`, `Dc 2215 "Begin n-ary tuple list (ordered, contiguous, typed slots, length n)"`..`Dc 2216 "End n-ary tuple list"`, including `dc_mixed_serde` support for `0`-, `1`-, and `3..=12`-tuples alongside preserved 2-ary `Dc 514 "Key-value pair (tuple)"`..`Dc 515 "End key-value pair"`), nominal positional `struct`s (`Dc 2673 "Built-in type identifier: Struct"`, `Dc 2217 "Begin positional struct declaration"`..`Dc 2220 "End positional struct value"`), and `field` / `with` (`Dc 2547 "Get positional field"`, `Dc 2548 "With positional field"`) across StageL 3 AST, parser, Token-Dc codec, validator, evaluator, Rust transpiler, host `wasm_codegen.rs`, and `stagel3_to_wasm.wat`.
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
  - Restored unmodified upstream `tinywasm 0.8.0` and implemented `instrument_wasm_module_bytes` (`src/formats/dcstring/converters/wasm_quota.rs`) with deterministic V2 structural-checkpoint `deduct-before-effect` compute-quota accounting in appended WASM globals (`$__ctb_deduct`, `$__ctb_deduct_1`), recording `last_interrupted_op_ordinal` and `last_interrupted_op_required_cost` on `WasmResourceInterruption::ComputeQuotaExhausted` (`Privilege::ExcessiveCompute`), live worker-backed continuation suspension (`SessionWorkerHandle` and `__ctb_suspend_on_quota`), `start`-section trap recording (`WasmStartSectionTrap`), trace-replay verification of recorded resource interruptions (`ExecutionContext::replay_wasm_session_trace` / `replay_wasm_bytes_trace`), `BootstrapKernelVersion::V1` opcode gating (`0xFB` GC, exception handling, and Relaxed SIMD) with V8 cross-engine parity (`scripts/test-wasm-compat`), and System-graph `.WASM` / `.Formats:transformation` boundary linting (`src/formats/dcdata/support/{validation,stagel_dc_alloc,format,wat}.rs`).
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
  - [ ] Remaining gap: Move default logical resource limits (`DcWasmResourceLimits`, `DEFAULT_MAX_WASM_*`), operation cost weights, `__ctb_*` bytecode instrumentation (`instrument_wasm_module_bytes`), and task-tree quota management (`SharedWasmTaskQuota`) out of transitional host Rust scaffolding (`wasm.rs` / `wasm_quota.rs`) into the self-hosted System supervisor `.WASM` program so bytecode transformation and cost tables are versioned entirely by the recorded System supervisor revision rather than the host binary. Once migrated, `wasm.rs` / `runtime.rs` should retain only zero-table Dc-WASM lowering (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`), the 32-byte linear-memory mailbox ABI (`0..32`), and host capability mediation / emergency process watchdogs.

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
  checkpoint placement rules, and the `__ctb_*` instrumentation and continuation transform
  in [wasm_quota.rs](../src/formats/dcstring/converters/wasm_quota.rs) and
  [wasm.rs](../src/formats/dcstring/converters/wasm.rs) must be versioned by
  the supervisor revision rather than the host binary.
- **Task-tree quota state**: `SharedWasmTaskQuota` is transitional; the
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
- [ ] **Option B `DcWasmSession` `SeedGraphIndex` capability ABI & StageR 3 `Dc 2608 "Convert Dc string to WASM"` (`Partially Implemented`)**:
  - Implemented the snapshot-scoped Option B host capability ABI on `DcWasmSession` (`Dc 1939 "Minimal query of system graph"`, `Dc 1940 "Execute pure WASM program"`, `Dc 2616 "Request host capability"`), the graph-resident StageR 3 implementation of `Dc 2608 "Convert Dc string to WASM"` in `src/formats/dcdata/data-src/impls/std/convert_dc_string_to_wasm.sr3`, and Guix-style native Rust WASM substitutes (`ctb-wasm-substitutes` / `ctb-wasm-substitutes-gen`, including Merkle-deduplicated shared functions, auto-discovered runnable System-block snapshots, and `mem0` fast-path access) with deterministic V2 checkpoint-ordinal compute-quota metering.
  - The host Rust `eval_convert_dc_string_to_wasm` fallback in `src/formats/dcstring/navigator/eval.rs` is now reached only for pre-migration historical `Dc 641 "Global graph system block most recent node ID at time of document creation"` pins that predate `Dc 2608`'s StageR 3 decomposition.
  - **Remaining Gaps**: Fine-grained `Dc 2616` permission Dcs, checksum-indexed reverse lookup option for `Dc 1939`, and optional profiling-gated `#[inline(never)]`-threshold / `vm.deduct` micro-tuning in `ctb-wasm-substitutes` (`issues/issues.md`).

---

## Deferred Items (Not Required for Compatibility Closure)

### 1. StageL 3, StageR 3, and EM3 Textual Languages
- **Status**: `Partially Implemented` (StageL 3 core, `n`-ary tuples, positional `struct`s, pure anonymous routine value nodes `Dc 2240 "Begin routine"`..`Dc 2241 "End routine"` + `Dc 599 "Self reference"` + decoupled `Dc 1942 "Routine test suite"` / `Dc 573 "Description"` / `Dc 574 "Programmatic identifier"` / `Dc 598 "Is associated with"` metadata across all active routines, `stage_1_impls/std/` modules including `u128`/`i128`/`dctf`/`bigint`/`rat`/`dcs`/`dcmst`, `: @dc` ledger auto-allocator, and self-hosted Stage 0 `f1680_stagel3_to_wasm.wat` `StageL3ToWasm` converter implemented, alongside StageR 3 `.sr3`/`.sri3` Token Dc repertoire, direct parser/codec, static totality/exhaustiveness validator, reference evaluator, `: @dc` allocator, `SeedGraphIndex` linker, self-hosted Stage 1 `stage_1_impls/std/stager3_to_stagel3.sl3` `pub dc/f1681` `StageR 3 -> StageL 3` transformation chained with `f1680` -> `.WASM`, Option B `Dc 1939`/`Dc 1940`/`Dc 2616` capability ABI + graph-resident StageR 3 `Dc 2608 "Convert Dc string to WASM"` in `impls/std/convert_dc_string_to_wasm.sr3`, and Guix-style native Rust WASM substitutes `ctb-wasm-substitutes` / `ctb-wasm-substitutes-gen` with deterministic V2 checkpoint-ordinal compute-quota metering, Merkle-deduplicated shared functions, auto-discovered System-block snapshots, and `mem0` fast-path access; `Dc 2616` permission Dcs, checksum-indexed `Dc 1939` reverse lookup, remaining Unicode-ism audit items, host-windowed `view`/`cursor`/`emitExtent` streaming verbs + capability ABI, ASCII `.sl3`/`.sr3` text-to-Dc syntax converters, native Rust/JS StageR backends, in-graph routine `test`-suite execution, CLI document execution/rendering wiring, and EM3 remain pending).
- **Implemented So Far**:
  - Added `ctb-formats-stagel::stagel3` and `ctb-formats-dcdata::stagel3` modules ([`ast.rs`](../src/formats/dcdata/stagel3/ast.rs), [`parser.rs`](../src/formats/dcdata/stagel3/parser.rs), [`validator.rs`](../src/formats/dcdata/stagel3/validator.rs), [`dc_tokens.rs`](../src/formats/dcdata/stagel3/dc_tokens.rs), [`eval.rs`](../src/formats/dcdata/stagel3/eval.rs), [`rust_codegen.rs`](../src/formats/stagel/stagel3/rust_codegen.rs), [`wasm_codegen.rs`](../src/formats/stagel/stagel3/wasm_codegen.rs)) plus the self-hosted Stage 0 `.wat` document transformation [`f1680_stagel3_to_wasm.wat`](../src/formats/dcdata/data-src/stage_0_impls/f1680_stagel3_to_wasm.wat) (`f1680` `StageL3ToWasm`, `@converts(2192 1927)`): direct `.sl3` lexer/parser with pass-1 routine arities, `n`-ary tuples, positional `struct`s, Token-Dc `Dc 2240 "Begin routine"`..`Dc 2241 "End routine"` + `Dc 599 "Self reference"` support (with inline `v/RET[?] PARAM* :: BODY :: ARG*` source calls gated to `.sr3` and indented `+8` body / `+4` closer across continuation lines), and `do ... ::` / indented blocks; static validator (`ns`/PSR-4 path validation, acyclic `use` DAG resolution, non-shadowing `let`/`set`/`const`/definite-assignment across branches/loops/captures, strict typing with `never` unification, unguarded struct cycle rejection, local-only `buf/T`, tail-call span annotation); bidirectional lossless Token-Dc codec & `.sl3` formatter (stripping `: @dc` from comment frames while preserving `AtomDc` header slots); decoupled System-graph routine storage (`Dc 611`/`Dc 614` -> pure `Dc 2240..2241` value nodes, `Dc 1942 "Routine test suite"` -> `Dc 2206..2207`, and `Dc 574` + `Dc 598` on private module helpers); deterministic bounded compute-quota interpreter with shared capture cells, freeze-enforced `buf/T`, mutual tail-call trampolining, and inline `test` runner; self-hosted `.wat` StageL-Dc-to-WASM transformation invoked via `Dc 1938 "Resolve to pin"` + shared-quota `Dc 2608 "Convert Dc string to WASM"` (running via the StageR 3 graph implementation in `impls/std/convert_dc_string_to_wasm.sr3` and `ctb-wasm-substitutes` for active pins, with host Rust fallback preserved in `eval.rs` strictly for pre-migration historical `Dc 641` pins); host Dc-WASM reference compiler/linker/inline-test runner; and direct StageL 3-to-Rust smoke transpiler with defunctionalized constant-stack tail-call trampolines (across self, mutual, cross-module, and nested-capture calls) validated against `clippy-driver` and `rustc`.
  - Added StageR 3 foundation and self-hosted Stage 1 lowering in `ctb-formats-dcdata`, `ctb-formats-dcstring`, and `ctb-formats-stagel`: StageR 3 Token Dc repertoire in [`categories/em.csv`](../src/formats/dcdata/data-src/categories/em.csv) (`Dc 2221 "Begin StageR module"`..`Dc 2222 "End StageR module"`, nominal `brand`/`type`/named-field `struct`/`enum` decls & constructors, `match`, error propagation, `i128`/`u128`/`int`/`rat`/`dc`/`s`/`dcs`/`option`/`result`/`error`/`view`/`cursor` built-ins/types); [`stagel3.rs`](../src/formats/stagel/stagel3.rs) `normalize_stager3` (bare `"..."` Dcts protection and `? -> /` normalization) and public `.sr3` helpers; StageR 3 extensions across [`parser.rs`](../src/formats/dcdata/stagel3/parser.rs), [`dc_tokens.rs`](../src/formats/dcdata/stagel3/dc_tokens.rs), [`stager_strings.rs`](../src/formats/dcdata/stagel3/stager_strings.rs), [`validator.rs`](../src/formats/dcdata/stagel3/validator.rs) (strict totality disallowing `fault` / `*StageLInternal` / `unwrapSome` / `unwrapOk` / `unwrapErr` / transitive `may_fault` calls, opaque `type` encapsulation, shallow exhaustive `match`, definite assignment, recursive type cycle checks, exact `conv`, and Stage 1 carrier bridging for `i128Val`/`u128Val`/`bigInt`/`ratVal`/`dcVal`), and [`eval.rs`](../src/formats/dcdata/stagel3/eval.rs) (total checked arithmetic, Euclidean `mod` vs `truncRem`, `BigInt`/`BigRational`, `s`/`dcs`/`dc:` operations, match tail-call trampolining, and inline `test` runner); `: @dc` ledger auto-allocator & Stage 1/Stage 2 gating in [`stagel_dc_alloc.rs`](../src/formats/dcdata/support/stagel_dc_alloc.rs) and `SeedGraphIndex` multi-module / direct-`dc/<shorthand>` linking in [`seed_linker.rs`](../src/formats/dcdata/stagel3/seed_linker.rs); unified parameter-type-dispatched Stage 1 standard libraries (`stage_1_impls/std/{math,u128,i128,dctf,bigint,rat,dcs,dcmst}.sl3`); self-hosted Stage 1 `StageR 3 -> StageL 3` document transformation [`stager3_to_stagel3.sl3`](../src/formats/dcdata/data-src/stage_1_impls/std/stager3_to_stagel3.sl3) (`pub dc/f1681`, `@chain(2221 > 2192)`), chained end-to-end with `f1680` `StageL3ToWasm` in [`wasm.rs`](../src/formats/dcstring/converters/wasm.rs) and [`transform.rs`](../src/formats/dcstring/converters/transform.rs); Option B `DcWasmSession` capability ABI (`Dc 1939` / `Dc 1940` / `Dc 2616`) powering [`convert_dc_string_to_wasm.sr3`](../src/formats/dcdata/data-src/impls/std/convert_dc_string_to_wasm.sr3) (`pub dc/2608`); and Guix-style native Rust WASM substitutes (`ctb-wasm-substitutes` / `ctb-wasm-substitutes-gen`) with deterministic V2 checkpoint-ordinal compute-quota metering.
- **Pending Work**: Complete remaining StageR 3 work (`Dc 2616` permission Dcs, checksum-indexed `Dc 1939` reverse lookup, remaining Unicode-ism audit items, host-windowed `view`/`cursor` `peekU8`/`readDc`/`emitExtent` streaming verb Dcs + host capability ABI, ASCII `.sl3`/`.sr3` text-to-Dc syntax converters, and native Rust/JS StageR backends), wire end-to-end CLI Dc document execution and rendering (`ctoolbox` CLI -> `ctb-runtime` -> `ctb-renderer`, writing core rendering/runtime passes in StageR 3 or EM3), implement EM3 (`f658` / `f661`) in StageR 3 (lowering EM3 first-class functions and closures into StageR 3 / StageL 3 helper routines and typed dispatchers), and add in-graph routine `test`-suite execution, following the [language layers](#language-layers-2026-11-11). Expression values, lexical capture, and stack-safe tail calls are required throughout; first-class function values are deliberately omitted in StageL 3 and StageR 3 for reasoning clarity and supported exclusively in EM3. The older assembler/VSO tier labels below do not replace this implementation hierarchy.
- **Preserved Design Notes from [`docs/chats/2026-10-31-language.md`](../docs/chats/2026-10-31-language.md) & Subsequent Clarifications**:
  - **`Dcsh`+ assembler (earlier Tier 0 proposal)**: Minimal human-writable assembler extending `src/formats/dcstring/converters/dcal.rs` with `@ident` resolution, inline literals, and automatic closer insertion via `Dc 590 "Is a closing delimiter of syntactic block for"`.
  - **EM3 (VSO / Polish Prefix Semantic & Programming Language, implemented in StageR 3)**:
    - General-purpose English-like VSO / Polish prefix language usable for both code and UI localization (transpiling to semantic Dcs and rendering into natural languages or compiling to WASM).
    - **Implementation & lowering boundary**: EM3 is planned to be implemented in StageR 3 (`impls/`), lowering EM3 first-class functions and closures into StageR 3 / StageL 3 helper routines and environment records (whereas StageL 3 and StageR 3 deliberately omit first-class function values so low-level code remains easier to reason about). Most higher-level runtime, renderer, workspace, and application logic (and potentially higher-level `csc` archive conversion/orchestration on top of StageR 3 streaming primitives) is expected to be written in StageR 3 or EM3.
    - Two lexical profiles mapping to the same token stream (aligned with `const` for constants, `let` for mutable variables, `set` for mutation, and `v/...` for verb declarations):
      - **Speakable profile (`.em3`)**: `Doing see I you.`, `Is to dipre meet I you par good.`, `Const var i32 var foo add i32:3 i32:4.`, `Let var i32 var bar var i32 var foo.`, `Set var i32 var bar mul var i32 var bar dipre daun.`, `Verb i32 sum i32 p1 i32 p2 par add i32 p1 i32 p2.`
      - **Compact code profile (`.emc`)**: `doing see I you`, `is to is meet I you } good`, `const i32/foo add i32:3 i32:4`, `let i32/bar i32/foo`, `set i32/bar mul i32/bar i32/bar`, `v/i32/sum i32/p1 i32/p2 } add i32/p1 i32/p2;`
    - **EM3 function / verb literal conventions**:
      - Named routine: `v/i32/sum i32/p1 i32/p2 } add i32/p1 i32/p2;`
      - Anonymous verb literal (first-class value, EM3-only): `v/i32/ i32/p1 i32/p2 } ...`
      - Untyped/any return (`Dc 275 "Built-in type identifier: * (Object or any subtype)"`): `v//...`
      - Unit-returning routines: explicit `v/void/log`; routines that never return normally: `v/never/log` (neither uses `v//log`).
    - **Repetition Dcs (`dipre`, `daun`, `dato`)**: Compact 1-token repetition of the word before previous (`dipre`) or previous word (`daun`, `dato`) to provide elegant variation when spoken.
    - **Possibly new anaphoric / backreference Dcs**: Compact 1-token backreferences to the previous clause or previous identifier to avoid repeating long identifiers or assigning intermediate node IDs.

### 2. Wholesale CST (`DcSyntaxKind`) Rewrite
- **Status**: `Deferred`.
- Replacing `DcSyntaxKind` variants in `src/formats/dcstring/navigator/arena.rs` or removing compiled lookup caches in `DatasetRuleResolver` is not required for backwards compatibility; compiled Rust tables and AST caches are acceptable wherever they faithfully and deterministically reflect the document's pinned graph snapshot.

### 3. Rust-to-EM3 Transpiler Target
- **Status**: `Deferred` (intentionally deferred — do not implement now).
- Making EM3 viable as a transpilation target from Rust could eventually allow reusing existing Rust code inside the graph. However, the design approach is uncertain and the append-only immutability of the global graph makes avoiding any plagiarism or licensing contamination mission-critical: once code is published to the global graph, it cannot be rolled back. Restricting graph-resident code to languages authored specifically for Collective Toolbox (`.wat`, StageL 3, StageR 3, EM3) provides an important structural provenance and licensing safeguard, so Rust-to-EM3 transpilation is deferred indefinitely.

## Possible design considerations

1. **Shared Memory Between Isolated Document Processes vs. Deterministic Replay & Torn Reads**:
   - If two concurrent document processes share **writable** memory (`MAP_SHARED` read/write) directly, OS scheduling jitter makes reads non-deterministic (breaking bit-for-bit replay) and causes torn-read / Rust `&[u8]` aliasing UB if a native Rust fast path validates DcTF bounds while another process mutates the bytes.
   - **Recommendation**: Make cross-document shared memory **immutable/write-sealed snapshots** (`memfd` + `F_SEAL_WRITE` on Linux) or exchange buffers only at explicit host capability yield points. That preserves ~95% of zero-copy performance (passing subdocuments, rendered frames, fonts, and large `view` slices without copying) with zero data races and 100% deterministic replay.

2. **Precompiled Built-In Rust Dcs vs. `Dc 641` Pinning & Compute Quotas**:
   - **Pinning trap**: Never key built-in Rust fast paths by **Short Dc ID** alone. If a Dc's implementation is later updated via `Dc 591 "Supersedes"`, an older document pinned via `Dc 641 "Global graph system block most recent node ID at time of document creation"` must run the historical revision, not `HEAD`'s compiled Rust code. Key the built-in native table by the **resolved implementation node ID** `(impl_node_id, dep_closure_hash)` so matching pins hit native code and superseded pins fall back cleanly to their pinned graph WASM.
   - **Infinite loops & permissive mode**: Native Rust lacks TinyWasm's automatic per-opcode metering (`wasm_quota.rs`). The StageR-to-Rust transpiler should emit a cheap `&mut u64` compute-quota decrement on loop backedges and non-inlined calls (plus a recursion depth counter) and return `Result<_, DcFault>` rather than panicking, so untrusted loops in precompiled Dcs can still suspend and prompt the user in permissive mode.

3. **One Dc per Function: Separate Graph Granularity from Runtime Call Granularity (and Conserve Short IDs)**:
   - Allocating Dcs for small functions is great for graph storage DRYness and caching, **provided** calling a helper Dc doesn't spin up a separate `DcWasmSession` and serialize DcTF through linear-memory mailbox `0..32` on every call. The StageL/StageR/WAT compilers should **link and direct-call/inline** resolved helper Dcs into one compiled module at snapshot-compile time.
   - Also keep an eye on ID namespaces: the Short Dc region (`1_114_112..=2_228_223`, ~1.1M IDs) is finite, whereas the System block (`l<id>`, 4.29B IDs) is huge. Reserving Short Dcs for canonical semantic/library concepts and letting internal compiler/shaping helpers live as System-block or dedicated library-block nodes avoids fragmenting Short Dc blocks.

4. **Words, Characters, Contextual Shaping, and Immediate Test-Speed Wins**:
   - **Mixed word-Dc vs. character-run text**: When one string encodes `"button"` as 6 Unicode character Dcs and another uses a single word Dc linked via `Dc 397 "Equivalent representations"` / `Dc 561 "Partially equivalent representations"`, structural `dcs` token operations (`len`, `eq`, `explode`) will see different lengths unless kept explicitly distinct from equivalence-expanded/orthographic text operations. Keeping `dcs` strictly token-exact (for lossless round-tripping) and exposing separate orthographic/equivalence-expansion helpers prevents surprises.
   - **Character -> Glyph is contextual (`M`-to-`N`), not 1-to-1**: Ligatures, kerning, Arabic contextual forms, Indic conjuncts, and emoji ZWJ sequences mean glyph selection depends on neighboring characters. Shape and cache at the **text-run / word / cluster** boundary `(font_dc, shaping_features, dc_run) -> positioned_glyph_dcs` (with a 1-to-1 ASCII/CJK fast path only when a font declares no contextual rules for those code points).
   - **Near-term fix for slow tests**: Even before Rust AOT lands, process-wide memoization of parsed `SeedGraphIndex` / `DatasetRuleResolver` snapshots and compiled `tinywasm::Module` artifacts keyed by `(bundle_hash, pin_node_id)` is completely sound under append-only pinning and should cut repeated setup overhead substantially in tests.

5. **Role of `.sli3` / `.sri3` vs. Lossless AppleScript-Style Token Dc Reconstruction**:
   - Because `stage_0_impls/f1680_stagel3_to_wasm.wat` transpiles **from StageL Token Dcs** (already tokenized and block-framed!) rather than parsing ASCII text, `.sl3` and `.sr3` seed files tokenize **directly** into StageL / StageR Token Dcs without passing through case-folding `.sli3` / `.sri3` first. Otherwise `.sli3`'s `UPPERCASE` folding and comment `:CHAR:...:` escaping would discard camelCase on local variables (`let u32/maxHeaderBytes`) and comments during AppleScript-style source reconstruction. `.sli3` / `.sri3` remain optional 32-character ASCII-armored interchange formats.

6. **Sharing Semantic Token Dcs Across StageL 3, StageR 3, and EM3 (Without Colliding With Incompatible Legacy `.EL` Rules)**:
   - StageL 3, StageR 3, and EM3 should share **one Dc per identical semantic construct** (`add`, `truncRem`, `if`, `let`, `set`, `const`, `pub`, `i32`, etc.) rather than allocating duplicate per-language token Dcs. Existing `.Semantic` / `.EL` Dcs (`Dc 517 "Boolean true"`, `Dc 518 "Boolean false"`, `Dc 6 "Begin number"`..`Dc 7 "End number"`, `Dc 310 "Begin list (ordered, contiguous, typed)"`..`Dc 311 "End list"`, `Dc 246 "Begin single-line source comment on current line"`..`Dc 248 "End single-line source comment"`) can be reused where semantics match, while constructs whose legacy `.EL` rules conflict—such as `Dc 541 "Enter identifier scope"`, which permits variable shadowing that StageL 3 forbids—need distinct Dcs.

7. **Type-Polymorphic StageL Built-In Token Dcs (`add`, `mod`, `explode`) vs. Monomorphic `.WASM` Implementations**:
   - In Token Dcs, `add` or `explode` is a single semantic operator Dc (so source reconstruction always prints `add` / `explode`), whereas `.WASM` opcodes and memory layouts are monomorphic (`u8` vs `i32` vs `i64`, `l/u8` vs `t`). Non-trivial built-in `.WASM` helpers in the graph should therefore be indexed by `(builtin_dc, type_dc)` (via `Dc 646 "In context of"`) while 1-opcode built-ins (`truncRem` -> `Dc 758 "WASM i32.rem_s"` / `Dc 776 "WASM i64.rem_s"`) lower directly or carry per-type 1-instruction decompositions.

8. **`: @dc <shorthand>` Comments Must Anchor Identity on Read While Being Stripped on Emit**:
   - Treating `: @dc l<id>` solely as an ignored comment on read would cause renaming a verb or reordering declarations in a `.sl3` file to allocate a brand-new Dc ID instead of superseding the existing one (`Dc 591 "Supersedes"`). Reading `: @dc l<id>` as the declaration's identity anchor (exactly like Column 1 of `categories/*.csv`) while stripping `: @dc ...` from the emitted Token Dc body gives stable IDs across edits/renames **and** zero extra graph revisions. Auto-assignment should run only during explicit generator/allocator invocations so read-only `cargo test` runs never dirty working-tree `.sl3` files.

9. **Pure / Compile-Time Constraint on Top-Level `const` and `pub` Value Initializers**:
   - Allowing `pub TYPE/name EXPR.` alongside `const TYPE/name EXPR.` works cleanly without parse ambiguity (value references use `TYPE/name` whereas 0-arg verb calls use bare `name`, and redundant `const v/...` is disallowed). Restricting top-level `const` / `pub` value initializers to pure, side-effect-free expressions with acyclic dependencies ensures deterministic, order-independent module loading and lets constants materialize directly as immutable graph value nodes.


## 12 Nov 2026

- **Compute-quota metering in the direct StageL 3 smoke transpiler (`rust_codegen.rs`)**: Unlike the StageL interpreter in [eval.rs](file:///workspaces/ctoolbox/src/formats/dcdata/stagel3/eval.rs) and the WASM-to-Rust native substitutes in [wasm_to_rust.rs](file:///workspaces/ctoolbox/src/formats/dcstring/converters/wasm_to_rust.rs) (which enforce deterministic V2 checkpoint-ordinal compute-quota metering), smoke-test Rust code emitted directly by [rust_codegen.rs](file:///workspaces/ctoolbox/src/formats/stagel/stagel3/rust_codegen.rs) executes natively without compute-quota accounting (so unbounded tail loops run until externally terminated rather than interrupting on a compute quota). Production direct StageR 3-to-Rust AOT compilation must include deterministic compute-quota accounting.
- **Local `buf/T` parameters on tail-called routines**: StageL 3 static validation ([validator.rs](file:///workspaces/ctoolbox/src/formats/stagel/stagel3/validator.rs) and [sl3.md](file:///workspaces/ctoolbox/src/formats/stagel/docs/sl3.md#L165-L174)) restricts `buf/T` to uncaptured local bindings and forbids passing `buf/T` as routine parameters or captures, so `buf/T` never crosses tail-call boundaries; if future language layers (such as StageR 3 or EM3) introduce linear/region reference parameters, trampoline frames will need corresponding lifetime or handle lowering.