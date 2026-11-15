# StageR 3 Specification (`f657` `.sr3` / `f660` `.sri3`) & EM3 Boundary

**Implementation Status**: `Partially Implemented`
- **Implemented**: Format registrations (`FormatId::StageR3` `f657` `.sr3`, `FormatId::StageR3Intermediate` `f660` `.sri3`, `FormatId::Em3` `f658` `.em3`, `FormatId::Emc` `f661` `.emc`), StageR 3 Token Dc repertoire (`Dc 2221 "Begin StageR module"`..`Dc 2222 "End StageR module"`, nominal decls/values, `match`, error propagation, and StageR built-ins/types in [`categories/el.csv`](../../dcdata/data-src/categories/el.csv)), StageR 3 dialect support in the 32-character normalizer (`ctb_formats_stagel::stagel3::normalize_stager3`) and public `.sr3` APIs, normalized StageR 3 fixture (`src/formats/stagel/fixtures/sr3/`), direct `.sr3` parser (`stager_parser.rs`), bidirectional `.sr3 <-> StageR Token Dcs` codec (`stager_tokens.rs`, `dc_tokens.rs`), static totality/exhaustiveness/definite-assignment/cycle validator (`stager_validator.rs`), bounded deterministic reference evaluator & inline `test` runner (`stager_eval.rs`), `: @dc l<id>` ledger auto-allocator + System-graph Token-Dc compiler (`stagel_dc_alloc.rs`), and `SeedGraphIndex` workspace/direct-`dc/<shorthand>` linker (`seed_linker.rs`).
- **Pending Work**: `StageR 3 -> StageL 3` / `.WASM` heterogeneous lowering, host-windowed `view` / `cursor` (`Dc 2677 "Built-in type identifier: Extent view"`, `Dc 2678 "Built-in type identifier: Stream cursor"`) and output-extent (`emitExtent`) streaming verb Dcs + host capability ABI, in-graph inline `test`-block execution, the Stage 1 self-hosted compiler (`stage_1_impls/f657_*.sl3`), and native Rust/JS backends.

---

## 1. Relationship to StageL 3 and EM3

StageR 3 builds on top of [StageL 3 (`sl3.md`)](./sl3.md) while enforcing **strict totality**:
- **Shared module, binding, and control-flow core**: StageL 3 types, literals, `nop` (`Dc 2455 "No-op"`), control-flow forms, `ns` / `use` module rules, unified top-level `const` / `pub` declarations (`: @dc l<id>` auto-assigned directive comments), single-definition constant names, optional types on value references (`/name`) and `set` (`set /name`), lexical capture rules, and mandatory constant-stack tail calls carry over directly to StageR 3.
- **Strictly total functions (`fault` and `*StageLInternal` prohibited)**: Every StageR 3 routine (and every verb declared in or imported into `impls/` or `.sr3`) is **strictly total**:
  - `fault` (`Dc 2454 "Fault trap"`), `unimplemented`, `unwrapSome` / `unwrapOk` / `unwrapErr`, and `*StageLInternal` library symbols (`modStageLInternal`, `assertStageLInternal`) are **compile-time errors** in StageR 3 (only hardware/quota exhaustion in the host supervisor can interrupt execution).
  - Fallible arithmetic, buffer updates, constructors, and narrowing/lossy conversions in StageR 3 (`add`, `sub`, `mul`, `div`, `mod`, `truncRem`, `neg`, `bufSet`, `makeRat`, `dcFromU128`, `dcsFromS`, and narrowing/sign-changing/`rat -> int` `conv`) return `result/T` directly, while indexed lookup and slicing (`get` and `slice`) return `option/...`—ensuring every failure path is surfaced in the static type system rather than trapping.
  - Domain invariants that make an invalid state unreachable are pushed into the static type system via module-opaque newtypes (`type BASE_TYPE/name.`, e.g. `type u64/nonZeroU64.` validated once on construction so division by `nonZeroU64` returns `u64` directly) or uninhabited 0-variant `enum` / `Dc f1426 "Never"` (`match /impossible ::`).
