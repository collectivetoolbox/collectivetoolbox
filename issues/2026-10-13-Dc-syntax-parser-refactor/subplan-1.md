# Data-Driven Dc Syntax Parser & Arena Projection Refactor Plan

## Goal Description

Complete the Document Character (`Dc`) syntax parser and deserialization refactor described in [plan.md](file:///workspaces/ctoolbox/issues/2026-10-13-Dc-syntax-parser-refactor/plan.md), addressing all points from the implementation review feedback:

1. **Truly Data-Driven Stage 1 Parsing (No Hardcoded Dc Lists or Arity Approximations)**:
   - Remove the hardcoded `is_value_operand` short-Dc match list in [dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L862-L876) and the `prefix_rule_fixed_arity` approximation in [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L1864-L1919).
   - Drive [DcDocumentNavigator](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L600-L608) and [DcSyntaxArena](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L461-L465) directly from the CSV-defined [`DcSyntaxRule`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs#L172-L180) and [`SyntaxPattern`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs#L139-L145) ASTs in [`DatasetRuleResolver`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs#L203-L211) (loaded from [README-named-types.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/README-named-types.csv), [categories/*.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories), and [categories/formats/*.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/formats)).
   - Precompute starter token sets for named constructs (`[value]`, `[type]`, `[statement]`, `[boolean]`, etc.) and infix operator rules (`269`) in `DatasetRuleResolver::load()` directly from the parsed CSV syntax ASTs.

2. **True DRY Consolidation (Deleting Redundant Parsers Instead of Layering Alongside Them)**:
   - **Rebase Stage 2 (`DcMixedDecode`, `#[derive(DcMixed)]`, `DcInspection`) onto `DcSyntaxArena`**:
     - In commits `a759cf9eb` and `a613af97b`, `DcSyntaxArena` and `expand_frame` were added to [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs) but never connected to `DcMixedDecode` or `#[derive(DcMixed)]`; `decode_frame` simply sliced bytes and handed them back to `DcMixedReader`.
     - By projecting `DcMixedDecode` types and `#[derive(DcMixed)]` structs/enums directly from `DcSyntaxArena` nodes (`NodeId`), container child framing and error recovery are handled once by Stage 1 (`DcDocumentNavigator`), allowing us to **delete `DcMixedReader`'s pull-parser and ad-hoc recovery methods**.
   - **Delete `source_edit::read_nodes` (`Node` / `Frame` / `map_entries`)**:
     - Rebase [source_edit.rs](file:///workspaces/ctoolbox/src/formats/dcstring/source_edit.rs) directly onto `DcDocumentNavigator` + `DcSyntaxArena`.
   - **Delete the `&[u128]` Document Parser in `dcdata::syntax`**:
     - Replace `validate_dc_document_strict(&tokens)` (which required `DcMstr::to_dc_tokens()` to base64-encode all binary payloads into a `Vec<u128>`) with direct zero-copy validation over `&DcMstr` / `&DcStr` via `DcDocumentNavigator`, and delete [framing.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/framing.rs), [matcher.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/matcher.rs), and `parse_document_tokens` in [parser.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/parser.rs#L703-L1346).

3. **No Special-Casing of Development-Era v4 Serializer Bugs**:
   - Remove legacy unframed fallbacks from `PathBuf::decode_dc_mixed`, `LinkPath::decode_dc_mixed`, and `FileTimestamps::decode_dc_mixed` so `DcProjectionPolicy::strict()` enforces the CSV syntax (`[filepath]`, `[date]`, `[datemagnitude]`, `[ipaddr]`) and rejects the broken dev-era fixtures (`v4_dcmixed-20261011.cscjournal`, `v4_dcmixed_only-20261010.dcmm`), while `DcProjectionPolicy::forgiving()` salvages valid fields and substitutes `DcForgivingDefault` for malformed fields via standard Stage 1/Stage 2 recovery.
   - Keep `v4_dcmixed-20261011.cscjournal` and `v4_dcmixed_only-20261010.dcmm` untouched to test strict rejection + forgiving recovery, and add a valid v4 journal test case for strict-mode roundtrip verification.

4. **Phased Execution to Prevent Context Compaction Issues**:
   - Each phase below is scoped to a self-contained vertical slice that compiles, passes tests, and deletes the obsolete code it replaces. We can execute one phase at a time (or use focused subagents per phase) and checkpoint with you between phases.

---

## Proposed Changes

### Phase 1: Data-Driven `SyntaxPattern` Evaluator in `DcDocumentNavigator` & Deletion of `&[u128]` Parser (`dcdata` + `dcstring`)

#### [MODIFY] [resolver.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs)
- Extend `DatasetRuleResolver::load()` to precompute from the CSV `DcSyntaxRule` and `README-named-types.csv` `SyntaxPattern` ASTs:
  - `named_type_starters: HashMap<String, StarterSet>`: exact set of starter tokens/scripts/ranges derived recursively from each named type's `SyntaxPattern` (`value`, `type`, `statement`, `number`, `string`, `boolean`, `data`, `checksum`, `date`, `duration`, `datemagnitude`, `list`, `key_value`, `identifier`, `filepath`, `filename`, `ipaddr`).
  - `infix_rules: HashMap<(NamedOrTarget, u128), DcSyntaxRule>`: rules where `~` (`SyntaxTerm::SelfChar`) is preceded by a left-hand operand (such as `Dc 269`: `:[identifier $ident] ~ [value $val]`).
  - `container_header_patterns: HashMap<CharTarget, Vec<SyntaxElement>>`: leading header elements between `~` and the body/closer of a container rule (e.g. `[number $length]? [type]` for `310`, `[number $length]? [type $keyType] [type $valType]` for `559` and `313`, `[format]` for `511`, `[formats:date]` for `390`, `[formats:duration]` for `393` and `395`).
- Expose `matches_named_type_starter(&self, name: &str, token: u128) -> bool` and `container_header_elements(&self, opener: &CharTarget) -> &[SyntaxElement]` on `SyntaxRuleResolver`.

#### [MODIFY] [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs)
- Replace `prefix_rule_fixed_arity` and hardcoded tag checks with a general, recursive `SyntaxPattern` / `SyntaxElement` / `SyntaxTerm` evaluator over `DcDocumentNavigator` that both validates and builds `DcFrameStub` / `DcSyntaxArena` nodes:
  - Evaluates `SyntaxPattern::Sequence` and `SyntaxPattern::Alternation` with `Quantifier` (`ExactOne`, `Optional`, `ZeroOrMore`, `OneOrMore`, `Range`).
  - Evaluates `SyntaxTerm::SelfChar`, `CharRef`, `CharSet`, `CharRange`, `Group`, `NamedConstruct` (including `[data:bytes=N]`, `[data:f84;bytes=N]`, `[number:Natural0&<8]`, `[string:[asciiprintable]{4}]`, `[script:...]`, `[format:...]`, `[formats:...]`, `[type]`, `[value]`), and `RuleRef` (`[310:]`, `[511:]`, `[f229:]`, etc.).
  - Handles universal wrappers defined in the CSVs (`397` `EquivalentRepresentations`, `561` `PartiallyEquivalentRepresentations`, `315` `OptionalPresent`, `316` `OptionalAbsent`) uniformly when matching any value term.
  - When a rule fails partway through inside an open container under `forgiving` mode, uses `resolver.matches_named_type_starter("value", tok)` / `resolver.matches_named_type_starter("type", tok)` (computed from CSV rules, zero hardcoded IDs) to consume dangling value/type operands up to the next statement tag or enclosing closer.
- Provide `validate_dc_mixed_strict(&DcMstr) -> Result<()>` and `validate_dc_str_strict(&DcStr) -> Result<()>` using `DcDocumentNavigator` + `DcSyntaxArena` directly without allocating `Vec<u128>` or base64-encoding binary chunks.

#### [DELETE] [framing.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/framing.rs)
#### [DELETE] [matcher.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/matcher.rs)
#### [MODIFY] [parser.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/parser.rs), [validator.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/validator.rs), [syntax.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax.rs), [dcdata.rs](file:///workspaces/ctoolbox/src/formats/dcdata/dcdata.rs)
- Remove `parse_document_tokens`, `parse_document_tokens_with_resolver`, `validate_dc_document_strict`, and `validate_dc_stream_strict` from `ctb_formats_dcdata` (keeping `parse_dc_syntax` and `validate_dc_syntax` for CSV schema validation).
- Migrate document-syntax validation tests from `dcdata/validation.rs` to `dcstring/navigator.rs` and update callers in `dcstring/encapsulation.rs`, `dcstring/dc_mixed_serde.rs`, `io/csc/journal.rs`, and `formats/apple_single_double/apple_single_double.rs` to use `validate_dc_mixed_strict` / `validate_dc_str_strict`.

---

### Phase 2: Rebase `DcMixedDecode`, `DcInspection`, & `source_edit` onto `DcSyntaxArena` and Delete `DcMixedReader` (`dcstring`)

#### [MODIFY] [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs) & [dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs)
- Introduce `DcProjectionContext<'a, 'ctx>` holding:
  - `arena: &'a DcSyntaxArena<'a>`
  - `policy: &'ctx DcProjectionPolicy`
  - `shared_context: Option<&'ctx SharedValueContext>`
  - `reference_scope: SharedValueContext` (for lexical `541..542` scopes and `269` assignments)
  - `annotations: Vec<DcFieldAnnotation>` + `annotating: bool` + `record_depth: usize` (for `DcInspection`)
- Change `DcMixedDecode` to project directly from a parsed `NodeId` in `DcSyntaxArena`:
  ```rust
  pub trait DcMixedDecode: Sized {
      fn decode_dc_node(
          ctx: &mut DcProjectionContext<'_, '_>,
          node_id: NodeId,
      ) -> Result<Self>;
  }
  ```
- Implement `DcMixedDecode::decode_dc_node` for all primitive and standard library types (`String`, `bool`, integer types, `Natural`, `Integer`, `Option<T>`, `Vec<T>`, `[T; N]`, `(A, B)`, `BTreeMap<K, V>`, `Ipv4Addr`, `Ipv6Addr`, `IpAddr`, `FormatId`, `PathBuf`, `SystemTime`, `Duration`, `BinaryPayload`, `Box<T>`, `Arc<T>`) directly over `DcSyntaxNode`:
  - Transparently unwraps `397`/`561` equivalent lists, `511..512` format representations, and `540`/`276` bound references in `DcProjectionContext`.
  - Enforces strict vs forgiving rules for `PathBuf` (`511 [f229:] 512`), `Ipv4Addr` / `Ipv6Addr` / `IpAddr` (`511 [f226:] 512`), `SystemTime` (`390..391`), and `Duration` (`393..394` / `395..396`).
- Delete `DcMixedReader` and all its pull-parsing/recovery methods (`peek_char`, `next_char`, `skip_syntax_unit`, `skip_unknown_field_in_container`, `recover_malformed_field_in_container`, `read_string`, `read_i128`, `read_u128`, `read_encapsulated_payload`, `read_textual_binary`).

#### [MODIFY] [source_edit.rs](file:///workspaces/ctoolbox/src/formats/dcstring/source_edit.rs) & [dc_source.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_source.rs)
- Delete `struct Node`, `struct Frame`, `fn read_nodes`, and `fn map_entries` in `source_edit.rs`.
- Rebase `preserve_source` and `preserve_text` directly onto `DcDocumentNavigator` + `DcSyntaxArena`.
- Update `DcInspection::read` in `dc_source.rs` to expand the document into `DcSyntaxArena` and project via `DcProjectionContext`.

#### [MODIFY] [kv_map.rs](file:///workspaces/ctoolbox/src/formats/dcstring/kv_map.rs) & [shared.rs](file:///workspaces/ctoolbox/src/formats/dcstring/shared.rs)
- Replace `DcMixedReader`-based `decode_header`/`decode_entry_begin`/`decode_entry_end` in `kv_map.rs` and `decode_identifier`/`decode_shared_reference` in `shared.rs` with `DcSyntaxArena` node helpers.

---

### Phase 3: Refactor `#[derive(DcMixed)]` to Project from `DcSyntaxArena` & Deduplicate Codegen (`ctb_dc_macro` + `io/file`)

#### [MODIFY] [lib.rs](file:///workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs)
- Refactor struct and enum `DcMixedDecode` generation to operate on `(ctx: &mut DcProjectionContext, node_id: NodeId)`:
  - For a `#[dc(begin = B, end = E)]` struct: verifies `node_id` is `DcSyntaxKind::Container { opener: B, closer: E, children, .. }` and iterates `&child_id in &children`:
    - Dispatches each `child_id` by its tag (`DcSyntaxKind::Tagged { tag, args }` or `DcSyntaxKind::Container { opener, .. }` or `DcSyntaxKind::Atom(tag)`).
    - Records `ctx.annotate_field(..., child_node.span)` on success.
    - On unknown tag: calls `ctx.policy.handle(ctx.policy.unknown_field, ...)` and continues to next `child_id` (no manual stream skipping needed!).
    - On field projection error: calls `ctx.policy.handle(ctx.policy.malformed_field, ...)` and continues to next `child_id` (no manual stream recovery needed!).
  - Deduplicate struct field projection, `decode_field`, and enum named-variant field projection into a single `generate_field_projection_arms` helper.

#### [MODIFY] [metadata.rs](file:///workspaces/ctoolbox/src/io/file/metadata.rs), [link_path.rs](file:///workspaces/ctoolbox/src/io/file/link_path.rs), [streams.rs](file:///workspaces/ctoolbox/src/io/file/streams.rs), [apple_single_double.rs](file:///workspaces/ctoolbox/src/formats/apple_single_double/apple_single_double.rs)
- Migrate manual `DcMixedDecode` implementations (`FileTimestamps`, `AppleMetadata`, `NativeMetadata`, `LinkPath`, `StreamName`, `AppleSingleDoubleFile`) from `DcMixedReader` to `DcSyntaxArena` (`DcProjectionContext` + `NodeId`).
- Remove silent legacy bare-number / bare-string fallbacks from `FileTimestamps` and `LinkPath` so `strict()` requires valid syntax (`390..391`, `393..394`, `511..512`).

---

### Phase 4: Migrate `csc::journal` Session/Error/Warning Records to `#[derive(DcMixed)]` & Verify Fixtures (`io/csc`)

#### [MODIFY] [journal.rs](file:///workspaces/ctoolbox/src/io/csc/journal.rs)
- Replace manual token loops in `write_session_header`, `record_error`, `record_warning`, and `read_v4_journal_snapshot` (`547..548` `DC_BEGIN_OPERATION_SESSION`, `552..553` `DC_BEGIN_ERROR_RECORD`, `554..555` `DC_BEGIN_WARNING_RECORD`) with `#[derive(DcMixed)]` record structs decoded via `nav.decode_frame_with_arena(&stub, &mut arena, Some(&shared_context), policy)`.
- Reuse a single `DcSyntaxArena` across all records in `read_v4_journal_snapshot` (`arena.clear()` per record) so steady-state memory is $O(1)$.
- Replace `mst.to_dc_tokens()` in `JournalWriter::validate_segment` with `ctb_formats_dcstring::validate_dc_mixed_strict(mst)`.
- Verify:
  1. `v4_dcmixed-20261011.cscjournal` and `v4_dcmixed_only-20261010.dcmm` remain unmodified, fail in `strict()` mode, and parse salvageable sections in `forgiving()` mode.
  2. Newly written v4 journals pass `strict()` validation and roundtrip all session, batch, entity, error, and warning records losslessly.

---

## Verification Plan

### Automated Tests
After each phase, run crate-scoped tests and lints:
- **Phase 1**:
  - `cargo test -p ctb-formats-dcdata`
  - `cargo test -p ctb-formats-dcstring navigator`
- **Phase 2 & Phase 3**:
  - `cargo test -p ctb-formats-dcstring`
  - `cargo test -p ctb-formats-apple-single-double`
  - `cargo test -p ctb-io file::`
- **Phase 4**:
  - `cargo test -p ctb-io-csc`
  - `cargo test -p ctb-formats-conversion`
  - `./lint --quick`
