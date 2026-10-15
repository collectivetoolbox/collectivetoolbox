# Dc Syntax Parser & Deserialization Refactor: Test Infrastructure Specification

## 1. Overview & Testing Philosophy

This document defines the testing architecture, runner infrastructure, tier hierarchy, and verification procedures for the Document Character (`Dc`) syntax parser, `DcSyntaxArena` projection engine, `#[derive(DcMixed)]` procedural macro, and `csc::journal` v4 streaming subsystem.

### 1.1 Guiding Principles

1. **Opaque-Box Specification Verification**:
   Tests must treat the system under test (SUT) as an opaque box, exercising public interfaces, data-driven grammar rules, and serialization formats without coupling to transient internal implementations.
2. **Authoritative Expected Output Derivation**:
   Every test case derives its inputs and expected outputs from authoritative sources:
   - The CSV syntax definitions in `src/formats/dcdata/data/categories/*.csv` and `README-named-types.csv`.
   - The interface contracts defined in `PROJECT.md § Interface Contracts`.
   - The authentic dev-era test fixtures `v4_dcmixed-20261011.cscjournal` and `v4_dcmixed_only-20261010.dcmm`.
   - The normative requirements in `ORIGINAL_REQUEST.md`.
3. **Repository Standards Compliance (`AGENTS.md`)**:
   - Every Rust test must be annotated with `#[crate::ctb_test]` or `#[crate::ctb_test("tokio")]` (never bare `#[test]` or `#[tokio::test]`).
   - Strict avoidance of `as` casts: all numeric and type conversions use `try_from`, `From`, or `utilities::math` fallible/checked conversion routines.
   - No unchecked infix arithmetic (`+`, `-`, `*`, `/`, `%` are denied by Clippy): all arithmetic operations use `.checked_add()`, `.saturating_add()`, `.checked_sub()`, etc.
   - No `unwrap()` or `expect()` in non-test production logic; tests propagate errors with `anyhow::Result<()>`.
   - Tests do not listen on network sockets; in-memory channels, mocks, and buffer streams are used.
   - Tests never write or reside in `.agents/teamwork/`.
4. **Progressive Testability & Isolation**:
   - Each test is self-contained and independently executable.
   - Tests create isolated temporary files or in-memory buffers and clean up after themselves.
   - Tests must run deterministically in any order.
5. **$O(1)$ Memory Stability Verification**:
   - The test infrastructure verifies that streaming readers (such as `read_v4_journal_snapshot`) reuse a single `DcSyntaxArena` across records without amortized allocation growth.

---

## 2. Multi-Tier Test Architecture

The test suite is structured into five distinct tiers, reflecting progressive integration depth:

```
+-------------------------------------------------------------------------------+
| Tier 5: Adversarial Hardening (Tampering, Depth Limits, Fuzzing)              |
+-------------------------------------------------------------------------------+
| Tier 4: Real-World Workload Scenarios (csc::journal Streams, Fixture Salvage)  |
+-------------------------------------------------------------------------------+
| Tier 3: Cross-Feature Combinations (Nested Envelopes, Shared Context, Maps)   |
+-------------------------------------------------------------------------------+
| Tier 2: Boundary & Corner Cases (Empty Containers, Strict vs Forgiving Rej.)  |
+-------------------------------------------------------------------------------+
| Tier 1: Feature Coverage (Data-Driven Syntax, Framing, Delimiters, Nav)       |
+-------------------------------------------------------------------------------+
```

### 2.1 Tier 1: Feature Coverage (Stage 1 Data-Driven Syntax & Framing)
- **Target Subsystems**: `ctb-formats-dcdata`, `ctb-formats-dcstring` (`navigator.rs`).
- **Scope**:
  - Verification of data-driven `SyntaxPattern` parsing directly from CSV grammar rules loaded by `DatasetRuleResolver`.
  - Opener and closer boundary containment: sacred framing ensures outer containers do not bleed into inner delimiters.
  - Elimination of `suppress_trailing_dc_512`: format rules (`CharTarget::Format(_)`) match only payload syntax; trailing `Dc 512` is handled strictly by enclosing rules.
  - Elimination of `prefix_rule_fixed_arity`: prefix operators evaluate variable arity patterns dynamically via AST evaluation.
  - Verification that document validation (`validate_dc_mixed_strict`, `validate_dc_str_strict`, `validate_dc_tokens_strict`) operates strictly via `DcDocumentNavigator` without legacy `framing.rs` or `matcher.rs`.