- **Minimal nominal value types without generics or OOP**: StageR 3 adds monomorphic, immutable nominal value types—branded types/opaque newtypes (`brand` / `type`), named-field `struct`s, and tagged unions (`enum`) with shallow exhaustive `match`—so domain invariants are enforced statically without bloating StageL 3's hand-written WAT compiler (`stage_0_impls/stagel3_to_wasm.wat`, which implements anonymous `n`-ary tuples and positional `struct`s), while leaving user-defined generics, traits/methods, subtyping, and higher-order abstractions out of StageR 3.
- **No first-class functions in StageR 3**: Like StageL 3, named nested verbs are directly callable and capture shared lexical bindings, but verbs cannot be assigned, passed as arguments, or returned as values. First-class functions (`v/T[A,B]`, `/name` references, `call /fn ...`, and escaping closures) belong exclusively to **EM3 (`f658`)**.
- **Tokenized Dc Graph Storage & Transpilation Hierarchy**:
  - **Token Dcs in the System graph (no `.WASM` bloat)**: Only foundational Stage 0 `.wat` routines (`Dc 1936 "Dc document execution kernel"`, `Dc 2608 "Convert Dc string to WASM"`, standalone `@converts(2192 1927)` converter routines such as `stage_0_impls/stagel3_to_wasm.wat`, and StageL built-in primitives) are stored as `.WASM` Dcs (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`) in the graph. Untokenized ASCII `.sr3` seed files in `data-src/` tokenize into **StageL / StageR Token Dcs** in the System block (losslessly reconstructable back to `.sr3` text as an alternate representation for AppleScript-style Dc-native editing).
  - **StageR 3 is implemented using StageL 3** (`stage_1_impls/f657_*.sl3`). StageR 3 Token Dcs lower via StageL 3 Token Dcs -> `.WASM` for sandboxed execution, and later transpile directly into **Rust**, **JavaScript**, and other target languages (enabling common graph Dcs to ship precompiled as native machine code in `ctoolbox`).
  - **EM3 (`f658`)** can always run by converting the needed StageR 3 functions and transpiling/compiling those, while also supporting higher-level construct refinements for cleaner target-language transpilation when semantics match; runtime execution errors never silently retry via a fallback implementation.
- **Single-Threaded Execution & Graph DRYness**:
  - StageR 3 (like StageL 3) has no intra-document multithreading; concurrency across isolated Dc document processes (with optional shared memory) and function-granularity Dc allocation for append-only graph DRYness and caching are detailed in [`issues/language.md`](../../../../issues/language.md).

---

## 2. Bare `"..."` Dcts Literals, `?` Propagation Alias, & `.sri3` Normalization

`FormatId::StageR3Intermediate` (`.sri3`, `f660`) uses the **exact same 32-character alphabet** (`A-Z`, `/`, `:`, `' '`, `'\n'`, `'"'`, `'.'`) and normalizer engine as `.sli3`, with `StageDialect::StageR3` enabling bare `"..."` string literals and the `? -> /` trailing error-propagation alias:

1. **Literal Kind Distinction**:
   - `dc:<shorthand>` -> `dc` (standalone Document Character scalar literal, e.g. `dc:256`, `dc:f657`, `dc:u41`).
   - `b"..."` -> `l/u8` (raw byte list, case-preserving).
   - `"..."` -> `s` (`DcMst` mixed-mode document string, constructed from a **Dcts** `FormatId::Dcts` `f461` payload—**not** JSON string syntax).
2. **Two-Layer Framing Without Double-Unescaping**:
   - **Outer framing (`.sr3 -> .sri3` normalizer)**: Scans `"..."` using StageL slash escapes (`/"`, `//`, `/0`, `/a`, `/b`, `/t`, `/n`, `/v`, `/f`, `/r`, `/xHH`) plus raw UTF-8 characters, encoding non-`{A-Z, ' ', ':', '.'}` bytes losslessly as `/X<HI><LO>` (`A..P` nibbles).
   - **Inner Dcts parsing (compile-time / runtime `s` construction)**: Calling `decode_sli3_byte_literal_body` recovers the exact UTF-8 byte sequence, which is parsed **once** by Dcts (`@256@`, `@f656@`, `@@` for literal `@`, etc.) into `DcMstring`. Outer quotes `/"` and slashes `//` never interfere with `@...@` Dcts tokens and are never double-decoded.
3. **Trailing `? -> /` Error-Propagation Alias**:
   - Outside comments and string literals, a trailing `?` attached immediately after an ASCII alphanumeric character at the end of a verb, value reference, or `do` block opener (`add? /x /y`, `get? /xs /idx`, `/optVar?`, `do? ... ::`) normalizes in `.sri3` to canonical `/` (`ADD/ /X /Y`, `GET/ /XS /IDX`, `/OPTVAR/`, `DO/ ... ::`), analogous to `; -> .` and `} -> ::`. Standalone `?` or mid-token `foo?bar` is rejected.

---

## 3. StageR 3 Extended Types

*(Rust borrowing vs ownership types such as `DcMstr` vs `DcMstring` and `DcStr` vs `DcString` are backend implementation details, **not** separate StageR 3 language types.)*

| Type | Semantics & Invariants |
| :--- | :--- |
| `i128`, `u128` | Fixed-width signed and unsigned 128-bit integers (`i128:-42`, `u128:0`). Support total checked `add`/`sub`/`mul`/`div`/`mod` -> `result/T` (plus `truncRem` and `neg` on signed `i128`), `can*`, `wrapAdd`/`wrapSub`/`wrapMul`, bitwise ops, shifts, comparisons, and explicit conversions. |
| `int` | Arbitrary-precision signed integer (`int:0`, `int:-100000000000000000000`). Infallible `add`/`sub`/`mul`/`neg` -> `int`; `div`/`mod`/`truncRem` -> `result/int` (`err error:divisionByZero` when `rhs == 0`; memory exhaustion is a host/supervisor resource interruption, never wrapped). |
| `rat` | Exact arbitrary-precision rational (`num: int`, `den: int`) maintained in canonical reduced form: `den > 0`, `gcd(|num|, den) == 1`, and zero represented uniquely as `0/1`. Constructed via `makeRat int/num int/den -> result/rat` (`Dc 2609 "Construct rational number"`, `err error:divisionByZero` when `den == 0`). |
| `dc` | Strongly typed 128-bit Document Character (`DcChar`). Supports standalone `dc:<shorthand>` literals (`dc:256`, `dc:f657`, `dc:u41`), `eq`, `ne`, and explicit fallible/lossless conversions (`dcFromU128 -> result/dc` `Dc 2610 "Convert u128 to Document Character"` returning `err error:invalidDc` on UTF-16 surrogates `0xD800..=0xDFFF`, `u128FromDc -> u128` `Dc 2611 "Convert Document Character to u128"`), never implicit integer arithmetic. |
| `s` | Mixed-mode Document Character + binary string (`DcMst`, corresponding to Rust `DcMstr`/`DcMstring`). Can hold arbitrary Document Characters and encapsulated binary runs (`Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`). |
| `dcs` | Pure Document Character sequence (`DcStr`/`DcString`, containing no raw unencapsulated binary runs). Converting `s` to `dcs` is explicit and fallible (`dcsFromS s/x -> result/dcs`); widening `dcs` to `s` is infallible (`sFromDcs dcs/x -> s`). |
| `view` | Immutable checked extent descriptor `(source_id: u32, offset: u64, len: u64)` over a host/graph input stream or value node, satisfying `offset <= source_len` and `len <= source_len - offset` (`Dc 2677 "Built-in type identifier: Extent view"`). |
| `cursor` | Mutable sequential/random-access reader `(view: view, pos: u64)` enforcing `0 <= pos <= view.len` (`Dc 2678 "Built-in type identifier: Stream cursor"`). Failed reads leave `pos` unchanged. |
| `option/T` | Tagged optional value: `option/T:none` or `option/T:some VALUE`. |
| `result/T` | Tagged fallible result: `result/T:ok VALUE` or `result/T:err ERROR`. Ordinary values—constructing an `err` does **not** implicitly return from the enclosing verb unless propagated with trailing `/` (`?`). |
| `error` | Structured semantic error value using the 8 canonical camelCase `.sri3`-compatible tags: `error:overflow`, `error:divisionByZero`, `error:outOfBounds`, `error:invalidEncoding`, `error:invalidDc`, `error:unexpectedNone`, `error:frozenBuffer`, and `error:parseFailure`. |

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
  - Any module with `name` in scope may construct and extract values via fixed-arity `wrap EXPR name` (`BASE_TYPE -> name`, arity 2, `Dc 2551 "Wrap nominal value"`, mirroring `conv EXPR TARGET_TYPE` `Dc 2562 "Convert value"`) and `unwrap EXPR` (`name -> BASE_TYPE`, arity 1, `Dc 2552 "Unwrap nominal value"`).
- **Module-opaque newtype (`[pub [dc/<shorthand>]] type BASE_TYPE/name.`)**:
  - Outside the defining `ns` module, `name` is an opaque nominal value type: callers may pass, store, return, and (if `BASE_TYPE` supports `eq`) compare `name` values, while **only** routines inside the defining module may apply `wrap` or `unwrap`, enforcing domain invariants via module verbs (mirroring built-in `dc` and `rat`).
- **Strict non-coercion**: Distinct brands over the same underlying type (`byteOffset` vs. `charIndex`) never unify; numeric, bitwise, and ordering operators are not implicitly inherited (unwrap explicitly or provide domain verbs); structural `eq` (`Dc 2535 "Equal comparison"`) and `ne` (`Dc 2536 "Not equal comparison"`) hold iff `BASE_TYPE` supports `eq`.

### 4.3 Immutable Named-Field Structs (`struct`)
- **Declaration**: `[pub [dc/<shorthand>]] struct name` declares an immutable named-field `struct` with one or more uniquely named typed fields `FIELD_TYPE/fieldName` (indented block or inline `:: ... .`, mirroring `Dc 2205 "Routine parameter"`; tokenized via `Dc 2224 "Begin named struct declaration"`..`Dc 2225 "End named struct declaration"` and `Dc 2226 "Named struct field declaration"`, distinct from StageL 3 positional `struct`s `Dc 2217 "Begin positional struct declaration"`..`Dc 2218 "End positional struct declaration"`).
- **Delimited construction (`structNew name: field1 EXPR_1 field2 EXPR_2 ::`)**: Tokenized via `Dc 2227 "Begin named struct value"`..`Dc 2228 "End named struct value"` and `Dc 2229 "Named struct field initializer"`, and framed with `:` .. `::` (analogous to `l/T: ... ::`) so parenthesis-free prefix parsing never requires cross-module field-count lookahead. Every declared field must be supplied exactly once; field expressions evaluate left-to-right in written order.
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
   - Heterogeneous named-field `struct`s and payload `enum`s mixing scalars and sequences (alongside built-in `option/T`, `result/T`, `rat`, and lowered EM3 closure structs) lower directly onto StageL 3's anonymous `n`-ary tuples (`Dc 2672 "Built-in type identifier: Tuple"`, `Dc 2215 "Begin n-ary tuple list (ordered, contiguous, typed slots, length n)"`..`Dc 2216 "End n-ary tuple list"`) and nominal positional `struct`s (`Dc 2673 "Built-in type identifier: Struct"`, `Dc 2217 "Begin positional struct declaration"`..`Dc 2220 "End positional struct value"`, `Dc 2547 "Get positional field"`, `Dc 2548 "With positional field"`), using `O(1)` linear-memory bump allocation and constant-offset loads/updates while keeping StageL 3 free of named fields. Preserving resolved semantic Token Dc constructs until target rule selection lets `StageR 3 -> Rust` and `StageR 3 -> JS` emit native `struct` and `enum` layouts directly without flattening.

---

## 5. StageR 3 Core Operations

### 5.1 Trailing `/` Error-Propagation Operator (`?` Source Alias)
- Attaching a trailing `/` (or its non-canonical `.sr3` source alias `?`, which normalizes to `/` in `.sri3`, tokenized as `Dc 2459 "Propagate error or absence operator"`) to a head verb, value reference, or `do` block opener (`add/ /x /y`, `get? /xs /idx`, `myVerb/ /a /b`, `/optVar/`, `do/ ... ::`) evaluates the fallible sub-expression and either unwraps its success payload or early-returns from the enclosing routine:
  - **In a routine returning `option/R`**:
    - `OPT/` (`option/T -> T`): unwraps `some VALUE` to `VALUE`, or early-returns `option/R:none` on `none`.
    - Applying `/` to `result/T` in an `option/R`-returning routine is a **compile-time error** (error information is never silently discarded).
  - **In a routine returning `result/R`**:
    - `RES/` (`result/T -> T`): unwraps `ok VALUE` to `VALUE`, or early-returns `result/R:err ERROR` on `err ERROR`.
    - `OPT/` (`option/T -> T`): unwraps `some VALUE` to `VALUE`, or early-returns `result/R:err error:unexpectedNone` on `none`.
  - Applying `/` inside a routine whose return type is neither `option/R` nor `result/R` (or on an expression whose result type is neither `option/T` nor `result/T`) is a **compile-time error**.

### 5.2 Total Checked Arithmetic, Conversions, & Sequence Indexing
Because StageR 3 routines are strictly total (`fault` `Dc 2454 "Fault trap"` and `*StageLInternal` symbols are prohibited), operations with invalid inputs or overflow return `result/T` or `option/T` directly:
- `add X Y`, `sub X Y`, `mul X Y`, `div X Y`, `mod X Y` -> `(T, T) -> result/T` on fixed-width integers (plus `truncRem X Y` and `neg X` -> `result/T` on signed fixed-width integers `i32`, `i64`, `i128`; on arbitrary-precision `int`, `add`/`sub`/`mul`/`neg` are infallible `-> int` while `div`/`mod`/`truncRem` return `result/int`):
  - Fixed-width overflow (including `neg MIN` and `MIN / -1` in `div`) returns `result/T:err error:overflow` (`mod` and `truncRem` on `(MIN, -1)` return `result/T:ok 0`).
  - Zero divisor returns `result/T:err error:divisionByZero`.
  - Operands evaluate once left-to-right; bounds are checked without executing a trapping core instruction first (`wrapAdd`, `wrapSub`, `wrapMul`, and `canAdd`/`canSub`/`canMul`/`canDiv` remain infallible `-> T` / `-> b` when modular or pre-checked arithmetic is intended).
- `conv EXPR TARGET_TYPE` (`Dc 2562 "Convert value"`): exact value-preserving conversion across fixed-width integers, `int`, and `rat`. Infallible widening conversions return `TARGET_TYPE` directly; narrowing, sign-changing, or `rat -> int`/fixed-width integer conversions return `result/TARGET_TYPE` (`err error:overflow` when out of range or non-integral).
- `get SEQ IDX`: returns `option/T` (`option/dc` on `dcs`; `none` when `IDX >= len SEQ`, `some ELEM` otherwise) across `l/T`, `buf/T`, and `dcs`.
- `slice SEQ START END`: returns `option/SEQ_TYPE` (`none` when `START > END` or `END > len SEQ`) across `l/T` and `dcs`.
- `bufSet BUF IDX VAL`: returns `result/void` (`err error:outOfBounds` when `IDX >= len BUF`, `err error:frozenBuffer` if `bufFreeze` has already frozen `BUF`); `result/void` values construct via `result/void:ok nop` / `result/void:err ERROR` and match via unit `case ok BODY` + `case err error/e BODY`.

### 5.3 `option/T` and `result/T` Total Helpers
- Alongside shallow exhaustive `match` (Section 4.4) and trailing `/` propagation (Section 5.1), StageR 3 provides total predicates and fallback extractors:
  - `isSome OPT` (`option/T -> b`), `isNone OPT` (`option/T -> b`), `unwrapOr OPT DEFAULT` (`(option/T, T) -> T`), `okOr OPT ERR` (`(option/T, error) -> result/T`).
  - `isOk RES` (`result/T -> b`), `isErr RES` (`result/T -> b`), `unwrapOr RES DEFAULT` (`(result/T, T) -> T`).
  - Trapping extractors (`unwrapSome`, `unwrapOk`, `unwrapErr`) do **not** exist in StageR 3.

### 5.4 `s` and `dcs` String Operations
Because `s` (`DcMst`) can contain both encoded Document Characters and encapsulated binary runs, StageR 3 distinguishes encoded byte length from character length instead of overloading ambiguous `len s/x`:
- `byteLen S` -> `s -> u64` (encoded byte length of `DcMst` payload; `eq u64:0 byteLen /x` tests emptiness in `O(1)`).
- `len DCS` -> `dcs -> u64` (exact Document Character count in `dcs`), `get DCS IDX` -> `(dcs, u64) -> option/dc`, `slice DCS START END` -> `(dcs, u64, u64) -> option/dcs`.
- `concat A B` -> `(s, s) -> s` and `(dcs, dcs) -> dcs`.
- `explode SEP INPUT` and `implode SEP PARTS`: extended to `s` (`(s, s) -> l/s`, `(s, l/s) -> s`) and `dcs` (`(dcs, dcs) -> l/dcs`, `(dcs, l/dcs) -> dcs`) with the same total contract as StageL 3: when `SEP` is empty, `explode` splits `INPUT` into individual single-element slices (`[]` when `INPUT` is also empty); when `SEP` is non-empty, `explode` scans left-to-right without overlap and retains empty pieces so `implode /sep explode /sep /x == /x`. On `s`, both empty-separator splitting and non-empty separator matching respect DcTF character/binary-run boundaries so a split never cuts mid-codepoint or mid-encapsulation header.

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
