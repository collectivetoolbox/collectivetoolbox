# Graph-Defined Dc Ontology & Pattern Matcher Implementation Plan

**Overall Status**: `Partially Implemented`

## Completed Phases (Summary)
- **Phase 1 (System-Block Node Ledger — Implemented in `src/formats/dcdata/graph_ledger.rs`)**:
  - Compiles CSV tables in `src/formats/dcdata/fixtures/` into `assets/data/graph/85899_*.ctbnodes` and `assets/data/graph/85899_34592.ctbnbdl` with append-only revision tracking via `Dc 591` (`Supersedes`) and `Dc 592` (`Retracts`), value node interning, and `Dc 584` (`Has a`) + `Dc 610` (`Has value`) statement triples.
- **Phase 2 (`SeedGraphIndex` & Runtime Migration — Partially Implemented in `src/formats/dcdata/seed_graph.rs`)**:
  - Runtime loading and indexing of `85899_34592.ctbnbdl`, active statement resolution across `Dc 591` (`Supersedes`) and `Dc 592` (`Retracts`), `.Semantic:Pattern` decoding (`Dc 599`–`Dc 607`), format metadata indexing, and migration of `DatasetRuleResolver::from_seed_graph`, `lookup.rs`, `format_info.rs`, and `mime_derivation.rs` are implemented.
  - CSV fixtures were moved from `src/formats/dcdata/data/` to `src/formats/dcdata/fixtures/` so `dcdata.rs` no longer embeds CSVs in production builds.
- **Phase 3 (Build-Time Graph Codegen & Retirement of `syntax_table.generated.rs` — Implemented)**:
  - `syntax_table.generated.rs` and `src/formats/dcdata/syntax/resolver/codegen.rs` have been removed.
  - `BuildGraphIndex` (`src/build_support/graph_index.rs`) reads `assets/data/graph/85899_*.ctbnodes` at build time to drive `dc_codegen.rs`, `format_id_codegen.rs`, `extension_codegen.rs`, `encoding_codegen.rs`, and `format_support_codegen.rs`.
- **Phase 4 (`DcDocumentNavigator` Simplification — Partially Implemented in `src/formats/dcstring/navigator/`)**:
  - Split `navigator.rs` into focused submodules (`navigator.rs`, `policy.rs`, `resolver.rs`, `arena.rs`, `scan.rs`, `eval.rs`, `tests.rs`).
  - Unified `spliced_tokens` and byte-backed token scanning across `scan_sibling_stub_with_resolver`, `eval_literal_frame`, `eval_identifier_frame`, `eval_number_frame`, and `eval_term_single`, and collapsed `eval_type_constructor` into the iterative `eval_pattern` frame machine.
  - Unified paired container scanning in `scan_sibling_stub_with_resolver` and `scan_balanced_container` via `Dc 590` (`Is a closing delimiter of syntactic block for`) + `SyntaxRuleResolver::is_opaque_container`, removing the separate `BeginLiteral`, `BeginIdentifier`, and `BeginNumber` scanner branches and `scan_escaped_text_frame`.
  - Replaced the bespoke `OptionalPresent` and `is_representation_set` fallback branches in `scan_sibling_stub_with_resolver` (`src/formats/dcstring/navigator.rs`) with generic `Dc 594` (`Syntax rule`) pattern evaluation and expanded `[value]` evaluation in `eval_term_single` (`src/formats/dcstring/navigator/eval.rs`) to cover paired non-identifier containers (`Dc 590`) and domain/primitive value atoms.
  - Added `: [number:...]` and `@implies(...)` rules for numeric format types (`Natural0`, `Positive`, `Negative`, `U8`..`I128`, `Natural1`) to the system graph ledger (`85899_34592.ctbnbdl`), indexed `Dc 587` (`Is a subtype of`) in `DatasetRuleResolver`, and consolidated `type_tokens_satisfy`, `number_satisfies_subtype`, and `enter_list_body_after_header` in `eval.rs`.

## Remaining Gaps / Pending Work