### 2.2 Tier 2: Boundary & Corner Cases
- **Target Subsystems**: `ctb-formats-dcstring` (`navigator.rs`, `dc_mixed_serde.rs`).
- **Scope**:
  - Empty containers: `317..318` (empty file), `310..311` (empty list), `511..512` (empty format payload), `543..544` (empty checksummed batch).
  - Unclosed and malformed container delimiters: missing closers, premature EOF, mismatched opener/closer pairs.
  - Nested and duplicate equivalent representations (`397`/`561`).
  - Format identifier mismatches: payload failing format-specific validation (e.g. invalid date magnitude or bad IP octet).
  - Strict syntax enforcement: strict rejection of bare strings and unformatted numbers where typed format envelopes are required (e.g. `PathBuf`, `LinkPath`, `FileTimestamps`).
  - Expansion depth limits: enforcing `MAX_SYNTAX_EXPANSION_DEPTH` (64) against hostile recursive syntax definitions.

### 2.3 Tier 3: Cross-Feature Combinations
- **Target Subsystems**: `ctb-formats-dcstring` (`kv_map.rs`, `shared.rs`, `dc_mixed_serde.rs`), `ctb_dc_macro`, `ctb-io` (`file::metadata`, `link_path`).
- **Scope**:
  - Key-value maps (`559..560` and `313..314`) nested within checksummed sections (`543..544 + 545`).
  - Format representations (`511..512`) embedded inside optional wrappers (`315..316`).
  - Lexical and shared value references (`540`/`276`) resolved across container boundaries via `SharedValueContext`.
  - Complex struct and enum projections generated by `#[derive(DcMixed)]`:
    - Named fields with custom tag annotations (`#[dc(tag = ...)]`).
    - Raw un-tagged fields (`#[dc(raw)]`).
    - Flattened structs (`#[dc(flatten)]`).
    - Key-value structs (`#[dc(key_value)]`).
    - Enum unit, unnamed, and named variants.
    - Transparent unwrapping of `397`/`561` equivalents and `315`/`316` optionals.

### 2.4 Tier 4: Real-World Workload Scenarios
- **Target Subsystems**: `ctb-io-csc` (`journal.rs`, `legacy.rs`), `ctb-formats-conversion`.
- **Scope**:
  - `csc::journal` multi-record stream encoding and decoding using `#[derive(DcMixed)]` record structs:
    - Session headers (`547..548`).
    - Checksummed entity batches (`543..544 + 545`).
    - Error records (`552..553`).
    - Warning records (`554..555`).
  - $O(1)$ arena reuse: verifying `read_v4_journal_snapshot` processes large multi-record journals using `arena.clear()` without increasing heap vector allocations.
  - Authentic Dev-Era Fixture Invariant:
    - `v4_dcmixed-20261011.cscjournal` and `v4_dcmixed_only-20261010.dcmm` must fail under `DcProjectionPolicy::strict()` due to legacy bare string/number paths.
    - Fixtures must succeed under `DcProjectionPolicy::forgiving()` (e.g. `--allow-journal-errors`), salvaging valid session and entity records.
  - Lossless strict roundtrip of newly serialized v4 journals containing entities, metadata, error logs, and session details.

### 2.5 Tier 5: Adversarial Hardening
- **Target Subsystems**: Cross-crate integration (`ctb-formats-dcstring`, `ctb-io-csc`).
- **Scope**:
  - Checksum tampering: flipping single bytes in checksummed sections (`543..544 + 545`) and verifying strict SHA256 digest validation fails.
  - Deeply nested framing attacks: inputs designed to exhaust recursion limits or trigger stack overflows.
  - Truncated chunk fuzzing: simulating power-loss truncation mid-record.
  - Unknown forward-compatible tag injection in open containers: verifying policy-driven skipping in forgiving mode vs error in strict mode.

---

## 3. Test Runner Commands & Suite Invocations

### 3.1 Crate-Specific Commands (Development / CI)

```bash
# Tier 1 & Syntax Rules
cargo test -p ctb-formats-dcdata
cargo test -p ctb-formats-dcstring navigator::

# Tier 2 & 3: Syntax Arena, Projections & Macro
cargo test -p ctb-formats-dcstring --test e2e_dc_syntax

# Tier 3: Macro Consumer Crates
cargo test -p ctb-formats-apple-single-double
cargo test -p ctb-io file::

# Tier 4: csc::journal Records, Fixtures & Snapshots
cargo test -p ctb-io-csc --test e2e_journal_v4
cargo test -p ctb-io-csc journal::
cargo test -p ctb-formats-conversion

# Full Regression Quick Linter
./lint --quick
```

### 3.2 Individual Test Target Invocations

