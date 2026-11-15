# StageL 3 Specification (`f656` `.sl3` / `f659` `.sli3`)

**Implementation Status**: `Partially Implemented`
- **Implemented**: Format registrations (`FormatId::StageL3` `f656` `.sl3`, `FormatId::StageL3Intermediate` `f659` `.sli3`, `FormatId::Boolean` `f1424`, `FormatId::Void` `f1425`, and `FormatId::Never` `f1426`), shared `.EL` and `.EL StageL` Token Dc repertoire (`Dc 2192 "Begin module"`..`Dc 2220 "End positional struct value"`, `Dc 2448 "If conditional"`..`Dc 2455 "No-op"`, `Dc 2512 "Add (faulting)"`..`Dc 2548 "With positional field"`, `Dc 2576 "StageL list type"`..`Dc 2585 "StageL buffer freeze"`, `Dc 2608 "Convert Dc string to WASM"`, `Dc 2672 "Built-in type identifier: Tuple"`, and `Dc 2673 "Built-in type identifier: Struct"`), `.Semantic` snapshot scoping (`Dc 1938 "Resolve to pin"`), literal-aware 32-character normalizer (`src/formats/stagel/stagel3.rs`), normalized fixture (`src/formats/stagel/fixtures/sl3/`), lossless `.sl3 <-> StageL Token Dcs` tokenizer/decompiler (`src/formats/dcdata/stagel3/dc_tokens.rs`), `: @dc l<id>` ledger auto-allocator & PSR-4 System-graph Token-Dc emitter (`src/formats/dcdata/support/stagel_dc_alloc.rs`), static type, definite-assignment, unguarded-struct-cycle, `*StageLInternal` stage-boundary gating, and fixed-arity parameter-type dispatch validator (`src/formats/dcdata/stagel3/validator.rs`), `SeedGraphIndex` snapshot Token-Dc linker & self-hosted converter bundler (`src/formats/dcdata/stagel3/seed_linker.rs`), reference evaluator (`src/formats/dcdata/stagel3/eval.rs`), Rust transpiler (`src/formats/stagel/stagel3/rust_codegen.rs`), host Dc-WASM reference compiler & inline `test` runner (`src/formats/stagel/stagel3/wasm_codegen.rs`), all 39 Stage 0 `.wat` built-in `.WASM` primitives, `Dc 2608 "Convert Dc string to WASM"`, and the self-hosted Stage 0 `.wat` StageL-Dc-to-WASM converter routine (`src/formats/dcdata/data-src/stage_0_impls/stagel3_to_wasm.wat`, `@dc l8589979107`, `@converts(2192 1927)`), Stage 1 standard library modules (`src/formats/dcdata/data-src/stage_1_impls/std/{assert,bytes,math}.sl3`), staged bootstrap loader & standalone `@converts(...)` converter-routine support (`src/formats/dcdata/support/wat.rs`), and `tinywasm` constant-stack tail-call execution (`return_call` / `return_call_indirect`).
- **Pending Work**: In-graph execution of inline routine `test` blocks (`Dc 2206 "Begin routine test block"`..`Dc 2207 "End routine test block"`, currently skipped during target `.WASM` emission by `stagel3_to_wasm.wat` and executed on the host by `run_dc_wasm_module_tests` in `src/formats/stagel/stagel3/wasm_codegen.rs`) and snapshot-keyed Rust AOT precompilation/caching (`issues/language.md`).

---

## 1. Role in the Language Hierarchy & Graph Storage Model

The language stack is split into three distinct languages (separate from the bootstrap directory numbers):

1. **StageL 3 (`f656`, `.sl3`)**: Small, strictly typed, expression-oriented Polish-prefix core language. Supports named nested verbs with shared lexical capture and mandatory constant-stack tail calls, but **not** first-class function values.
2. **StageR 3 (`f657`, `.sr3`)**: Implemented using StageL 3; extends StageL 3 with 128-bit and arbitrary-precision numbers (`i128`, `u128`, `int`, `rat`), Document Character and string types (`dc`, `dcs`, `s`), zero-copy `view`/`cursor` handles, `option/T`, `result/T`, monomorphic nominal value types (`brand` / `type`, `struct`, `enum`) with shallow exhaustive `match`, and bare `"..."` Dcts literals. See [`sr3.md`](./sr3.md).
3. **EM3 (`f658`, `.em3` / compact `f661`, `.emc`)**: Full VSO semantic and programming language built on StageR 3, adding speakable/compact profiles and **first-class functions** (anonymous verbs, passed callbacks, returned/escaping closures).

