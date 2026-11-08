# Minimal Trusted Bootstrap Kernel Specification (`BootstrapKernelVersion::V1`)

Status: **Draft / Partially Implemented**. This document inventories the current
`BootstrapKernelVersion::V1` host implementation; it is not yet an exhaustive,
frozen execution specification. The wire-format corrections below describe the
current implementation, not a new encoding.

## 0. Version Contract & Bootstrap Selection

The host exposes [`BootstrapKernelVersion`](../src/formats/dcstring/navigator/policy.rs)
and rejects unsupported version numbers (`BootstrapKernelVersion::try_from_u32`).
Only `BootstrapKernelVersion::V1` (`1`) currently exists.

To ensure version selection happens before any leading `Dc 641` (`"Global graph system block most recent node ID at time of document creation"`) pin inspection or token decoding runs, [`DcDocumentNavigator`](../src/formats/dcstring/navigator.rs) provides upfront bootstrap constructors:
- `DcDocumentNavigator::try_from_mstr_with_kernel_version_number(mstr, version)` and `DcDocumentNavigator::from_mstr_with_kernel_version(mstr, kernel_version)`
- `DcDocumentNavigator::try_from_dc_str_with_kernel_version_number(text, version)` and `DcDocumentNavigator::from_dc_str_with_kernel_version(text, kernel_version)`
- `DcDocumentNavigator::from_mstr_with_policy(mstr, policy)` and `DcDocumentNavigator::from_dc_str_with_policy(text, policy)`

These constructors validate and install `self.kernel_version` prior to invoking `detect_leading_document_most_recent()`.

Many syntax rules and WASM definitions live in the append-only System graph.
Procedural evaluators and implicit rule synthesis still exist in Rust; their
migration is tracked in Gates 3 and 4 of [the language plan](../issues/language.md).
The four layers below are the intended kernel boundary, not proof that the
current implementation has already been reduced to that boundary.

---

## 1. Physical Token Codecs (`DcTF`, `.dcmm` Header, and Binary Encapsulation)

### 1.1 Global Graph ID Space and `DcTF` (`f641`)
Every token in a Dc stream is a 128-bit unsigned Global Graph ID (`u128` / [`DcChar`](../src/formats/dcdata/dc_char.rs)):
- **Unicode region** (`0x0000_0000..=0x0010_FFFF`, `u<hex>`): standard Unicode scalar values and surrogate code points.
- **Short Dc region** (`1_114_112..=2_228_223`, `<decimal>`): the short ID plus `1_114_112`.
- **Format region** (`2_228_224..=3_342_335`, `f<N>`): strongly typed `FormatId` values, whose graph IDs are `2_228_224 + N`.
- **Additional reserved Dcs** (`3_342_336..=4_294_967_295`).
- **Reserved Data block** (`4_294_967_296..=8_589_934_591`): not the format region; per-block pinning support remains pending.
- **System graph block** (`8_589_934_592..=12_884_901_887`, `l<id>`).

The authoritative layout is [global-graph-layout.csv](../src/storage/minimal/data/global-graph-layout.csv).

`DcTF` (`ctb_formats_dctf`, [`src/formats/dctf/codec.rs`](../src/formats/dctf/codec.rs)) encodes each `u128` token as follows:
1. Values through `0x7F`, `0x7FF`, `0xFFFF`, `0x1F_FFFF`, `0x3FF_FFFF`,
   `0x7FFF_FFFF`, and `0xF_FFFF_FFFF` use respectively 1 through 7 bytes.
   Their lead-byte prefixes follow extended UTF-8; continuation bytes carry
   six bits. Surrogate values are permitted.
2. Larger values use `0xFF`, a second byte `10_kkkk_bb`, and `k + 6`
   continuation bytes, for a total of `8 + k` bytes (`k = 0..=15`). The
   second byte carries the high two payload bits and the tier index.
3. Each graph ID is encoded directly as one integer. There is no separate
   long-Dc escape token in DcTF. This differs from the legacy UTF-8e-128 codec.

Canonical-length rejection, malformed-input recovery, and legacy conversion
need explicit conformance vectors before this codec contract can be frozen.

