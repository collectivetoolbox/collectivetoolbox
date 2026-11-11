# StageL 3 Specification (`f656` `.sl3` / `f659` `.sli3`)

**Implementation Status**: `Partially Implemented`
- **Implemented**: Format registrations (`FormatId::StageL3` `f656` `.sl3` and `FormatId::StageL3Intermediate` `f659` `.sli3`), literal-aware 32-character normalizer (`src/formats/stagel/stagel3.rs`), normalized fixture (`src/formats/stagel/fixtures/sl3/`), staged bootstrap loader support (`src/formats/dcdata/support/wat.rs`), and `tinywasm` constant-stack tail-call execution (`return_call` / `return_call_indirect`).
- **Pending Work**: StageL 3 parser, static type checker, definite-assignment checker, inline `test` runner, and the Stage 0 `.wat` compiler (`src/formats/dcdata/data-src/stage_0_impls/f656_*.wat`).

---

## 1. Role in the Language Hierarchy

The language stack is split into three distinct languages (separate from the bootstrap directory numbers):

1. **StageL 3 (`f656`, `.sl3`)**: Small, strictly typed, expression-oriented Polish-prefix core language designed to be compiled directly in hand-written WebAssembly Text (`stage_0_impls/f656_*.wat`) into `.WASM` Document Characters (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`). Supports named nested verbs with shared lexical capture and mandatory constant-stack tail calls, but **not** first-class function values.
2. **StageR 3 (`f657`, `.sr3`)**: Extends StageL 3 with 128-bit and arbitrary-precision numbers (`i128`, `u128`, `int`, `rat`), Document Character and string types (`dc`, `dcs`, `s`), zero-copy `view`/`cursor` handles, `option/T`, `result/T`, and bare `"..."` Dcts literals. See [`sr3.md`](./sr3.md).
3. **EM3 (`f658`, `.em3` / compact `f661`, `.emc`)**: Full VSO semantic and programming language built on StageR 3, adding speakable/compact profiles and **first-class functions** (anonymous verbs, passed callbacks, returned/escaping closures).

### Bootstrap Stage Directories (`src/formats/dcdata/data-src/`)
- **`stage_0_impls/` (`.wat` only)**: Hand-written WAT foundations, including `1936_dc_document_execution_kernel.wat` and the StageL 3 compiler (`f656_*.wat`).
- **`stage_1_impls/` (`.dcsh`, `.sl3`)**: StageR 3 compiler (`f657_*.sl3`) and core bootstrap routines written in StageL 3.
- **`impls/` (`.dcsh`, `.sl3`, `.sr3`)**: Higher-level Document Character and EM3 implementations.

---

## 2. Lexical Rules & 32-Character Intermediate Format (`.sli3`)

StageL 3 source (`.sl3`) normalizes into `FormatId::StageL3Intermediate` (`.sli3`, `f659`), whose entire byte repertoire consists of **32 ASCII characters**:

- Uppercase letters: `A`–`Z` (26)
- Punctuation & whitespace: `/` `: ` `' '` `'\n'` `'"'` `'.'` (6)

### Source-to-`.sli3` Normalization Rules (`ctb_formats_stagel::stagel3::normalize`)
1. **Line endings**: `\r\n` and standalone `\r` normalize to `\n` outside string literals. Raw `\r` or `\n` inside an unclosed string literal is a lexical error.
2. **Comments**: After optional leading spaces at the start of a line (including line 1, byte 0), a single `:` followed by `' '`, `\r`, `\n`, or EOF starts a comment through the end of the line.
   - Leading `::` is an inline-header / block-closer delimiter, **never** a comment.
   - Mid-line colons followed by spaces (such as list openers `l/u32: u32:1 ::`) are **not** comments.
   - Inside comments, `a-z` fold to `A-Z`, `;` folds to `.`, digits `0-9` encode as `N:A:`..`N:J:`, and any character outside `{A-Z, ' ', '.', '/', ':'}` (including `"`) escapes as `:CHAR:<shifted_dec>:` where decimal digits `'0'..'9'` of the byte value map to `'A'..'J'`. Consequently, `"` **never** appears inside a `.sli3` comment, allowing a Stage 0 WAT lexer to scan string quotes without comment ambiguity.
3. **Delimiters & Identifiers**:
   - Outside comments and literals, `;` normalizes to `.` (statement separator) and `}` normalizes to `::` (explicit block/list closer). Opening `{` and tabs `\t` are rejected outside comments/literals.
   - Identifiers are ASCII case-insensitive (`a-z` -> `A-Z`).
   - Decimal digits `0`–`9` outside literals encode as `N:A:` (`0`) through `N:J:` (`9`).
   - A leading minus sign immediately after `:` before a digit in a typed negative number (e.g. `i32:-2147483648`) encodes as `N:M:` (yielding `IN:D:N:C::N:M:...`), allowing minimum two's-complement literals without permitting hyphens inside identifiers.
4. **`letters` Literals (`t"..."` -> `T"..."`)**:
   - Must start at an expression boundary (`t"` or `T"`).
   - Lowercase `a-z` fold to `A-Z` (`eq t"a" t"A"` is `true`).
   - Permitted `/` escapes: `/"` (`"`), `//` (`/`), and `/n` -> `/N` (`\n`).
   - Digits, arbitrary byte escapes (`/xHH`, `/0`), and characters outside the 32-character `letters` repertoire are rejected.
5. **Byte Literals (`b"..."` -> `B"..."`)**:
   - Denotes a literal of type `l/u8`. Payload byte values are **case-preserving** (`b"a"` is `[0x61]`, distinct from `b"A"` `[0x41]`).
   - Source slash escapes: `/"` (`0x22`), `//` (`0x2F`), `/0` (`0x00`), `/a` (`0x07`), `/b` (`0x08`), `/t` (`0x09`), `/n` (`0x0A`), `/v` (`0x0B`), `/f` (`0x0C`), `/r` (`0x0D`), and `/xHH` (exactly two hex digits `0x00..=0xFF`, e.g. `/x1a`, `/x5c` for backslash).
   - In `.sli3`, `A-Z`, `' '`, `:`, and `.` remain verbatim; `/"`, `//`, `/A`, `/B`, `/T`, `/N`, `/V`, `/F`, `/R` represent their respective escapes; and all other bytes (including lowercase `a-z`, digits `0-9`, `/0`, and `/xHH`) normalize losslessly to `/X<HI><LO>`, where `<HI>` and `<LO>` map nibbles `0..=15` to `A..=P`.
   - Raw control bytes and raw non-ASCII bytes (`>= 0x80`) are rejected in `b"..."` (use `/xHH`).
   - Bare `"..."` literals are **rejected** in StageL 3 (they belong to StageR 3 Dcts `s` literals).

---

## 3. Type System

Typing is static and strict with **zero implicit coercions**:

| Type | Description | Literal Syntax |
| :--- | :--- | :--- |
| `b` | Boolean | `true`, `false` |
| `t` | `letters`: opaque sequence over the 32-char `.sli3` alphabet | `t"abc"` |
| `u8` | Unsigned 8-bit integer (`0..=255`) | `u8:0`, `u8:255` |
| `i32` | Signed 32-bit two's-complement integer | `i32:-2147483648`, `i32:42` |
| `u32` | Unsigned 32-bit integer | `u32:0` |
| `i64` | Signed 64-bit two's-complement integer | `i64:-1`, `i64:0` |
| `u64` | Unsigned 64-bit integer | `u64:0` |
| `l/T` | Immutable/value-semantics list of supported element type `T` (`b"..."` has type `l/u8`) | `l/u32: u32:1 u32:2 ::`, `b"abc"` |
| `void` | Unit type (single unit value returned by declarations, `set`, loops, and empty `do ::`) | `do ::` |
| `never` | Bottom / non-returning type (`break`, `continue`, `return`, `fault`) — compatible with any branch type in `if` | *(control transfer)* |

Named verbs (`v/RETURN/name ...`) are declarations, **not** value types. Assigning, passing, or returning a verb is prohibited in StageL 3 and StageR 3.

---

## 4. Declarations, Mutability, Scoping, and Definite Assignment

### Verb Declarations
```text
v/RETURN/name TYPE/param1 TYPE/param2 ...
    test optional test label
        in ARG1 ARG2
        out EXPECTED
    BODY_STATEMENT_1.
    FINAL_EXPRESSION.
```
- An inline single-statement body uses `::` after the parameter list:
  `v/b/nne i64/a i64/b :: ne i64/a i64/b.`
- Prefix calls have fixed arity. Verb signatures in a block are collected before parsing bodies so forward and mutually recursive calls work throughout the enclosing block. Overloading is prohibited.

### Variable & Constant Bindings (`let`, `set`, `const`)
1. **`let TYPE/name [EXPR]`** — Declares a new **mutable** variable in the current block and returns `void` (unit).
   - With `EXPR` (`let u32/x u32:0.`): declares and initializes `x`.
   - Without `EXPR` (`let u32/x.`): declares `x` in an uninitialized state; `x` **must** be definitely assigned via `set` on all control-flow paths before any read (including reads via nested verbs that capture `x`). Reading a potentially uninitialized `let` variable is a **compile-time error**.
   - Once declared with `let`, `name` **cannot** be re-declared in the same block or shadowed in nested scopes within that block; mutate it with `set`.
2. **`set TYPE/name EXPR`** — Mutates an already-declared `let` variable `name` of matching `TYPE` and returns `void` (unit). Using `set` on a `const` binding, parameter, `foreach` `this`, or undeclared identifier is a **compile-time error**.
3. **`const TYPE/name EXPR`** — Declares and initializes a new **immutable** constant in the current block and returns `void` (unit). Initializer `EXPR` is mandatory; re-declaration and shadowing are prohibited.

---

## 5. Expression-Oriented Control Flow & Tail Calls

Every normally completing construct is an expression returning a value:
- **Blocks (`do ... ::` and indented bodies)**: Evaluate statements left-to-right / top-to-bottom, discarding intermediate values and returning the final expression's value (`.` and `;` do **not** suppress the final value). An empty block `do ::` returns unit of type `void`.
- **`if COND THEN ELSE`**: Evaluates `COND` (`b`), then evaluates **only** the selected branch. Result types of `THEN` and `ELSE` must match, except `never` unifies with any type `T` to yield `T`.
- **`while COND BODY`**: Repeatedly evaluates `COND` (`b`) and `BODY` (`void`); returns `void` when `COND` is `false` or `break` executes.
- **`foreach LIST BODY`**: Evaluates `LIST` (`l/T`) once (iterating a snapshot) and evaluates `BODY` (`void`) for each element with read-only `T/this` bound. Nested `foreach` is prohibited unless extracted into a helper verb (preserving unambiguous `T/this`).
- **`break` / `continue`**: Type `never`; target the nearest enclosing loop within the current verb (cannot cross a verb boundary).
- **`return EXPR`**: Type `never`; immediately returns `EXPR` (which must match the enclosing verb's declared `RETURN` type; `return do ::` for `void`). A call in `return CALL` is in tail position.

### Mandatory Constant-Stack Tail Calls & Lexical Capture Lowering
- **Tail-position guarantee**: Any direct call whose result is returned directly by the enclosing verb—including at the end of a verb body, inside `return`, at the end of a tail-position `do ... ::` block, or in either branch of a tail-position `if`—**must not grow the WASM call stack**, including across mutual recursion (`Dc 665 "WASM return_call"`).
- **Lexical capture**: Named nested verbs capture shared bindings from their **definition site**, not their call site; mutations via `set` are visible to all readers/writers sharing that binding, and bindings never outlive the defining verb invocation in StageL 3 / StageR 3.
- **WAT lowering without leaking linear memory across tail calls**:
  - Uncaptured `const` and `let` locals live in native WASM locals.
  - Only bindings actually captured by a nested verb are placed in a small linear-memory **shadow frame** allocated on entry to the defining verb and popped before returning.
  - When verb `F` makes a tail call to `G`:
    - If `G` is **not** nested inside `F` (so `G` cannot reference `F`'s shadow frame), `F` pops its shadow frame **before** executing `return_call $G`, preserving both constant WASM call stack **and** constant shadow-stack memory!
    - If `F` calls a nested child `H` in tail position where `H` captures `F`'s frame, `F`'s shadow frame remains live as `H`'s lexical environment; if `H` tail-calls itself or siblings within `F`, no new shadow frame is allocated for `F`.

### Lowering EM3 First-Class Functions Without New `.WASM` Table Dcs
StageL 3 and `.WASM` (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`) intentionally omit user-authored WASM `table` and `elem` sections. EM3 first-class closures compile cleanly through StageR 3 / StageL 3 via **typed defunctionalization**:
1. An EM3 closure value of type `v/R[A, B]` lowers to a 2-field runtime record `{ fn_id: u32, env_ptr: u32 }` (with `env_ptr` pointing to a heap-allocated capture record).
2. For each distinct closure signature `(A, B) -> R` in the program, the compiler synthesizes one direct-call dispatcher function `__dispatch_R_A_B(fn_id, env_ptr, a, b)` using `Dc 661 "WASM br_table"` + `Dc 665 "WASM return_call"` to jump in `O(1)` time and `O(1)` stack space to the concrete lifted target function.

---

## 6. Core Built-in Operators

Operands evaluate strictly **left-to-right** (except short-circuiting `if`, `and`, `or`). User-defined verbs cannot overload names, whereas the built-in operators below are type-directed over fixed primitive families.

### 6.1 Checked Arithmetic (`u8`, `i32`, `u32`, `i64`, `u64`)
Invalid arithmetic (overflow, division/modulus by zero, or `MIN / -1`) terminates evaluation with a deterministic fault (**never** silent two's-complement wrapping):
- `add X Y`, `sub X Y`, `mul X Y`, `div X Y`, `mod X Y` -> `(T, T) -> T`
  - `div` truncates toward zero (matching WASM `div_u` / `div_s`, with `MIN / -1` faulting).
  - **`mod` semantics**: Identical to remainder on unsigned types (`u8`, `u32`, `u64`: `17 mod 5 == 2`). On signed types (`i32`, `i64`), **Euclidean modulus** (`0 <= (a mod b) < |b|` for `b != 0`; fault when `b == 0`) is used so `mod` always yields a non-negative residue (`-7 mod 3 == 2`), avoiding confusion with truncated C/WASM `rem_s` (`-1`). *(Note: StageR 3 also provides `tryDiv` and `tryMod` returning `result/T`.)*
- `neg X` -> `T -> T` (signed `i32`, `i64` only; faults on `i32::MIN` / `i64::MIN`).
- Pure pre-check predicates (`(T, T) -> b`, never faulting):
  `canAdd X Y`, `canSub X Y`, `canMul X Y`, `canDiv X Y` (`false` when `Y == 0` or signed `X == MIN && Y == -1`).

### 6.2 Bitwise & Shift Operations (`u8`, `i32`, `u32`, `i64`, `u64`)
- `bitAnd X Y`, `bitOr X Y`, `bitXor X Y` -> `(T, T) -> T`
- `bitNot X` -> `T -> T`
- `shl X COUNT`, `shr X COUNT` -> `(T, u32) -> T`
  - `COUNT` must satisfy `COUNT < bit_width(T)` (`8`, `32`, or `64`); out-of-range shift counts fault rather than silently masking `COUNT`.
  - `shr` performs logical zero-fill right shift on unsigned types (`u8`, `u32`, `u64`) and arithmetic sign-extending right shift on signed types (`i32`, `i64`).

### 6.3 Booleans, Equality, and Ordering
- `not P` (`b -> b`), lazy `and P Q` (`(b, b) -> b`, lowers to `if P Q false`), lazy `or P Q` (`(b, b) -> b`, lowers to `if P true Q`).
- `eq X Y`, `ne X Y` -> `(T, T) -> b` for `b`, `t`, `u8`, `i32`, `u32`, `i64`, `u64`, and `l/E` (structural element-by-element equality where `E` supports `eq`).
- `lt X Y`, `le X Y`, `gt X Y`, `ge X Y` -> `(T, T) -> b` for integer types `u8`, `i32`, `u32`, `i64`, `u64` (and lexicographic order on `t`).

### 6.4 Explicit Numeric Conversions
- Lossless widening: `u32FromU8`, `i32FromU8`, `u64FromU8`, `i64FromU8`, `u64FromU32`, `i64FromU32`, `i64FromI32`.
- Checked narrowing (faults if value is outside target range): `u8FromU32`, `u8FromI32`, `u8FromU64`, `u8FromI64`, `u32FromI32`, `i32FromU32`, `u32FromU64`, `u32FromI64`, `i32FromU64`, `i32FromI64`, `u64FromI64`, `i64FromU64`.
- Lossless bit-pattern reinterpretation (same width, never faults): `u32BitsFromI32`, `i32BitsFromU32`, `u64BitsFromI64`, `i64BitsFromU64`.

### 6.5 Lists (`l/T`), Byte Slices (`l/u8`), and `letters` (`t`)
- `len XS` -> `l/T -> u64` (for `l/T`, including `l/u8`) and `t -> u64` (character count in `letters`).
- `get XS IDX` -> `(l/T, u64) -> T` (0-based; faults when `IDX >= len XS`).
- `slice XS START END` -> `(Seq, u64, u64) -> Seq` for `Seq` in `{l/T, t}` (half-open `[START..END)`; faults unless `START <= END <= len XS`).
- `concat A B` -> `(Seq, Seq) -> Seq` for `Seq` in `{l/T, t}` (single canonical concatenation name—no separate `catB` alias).
- `explode SEP INPUT` -> `(Seq, Seq) -> l/Seq` and `implode SEP PARTS` -> `(Seq, l/Seq) -> Seq` for `Seq` in `{l/u8, t}` (and extended to `{s, dcs}` in StageR 3):
  - **Why generic `explode`/`implode` works cleanly**: Because `l/u8`, `t`, `s`, and `dcs` are mutually disjoint static types (and `l/l/u8`, `l/t`, `l/s`, `l/dcs` are likewise disjoint), built-in type dispatch determines the exact concrete signature from the first operand with zero type ambiguity and no user-verb overloading.
  - **Edge-case contract**:
    1. `SEP` must be non-empty (`len SEP > 0`); an empty separator faults deterministically.
    2. Matching scans **left-to-right without overlap**.
    3. Leading, trailing, and adjacent separator matches **retain empty pieces** (`explode t":" t":A:"` -> `l/t: t"" t"A" t"" ::`; `explode t":" t""` -> `l/t: t"" ::`), guaranteeing the round-trip identity `implode sep (explode sep x) == x` for every `x` and non-empty `sep` (with `implode b"" parts` / `implode t"" parts` joining without a separator).
    4. `implode` precomputes total output length and fills in `O(total length)` time rather than quadratic repeated `concat`.
- `assert COND` -> `b -> void` (returns unit on `true`; faults on `false`).

---

## 7. Entry-Point & Dc-WASM Linear-Memory Mailbox ABI

Every compiled StageL 3 `.WASM` program executes inside `DcWasmSession` (`src/formats/dcstring/converters/wasm.rs`) using the standard 32-byte mailbox header at linear-memory addresses `0..32`:

| Byte Offset | Type | Name | Description |
| :--- | :--- | :--- | :--- |
| `0..4` | `u32` LE | `MB_IN_PTR` | Byte offset of host-provided DcTF input payload (`>= 32`, placed above static data segments) |
| `4..8` | `u32` LE | `MB_IN_LEN` | Byte length of input payload |
| `8..12` | `u32` LE | `MB_OUT_PTR` | Byte offset of output DcTF slice (pre-initialized by host to `align8(MB_IN_PTR + MB_IN_LEN)`) |
| `12..16` | `u32` LE | `MB_OUT_LEN` | Byte length of output DcTF slice written by guest |
| `16..32` | `u128` LE | `MB_PIN_NODE_ID` | Active `Dc 641 "Global graph system block most recent node ID at time of document creation"` pin (`0` if unpinned) |

### Dynamic Heap Floor & Linear Memory Growth
- `.WASM` modules instantiate 1 initial 64 KiB page (grown automatically by `DcWasmSession::provide_input` to fit the input slice plus initial output reserve).
- Compiled StageL 3 programs **must never** hardcode a fixed heap start such as `4096`, which would collide with inputs larger than 4 KiB. Instead, on entry to `main` / `_start`, the runtime initializes its bump/arena pointer dynamically above both static data and the input buffer:
  ```wat
  ;; Dynamic 8-byte-aligned heap floor above [MB_IN_PTR .. MB_IN_PTR + MB_IN_LEN)
  (global.set $heap_ptr
    (i32.and
      (i32.add (i32.add (i32.load (i32.const 0)) (i32.load (i32.const 4))) (i32.const 7))
      (i32.const -8)))
  ```
- Allocations check remaining byte capacity against `(memory.size) * 65536` and invoke `(memory.grow ...)` on demand; if `memory.grow` returns `-1` (logical page cap reached), execution faults cleanly via `unreachable`.

### Entry Convention
- A StageL 3 program used as a staged Dc/format implementation defines `v/l/u8/main l/u8/input` (operating on raw DcTF/byte payload slices) or exports `_start` / `main () -> i32` returning Dc-WASM `StatusCode` (`0 = Complete`, `1 = YieldFrame`, `2 = RequestCapability`), writing the result slice descriptor to `MB_OUT_PTR` (`8..12`) and `MB_OUT_LEN` (`12..16`).