### Tokenized Dc Graph Representation vs. Seed ASCII Source (`.WASM` Restriction)
- **Strict `.WASM` boundary in the graph**: **Only** foundational Stage 0 `.wat` foundations (`Dc 1936 "Dc document execution kernel"`, `Dc 2608 "Convert Dc string to WASM"`, standalone `@converts(2192 1927)` converter routines such as `stagel3_to_wasm.wat`, and **StageL built-in primitives**) are emitted into the graph as `.WASM` Document Characters (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`, optionally paired with `Dc 397 "Equivalent representations"`). Because multiple compilers or implementations may convert StageL Token Dcs to `.WASM`, the compiler is modeled as a standalone Routine (`Dc 280 "Built-in type identifier: Routine"`) entity declaring `Dc 302 "Type conversion"` rather than attaching a `.WASM` decomposition directly to `Dc f656 "StageL 3"`; `Dc 1936 "Dc document execution kernel"` scans the input document for its leading `pin_description` (`Dc 641 "Global graph system block most recent node ID at time of document creation"`), evaluates `Dc 1938 "Resolve to pin"` over `Dc 2608 "Convert Dc string to WASM"` under the same compute/memory quota as execution, and then runs the resulting `.WASM` program.
- **StageL & StageR Token Dcs for everything else**: Anything that can be implemented using StageL 3 or StageR 3 is stored in the System block using **StageL / StageR Token Dcs** (dedicated semantic Dcs for keywords, types, and built-in operators such as `add`, `truncRem`, etc., plus direct Dc references to called functions/constants), **never** expanded into `.WASM` Dcs in `.ctbnodes`. This keeps *meanings* in the System block and dramatically reduces append-only graph size.
- **Seed ASCII files & AppleScript-style lossless reconstruction**: Untokenized ASCII `.sl3` and `.sr3` files under `src/formats/dcdata/data-src/` serve as **seed authoring sources** in the same manner as `categories/*.csv`; the generator parses them and emits their corresponding Token Dcs into the graph. Textual `.sl3` / `.sr3` source and the tokenized Dc stream are true alternate representations: textual source is losslessly reconstructable from the Dcs (preserving comments, local identifier names, and structure), and once a Dc-native editor is available, editing will operate directly on tokenized Dcs in the manner of Apple's AppleScript Script Editor.

### Bootstrap Stage Directories (`src/formats/dcdata/data-src/`)
- **`stage_0_impls/` (`.wat` only)**: Hand-written WAT foundations (`1936_dc_document_execution_kernel.wat`, `2608_convert_dc_string_to_wasm.wat`, the standalone `@converts(2192 1927)` StageL-Dc-to-WASM converter routine `stagel3_to_wasm.wat`, and non-trivial StageL built-in `.WASM` primitives).
- **`stage_1_impls/` (`.dcsh`, `.sl3`)**: StageR 3 compiler (`f657_*.sl3`) and core bootstrap libraries written in StageL 3 (emitted into the graph as StageL Token Dcs).
- **`impls/` (`.dcsh`, `.sl3`, `.sr3`)**: Higher-level Document Character, standard-library, UI, and EM3 implementations (emitted into the graph as StageL / StageR / EM3 Token Dcs).

### Execution, Concurrency, and Graph DRYness
- **Single-threaded language semantics**: StageL 3 and StageR 3 have no built-in multithreading; parallelism is handled at the host level by running separate Dc documents as isolated processes (optionally sharing memory/extents), while intra-document concurrency is deferred to future EM3 design.
- **Function-granularity Dcs**: Because the graph is append-only, reusable helpers should be allocated their own Dcs rather than duplicated across implementations, enabling both compact graph storage and per-Dc native Rust precompilation/caching (see [`issues/language.md`](../../../../issues/language.md)).

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
4. **Byte Literals (`b"..."` -> `B"..."`)**:
   - Denotes a literal of type `l/u8`. Payload byte values are **case-preserving** (`b"a"` is `[0x61]`, distinct from `b"A"` `[0x41]`).
   - Source slash escapes: `/"` (`0x22`), `//` (`0x2F`), `/0` (`0x00`), `/a` (`0x07`), `/b` (`0x08`), `/t` (`0x09`), `/n` (`0x0A`), `/v` (`0x0B`), `/f` (`0x0C`), `/r` (`0x0D`), and `/xHH` (exactly two hex digits `0x00..=0xFF`, e.g. `/x1a`, `/x5c` for backslash).
   - In `.sli3`, `A-Z`, `' '`, `:`, and `.` remain verbatim; `/"`, `//`, `/A`, `/B`, `/T`, `/N`, `/V`, `/F`, `/R` represent their respective escapes; and all other bytes (including lowercase `a-z`, digits `0-9`, `/0`, and `/xHH`) normalize losslessly to `/X<HI><LO>`, where `<HI>` and `<LO>` map nibbles `0..=15` to `A..=P`.
   - Raw control bytes and raw non-ASCII bytes (`>= 0x80`) are rejected in `b"..."` (use `/xHH`).
   - Bare `"..."` literals are **rejected** in StageL 3 (they belong to StageR 3 Dcts `s` literals).

---

## 3. Type System

Typing is static and strict with **zero implicit coercions**:

| Type | Token Dc / Format | Description | Literal / Constructor Syntax |
| :--- | :--- | :--- | :--- |
| `b` | `f1424` (`Boolean`) | Boolean | `true` (`Dc 517 "Boolean true"`), `false` (`Dc 518 "Boolean false"`) |
| `u8` | `f203` (`U8`) | Unsigned 8-bit integer (`0..=255`) | `u8:0`, `u8:255` |
| `i32` | `f208` (`I32`) | Signed 32-bit two's-complement integer | `i32:-2147483648`, `i32:42` |
| `u32` | `f207` (`U32`) | Unsigned 32-bit integer | `u32:0` |
| `i64` | `f210` (`I64`) | Signed 64-bit two's-complement integer | `i64:-1`, `i64:0` |
| `u64` | `f209` (`U64`) | Unsigned 64-bit integer | `u64:0` |
| `l/T` | `Dc 2576 "StageL list type"` | Immutable value-semantics list of element type `T` (`b"..."` has type `l/u8`) | `l/u32: u32:1 u32:2 ::`, `b"abc"` |
| `buf/T` | `Dc 2577 "StageL buffer type"` | Mutable local StageL builder buffer over element type `T` (`O(1)` amortized push/set, zero-copy `bufFreeze -> l/T`; local-only, disallowed in exported `pub` constants/signatures and `l/T` elements) | `bufNew u8 u64:64`, `bufFill u8:0 u64:16` |
| `tup: T_0 ... T_{n-1} ::` | `Dc 2672 "Built-in type identifier: Tuple"` | Anonymous immutable `n`-ary structural tuple (`n >= 0` ordered typed slots; distinct from binary `Dc 514 "Key-value pair (tuple)"`..`Dc 515 "End key-value pair"`, which remains fixed at arity 2) | `tupNew: ::`, `tupNew: u32:7 b"ok" ::` (`Dc 2215 "Begin n-ary tuple list (ordered, contiguous, typed slots, length n)"`..`Dc 2216 "End n-ary tuple list"`) |
| `NAME` / `dc/<shorthand>` | `Dc 2673 "Built-in type identifier: Struct"` | Nominal immutable positional `struct` declared at module top level (`[pub [dc/<sh>]] struct NAME :: T_0 ... T_{n-1}.`, `n >= 1` slots; `Dc 2217 "Begin positional struct declaration"`..`Dc 2218 "End positional struct declaration"`) | `structNew NAME: E_0 ... E_{n-1} ::` (`Dc 2219 "Begin positional struct value"`..`Dc 2220 "End positional struct value"`) |
| `void` | `f1425` (`Void`) | Unit type (single unit value returned by declarations, `set`, buffer mutators, loops, `nop`, and empty `do ::`) | `nop` (`Dc 2455 "No-op"`), `do ::` |
| `never` | `f1426` (`Never`) | Bottom / non-returning type (`break`, `continue`, `return`, `fault`) — compatible with any branch type in `if` | *(control transfer)* |

Named verbs (`v/RETURN/name ...`) are declarations, **not** value types. Assigning, passing, or returning a verb is prohibited in StageL 3 and StageR 3.

---

## 4. Namespaces, Declarations, Visibility (`const` / `pub`), and `: @dc` Allocation

### 4.1 File Organization: Standalone Dc Implementations vs. PSR-4 Namespaces (`ns` & `use`)
StageL 3 and StageR 3 support two file layouts under `stage_1_impls/` and `impls/`:
1. **Standalone implementation of an already-assigned Dc (`<dcsh>_<slug>.sl3` / `.sr3`)**:
   - Placed at the stage directory root with the target `DcShorthand` prefix in the filename (e.g. `f657_stager3_compiler.sl3`, `256_lang_say.sr3`), matching `stage_0_impls/1936_dc_document_execution_kernel.wat`.
2. **Multi-definition namespace modules (`ns` + PSR-4 directory path)**:
   - Placed in a subdirectory matching its declared `/` namespace path (`stage_1_impls/std/dctf/codec.sl3` -> `ns std/dctf/codec.`):
     ```text
     ns std/dctf/codec.
     use std/bytes/slice: startsWith endsWith ::.
     use std/ascii/char: isDigit as isAsciiDigit ::.
     ```
   - `use` imports explicitly named `pub` verbs and `pub` constant values from other namespaces (wildcard `*` imports are disallowed so pass-1 signature collection always knows every verb's fixed arity, and `use` dependencies must form an acyclic DAG).

### 4.2 Unified Top-Level `const` / `pub` Model & Auto-Assigned `: @dc` Annotations
At file top level, all declarations are immutable (`let` and `set` are local-block-only). In the tokenized AST (`Dc 2192 "Begin module"`..`Dc 2193 "End module"`), `let` (`Dc 2198 "Declare mutable binding"`), `const` (`Dc 2199 "Declare constant binding"`), and `pub` (`Dc 2200 "Declare public binding"`) are three peer binding-declarator verbs (`pub` never wraps `const`), while routines (`Dc 2203 "Begin routine declaration"`..`Dc 2204 "End routine declaration"`) carry a boolean export visibility flag (`Dc 517 "Boolean true"` for `pub v/...`, `Dc 518 "Boolean false"` for private `v/...`):
- **Bare top-level `v/RETURN/name ...` is an implicit private routine (`Dc 2203` with `false`)**: Declares a file-private verb (do **not** write redundant `const v/...`).
- **Top-level `[pub [dc/<shorthand>]] struct NAME :: TYPE_0 ... TYPE_{n-1}.` (`Dc 2217 "Begin positional struct declaration"`..`Dc 2218 "End positional struct declaration"`)**: Declares a module-private or exported positional `struct` with one or more non-`void`, non-`never` slot types. Unguarded zero-indirection value cycles across positional `struct`s and `tup: ... ::` slots are rejected at compile time, while recursion guarded behind `l/T` (`struct tree :: point l/tree.`) is valid. Transitive `buf/T` containment is tracked across tuples and positional `struct`s so `buf`-containing composites remain local-only (forbidden on `pub struct`, `pub`/`const` top-level values, `pub` routine signatures, captured variables, tail-call parameters, and `l/T` elements).
- **`const TYPE/name EXPR.` (`Dc 2199`)**: Declares a file-private constant value/data/list (evaluated as a pure constant expression).
- **`pub` is the exported declaration form (`Dc 2200` for values, `Dc 2203` with `true` for routines)**:
  - `pub v/RETURN/name ...` exports a verb from the file's `ns` module.
  - `pub TYPE/name EXPR.` exports a reusable constant value/data/list (referenced via `Dc 2202 "Typed value reference"` `/name` or `TYPE/name`, which never collides with 0-arg verb calls `Dc 2211 "Begin routine call"`..`Dc 2212 "End routine call"`).
  - `pub dc/<shorthand> v/...` or `pub dc/<shorthand> TYPE/name EXPR.` binds an exported namespace item as the implementation of an already-allocated Core Short Dc or Format Dc from `categories/*.csv`.

| Declaration Scope & Form | Visibility | Graph Node Allocation |
| :--- | :--- | :--- |
| **Nested `const v/RET/name ...`** (inside a verb or `do ... ::` block) | Enclosing block | **No separate Dc** (compiled inside enclosing verb's Dc; shares lexical frame) |
| **Top-level `v/...` or `const TYPE/name ...`** | File-private | **Dedicated System-region Dc (`l<id>`)** (`Dc 586 "Is an instance of"` `Dc 615 "Enclosing document"` `<type>` + `Dc 611 "Canonical decomposition"`) |
| **Top-level `pub v/...` or `pub TYPE/name ...`** | Exported from `ns` | **Dedicated System-region Dc (`l<id>`)** + `Dc 584 "Has a"` `Dc 574 "Programmatic identifier"` qualified by `Dc 646 "In context of"` namespace filepath |
| **Top-level `pub dc/<id> ...`** (or `<id>_*.sl3` root file) | Exported / Global Dc | **Attaches `Dc 611 "Canonical decomposition"` to pre-assigned Short/Format Dc `<id>`** |

#### Auto-Assigned `: @dc <shorthand>` Directive Comments
Just as leaving Column 1 (`Dc`) blank in `categories/*.csv` causes the generator to assign and write back the permanent ID, top-level verbs and constants in `.sl3` / `.sr3` seed files without an ID receive an auto-assigned System-block `l<id>` written onto a dedicated comment line immediately before the declaration:
```text
: @dc l8590012345
pub l/u8/dcmmMagic b"DcMM/0".

: @dc l8590012346
pub v/b/hasDcmmMagic l/u8/buf
    startsWith l/u8/buf l/u8/dcmmMagic.
```
- **Identity anchor without revision churn**: The generator reads `: @dc l<id>` to preserve the item's permanent graph identity across body edits (`Dc 591 "Supersedes"`), renames, or file reordering, **while stripping `: @dc ...` lines from the emitted Token Dc body and content-change hash** so inserting `: @dc l<id>` never triggers a second graph revision. When rendering Token Dcs back to ASCII source, `: @dc <id>` is reconstructed from the node header.

### 4.3 Verb Syntax & Local Bindings (`let`, `set`, `const`)
```text
v/RETURN/name TYPE/param1 TYPE/param2 ...
    test optional test label
        in ARG1 ARG2
        out EXPECTED
    BODY_STATEMENT_1.
    FINAL_EXPRESSION.
```
- An inline single-statement body uses `::` after the parameter list and still ends with `.` (or `;`), even when its body expression ends with a `::` / `}` closer:
  `v/b/nne i64/a i64/b :: ne i64/a i64/b.` / `v/u32/braceBlock u32/x :: do u32/x. }.`
- Prefix calls have fixed arity. Verb signatures in a block are collected before parsing bodies so forward and mutually recursive calls work throughout the enclosing block.
- **Fixed-arity exact-parameter-type implementation dispatch & single-definition constant names**:
  - Multiple top-level (`v/...` / `pub v/...`) or `use`-imported routines may share an identifier **iff** they have the **exact same parameter arity** and **pairwise-distinct concrete parameter type tuples** `(T_1, ..., T_n)`. Return-type-only overloading and arity overloading are prohibited, no identifier may denote both a constant and a routine, and a call with a `never` argument (`fault`, `return`, `break`, `continue`) must still select a **unique** candidate overload.
  - Constant and variable names have **at most one definition** in scope (constant-by-type overloading is disallowed; use descriptive camelCase names such as `i32Min` / `i64Min` so `/i32Min` is always unambiguous).
  - Nested lexical routines (`const v/...` inside a verb or `do ... ::` block) and local bindings (`let` / `const`) remain strictly non-overloaded and non-shadowing.
  - Each overloaded top-level routine receives its own System-block `l<id>` node, and calls/references inside System-graph bodies resolve statically at compile time to direct `dc/l<id>` references with zero runtime dispatch overhead.
- **Optional type prefix on value references (`/name` vs `TYPE/name`)**:
  - Value reads (`Dc 2202 "Typed value reference"`) may omit the type prefix (`/lhs`, `/dc/l8589978241`), inferring the type from the single declaration in scope, or include an explicit type annotation (`u32/lhs`, `i32/dc/l8589978241`), which the validator verifies against the declared type.
  - Explicit types remain mandatory on all new binding declarations (`let`, `const`, `pub`, routine parameters, and `foreach` element bindings).
- **`*StageLInternal` stage-boundary gating**:
  - Identifiers ending in `StageLInternal` (case-insensitively, e.g. `modStageLInternal`, `assertStageLInternal`) are Stage 1 bootstrap/StageL-only helpers permitted in `stage_1_impls/` and prohibited from being declared, `use`-imported (including with `as` or `dc/`), or called in Stage 2 `impls/` or `.sr3` files.
- **Namespace-scoped identifier validation (`StageL 3 ⊂ StageR 3`)**:
  - Any valid `[A-Za-z_][A-Za-z0-9_]*` identifier is permitted for `/`-prefixed bindings, named struct fields, enum variants, and `use`-imported items (so words such as `in`, `out`, `s`, `b`, `len`, `type`, or `error` remain usable in delimited contexts without a global reserved-word table).
  - Type declarations (`struct`, `brand`, `type`, `enum`) reject the built-in type keywords (`BUILTIN_TYPE_WORDS`, checked ASCII case-insensitively), and routine declarations (`v/RETURN/name`) reject the built-in head forms and core verbs (`BUILTIN_HEAD_WORDS`, checked ASCII case-insensitively). Both tables are unified across StageL 3 and StageR 3 so every valid StageL 3 user declaration remains valid in StageR 3 (preserving the narrow StageL 3 bootstrap exception for `std/bytes::{explode, implode}`).

Inside a verb or `do ... ::` block:
1. **`let TYPE/name [EXPR]`** — Declares a new **mutable** local variable in the current block and returns `void` (unit).
   - With `EXPR` (`let u32/x u32:0.`): declares and initializes `x`.
   - Without `EXPR` (`let u32/x.`): declares `x` in an uninitialized state; `x` **must** be definitely assigned via `set` on all control-flow paths before any read (including reads via nested verbs that capture `x`). Reading a potentially uninitialized `let` variable is a **compile-time error**.
   - Once declared with `let`, `name` **cannot** be re-declared in the same block or shadowed in nested scopes within that block; mutate it with `set`.
2. **`set [/name | TYPE/name] EXPR` (`Dc 2201 "Assign mutable binding"`)** — Mutates an already-declared `let` variable `name` and returns `void` (unit). `set` may **never** change an existing variable's declared type (if `TYPE/name` is written, `TYPE` must equal `name`'s declared type, and `EXPR` must match that type). Using `set` on a `const` / `pub` binding, parameter, `foreach` element binding, or undeclared identifier is a **compile-time error**.
3. **`const TYPE/name EXPR` / `const v/RETURN/name ...`** — Declares an **immutable** local constant value (`const TYPE/name EXPR`, mandatory initializer) or nested routine (`const v/RETURN/name ...`; bare `v/...` is top-level-only) in the current block and returns `void` (unit). Re-declaration, shadowing, and mutation via `set` are prohibited.

---

## 5. Expression-Oriented Control Flow & Tail Calls

Every normally completing construct is an expression returning a value:
- **Blocks (`do ... ::` and indented bodies, `Dc 2213 "Begin code block"`..`Dc 2214 "End code block"`)**: Evaluate statements left-to-right / top-to-bottom, discarding intermediate values and returning the final expression's value (`.` and `;` do **not** suppress the final value). An empty block `do ::` returns unit of type `void`.
- **`nop` (`Dc 2455 "No-op"`)**: Evaluates to unit of type `void` (`f1425`). In Token-Dc-to-`.sl3` decompilation, both `Dc 2455 "No-op"` and an empty `Dc 2213 "Begin code block"` `Dc 2214 "End code block"` pair render canonically as `nop` (while blocks with comments or statements remain `do ... ::`); parsing `nop` back to Token Dcs emits single `Dc 2455 "No-op"`.
- **`if COND THEN ELSE` (`Dc 2448 "If conditional"`)**: Evaluates `COND` (`b`), then evaluates **only** the selected branch. Result types of `THEN` and `ELSE` must match, except `never` unifies with any type `T` to yield `T`.
- **`while COND BODY` (`Dc 2449 "While loop"`)**: Repeatedly evaluates `COND` (`b`) and `BODY` (`void`); returns `void` when `COND` is `false` or `break` executes.
- **`foreach T/elem LIST BODY` (`Dc 2450 "For each"`)**: Evaluates `LIST` (`l/T`) once (iterating a snapshot) and evaluates `BODY` (`void`) for each element with read-only non-shadowing `T/elem` bound (`: ~ [stagel_type] [identifier] [el_expr] [el_expr]`), supporting arbitrary nested `foreach` loops. (Distinct from declarative graph-relationship `Dc 638 "Distribute statement over each element"` + `Dc 639 "This element"`.)
- **`break` (`Dc 2451 "Break loop"`) / `continue` (`Dc 2452 "Continue loop"`)**: Type `never`; target the nearest enclosing loop within the current verb (cannot cross a verb boundary).
- **`return [EXPR]` (`Dc 2453 "Return from routine"`)**: Type `never`; immediately returns `EXPR` (or `absent` `Dc 316 "Optional value absent"` for `void` routines). A call in `return CALL` is in tail position.
- **`fault` (`Dc 2454 "Fault trap"`)**: Type `never`; deterministically aborts execution with an unrecoverable trap.

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

## 6. Core Built-in Verbs vs. Standard Library Verbs

Operands evaluate strictly **left-to-right** (except short-circuiting `if`, `and`, `or`) in **VSO (Verb–Subject–Object)** prefix order. Core Token Dc built-in verbs below are type-directed over fixed primitive families (`categories/el.csv` across `.EL Builtins` and `.EL StageL`), while top-level and `use`-imported StageL 3 verbs support fixed-arity exact-parameter-type implementation dispatch across distinct concrete parameter tuples. Higher-level routines that can be written performantly in StageL 3 (`mod`, `can*`, `explode`, `implode`, `assert`) are Stage 1 library verbs (`stage_1_impls/std/`) rather than core Token Dcs.

### 6.1 Checked & Wrapping Arithmetic (`u8`, `i32`, `u32`, `i64`, `u64`)
Invalid checked arithmetic (overflow, division/remainder by zero, or signed `MIN / -1` and `MIN % -1`) terminates evaluation with a deterministic fault (**never** silent two's-complement wrapping):
- `add X Y` (`Dc 2512 "Add (faulting)"`), `sub X Y` (`Dc 2513 "Subtract (faulting)"`), `mul X Y` (`Dc 2514 "Multiply (faulting)"`), `div X Y` (`Dc 2515 "Divide (faulting)"`), `rem X Y` (`Dc 2516 "Truncating remainder (faulting)"`) -> `(T, T) -> T`
  - `div` and `rem` truncate toward zero across all integer types (`u8`, `i32`, `u32`, `i64`, `u64`), faulting when `Y == 0` or signed `X == MIN && Y == -1`.
- `neg X` (`Dc 2517 "Negate (faulting)"`) -> `T -> T` (signed `i32`, `i64` only; faults on `i32::MIN` / `i64::MIN`).
- Explicit modular two's-complement wrapping arithmetic (`2^width`, never faulting; needed for hashing, PRNGs, and multi-limb StageR `i128`/`u128`/`int` lowering):
  - `wrapAdd X Y` (`Dc 2518 "Wrapping add"`), `wrapSub X Y` (`Dc 2519 "Wrapping subtract"`), `wrapMul X Y` (`Dc 2520 "Wrapping multiply"`) -> `(T, T) -> T`.
- **Standard library arithmetic (`stage_1_impls/std/math.sl3`, not core Token Dcs)**:
  - StageL-internal Euclidean modulus (`modStageLInternal` across `u8`, `i32`, `u32`, `i64`, `u64`, yielding `0 <= (modStageLInternal lhs rhs) < |rhs|` for `rhs != 0`, including signed `modStageLInternal i32Min -1 == 0` and `modStageLInternal -1 i32Min == i32Max`; restricted to `stage_1_impls/`), single-definition camelCase integer bounds constants (`u8Min`, `u8Max`, `i32Min`, `i32Max`, `u32Min`, `u32Max`, `i64Min`, `i64Max`, `u64Min`, `u64Max`), and total pre-check predicates (`canAdd`, `canSub`, `canMul`, `canDiv`) are exported from a single unified `std/math` namespace.

### 6.2 Bitwise, Shift, Rotate, and Bit-Count Operations (`u8`, `i32`, `u32`, `i64`, `u64`)
- `bitAnd X Y` (`Dc 2521 "Bitwise AND"`), `bitOr X Y` (`Dc 2522 "Bitwise OR"`), `bitXor X Y` (`Dc 2523 "Bitwise XOR"`) -> `(T, T) -> T`
- `bitNot X` (`Dc 2524 "Bitwise NOT"`) -> `T -> T`
- `shl X COUNT` (`Dc 2525 "Shift left (faulting)"`), `shr X COUNT` (`Dc 2526 "Shift right (faulting)"`) -> `(T, u32) -> T`
  - `COUNT` must satisfy `COUNT < bit_width(T)` (`8`, `32`, or `64`); out-of-range shift counts fault rather than silently masking `COUNT`.
  - `shr` performs logical zero-fill right shift on unsigned types (`u8`, `u32`, `u64`) and arithmetic sign-extending right shift on signed types (`i32`, `i64`).
- `rotl X COUNT` (`Dc 2527 "Rotate left"`), `rotr X COUNT` (`Dc 2528 "Rotate right"`) -> `(T, u32) -> T` (unsigned `u8`, `u32`, `u64`).
- `clz X` (`Dc 2529 "Count leading zeros"`), `ctz X` (`Dc 2530 "Count trailing zeros"`), `popcnt X` (`Dc 2531 "Population count"`) -> `T -> u32` (single-instruction WASM bit-scan primitives, e.g. `O(1)` DcTF byte-length decoding).

### 6.3 Booleans, Equality, and Ordering
- `not P` (`Dc 2532 "Boolean NOT"`, `b -> b`), lazy `and P Q` (`Dc 2533 "Boolean AND"`, `(b, b) -> b`), lazy `or P Q` (`Dc 2534 "Boolean OR"`, `(b, b) -> b`).
- `eq X Y` (`Dc 2535 "Equal comparison"`), `ne X Y` (`Dc 2536 "Not equal comparison"`) -> `(T, T) -> b` for `b`, `u8`, `i32`, `u32`, `i64`, `u64`, `l/E`, `tup: T_0 ... ::`, and positional `struct`s (coinductive structural slot-by-slot / element-by-element equality whenever no constituent slot contains `buf/T`, including `l/T`-guarded recursive `struct`s).
- `lt X Y` (`Dc 2537 "Less than comparison"`), `le X Y` (`Dc 2538 "Less than or equal comparison"`), `gt X Y` (`Dc 2539 "Greater than comparison"`), `ge X Y` (`Dc 2540 "Greater than or equal comparison"`) -> `(T, T) -> b` for integer types `u8`, `i32`, `u32`, `i64`, `u64`.

### 6.4 Explicit Numeric Conversions (VSO Order: Value First, Target Type Second)
All 20 numeric widening/narrowing conversions and 4 same-width bit reinterpretations tokenize to **two** VSO built-in verbs (`: ~ [el_expr] [stagel_type]`):
- `conv X TARGET_TYPE` (`Dc 2541 "Convert value (faulting)"`): Lossless when widening (`u8 -> u32/i32/u64/i64`, `u32 -> u64/i64`, `i32 -> i64`); checked narrowing (faults when `X` is outside `TARGET_TYPE`'s range).
- `bitcast X TARGET_TYPE` (`Dc 2542 "Bitcast value"`): Lossless same-width bit-pattern reinterpretation (`i32 <-> u32`, `i64 <-> u64`; never faults).

### 6.5 Immutable Lists (`l/T`), Local Builder Buffers (`buf/T`), and Library Helpers
- **Immutable list construction & sequence operations**:
  - `l/T: ELEM_1 ... ::` (`Dc 2578 "Begin StageL list"`..`Dc 2579 "End StageL list"`) -> `l/T`
  - `len XS` (`Dc 2543 "Sequence length"`) -> `l/T | buf/T -> u64`
  - `get XS IDX` (`Dc 2544 "Get element (faulting)"`) -> `(l/T | buf/T, u64) -> T` (0-based; faults when `IDX >= len XS`)
  - `slice XS START END` (`Dc 2545 "Slice sequence (faulting)"`) -> `(l/T, u64, u64) -> l/T` (half-open `[START..END)`; faults unless `START <= END <= len XS`)
  - `concat A B` (`Dc 2546 "Concatenate sequences"`) -> `(l/T, l/T) -> l/T`
- **Mutable local StageL builder buffers (`buf/T`, enabling `O(N)` construction & `O(1)` lookup tables without hidden refcounting)**:
  - `bufNew T CAP` (`Dc 2580 "StageL buffer new"`) -> `buf/T` (empty buffer with `u64` initial capacity `CAP`)
  - `bufFill VAL LEN` (`Dc 2581 "StageL buffer fill"`) -> `buf/T` (buffer of length `LEN` initialized with `VAL`)
  - `bufPush BUF VAL` (`Dc 2582 "StageL buffer push"`) -> `void` (amortized `O(1)` append; faults if `BUF` is frozen)
  - `bufExtend BUF LIST` (`Dc 2583 "StageL buffer extend"`) -> `void` (bulk `memcpy` append of `l/T`; faults if `BUF` is frozen)
  - `bufSet BUF IDX VAL` (`Dc 2584 "StageL buffer set (faulting)"`) -> `void` (`O(1)` in-place element write at `IDX < len BUF`; faults if out of bounds or frozen)
  - `bufFreeze BUF` (`Dc 2585 "StageL buffer freeze"`) -> `l/T` (`O(1)` zero-copy conversion to immutable `l/T`, marking `BUF` frozen so later mutations fault)
- **Standard library sequence & assertion helpers (`stage_1_impls/std/{bytes,assert}.sl3`, not core Token Dcs)**:
  - `explode` and `implode` in `stage_1_impls/std/bytes.sl3` are total linear-time byte-slice splitting and joining verbs built over `slice`, `bufNew`, `bufPush`, `bufExtend`, and `bufFreeze`: when `sep` is non-empty, `explode` retains leading, trailing, and adjacent empty slices (`implode sep (explode sep x) == x`); when `sep` is empty (`b""`), `explode` splits `input` into individual 1-byte slices (`explode b"" b"" == []`, `explode b"" b"abc" == [b"a", b"b", b"c"]`). `assertStageLInternal` in `stage_1_impls/std/assert.sl3` (`if /cond nop fault`) is gated to `stage_1_impls/`.

### 6.6 `n`-Ary Tuples (`tup: ... ::`) and Positional `struct`s
- **Construction**:
  - `tupNew: ELEM_0 ... ELEM_{n-1} ::` (`Dc 2215 "Begin n-ary tuple list (ordered, contiguous, typed slots, length n)"`..`Dc 2216 "End n-ary tuple list"`) evaluates `n >= 0` slot expressions left-to-right and returns an immutable `tup: T_0 ... T_{n-1} ::` value (`Dc 2672 "Built-in type identifier: Tuple"`).
  - `structNew NAME: ELEM_0 ... ELEM_{n-1} ::` (`Dc 2219 "Begin positional struct value"`..`Dc 2220 "End positional struct value"`) evaluates `n >= 1` slot expressions left-to-right and returns an immutable nominal positional `struct` instance (`Dc 2673 "Built-in type identifier: Struct"`).
- **Constant-index projection & functional update**:
  - `field BASE u32:IDX` (`Dc 2547 "Get positional field"`) projects zero-based compile-time integer slot `IDX` (`0 <= IDX < n`) from an `n`-ary tuple or positional `struct` in `O(1)` time.
  - `with BASE u32:IDX NEW_VAL` (`Dc 2548 "With positional field"`) evaluates `BASE` then `NEW_VAL` left-to-right and returns a new tuple or positional `struct` with slot `IDX` (`0 <= IDX < n`) replaced by `NEW_VAL`.

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

### Entry Convention & Stage 0 Built-in Linking
- A StageL 3 program used as a staged Dc/format implementation defines `v/l/u8/step l/u8/input` or `v/l/u8/main l/u8/input` (operating on raw DcTF/byte payload slices) or exports `_start` / `step` / `main () -> i32` returning Dc-WASM `StatusCode` (`0 = Complete`, `1 = YieldFrame`, `2 = RequestCapability`), writing the result slice descriptor to `MB_OUT_PTR` (`8..12`) and `MB_OUT_LEN` (`12..16`).
- All 39 non-short-circuiting StageL 3 built-in verbs and `buf/T` primitives (`Dc 2512 "Add (faulting)"`..`Dc 2532 "Boolean NOT"`, `Dc 2535 "Equal comparison"`..`Dc 2546 "Concatenate sequences"`, and `Dc 2580 "StageL buffer new"`..`Dc 2585 "StageL buffer freeze"`) have readable Stage 0 `.wat` implementations in `src/formats/dcdata/data-src/stage_0_impls/`, emitted into the System graph as `Dc 646 "In context of"` `.WASM` definitions (`tup: ... ::` and positional `struct` linear-memory pointers reuse the 32-bit pointer sequence/buffer helpers), and Stage 1 standard library namespaces (`src/formats/dcdata/data-src/stage_1_impls/std/`) are emitted as lossless StageL Token Dcs and linked directly from `SeedGraphIndex` snapshots by `src/formats/dcdata/stagel3/seed_linker.rs` across the evaluator, Rust transpiler, host `src/formats/stagel/stagel3/wasm_codegen.rs`, and the self-hosted Stage 0 `.wat` StageL-Dc-to-WASM converter routine (`src/formats/dcdata/data-src/stage_0_impls/stagel3_to_wasm.wat`, `@dc l8589979107`, `@converts(2192 1927)`), invoked under `Dc 1938 "Resolve to pin"` and shared-quota `Dc 2608 "Convert Dc string to WASM"` by `Dc 1936 "Dc document execution kernel"`.
