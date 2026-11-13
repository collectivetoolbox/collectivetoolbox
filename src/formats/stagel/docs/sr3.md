# StageR 3 Specification (`f657` `.sr3` / `f660` `.sri3`) & EM3 Boundary

**Implementation Status**: `Partially Implemented`
- **Implemented**: Format registrations (`FormatId::StageR3` `f657` `.sr3`, `FormatId::StageR3Intermediate` `f660` `.sri3`, `FormatId::Em3` `f658` `.em3`, `FormatId::Emc` `f661` `.emc`), StageR 3 dialect support in the 32-character normalizer (`ctb_formats_stagel::stagel3::normalize_stager3`), normalized StageR 3 fixture (`src/formats/stagel/fixtures/sr3/`), and staged bootstrap loader rules (`stage_1_impls/*.sl3` -> `impls/*.sr3`).
- **Pending Work**: StageR 3 Token Dc repertoire allocation (requires operator approval before CSV assignment), bidirectional `.sr3 <-> StageR Token Dcs` tokenizer/decompiler, static type & exhaustiveness/cycle validator, nominal user-defined value types (`brand` / `type`, `struct`, `enum`) and shallow `match`, runtime data representations and heterogeneous StageL/WASM lowering (`int`, `rat`, `s`, `dcs`, `view`, `cursor`, `option/T`, `result/T`, structs, and payload enums), zero-copy window/extent host calls, and the Stage 1 compiler (`stage_1_impls/f657_*.sl3`) + Rust/JS backends.

---

## 1. Relationship to StageL 3 and EM3

