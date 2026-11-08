# Minimal Trusted Bootstrap Kernel Specification (`BootstrapKernelVersion::V1`)

This document specifies the frozen minimal trusted bootstrap kernel (`BootstrapKernelVersion::V1` = `1`) implemented by the host in Rust (`ctb-formats-dctf`, `ctb-formats-dcdata`, `ctb-formats-dcstring`, and `ctb-runtime`).

## 0. Version Contract & Bootstrap Selection

The foundational execution contract is explicitly versioned via [`BootstrapKernelVersion`](../src/formats/dcstring/navigator/policy.rs) (`BootstrapKernelVersion::V1 = 1`, `BootstrapKernelVersion::CURRENT = Self::V1`) and is selected on [`DcProjectionPolicy`](../src/formats/dcstring/navigator/policy.rs) (`DcProjectionPolicy::with_kernel_version` / `DcProjectionPolicy::try_with_kernel_version_number`) or [`DcDocumentNavigator::select_bootstrap_kernel_version`](../src/formats/dcstring/navigator.rs) during bootstrap **before** any document or graph interpretation that depends on it. Requesting an unknown kernel version fails immediately at bootstrap before token or graph evaluation begins.

All higher-level language definitions, syntax rules (`Dc 594` `"Syntax rule"`), format expressions, and conversions live in the append-only System-block seed graph (`data/graph/85899_34592.ctbnbdl`, pinned per document via `Dc 641` `"Global graph system block most recent node ID at time of document creation"`). Those graph-defined rules and `.WASM` programs rely exclusively on the four host-implemented primitive layers specified below.

---

## 1. Physical Token Codecs (`DcTF`, `.dcmm` Header, and Binary Encapsulation)

### 1.1 Global Graph ID Space and `DcTF` (`f549`)
Every token in a Dc stream is a 128-bit unsigned Global Graph ID (`u128` / [`DcChar`](../src/formats/dcdata/dc_char.rs)):
- **Unicode region** (`0x0000_0000..=0x0010_FFFF`, `u<hex>`): standard Unicode scalar values and surrogate code points.
- **Short Dc region** (`0x0011_0000..=0x0021_FFFF` in `DcChar` / `DcTF`, `<decimal>`): compact Document Characters `Dc 1`..`Dc 1114111`, with `Dc 308` (`"Begin long Dc"`, `DC_LONG_DC`) reserved as the `DcTF` escape into the full 128-bit space.
- **Format region** (`4_294_967_296..=8_589_934_591`, `f<N>`): format identifiers (`FormatId`, where `f<N>` has Global ID `4_294_967_296 + N`).
- **System graph block** (`8_589_934_592..=1_099_511_627_775`, `l<id>`): immutable/append-only System-block graph nodes.

`DcTF` (`ctb_formats_dctf`, [`src/formats/dctf/codec.rs`](../src/formats/dctf/codec.rs)) encodes each `u128` token as follows:
1. Code points in `0..=0x0010_FFFF` encode directly as `UTF-8e-128` (1 to 4 bytes).
2. Short Dcs `1..=1_114_111` (excluding `Dc 308` `"Begin long Dc"`) encode as `UTF-8e-128(0x0011_0000 + short_id)`.
3. Any other 128-bit Global ID `gid` (including `Dc 308` `"Begin long Dc"` itself, Format IDs `f<N>`, and System-block node IDs `l<id>`) encodes as the prefix token `Dc 308` `"Begin long Dc"` (`UTF-8e-128(0x0011_0000 + 308)`) immediately followed by `UTF-8e-128(gid)`.

### 1.2 `.dcmm` (`DcMixed2`, `f643`) File Header
Every binary `.dcmm` document ([`src/formats/dcstring/dc_mixed.rs`](../src/formats/dcstring/dc_mixed.rs)) begins with a fixed 21-byte header (`DCMM_HEADER_LEN = 21`):
- Bytes `0..5`: ASCII magic `b"DcMM\0"` (`[0x44, 0x63, 0x4D, 0x4D, 0x00]`).
- Bytes `5..21`: 16-byte format UUID (`b0327a84-b745-492c-b39d-77c2827e3a6f` for `DcMixed2` `f643`; legacy `d5ab819e-9b8e-4d61-adbc-933e8851e79a` for `DcMixed1` `f642`).
- Bytes `21..`: `DcTF` token stream interspersed with binary encapsulation frames.

### 1.3 Binary Encapsulation (`Dc 203`..`Dc 204`)
Raw binary payloads inside `.dcmm` streams are framed by `Dc 203` (`"Begin encapsulated binary data"`) and `Dc 204` (`"End encapsulated binary data"`):
1. Opener token: `Dc 203` (`"Begin encapsulated binary data"`).
2. Binary header following `Dc 203` in `DcMixed2`:
   - 8-byte little-endian `u64` total byte length of `(metadata_slice || raw_payload_bytes)`.
   - `metadata_slice` (encoded in `DcTF`), containing:
     - Optional `Dc 543` (`"Begin checksummed section"`) .. `Dc 544` (`"End checksummed section"`) with `Dc 545` (`"Section digest"`) checksum pairs (`FormatId` algorithm token + digest encapsulation).
     - Optional `Dc 511` (`"Begin format representation"`) .. `Dc 512` (`"End format representation"`) format specifier.
     - `Dc 260` (`"Begin literal value"`): marks the end of the `DcTF` metadata header and the exact byte offset where raw binary payload bytes begin.
