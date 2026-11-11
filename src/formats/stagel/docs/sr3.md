# StageR 3 Specification (`f657` `.sr3` / `f660` `.sri3`) & EM3 Boundary

**Implementation Status**: `Partially Implemented`
- **Implemented**: Format registrations (`FormatId::StageR3` `f657` `.sr3`, `FormatId::StageR3Intermediate` `f660` `.sri3`, `FormatId::Em3` `f658` `.em3`, `FormatId::Emc` `f661` `.emc`), StageR 3 dialect support in the 32-character normalizer (`ctb_formats_stagel::stagel3::normalize_stager3`), normalized StageR 3 fixture (`src/formats/stagel/fixtures/sr3/`), and staged bootstrap loader rules (`stage_1_impls/*.sl3` -> `impls/*.sr3`).
- **Pending Work**: StageR 3 parser, type checker, runtime data representations (`int`, `rat`, `s`, `dcs`, `view`, `cursor`, `option/T`, `result/T`), zero-copy window/extent host calls, and the Stage 1 compiler (`stage_1_impls/f657_*.sl3`) + Rust/JS backends.

---

## 1. Relationship to StageL 3 and EM3

StageR 3 builds strictly on top of [StageL 3 (`sl3.md`)](./sl3.md):
- **Unchanged core**: Every valid StageL 3 type, literal, control-flow form, `let`/`set`/`const` rule, lexical capture rule, constant-stack tail-call guarantee, and core operator signature retains **identical** syntax and return types in StageR 3. StageR 3 never alters inherited StageL 3 signatures (for instance, `add` still returns `T` and faults on overflow; recoverable checked arithmetic uses distinct `tryAdd` verbs returning `result/T`).
- **No first-class functions in StageR 3**: Like StageL 3, named nested verbs are directly callable and capture shared lexical bindings, but verbs cannot be assigned, passed as arguments, or returned as values. First-class functions (`v/T[A,B]`, `/name` references, `call /fn ...`, and escaping closures) belong exclusively to **EM3 (`f658`)**.
- **Compilation & Transpilation Hierarchy**:
  - Baseline native/web path: **EM3 -> StageR 3 -> Rust or JavaScript**.
  - Baseline graph/sandboxed WASM path: **EM3 -> StageR 3 -> StageL 3 -> `.WASM` (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`)**.
  - Construct-level target rules operate on resolved semantic Dc constructs (retaining types, lexical scopes, and effects) before falling back to lower-stage decompositions; backends never flatten prematurely and try to reverse-engineer high-level constructs afterward, and runtime execution errors never silently retry via a fallback implementation.

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
| `i128`, `u128` | Fixed-width signed and unsigned 128-bit integers (`i128:-42`, `u128:0`). Support checked `add`/`sub`/`mul`/`div`/`mod`, `can*`, `try*`, bitwise ops, shifts (`COUNT < 128`), comparisons, and explicit conversions. |
| `int` | Arbitrary-precision signed integer (`int:0`, `int:-100000000000000000000`). Arithmetic faults only on division/modulus by zero (memory exhaustion is a host/supervisor resource interruption, never wrapped or converted into `overflow`). |
| `rat` | Exact arbitrary-precision rational (`num: int`, `den: int`) maintained in canonical reduced form: `den > 0`, `gcd(|num|, den) == 1`, and zero represented uniquely as `0/1`. Constructed via `makeRat int/num int/den -> result/rat` (`err error:invalid_denominator` when `den == 0`). |
| `dc` | Strongly typed 128-bit Document Character (`DcChar`). Supports `eq`, `ne`, and explicit fallible/lossless conversions (`dcFromU128 -> option/dc`, `u128FromDc -> u128`), never implicit integer arithmetic. |
| `s` | Mixed-mode Document Character + binary string (`DcMst`, corresponding to Rust `DcMstr`/`DcMstring`). Can hold arbitrary Document Characters and encapsulated binary runs (`Dc 203 "Begin encapsulated binary data"`..`Dc 204 "End encapsulated binary data"`). |
| `dcs` | Pure Document Character sequence (`DcStr`/`DcString`, containing no raw unencapsulated binary runs). Converting `s` to `dcs` is explicit and fallible (`dcsFromS s/x -> result/dcs`); widening `dcs` to `s` is infallible (`sFromDcs dcs/x -> s`). |
| `view` | Immutable checked extent descriptor `(source_id: u32, offset: u64, len: u64)` over a host/graph input stream or value node, satisfying `offset <= source_len` and `len <= source_len - offset`. |
| `cursor` | Mutable sequential/random-access reader `(view: view, pos: u64)` enforcing `0 <= pos <= view.len`. Failed reads leave `pos` unchanged. |
| `option/T` | Tagged optional value: `option/T:none` or `option/T:some VALUE`. |
| `result/T` | Tagged fallible result: `result/T:ok VALUE` or `result/T:err ERROR`. Ordinary values—constructing an `err` does **not** implicitly return from the enclosing verb. |
| `error` | Structured semantic error value (`error:overflow`, `error:division_by_zero`, `error:invalid_denominator`, `error:invalid_encoding`, `error:out_of_bounds`, etc.). |

---

## 4. StageR 3 Core Operations

### 4.1 Fallible Checked Arithmetic (`try*`)
- `tryAdd X Y`, `trySub X Y`, `tryMul X Y`, `tryDiv X Y`, `tryMod X Y` -> `(T, T) -> result/T`:
  - Fixed-width overflow or `MIN / -1` returns `result/T:err error:overflow`.
  - Zero divisor returns `result/T:err error:division_by_zero`.
  - Operands evaluate once left-to-right; bounds are checked without executing a trapping core instruction first.

### 4.2 `option/T` and `result/T` Inspection
- Predicates & extractors (until general pattern-matching syntax is finalized):
  - `isSome OPT` (`option/T -> b`), `isNone OPT` (`option/T -> b`), `unwrapOr OPT DEFAULT` (`(option/T, T) -> T`), `unwrap OPT` (`option/T -> T`, faults on `none`).
  - `isOk RES` (`result/T -> b`), `isErr RES` (`result/T -> b`), `unwrapOk RES` (`result/T -> T`, faults on `err`), `unwrapErr RES` (`result/T -> error`, faults on `ok`).

### 4.3 `s` and `dcs` String Operations
Because `s` (`DcMst`) can contain both encoded Document Characters and encapsulated binary runs, StageR 3 distinguishes encoded byte length from character length instead of overloading ambiguous `len s/x`:
- `byteLen S` -> `s -> u64` (encoded byte length of `DcMst` payload; `eq u64:0 byteLen s/x` tests emptiness in `O(1)`).
- `len DCS` -> `dcs -> u64` (exact Document Character count in `dcs`), `get DCS IDX` -> `(dcs, u64) -> dc`, `slice DCS START END` -> `(dcs, u64, u64) -> dcs`.
- `concat A B` -> `(s, s) -> s` and `(dcs, dcs) -> dcs`.
- `explode SEP INPUT` and `implode SEP PARTS`: extended to `s` (`(s, s) -> l/s`, `(s, l/s) -> s`) and `dcs` (`(dcs, dcs) -> l/dcs`, `(dcs, l/dcs) -> dcs`) with the same non-empty-separator, left-to-right non-overlapping, empty-piece-retaining contract as StageL 3 (`implode sep (explode sep x) == x`). On `s`, separator matching respects DcTF character/binary-run boundaries so a separator never splits mid-codepoint or mid-encapsulation header.
- `unimplemented MESSAGE` -> `s -> never`: development-only stub that terminates with a defined `unimplemented` fault.

---

## 5. Streaming Large Files & Graph-Sliced Documents (`view` / `cursor` + Output Extents)

To process multi-gigabyte archives (e.g. `csc archive`) and large graph documents composed of deduplicated value-node slices within bounded WASM linear memory (16 MiB default logical cap):

1. **Sliding-Window Reads (`view` / `cursor`)**:
   - Host streams and graph slice trees present a logical 64-bit byte/token space `0..total_len`.
   - When the requested range `[pos .. pos + n)` lies inside the currently mapped linear-memory window, cursor reads (`peekU8`, `readU8`, `readU32Le`, `readDc`) execute directly via bounds-checked WASM `load` instructions with zero host transitions.
   - On a window miss or distant `seek`, the runtime requests a window page transition via the Dc-WASM capability/yield protocol, mapping the target 64 KiB window and resuming seamlessly.
2. **Zero-Copy Output Extents**:
   - Small newly computed headers or transformed tokens append directly to the output buffer (`emitBytes`, `emitDc`, `emitS`).
   - Unchanged large sub-ranges emit zero-copy extent references (`emitExtent view/slice`, corresponding to `Dc 376 "Extent byte offset"` and `Dc 377 "Extent byte length"`), allowing disk materialization (`copy_file_range`) and graph deduplication (reusing existing `DcShorthand::Long(node_id)` value nodes) without copying gigabytes through WASM linear memory.

---

## 6. EM3 Boundary Notes (`f658` `.em3` / `f661` `.emc`)

Features intentionally reserved for EM3 (above StageR 3):
- **First-class function values**: Type syntax `v/T[A,B]` (or `v/T[]` for 0-arg thunks), value reference `/name`, anonymous verb literals `v/T/ p1 p2 :: ...`, and invocation `call /fn ARGUMENTS`.
- **Escaping closures**: Heap-allocated capture environments that outlive the defining invocation (with cycle-safe arena/tracing lifetime management when closures form reference cycles).
- **Dual lexical profiles**: Speakable English-like VSO profile (`.em3`, including repetition backreferences `dipre`, `daun`, `dato`) and compact code profile (`.emc`) mapping to the same semantic Dc stream.