StageR 3 builds strictly on top of [StageL 3 (`sl3.md`)](./sl3.md):
- **Unchanged core & module system**: Every valid StageL 3 type, literal, control-flow form, `ns` / `use` module rule, unified top-level `const` / `pub` declaration rule (including `: @dc l<id>` auto-assigned directive comments), local `let`/`set`/`const` rule, lexical capture rule, constant-stack tail-call guarantee, and core operator signature retains **identical** syntax and return types in StageR 3. StageR 3 never alters inherited StageL 3 signatures (for instance, `add` still returns `T` and faults on overflow; recoverable checked arithmetic uses distinct `tryAdd` verbs returning `result/T`).
- **Minimal nominal value types without generics or OOP**: StageR 3 adds monomorphic, immutable nominal value types—branded types/opaque newtypes (`brand` / `type`), product records (`struct`), and tagged unions (`enum`) with shallow exhaustive `match`—so domain invariants are enforced statically without bloating StageL 3's hand-written WAT compiler (`stage_0_impls/f656_*.wat`), while leaving user-defined generics, traits/methods, subtyping, and higher-order abstractions out of StageR 3.
- **No first-class functions in StageR 3**: Like StageL 3, named nested verbs are directly callable and capture shared lexical bindings, but verbs cannot be assigned, passed as arguments, or returned as values. First-class functions (`v/T[A,B]`, `/name` references, `call /fn ...`, and escaping closures) belong exclusively to **EM3 (`f658`)**.
- **Tokenized Dc Graph Storage & Transpilation Hierarchy**:
  - **Token Dcs in the System graph (no `.WASM` bloat)**: Only the Stage 0 StageL-Dc-to-WASM transpiler (`stage_0_impls/f656_*.wat`), `Dc 1936 "Dc document execution kernel"`, and StageL built-in primitives are stored as `.WASM` Dcs (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`) in the graph. Untokenized ASCII `.sr3` seed files in `data-src/` tokenize into **StageL / StageR Token Dcs** in the System block (losslessly reconstructable back to `.sr3` text as an alternate representation for AppleScript-style Dc-native editing).
  - **StageR 3 is implemented using StageL 3** (`stage_1_impls/f657_*.sl3`). StageR 3 Token Dcs lower via StageL 3 Token Dcs -> `.WASM` for sandboxed execution, and later transpile directly into **Rust**, **JavaScript**, and other target languages (enabling common graph Dcs to ship precompiled as native machine code in `ctoolbox`).
  - **EM3 (`f658`)** can always run by converting the needed StageR 3 functions and transpiling/compiling those, while also supporting higher-level construct refinements for cleaner target-language transpilation when semantics match; runtime execution errors never silently retry via a fallback implementation.
- **Single-Threaded Execution & Graph DRYness**:
  - StageR 3 (like StageL 3) has no intra-document multithreading; concurrency across isolated Dc document processes (with optional shared memory) and function-granularity Dc allocation for append-only graph DRYness and caching are detailed in [`issues/language.md`](../../../../issues/language.md).

---

## 2. Bare `"..."` Dcts Literals & `.sri3` Normalization

`FormatId::StageR3Intermediate` (`.sri3`, `f660`) uses the **exact same 32-character alphabet** (`A-Z`, `/`, `:`, `' '`, `'\n'`, `'"'`, `'.'`) and normalizer engine as `.sli3`, with `StageDialect::StageR3` enabling bare `"..."` string literals:

1. **Literal Kind Distinction**:
   - `t"..."` -> `t` (`letters`, ASCII case-folded, 32-character identifier sequence).
   - `b"..."` -> `l/u8` (raw byte list, case-preserving).
   - `"..."` -> `s` (`DcMst` mixed-mode document string, constructed from a **Dcts** `FormatId::Dcts` `f461` payload—**not** JSON string syntax).
2. **Two-Layer Framing Without Double-Unescaping**:
   - **Outer framing (`.`sr3 -> `.sri3` normalizer)**: Scans `"..."` using StageL slash escapes (`/"`, `//`, `/0`, `/a`, `/b`, `/t`, `/n`, `/v`, `/f`, `/r`, `/xHH`) plus raw UTF-8 characters, encoding non-`{A-Z, ' ', ':', '.'}` bytes losslessly as `/X<HI><LO>` (`A..P` nibbles).
   - **Inner Dcts parsing (compile-time / runtime `s` construction)**: Calling `decode_sli3_byte_literal_body` recovers the exact UTF-8 byte sequence, which is parsed **once** by Dcts (`@256@`, `@f656@`, `@@` for literal `@`, etc.) into `DcMstring`. Outer quotes `/"` and slashes `//` never interfere with `@...@` Dcts tokens and are never double-decoded.

---

## 3. StageR 3 Extended Types

*(Rust borrowing vs ownership types such as `DcMstr` vs `DcMstring` and `DcStr` vs `DcString` are backend implementation details, **not** separate StageR 3 language types.)*

| Type | Semantics & Invariants |
| :--- | :--- |
| `i128`, `u128` | Fixed-width signed and unsigned 128-bit integers (`i128:-42`, `u128:0`). Support checked `add`/`sub`/`mul`/`div`/`mod` (plus `truncRem` and `neg` on signed `i128`), `can*`, `try*`, bitwise ops, shifts (`COUNT < 128`), comparisons, and explicit conversions. |
| `int` | Arbitrary-precision signed integer (`int:0`, `int:-100000000000000000000`). Supports `add`/`sub`/`mul`/`div`/`mod`/`truncRem`/`neg`; arithmetic faults only on division/modulus/remainder by zero (memory exhaustion is a host/supervisor resource interruption, never wrapped or converted into `overflow`). |
| `rat` | Exact arbitrary-precision rational (`num: int`, `den: int`) maintained in canonical reduced form: `den > 0`, `gcd(|num|, den) == 1`, and zero represented uniquely as `0/1`. Constructed via `makeRat int/num int/den -> result/rat` (`err error:invalidDenominator` when `den == 0`). |
| `dc` | Strongly typed 128-bit Document Character (`DcChar`). Supports `eq`, `ne`, and explicit fallible/lossless conversions (`dcFromU128 -> option/dc`, `u128FromDc -> u128`), never implicit integer arithmetic. |
| `s` | Mixed-mode Document Character + binary string (`DcMst`, corresponding to Rust `DcMstr`/`DcMstring`). Can hold arbitrary Document Characters and encapsulated binary runs (`Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`). |
| `dcs` | Pure Document Character sequence (`DcStr`/`DcString`, containing no raw unencapsulated binary runs). Converting `s` to `dcs` is explicit and fallible (`dcsFromS s/x -> result/dcs`); widening `dcs` to `s` is infallible (`sFromDcs dcs/x -> s`). |
| `view` | Immutable checked extent descriptor `(source_id: u32, offset: u64, len: u64)` over a host/graph input stream or value node, satisfying `offset <= source_len` and `len <= source_len - offset`. |
| `cursor` | Mutable sequential/random-access reader `(view: view, pos: u64)` enforcing `0 <= pos <= view.len`. Failed reads leave `pos` unchanged. |
| `option/T` | Tagged optional value: `option/T:none` or `option/T:some VALUE`. |
| `result/T` | Tagged fallible result: `result/T:ok VALUE` or `result/T:err ERROR`. Ordinary values—constructing an `err` does **not** implicitly return from the enclosing verb. |
| `error` | Structured semantic error value using camelCase `.sri3`-compatible tags (`error:overflow`, `error:divisionByZero`, `error:invalidDenominator`, `error:invalidEncoding`, `error:outOfBounds`, etc.). |

---

## 4. Nominal User-Defined Types (`brand` / `type`, `struct`, `enum`) & Shallow `match`

StageR 3 supports monomorphic nominal user-defined value types so `.sr3` codecs, AST passes, and standard-library modules do not fall back to untyped `u64` IDs, magic integer tags, or positional lists, and so target backends can emit native target types.

### 4.1 Module Declarations, `use` Resolution, and `: @dc` / `pub dc/<shorthand>` Anchors
- **Top-level scope & visibility**: Nominal types (`brand`, `type`, `struct`, `enum`) are declared at module top level only—file-private by default, or exported with `pub` and imported explicitly via `use path/to/mod: typeName ::.` (`Dc 2195 "Begin use import"`).
- **Slash-segmented type references**: Nominal type names are single identifiers (`nodeId`, `point`, `token`) that slot directly into StageL/StageR `/` type paths (`point/p`, `l/point/pts`, `option/nodeId/id`, `v/point/origin` in `Dc 2203 "Begin routine declaration"`) and normalize into `.sri3` (`f660`) with zero changes to the 32-character normalizer.
- **Graph identity & ontology binding**:
  - Module-defined algorithmic types carry auto-assigned `: @dc l<id>` System-block directive comments (revised across edits via `Dc 591 "Supersedes"` while historical snapshots pinned via `Dc 641 "Global graph system block most recent node ID at time of document creation"` resolve the layout active at their pin).
  - `pub dc/<shorthand>` on a `brand`, `type`, `struct`, or `enum` binds the StageR nominal type directly to an existing Format Dc (`f<N>`) or Semantic `Dc 565 "Type"` node rather than creating a duplicate ontology entry.

### 4.2 Branded Types & Opaque Newtypes (`brand` vs. `type`)
- **Transparent brand (`[pub [dc/<shorthand>]] brand BASE_TYPE/name.`)**:
  - Declares a distinct nominal type backed by concrete value type `BASE_TYPE`.
  - Any module with `name` in scope may construct and extract values via fixed-arity `wrap EXPR name` (`BASE_TYPE -> name`, arity 2, mirroring `conv EXPR TARGET_TYPE` `Dc 2541 "Convert value operation"`) and `unwrap EXPR` (`name -> BASE_TYPE`, arity 1).
- **Module-opaque newtype (`[pub [dc/<shorthand>]] type BASE_TYPE/name.`)**:
  - Outside the defining `ns` module, `name` is an opaque nominal value type: callers may pass, store, return, and (if `BASE_TYPE` supports `eq`) compare `name` values, while **only** routines inside the defining module may apply `wrap` or `unwrap`, enforcing domain invariants via module verbs (mirroring built-in `dc` and `rat`).
- **Strict non-coercion**: Distinct brands over the same underlying type (`byteOffset` vs. `charIndex`) never unify; numeric, bitwise, and ordering operators are not implicitly inherited (unwrap explicitly or provide domain verbs); structural `eq` (`Dc 2535 "Equal comparison"`) and `ne` (`Dc 2536 "Not equal comparison"`) hold iff `BASE_TYPE` supports `eq`.

### 4.3 Immutable Structs (`struct`)
- **Declaration**: `[pub [dc/<shorthand>]] struct name` declares an immutable value record with one or more uniquely named typed fields `FIELD_TYPE/fieldName` (indented block or inline `:: ... .`, mirroring `Dc 2205 "Routine parameter"`).
- **Delimited construction (`name: field1 EXPR_1 field2 EXPR_2 ::`)**: Framed with `:` .. `::` (analogous to `l/T: ... ::`) so parenthesis-free prefix parsing never requires cross-module field-count lookahead. Every declared field must be supplied exactly once; field expressions evaluate left-to-right in written order.
- **Projection & functional update**:
  - `field EXPR fieldName` (fixed arity 2) projects `fieldName` from a `struct` expression.
  - `with EXPR fieldName NEW_EXPR` (fixed arity 3) evaluates `EXPR` then `NEW_EXPR` and returns a new `struct` value with `fieldName` replaced.
- **Type invariants**: Fields cannot have type `void` (`f1425`), `never` (`f1426`), or—on `pub` structs or any struct used inside `l/T`—local-only `buf/T` (`Dc 2577 "StageL buffer type"`). Unguarded zero-indirection value cycles (`struct A` directly containing `A`) are compile-time errors. Structural `eq` / `ne` is defined iff all field types support `eq`.

### 4.4 Enums (`enum`) & Shallow Exhaustive `match`
- **Declaration**: `[pub [dc/<shorthand>]] enum name` declares a tagged union of one or more uniquely named variants, each with `0` payload (`variantName`) or `1` concrete payload type (`PAYLOAD_TYPE/variantName`; multi-field variant payloads wrap a `struct`).
- **Construction**: Matches built-in `error`, `option/T`, and `result/T`—`name:variant` (arity 0) for unit variants and `name:variant EXPR` (arity 1) for payload variants. Acyclic `use` resolution exports enum variant arities (`0` vs. `1`) alongside routine arities before body parsing so prefix calls parse unambiguously without parentheses.
- **Shallow exhaustive `match` (`match EXPR ... ::`)**:
  - Serves uniformly as the canonical elimination form for `option/T`, `result/T`, `error`, and user-defined `enum`s.
  - Each arm matches a single top-level variant (`case variant BODY` or `case variant TYPE/binding BODY`, where `binding` obeys StageL's non-shadowing immutable local binding rule), with an optional final `else BODY` wildcard arm.
  - Compile-time exhaustiveness is required (all variants covered without duplicates, or a non-redundant trailing `else` arm).
  - Arm result types unify identically to `if` (`Dc 2448 "If conditional"`, with `never` `f1426` bottom unification), definite-assignment knowledge intersects across completing arms, and calls at the end of arm bodies of a tail-position `match` preserve mandatory constant-stack tail calls.
  - Structural `eq` / `ne` is defined on an `enum` iff every variant's payload type (if any) supports `eq`.

### 4.5 Intentional Limitations & Lowering Constraints
To keep StageR 3 minimal over StageL 3 (`f656`) and cleanly compilation-compatible across WASM, Rust, and JS:
1. **No user-defined generics (`myType/T`)**: Parametricity is restricted to built-in `l/T`, `buf/T`, `option/T`, and `result/T`.
2. **Immutable value semantics only**: Struct fields and enum payloads cannot be mutated in place (`setField` does not exist); mutate `let` bindings via `set` or construct updated values with `with`.
3. **No methods, traits/interfaces, or subtyping**: Operations are plain VSO verbs in `ns` namespaces (`Dc 646 "In context of"`), with explicit `wrap`/`unwrap` rather than implicit `Dc 587 "Is a subtype of"` coercions.
4. **Shallow 1-level `match` only**: Nested constructor patterns, inline struct destructuring inside `case`, and boolean arm guards are excluded so `stage_1_impls/f657_*.sl3` does not need a decision-tree pattern compiler.
5. **Recursive type cycle bounds**: Unguarded value cycles across `struct`s (or `brand`/`type` chains) are rejected at compile time. Recursion through `l/T` (`Dc 2576 "StageL list type"`) is always finite-sized; direct recursion across payload `enum` variants must be tracked by the validator so native Rust codegen inserts explicit heap indirection (`Box` / `Arc`).
6. **`StageR 3 -> StageL 3` heterogeneous representation**:
   - `brand` / `type` wrappers and fieldless `enum`s erase at compile time to their underlying StageL representation or `u8`/`u32` discriminant (`0` runtime overhead in StageL/WASM and JS; `#[repr(transparent)]` newtypes and native `enum`s in Rust).
   - Heterogeneous `struct`s and payload `enum`s mixing scalars and sequences (alongside built-in `option/T`, `result/T`, `rat`, and lowered EM3 closure records) cannot pack into homogeneous `l/u64` in StageL 3 without `O(N)` byte copying, because StageL 3 forbids `l/T <-> u64` casts (`Dc 2542 "Bitcast value operation"` is fixed-width numeric only), forbids mutable module globals, and restricts `buf/T` (`Dc 2577 "StageL buffer type"`) to uncaptured local bindings outside tail-call trampoline frames. `StageR 3 -> StageL 3` lowering therefore requires either **(a)** a minimal anonymous fixed-slot structural tuple/record primitive in StageL 3 (`stage_0_impls/f656_*.wat`, `O(1)` linear-memory bump allocation + constant-offset loads, keeping StageL 3 free of nominal types and field names) or **(b)** scalarization (SROA) of non-escaping locals paired with handle-indexed runtime storage. Preserving resolved semantic Token Dc constructs until target rule selection lets `StageR 3 -> Rust` and `StageR 3 -> JS` emit native `struct`, `enum`, and plain-record layouts directly without flattening.

---

## 5. StageR 3 Core Operations

### 5.1 Fallible Checked Arithmetic (`try*`)
- `tryAdd X Y`, `trySub X Y`, `tryMul X Y`, `tryDiv X Y`, `tryMod X Y` -> `(T, T) -> result/T` (and `tryTruncRem X Y` -> `(T, T) -> result/T` on signed `i32`, `i64`, `i128`, `int`):
  - Fixed-width overflow or `MIN / -1` in `tryDiv` returns `result/T:err error:overflow` (`tryMod` and `tryTruncRem` on `(MIN, -1)` return `result/T:ok 0`).
  - Zero divisor returns `result/T:err error:divisionByZero`.
  - Operands evaluate once left-to-right; bounds are checked without executing a trapping core instruction first.

### 5.2 `option/T` and `result/T` Convenience Helpers
- Alongside shallow exhaustive `match` (Section 4.4), StageR 3 provides concise predicates and extractors:
  - `isSome OPT` (`option/T -> b`), `isNone OPT` (`option/T -> b`), `unwrapOr OPT DEFAULT` (`(option/T, T) -> T`), `unwrapSome OPT` (`option/T -> T`, faults on `none`; distinct from brand `unwrap`).
  - `isOk RES` (`result/T -> b`), `isErr RES` (`result/T -> b`), `unwrapOk RES` (`result/T -> T`, faults on `err`), `unwrapErr RES` (`result/T -> error`, faults on `ok`).

### 5.3 `s` and `dcs` String Operations
Because `s` (`DcMst`) can contain both encoded Document Characters and encapsulated binary runs, StageR 3 distinguishes encoded byte length from character length instead of overloading ambiguous `len s/x`:
- `byteLen S` -> `s -> u64` (encoded byte length of `DcMst` payload; `eq u64:0 byteLen s/x` tests emptiness in `O(1)`).
- `len DCS` -> `dcs -> u64` (exact Document Character count in `dcs`), `get DCS IDX` -> `(dcs, u64) -> dc`, `slice DCS START END` -> `(dcs, u64, u64) -> dcs`.
- `concat A B` -> `(s, s) -> s` and `(dcs, dcs) -> dcs`.
- `explode SEP INPUT` and `implode SEP PARTS`: extended to `s` (`(s, s) -> l/s`, `(s, l/s) -> s`) and `dcs` (`(dcs, dcs) -> l/dcs`, `(dcs, l/dcs) -> dcs`) with the same non-empty-separator, left-to-right non-overlapping, empty-piece-retaining contract as StageL 3 (`implode sep (explode sep x) == x`). On `s`, separator matching respects DcTF character/binary-run boundaries so a separator never splits mid-codepoint or mid-encapsulation header.
- `unimplemented MESSAGE` -> `s -> never`: development-only stub that terminates with a defined `unimplemented` fault.

---

## 6. Streaming Large Files & Graph-Sliced Documents (`view` / `cursor` + Output Extents)

To process multi-gigabyte archives (e.g. `csc archive`) and large graph documents composed of deduplicated value-node slices within bounded WASM linear memory (16 MiB default logical cap):

1. **Sliding-Window Reads (`view` / `cursor`)**:
   - Host streams and graph slice trees present a logical 64-bit byte/token space `0..total_len`.
   - When the requested range `[pos .. pos + n)` lies inside the currently mapped linear-memory window, cursor reads (`peekU8`, `readU8`, `readU32Le`, `readDc`) execute directly via bounds-checked WASM `load` instructions with zero host transitions.
   - On a window miss or distant `seek`, the runtime requests a window page transition via the Dc-WASM capability/yield protocol, mapping the target 64 KiB window and resuming seamlessly.
2. **Zero-Copy Output Extents**:
   - Small newly computed headers or transformed tokens append directly to the output buffer (`emitBytes`, `emitDc`, `emitS`).
   - Unchanged large sub-ranges emit zero-copy extent references (`emitExtent view/slice`, corresponding to `Dc 376 "Extent byte offset"` and `Dc 377 "Extent byte length"`), allowing disk materialization (`copy_file_range`) and graph deduplication (reusing existing `DcShorthand::Long(node_id)` value nodes) without copying gigabytes through WASM linear memory.

---

## 7. EM3 Boundary Notes (`f658` `.em3` / `f661` `.emc`)

Features intentionally reserved for EM3 (above StageR 3):
- **First-class function values**: Type syntax `v/T[A,B]` (or `v/T[]` for 0-arg thunks), value reference `/name`, anonymous verb literals `v/T/ p1 p2 :: ...`, and invocation `call /fn ARGUMENTS`.
- **Escaping closures**: Heap-allocated capture environments that outlive the defining invocation (with cycle-safe arena/tracing lifetime management when closures form reference cycles).
- **Dual lexical profiles**: Speakable English-like VSO profile (`.em3`, including repetition backreferences `dipre`, `daun`, `dato`) and compact code profile (`.emc`) mapping to the same semantic Dc stream.