### 1.2 `.dcmm` (`DcMixed2`, `f643`) File Header
Every binary `.dcmm` document ([`src/formats/dcstring/dc_mixed.rs`](../src/formats/dcstring/dc_mixed.rs)) begins with a fixed 21-byte header (`DCMM_HEADER_LEN = 21`):
- Bytes `0..5`: ASCII magic `b"DcMM\0"` (`[0x44, 0x63, 0x4D, 0x4D, 0x00]`).
- Bytes `5..21`: 16-byte format UUID (`b0327a84-b745-492c-b39d-77c2827e3a6f` for `DcMixed2` `f643`; legacy `d5ab819e-9b8e-4d61-adbc-933e8851e79a` for `DcMixed1` `f642`).
- Bytes `21..`: `DcTF` token stream interspersed with binary encapsulation frames.

### 1.3 Binary Encapsulation
The current binary DcMixed2 frame uses an opener followed by:
1. A one-byte type: `0` (raw), `2` (length-delimited metadata), or `3` (SHA-256).
2. A **16-byte big-endian `u128` raw-payload length**, excluding metadata and
   checksum bytes.
3. For type `2`, a 16-byte big-endian metadata length and that many DcTF
   metadata bytes; for type `3`, a 32-byte SHA-256 digest; for type `0`, neither.
4. Exactly the declared raw payload bytes. No closer token follows this binary
   frame, and no literal opener terminates its metadata.

The reader rejects unsupported legacy types `1` and `4`, size overflows,
truncation, invalid metadata, and checksum mismatches. Textual encapsulation
has a separate delimited representation; it must not be confused with this
binary header. See [dc_mixed.rs](../src/formats/dcstring/dc_mixed.rs) and
[encapsulation.rs](../src/formats/dcstring/encapsulation.rs).

---

## 2. Active Triple Resolution in `SeedGraphIndex`

`SeedGraphIndex` ([`src/formats/dcdata/seed_graph.rs`](../src/formats/dcdata/seed_graph.rs)) loads `.ctbnbdl` (`CtbNodeBundle2`, `f645`, magic `b"CTBNBDL\x02"`) node bundles up to an optional watermark `most_recent_node_id` (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).

### 2.1 Snapshot Filtering (`Dc 641`)
When pinned to `most_recent_node_id`, only `LedgerNode`s with `node.id <= most_recent_node_id` are included in the snapshot. Any supersession or retraction statement with `stmt_id > most_recent_node_id` is excluded, restoring the exact active graph state as of `most_recent_node_id`.

### 2.2 Lifecycle Resolution (`Dc 591` `"Supersedes"` and `Dc 592` `"Retracts"`)
1. **Supersession** (`591 <new_id> <old_id>`): Marks `<old_id>` as inactive (`inactive_ids.insert(old_id)`) and records `superseded_by[old_id] = new_id`. `SeedGraphIndex::resolve_latest_node_id(id)` follows `superseded_by` edges deterministically (with cycle detection) to the terminal active node ID.
2. **Retraction** (`592 <stmt_id> <target_id>`): Marks the target inactive and
   records its retraction relationship. The retraction statement does not
   become inactive merely by retracting another node; it can itself be
   retracted later. Dependent value statements cannot make an inactive property
   statement effective. Lifecycle resolution must account for retracted
   retractions and supersessions at the selected snapshot.

### 2.3 Value Nodes and Property Triples (`Dc 584` `"Has a"` and `Dc 610` `"Has value"`)
1. **String and Integer Value Nodes**:
   - A node whose tokens are `[DcShorthand::Unicode(cp), ...]` (or `[Dc 260 "Begin literal value", Dc 262 "Begin type definition", Dc 264 "Built-in type identifier: String", Dc 263 "End type definition", Dc 191 "Begin encapsulated UTF-8 string", ... Dc 192 "End encapsulated UTF-8 string", Dc 261 "End literal value"]`) is indexed in `string_by_node_id`.
   - A node whose tokens encode a `Dc 6` (`"Begin number"`)..`Dc 7` (`"End number"`) integer literal is decoded via base-64 digits (`Dc 127`..`Dc 190`) and indexed in `integer_by_node_id`.