3. Raw payload bytes (exact length = `total_len - metadata_len`).
4. Closer token: `Dc 204` (`"End encapsulated binary data"`), immediately following the raw payload bytes.

In textual `DcString` / `DcText` streams, `Dc 203`..`Dc 204` encapsulates the payload using Base64 digits (`Dc 127`..`Dc 190`) with `Dc 195` (`"Base64 padding"`).

---

## 2. Active Triple Resolution in `SeedGraphIndex`

`SeedGraphIndex` ([`src/formats/dcdata/seed_graph.rs`](../src/formats/dcdata/seed_graph.rs)) loads `.ctbnbdl` (`CtbNodeBundle2`, `f645`, magic `b"CTBNBDL\x02"`) node bundles up to an optional watermark `most_recent_node_id` (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).

### 2.1 Snapshot Filtering (`Dc 641`)
When pinned to `most_recent_node_id`, only `LedgerNode`s with `node.id <= most_recent_node_id` are included in the snapshot. Any supersession or retraction statement with `stmt_id > most_recent_node_id` is excluded, restoring the exact active graph state as of `most_recent_node_id`.

### 2.2 Lifecycle Resolution (`Dc 591` `"Supersedes"` and `Dc 592` `"Retracts"`)
1. **Supersession** (`591 <new_id> <old_id>`): Marks `<old_id>` as inactive (`inactive_ids.insert(old_id)`) and records `superseded_by[old_id] = new_id`. `SeedGraphIndex::resolve_latest_node_id(id)` follows `superseded_by` edges deterministically (with cycle detection) to the terminal active node ID.
2. **Retraction** (`592 <stmt_id> <target_id>`): Marks both `<target_id>` and the retraction statement node itself as inactive (`inactive_ids.insert(target_id)`), and records `retracted_by[target_id] = stmt_id`. If `<target_id>` is a `Dc 584` `"Has a"` statement node, any active `Dc 610` `"Has value"` statement targeting `<target_id>` is also marked inactive.

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
| `0..4` | `DC_WASM_MAILBOX_INPUT_PTR` (`0`) | `u32` LE | Byte offset of the input DcTF payload slice in linear memory (`>= 64`, placed after any static data segments). |
| `4..8` | `DC_WASM_MAILBOX_INPUT_LEN` (`4`) | `u32` LE | Byte length of the input DcTF payload slice (with the 21-byte `.dcmm` header stripped). |
| `8..12` | `DC_WASM_MAILBOX_OUTPUT_PTR` (`8`) | `u32` LE | Byte offset of the output DcTF/DcMixed slice in linear memory (pre-initialized by the host before each `step()` to an 8-byte-aligned scratch region after the input buffer; may be updated by the guest). |
| `12..16` | `DC_WASM_MAILBOX_OUTPUT_LEN` (`12`) | `u32` LE | Byte length of the output slice written by the guest during `step()` (reset to `0` by the host before each `step()`). |
| `16..32` | `DC_WASM_MAILBOX_MOST_RECENT_NODE_ID` (`16`) | `u128` LE | Pinned `Dc 641` (`"Global graph system block most recent node ID at time of document creation"`) watermark (`0` when unpinned). |

### 4.4 `step()` Return Status Codes and VM Resource Budgets
Each invocation of `"step"` returns an `i32` [`WasmStepStatus`](../src/formats/dcstring/converters/wasm.rs):
- `0` (`WasmStepStatus::Complete`): Execution finished; the host reads `output_ptr..output_ptr + output_len` as the final output [`DcMstring`].
- `1` (`WasmStepStatus::YieldFrame`): Emitted an intermediate streaming frame in `output_ptr..output_ptr + output_len`; linear memory and globals remain intact for the next `step()` call.
- `2` (`WasmStepStatus::RequestCapability`): Execution paused requesting a host capability described by the Dc document in `output_ptr..output_ptr + output_len`. The host evaluates the capability request against the current host [`ExecutionContext`](../src/runtime/runtime.rs), writes any allowed or mocked response into the input mailbox via `provide_input`, and calls `step()` again.

The host enforces deterministic VM resource budgets ([`DcWasmResourceLimits`](../src/formats/dcstring/converters/wasm.rs)) on every session:
1. **Per-step instruction fuel** (`max_instructions_per_step`, default `1_000_000` instructions): metered inside the `tinywasm` instruction loop (`Trap::InstructionFuelExhausted` -> `WasmResourceInterruption::InstructionFuelExhausted`).
2. **Linear memory page cap** (`max_memory_pages`, default `256` pages = `16 MiB`): enforced during module instantiation, `provide_input`, and WASM `memory.grow` (`WasmResourceInterruption::MemoryLimitExceeded`).
3. **Single-step output slice cap** (`max_step_output_bytes`, default `4 MiB`): checked before allocating/copying `out_len` bytes from linear memory (`WasmResourceInterruption::StepOutputSliceTooLarge`).
4. **Cumulative session output cap** (`max_session_output_bytes`, default `16 MiB`): accumulated across all `YieldFrame`, `RequestCapability`, and `Complete` outputs in a session (`WasmResourceInterruption::SessionOutputBudgetExceeded`).

Physical VM resource interruptions are exposed as [`WasmResourceInterruption`](../src/formats/dcstring/converters/wasm.rs) and recorded as external resource events in the host execution trace (`Privilege::ExcessiveCompute`), keeping physical VM metering distinct from logical document progress.