### 1. Direct Graph Pattern Evaluation (`src/formats/dcdata/seed_graph.rs` & `src/formats/dcdata/syntax/resolver.rs`)
- [ ] Evaluate `.Semantic:Pattern` graph nodes (`Dc 599` `Self reference` .. `Dc 607` `Pattern replacement`) directly without decoding them back into intermediate `SyntaxPattern` AST structures in `DatasetRuleResolver::from_seed_graph`.
- [ ] Precompute static active-statement, equivalence (`Dc 611` `Canonical decomposition`), and transitive `Dc 586` (`Is an instance of`) / `Dc 587` (`Is a subtype of`) closures at build time so startup does not need to replay and re-index `.ctbnbdl` into heap `HashMap`s in `SeedGraphIndex` and `DatasetRuleResolver`.

### 2. `DcDocumentNavigator` Graph-Driven Scanning & Evaluation (`src/formats/dcstring/navigator/`)
- [ ] Unify the remaining role-specific hooks inside prefix-rule scanning in `scan_sibling_stub_with_resolver` (`src/formats/dcstring/navigator.rs`: `.EL Routines`, `RoutineInvocation` trailing parameter scanning, and `is_representation_set` grammar verification) into `Dc 594` (`Syntax rule`) pattern definitions.
- [ ] Replace `number_satisfies_subtype` and `type_tokens_satisfy` in `src/formats/dcstring/navigator/eval.rs` with unified graph subtype/instance reachability (`Dc 586` / `Dc 587`) and `.Semantic:Pattern` rule evaluation (`Dc 594`).
- [ ] Replace bespoke evaluators in `src/formats/dcstring/navigator/eval.rs` (`eval_list_constraints`, `check_equivalent_list_grammar_with_resolver`, `eval_data_construct`, and `eval_typed_data_construct`) with general `Dc 606` (`Pattern capture`) and `Dc 607` (`Pattern replacement`) binding evaluation (`$length`, `$keyType`, `$valType`, and `[data:...]` / `[typed_data:...]`).

---

## Reference: `.Semantic:Pattern` Graph Encoding (`Dc 599`–`Dc 607`)

Every `DcSyntaxRule` and named construct in `README-named-types.csv` is encoded as a `.Semantic:Pattern` value node attached to its subject via `Dc 584` (`Has a`) + `Dc 594` (`Syntax rule`) + `Dc 610` (`Has value`):

| Construct | `.Semantic:Pattern` Graph Encoding |
| :--- | :--- |
| `SyntaxTerm::SelfRef` (`~`) | `Dc 599` (`Self reference`) |
| `SyntaxTerm::Any` (`*`) | `Dc 600` (`Wildcard pattern`) |
| `SyntaxTerm::Char(id)` (`261`, `u20`, `f199`) | Literal token `id` (escaped with `Dc 255` (`Ignore following Dc`) if `id` is a pattern operator `599`–`607`) |
| `SyntaxTerm::Range(start, end)` (`127..190`) | `4 <start> <end>` using `Dc 4` (`Range`) |
| `SyntaxPattern::Sequence(elems)` | Framed list `310 594 <elem_1> 312 <elem_2> ... 311` (`Dc 310` `Begin list`, `Dc 594` `Syntax rule`, `Dc 312` `List element separator`, `Dc 311` `End list`) |
| `SyntaxPattern::Alternation(branches)` (`A \| B`) | `602 <list_of_branches>` using `Dc 602` (`Pattern alternation`) |
| `ConstraintExpr::And(lhs, rhs)` (`A & B`) | `601 <list_of_conjuncts>` using `Dc 601` (`Pattern conjunction`) |
| `Quantifier` (`?`, `*`, `+`, `{n}`, `{m..n}`) | `603 <count_or_range> <inner_pattern>` using `Dc 603` (`Pattern repetition`), with unbounded upper bound `Dc 316` (`Optional value absent`) |
| `SyntaxTerm::Negated(inner)` (`[^261 255]`) | `604 <inner_pattern>` using `Dc 604` (`Negated pattern`) |
| `[262:]`, `[type]`, `[script:.EL Types]` | `605 <target_entity_or_type>` using `Dc 605` (`Subpattern match`) |
| `$ident` capture binding (`[number $length]`) | `606 270 <ident_chars> 271 <inner_pattern>` using `Dc 606` (`Pattern capture`) and `Dc 270`..`Dc 271` (`Begin identifier`..`End identifier`) |
| `SyntaxAction` (`: lang.assign($ident, $val)`) | `607 <match_pattern> <replacement_expr>` using `Dc 607` (`Pattern replacement`) |