2. **Two-Part Property Statements**:
   - Statement node `l<has_a_id>: 584 <subject> <property>` declares that `<subject>` has property `<property>` (`Dc 584` `"Has a"`).
   - Statement node `l<has_val_id>: 610 l<has_a_id> <value_token>` binds `<value_token>` to `l<has_a_id>` (`Dc 610` `"Has value"`).
   - Both `l<has_a_id>` and `l<has_val_id>` must be active (`!inactive_ids.contains(&id)`).
   - Single-valued properties (`Dc 572` `"Canonical name"`, `Dc 573` `"Description"`, `Dc 574` `"Programmatic identifier"`, `Dc 580` `"Canonical combining class"`, `Dc 581` `"Bidirectional class"`, `Dc 582` `"Case mapping partner"`, `Dc 594` `"Syntax rule"`, etc.) are indexed in `active_single_props[(subject, property)]`.
   - Multi-valued properties (including `Dc 611` `"Canonical decomposition"`) are indexed in `multi_props_by_subj_prop[(subject, property)]`, ordered with `Dc 570` (`"Preferred"`) entries first.
3. **Direct Binary Statements**:
   - Active 3-token nodes `l<stmt_id>: <verb> <subject> <object>` (where `<verb>` is `Dc 586` `"Is instance of"`, `Dc 587` `"Is subtype of"`, `Dc 588` `"Is based on"`, `Dc 589` `"Is deprecated"`, `Dc 590` `"Is closing delimiter for"`, `Dc 598` `"Is associated with"`, `Dc 645` `"Is stable"`, `Dc 646` `"In context of"`, etc.) are indexed in `active_direct_stmts[(verb, subject, object)]`.

### 2.4 Canonical Decomposition & Equivalence (`Dc 611` `"Canonical decomposition"`)
Active `Dc 611` `"Canonical decomposition"` properties on a subject define `<equiv>` relationships:
- Connected components of `<equiv>` pairs form equivalence classes in `DatasetRuleResolver` (`canonical_map` and `semantic_equivalents`), choosing the lowest non-deprecated terminal target as the canonical token.
- All syntax rules (`Dc 594` `"Syntax rule"`), `@closes` (`Dc 590` `"Is closing delimiter for"`), container headers, infix rules, and subtype relations are validated for parity and closed transitively across equivalent tokens.
- If a subject's active `Dc 611` `"Canonical decomposition"` (or `Dc 571` `"Decomposition"` / `Dc 614` `"Semantic decomposition"`) points to a node beginning with `Dc 1927` (`"Begin WASM program"`) or `wasm_routine` (`260 262 280 ...`), `SeedGraphIndex::get_wasm_definition_tokens` returns that `.WASM` token slice for pure-compute evaluation.

---

## 3. `.Semantic:Pattern` Operators (`Dc 599`..`Dc 607`, `Dc 642`, `Dc 643`, `Dc 644`)

Syntax rules (`Dc 594` `"Syntax rule"`) are stored in the seed graph as token sequences using `.Semantic:Pattern` operators and decoded by [`SeedGraphIndex::decode_syntax_rule_tokens`](../src/formats/dcdata/support/seed_graph_reconstruct.rs):

The following is an implementation inventory, not yet normative semantics.
Choice/backtracking order, capture scope, recursive references, malformed
patterns, range arity, and historical operator encodings still need independent
conformance fixtures. An encoding accepted by the current decoder is not by
itself evidence that its historical semantic identifier was used correctly.