```bash
# Run only Tier 1 Feature Coverage tests
cargo test -p ctb-formats-dcstring --test e2e_dc_syntax tier1_

# Run only Tier 2 Boundary & Corner Case tests
cargo test -p ctb-formats-dcstring --test e2e_dc_syntax tier2_

# Run only Tier 3 Cross-Feature Combination tests
cargo test -p ctb-formats-dcstring --test e2e_dc_syntax tier3_

# Run only Tier 4 Real-World Workload tests
cargo test -p ctb-io-csc --test e2e_journal_v4 tier4_
```

---

## 4. Test Fixtures & Asset Integrity Rules

1. **Authentic Fixtures Protection**:
   - Files `/workspaces/ctoolbox/src/io/csc/fixtures/journals/v4_dcmixed-20261011.cscjournal` and `v4_dcmixed_only-20261010.dcmm` must never be modified, deleted, or regenerated.
   - Any test that modifies these files in place or fails to assert strict rejection is an invalidation of the test suite.
2. **Synthetic In-Memory Fixtures**:
   - Synthetic test documents are constructed in memory using `DcMstring` or `DcByteSource` slices.
   - Deterministic test tokens are defined using typed constants from `ctb_formats_dcdata::dc::*`.
3. **No External Network Access**:
   - Tests must operate offline with zero port bindings.

---

## 5. Requirement Traceability Matrix

| Feature | Description | Primary Tier | Test File / Target | Invariant Tested |
| :--- | :--- | :--- | :--- | :--- |
| **F1** | Clean Up Commit `9105d5bae` | Tier 1 | `ctb-formats-dcdata`, `ctb-formats-dcstring` | Zero uncompilable/duplicate merge lines |
| **F2** | Remove `suppress_trailing_dc_512` | Tier 1 | `tests/e2e_dc_syntax.rs` | Format rules do not consume outer `Dc 512` |
| **F3** | Remove `prefix_rule_fixed_arity` | Tier 1 | `tests/e2e_dc_syntax.rs` | Variable-arity prefix operators parsed from AST |
| **F4** | Navigator Document Validation | Tier 1 | `tests/e2e_dc_syntax.rs` | `validate_dc_mixed_strict` validates zero-copy |
| **F5** | `DcProjectionContext` | Tier 2 | `tests/e2e_dc_syntax.rs` | Context tracks arena, policy, and shared scope |
| **F6** | `DcMixedDecode` Node Trait | Tier 2 | `tests/e2e_dc_syntax.rs` | Projection executes from `(ctx, node_id)` |
| **F7** | Primitive & Collection Projections | Tier 2 | `tests/e2e_dc_syntax.rs` | Primitives, `Duration`, collections decode |
| **F8** | Transparent Universal Unwrapping | Tier 2/3 | `tests/e2e_dc_syntax.rs` | `397`/`561`, `315`/`316`, `511..512` unwrap |
| **F9** | Strict Syntax Enforcement | Tier 2 | `tests/e2e_dc_syntax.rs` | Reject bare strings/numbers in strict mode |
| **F10** | Rebase `source_edit` onto Arena | Tier 2 | `ctb-formats-dcstring` (`source_edit`) | In-place rewriting without `read_nodes` |
| **F11** | Rebase `DcInspection` onto Arena | Tier 2 | `ctb-formats-dcstring` (`dc_source`) | Diagnostics and annotations over arena |
| **F12** | Delete `DcMixedReader` | Tier 1-3 | All crates | `git grep "DcMixedReader"` returns 0 matches |
| **F13** | Refactor `#[derive(DcMixed)]` | Tier 3 | `tests/e2e_dc_syntax.rs` | Macro generates `decode_dc_node` projection |
| **F14** | Deduplicate Field Codegen | Tier 3 | `tests/e2e_dc_syntax.rs` | Shared field spec handles structs & enums |
| **F15** | Migrate Manual Structs | Tier 3 | `apple_single_double`, `ctb-io` | Metadata and link paths decode over arena |
| **F16** | `csc::journal` Record Structs | Tier 4 | `tests/e2e_journal_v4.rs` | Session, error, warning use `#[derive(DcMixed)]` |
| **F17** | $O(1)$ Arena Reuse in `read_v4_journal_snapshot` | Tier 4 | `tests/e2e_journal_v4.rs` | Single arena reused across all stream records |
| **F18** | V4 Fixture Strict Rejection / Forgiving Salvage | Tier 4 | `tests/e2e_journal_v4.rs` | Strict fails, forgiving recovers v4 fixtures |
| **F19** | E2E Multi-Tier Coverage | Tiers 1-4 | Full Test Suite | 100% test pass across all affected crates |
| **F20** | Tier 5 Adversarial Coverage | Tier 5 | `tests/e2e_dc_syntax.rs`, `tests/e2e_journal_v4.rs` | Tampered digest, max depth, malformed frames |