| Operator Dc | Name | Kernel Semantics in `.Semantic:Pattern` |
|---|---|---|
| `Dc 599` | `"Self reference"` | Matches the subject token `self` (or any token equivalent to `self` via `Dc 611` `"Canonical decomposition"`). |
| `Dc 600` | `"Wildcard pattern"` | Matches any single token (`SyntaxTerm::Any`). |
| `Dc 601` | `"Pattern conjunction"` | Sequential/conjunctive composition within an associativity group (`Dc 298` `"Begin associativity group"` .. `Dc 299` `"End associativity group"`). |
| `Dc 602` | `"Pattern alternation"` | Inside `298 602 (branch1) (branch2) ... 299`, matches the first matching branch (`SyntaxPattern::Alternation`). Inside `298 602 tok1 tok2 ... 299` (where `tok1 != 298`), defines a positive character set (`SyntaxTerm::CharSet { negated: false, .. }`). |
| `Dc 603` | `"Pattern repetition"` | Postfix quantifier group `298 603 643 f200 <min_node> <max_node_or_316> 299` (or `298 603 4 <min_node> [<max_node>] 299`) specifying `Quantifier` (`ZeroOrMore` `0..`, `OneOrMore` `1..`, `Optional` `0..=1`, or bounded `Range { min, max }`). |
| `Dc 604` | `"Negated pattern"` | Inside `298 604 tok1 tok2 ... 299`, defines a negated character set (`SyntaxTerm::CharSet { negated: true, .. }`) matching any token not equivalent to a member. |
| `Dc 605` | `"Subpattern match"` | Inside `298 605 <target> ... 299`, invokes a named construct (`SeedEntityKey::NamedConstruct`), a script membership check (`SeedEntityKey::Script`), a constrained construct (`298 605 300 l<construct_id> <constraint_tokens> 299`), or another token's syntax rule (`SyntaxTerm::RuleRef`). |
| `Dc 606` | `"Pattern capture"` | Inside a `Dc 605` `"Subpattern match"` group (`... 606 l<var_name_str_node>`), binds the matched subpattern span/value to capture variable `<var_name>` for downstream bound-value constraints (`Dc 540` `"Reference bound value"`) or replacement actions (`Dc 607` `"Pattern replacement"`). |
| `Dc 607` | `"Pattern replacement"` | Trailing action group `298 607 279 l<method_str_node> <args...> 299` (`SyntaxAction`) specifying the semantic projection or `.WASM` routine transformation for the matched rule. |
| `Dc 642` | `"Data byte length"` | Inside `[data:...]` subtype constraints (`642 l<int_node>` or `300 <fmt_tok> 642 l<int_node>`), constrains the exact decoded byte length of a binary encapsulation payload. |
| `Dc 643` | `"Range"` | Token range `643 640 <start_tok> <end_tok>` (`SyntaxTerm::CharRange`), length range `643 f200 <min_node> <max_node>` (`SubtypeConstraint::LengthRange`), repetition bounds in `Dc 603` `"Pattern repetition"`, or numeric range `643 <domain_type> <lower_endpoint> <upper_endpoint>` (`NumberSubtypeConstraint`). |
| `Dc 644` | `"Exclusive range endpoint"` | Prefix qualifier (`644 <bound_node>`) on a lower or upper endpoint in a `Dc 643` `"Range"` constraint marking that bound as strictly exclusive (`>` or `<` rather than `>=` or `<=`). Unbounded endpoints use `Dc 316` (`"Optional value absent"`). |

---

## 4. Dc-WASM Binary Lowering (`Dc 647`..`Dc 1928`) & Linear-Memory Mailbox ABI

### 4.1 Zero-Table Arithmetic Opcode Lowering (`Dc 647`..`Dc 1926`)
Every WebAssembly instruction in a `.WASM` program (`Dc 1927` `"Begin WASM program"` .. `Dc 1928` `"End WASM program"`) lowers to standard WebAssembly binary bytecode via five contiguous 256-slot short-Dc pages ([`decode_wasm_opcode`](../src/formats/dcstring/converters/wasm.rs)):
1. **Core single-byte page** (`Dc 647`..`Dc 902`, `256` slots):
   - `byte = short_id - 647` (`0x00..=0xFF`, e.g., `Dc 647` `"WASM unreachable"` -> `0x00`, `Dc 650` `"WASM loop"` -> `0x03`, `Dc 659` `"WASM br"` -> `0x0C`, `Dc 753` `"WASM i32.add"` -> `0x6A`). Structured control-flow instructions (`Dc 649` `"WASM block"`, `Dc 650` `"WASM loop"`, `Dc 651` `"WASM if"`, `Dc 678` `"WASM try_table"`) encode an empty blocktype (`0x40`) via `Dc 316` (`"Optional value absent"`).
2. **`0xFB` GC prefix page** (`Dc 903`..`Dc 1158`, `256` slots):
   - Prefix byte `0xFB` followed by `u32` LEB128 opcode `short_id - 903`.
3. **`0xFC` miscellaneous prefix page** (`Dc 1159`..`Dc 1414`, `256` slots):
   - Prefix byte `0xFC` followed by `u32` LEB128 opcode `short_id - 1159`.
4. **`0xFD` SIMD low page** (`Dc 1415`..`Dc 1670`, `256` slots):
   - Prefix byte `0xFD` followed by `u32` LEB128 opcode `short_id - 1415` (`0..=255`).
5. **`0xFD` SIMD high page** (`Dc 1671`..`Dc 1926`, `256` slots):
   - Prefix byte `0xFD` followed by `u32` LEB128 opcode `256 + (short_id - 1671)` (`256..=511`).

### 4.2 `.WASM` Module Envelope (`Dc 1927`..`Dc 1928`)
A `.WASM` program is enclosed in `Dc 1927` (`"Begin WASM program"`) .. `Dc 1928` (`"End WASM program"`) and contains:
- Optional global variable declarations (`Dc 260` `"Begin literal value"`, `Dc 262` `"Begin type definition"`, `<valtype>`, `Dc 263` `"End type definition"`, `Dc 270` `"Begin identifier"` .. `Dc 271` `"End identifier"`, `<init_expr>`, `Dc 261` `"End literal value"`).
- Optional static linear-memory data segments (`Dc 260` `"Begin literal value"`, `Dc 6` `"Begin number"` .. `Dc 7` `"End number"`, `Dc 203` `"Begin encapsulated binary data"` .. `Dc 204` `"End encapsulated binary data"`, `Dc 261` `"End literal value"`).
- One or more `wasm_routine` definitions (`Dc 260` `"Begin literal value"`, `Dc 262` `"Begin type definition"`, `Dc 280` `"Built-in type identifier: Routine"`, ..., `Dc 261` `"End literal value"`), exporting `"step"` or `"main"` and default 1-page exported `"memory"`.

### 4.3 32-Byte Linear-Memory Mailbox ABI (`0..32`)
The host communicates with the instantiated WebAssembly module exclusively through linear memory `"memory"` and the exported `"step"` (or `"main"`) function without host FFI imports ([`DcWasmSession`](../src/formats/dcstring/converters/wasm.rs)):

| Byte Offset | Constant | Type | Description |
|---|---|---|---|
| `0..4` | `DC_WASM_MAILBOX_INPUT_PTR` (`0`) | `u32` LE | Byte offset of the input DcTF payload slice in linear memory (`base_io_offset`, minimum `256`, 8-byte aligned past the maximum of `MemoryInstance::static_data_end()`, optional exported `__heap_base`/`__data_end` globals, and any nonzero byte written during `start`). |
| `4..8` | `DC_WASM_MAILBOX_INPUT_LEN` (`4`) | `u32` LE | Byte length of the input DcTF payload slice (with the 21-byte `.dcmm` header stripped). |
| `8..12` | `DC_WASM_MAILBOX_OUTPUT_PTR` (`8`) | `u32` LE | Byte offset of the output DcTF/DcMixed slice in linear memory (pre-initialized by the host before each `step()` to an 8-byte-aligned scratch region after the input buffer; may be updated by the guest). |
| `12..16` | `DC_WASM_MAILBOX_OUTPUT_LEN` (`12`) | `u32` LE | Byte length of the output slice written by the guest during `step()` (reset to `0` by the host before each `step()`). |
| `16..32` | `DC_WASM_MAILBOX_MOST_RECENT_NODE_ID` (`16`) | `u128` LE | Pinned `Dc 641` (`"Global graph system block most recent node ID at time of document creation"`) watermark (`0` when unpinned). |

### 4.4 `step()` Return Status Codes, Numeric/Trap Semantics, and VM Resource Budgets
Each invocation of `"step"` returns an `i32` [`WasmStepStatus`](../src/formats/dcstring/converters/wasm.rs):
- `0` (`WasmStepStatus::Complete`): Execution finished; the host reads `output_ptr..output_ptr + output_len` as the final output [`DcMstring`].
- `1` (`WasmStepStatus::YieldFrame`): Emitted an intermediate streaming frame in `output_ptr..output_ptr + output_len`; linear memory and globals remain intact for the next `step()` call.
- `2` (`WasmStepStatus::RequestCapability`): Execution paused requesting a host capability described by the Dc document in `output_ptr..output_ptr + output_len`. The host evaluates the capability request against the current host [`ExecutionContext`](../src/runtime/runtime.rs), writes any allowed or mocked response into the input mailbox via `provide_input`, and calls `step()` again.

**Supported WebAssembly Feature Subset & Deterministic Numeric/Trap Semantics**:
- Supported value types: `i32` (`0x7F`), `i64` (`0x7E`), `f32` (`0x7D`), `f64` (`0x7C`), `v128` (`0x7B`), `funcref` (`0x70`), `externref` (`0x6F`). 64-bit linear memories (`memory64`) are rejected at instantiation (`Error::UnsupportedFeature`).
- Integer division by zero (`i32.div_s`, `i32.div_u`, `i64.div_s`, `i64.div_u`, `rem_s`, `rem_u`), signed division overflow (`i32::MIN / -1`, `i64::MIN / -1`), out-of-bounds linear memory or table accesses, `unreachable` (`0x00`), and indirect call signature mismatches trap deterministically (`tinywasm::Trap`).

The current host enforces backend resource limits
([`DcWasmResourceLimits`](../src/formats/dcstring/converters/wasm.rs)) across the
session store. These are not yet a runtime-independent language resource model:
1. **Instruction fuel** (`max_instructions_per_step`, default `1_000_000` instructions): metered inside the `tinywasm` instruction loop during module instantiation, the WebAssembly `start` section (`run_wasm_start_section`), and every exported `step()` call (`Trap::InstructionFuelExhausted` -> `WasmResourceInterruption::InstructionFuelExhausted`).
2. **Store-wide and per-memory linear memory page cap** (`max_memory_pages`, default `256` pages = `16 MiB` total across all `MemoryInstance`s in `Store`): enforced during module instantiation, `provide_input`, and WASM `memory.grow` (`Store::total_memory_pages()`). Guest `memory.grow` failure returns standard WASM `-1`; host allocation or cap-lowering violations raise `WasmResourceInterruption::MemoryLimitExceeded`.
3. **Store-wide and per-table element cap** (`max_table_elements`, default `65_536` elements total across all `TableInstance`s in `Store`): enforced during module instantiation, `table.grow`, and before each `step()` (`WasmResourceInterruption::TableLimitExceeded`; guest `table.grow` failure returns standard WASM `-1`).
4. **Single-step output slice cap** (`max_step_output_bytes`, default `4 MiB`): checked before allocating/copying `out_len` bytes from linear memory (`WasmResourceInterruption::StepOutputSliceTooLarge`).
5. **Cumulative session output cap** (`max_session_output_bytes`, default `16 MiB`): accumulated across all `YieldFrame`, `RequestCapability`, and `Complete` outputs in a session (`WasmResourceInterruption::SessionOutputBudgetExceeded`).

**Trace Preservation & Deterministic Replay**:
Live host capabilities default to denial unless explicitly granted; parse errors steer requests toward mocks. Historical graph pins cannot grant host permissions. `ExecutionContext::execute_wasm_session_with_trace` and `ExecutionContext::execute_wasm_bytes_with_trace` preserve all accumulated `frames` and `events` in the returned [`WasmExecutionTrace`](../src/runtime/runtime.rs) even when a resource interruption is denied (`PermissionDecision::Deny`, queryable via `WasmExecutionTrace::denied_resource_interruption` / `ensure_not_denied`) or occurs during module construction/start (`after_logical_steps == 0`), and `replay_wasm_session_trace` / `replay_wasm_bytes_trace` deterministically replay those finite prefixes.

**Remaining Gaps / Pending Work Before Freezing**:
- Fuel meters executed VM instructions, not initial module parsing/validation or per-byte cost inside a single bulk memory/table instruction (`memory.copy`, `memory.fill`, `table.copy`).
- Non-resource traps during module `start` or `step()` return `Err` rather than a structured trap event in `WasmExecutionTrace`.
- Gates 3 and 4 of [the language plan](../issues/language.md) (eliminating remaining procedural syntax rule synthesis and completing self-hosted `.WASM` definitions) must land before `BootstrapKernelVersion::V1` is frozen.

### 4.5 Portable Resource Semantics Versus Host Watchdogs

`max_instructions_per_step` currently counts `tinywasm` internal interpreter
dispatches, including lowered control-flow instructions. It does not count a
specified sequence of original WASM instructions. Neither another interpreter's
fuel nor a JIT's fuel may be substituted as an equivalent semantic budget.
Recording an interruption as an external event permits replay of the same
finite prefix; it does not prove that fresh runs on different engines interrupt
at the same point. Timeouts, allocator exhaustion and physical watchdog fuel
are host safety events, not language-level resource measurements.

Memory/table growth is different: its result is directly observable by guest
code. WASM permits allocation failure to return `-1` even below a declared
maximum. Portable document semantics therefore need explicit logical quotas
and allocation accounting defined by the selected trusted supervisor revision
and its effective configuration, including the host mailbox allocation. A
physical allocation failure below the logical quota must terminate with a host
resource event, not silently become a different guest-visible growth result.
Output quotas likewise need specified charging and delivery order. Current
page/table limits and output checks provide local enforcement, not this proof.

The intended owner of logical budget policy is the privileged System document.
The workspace hardcodes its bootstrap entry document: exactly one short Dc,
with no graph pin. On workspace startup, that entry resolves to the latest
trusted System-block definition associated with the short Dc and runs it as
the top-level supervisor. The application-bundled System graph is the trusted
base; Wikidata and other additional data resources are not bootstrap authority.
Resolution must check trusted statement provenance, not merely whether an ID
falls numerically inside the System block.

The selected System document defines resource limits, accounting and exhaustion
policy, including how user configuration is obtained and applied. These policy
decisions belong in that document, not in a second hardcoded host policy. The
host supplies the minimal bootstrap/execution primitives, isolation and external
capability mediation needed to run it and protect its state. This fixed
bootstrap choice grants the selected trusted document its supervisor role;
an arbitrary document's pin, declaration, or reference to the same short Dc
does not make that document privileged or let it replace the running supervisor.

Startup deliberately follows the latest trusted definition rather than an
older pinned revision. For reproducible replay, record the trusted supervisor
revision that actually ran, its dependencies and effective user configuration,
together with external responses/events. That execution record does not add a
pin to the one-Dc startup document or let an ordinary document's historical pin
restore an older supervisor policy.

Running that supervisor beside arbitrary native WASM cannot meter or preempt
the child by itself. The child
must either run through a supervisor-owned interpreter, or undergo a validated
transformation that inserts non-bypassable, standard-WASM metering and growth
checks. Instrumentation must be derived from the pinned source module before
engine-specific lowering/optimization, with a versioned cost model specifying
charge-before-effect, branches/calls/start, bulk-operation costs, overflow,
resets, and interruption delivery points. The supervisor decides exhaustion
policy; a minimal host boundary protects supervisor state and mediates external
capabilities. Independent emergency host watchdogs protect the process, but
are not a separate source of logical document budget policy.

This portable supervisor/metering path is **not implemented**. Required proof
includes running the same pinned module, budget policy and ordered external
events under independent WASM engines, comparing growth results, interruptions,
requests and output bytes. Existing same-engine tests and replay against a
NOP-padded recompilation do not establish that property. The `tinywasm` fork is
currently a backend safety mechanism, not the canonical definition of resource
use.
