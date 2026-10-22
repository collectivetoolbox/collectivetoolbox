# Dc Parser

**Partially Implemented.** This is the canonical backlog for Dc syntax parsing,
validation, projection, and derive work. The
[dated refactor records](2026-10-13-Dc-syntax-parser-refactor/README.md) provide
historical context, not current completion or test certification.

Reviewed against source at `2fe81e7027399dcbceabf3b394f5c94cc814100d` on
2026-10-21. Rust tests and Clippy were not rerun for this documentation cleanup.
Checked items below describe bounded implementation evidence, not full feature
completion. Unchecked review leads require a focused reproduction before fixes.

## Required Behavior

- Generate grammar, semantic roles, and equivalence handling from the Dc
  database. Navigation, validation, inspection, editing, and typed projection
  share parsing rather than implementing independent readers.
- Preserve container scope during recovery, including unknown future fields.
  Require explicit strict/forgiving policies and retain caller diagnostics.
- Use generic derives for structured codecs, not per-struct syntax parsers.
- Support hierarchical seeking, lazy binary spans, and bounded parsing memory.
  Per-record arena reuse does not make retained journal snapshots constant-size.
- Preserve malformed development fixtures: strict rejection and ordinary
  forgiving recovery, without fixture-specific rewrites.
- Keep raw inspection distinct from runtime execution. Normal document display
  executes through the capability-gated runtime; strict validation is not a
  prerequisite for attempting display.

## Pending Work

### Parser and Projection Ownership

- [ ] Finish auditing remaining retained diagnostics across projection,
  inspection, and journal consumers (`record_diagnostic` now records parser
  diagnostics in `DcDocumentNavigator` and drains them into `DcSyntaxArena` even
  on error; verify conversion failures, unknown fields, and missing fields).
- [ ] Audit remaining default-resolver boundaries rather than repeating the
  completed role migration. For example,
  [minimal number readers](../src/formats/dcdata/dc_number_minimal.rs#L71)
  use default roles/digits. Exercise custom aliases through the public
  API before deciding which helpers need explicit resolver variants.
- [ ] Validate ambiguous database grammar and ambiguous derive mappings at
  build time. Equivalence parity is already checked; establish which other
  ambiguity classes remain unsupported.
- [ ] Evaluate adding `@ident` for Dcs used in struct/enum derive attributes as well, to make compiler/diagnostic errors more obvious (note that this is different from the current accepted style guide, which forbids @ident only for enums/structs).
- [ ] Audit remaining handwritten structured codecs and grammar-independent
  encoding paths. Writer-based derives exist; do not equate this with all
  serialization being generated from syntax specifications.
- [ ] Reconcile `DcProjectionPolicy` naming/ownership: it governs both parsing
  and projection, despite the original distinction. Do not create parallel
  policy implementations merely to rename it.

### Value Correctness and Fidelity

- [ ] Preserve Dc-native literal/identifier payloads losslessly. Current
  [literal decoding](../src/formats/dcstring/navigator.rs#L4670) and identifier
  decoding convert to Rust `String`, including Plane 14 fallback for short Dcs.
  Cover escaped closers/escape Dcs, long/non-Unicode Dcs, aliases, and exact
  roundtrips; the fallback is progress, not proof of general losslessness.
- [ ] Reproduce the remaining converted-flattened-struct and optional-flattened-
  enum derive combinations identified in the October 17 review. Required-field
  presence/finalization regressions already exist in
  [projection tests](../src/formats/dcstring/tests/challenge_m2_projection.rs#L592).
- [ ] Qualify equivalent/partially equivalent representations generically,
  including timestamp resolution, extent fields, and nested collection/optional
  combinations. Existing equivalents and expression tests are not proof that
  every target type accepts every valid representation.
- [ ] Verify runtime-produced streams through both projection and splicing,
  including multi-token values, emitted outer closers, and quoted content.
  [Expression/overlay tests](../src/formats/dcstring/tests/e2e_dc_syntax.rs#L1010)
  already exist; extend them instead of reviving synthetic-node insertion.
- [ ] Audit timestamp decoding for omitted required fields defaulting to epoch,
  duplicate-field replacement, and retained unknown `EnvDescription.extra`
  fields. These historical CSC reports need reproduction against current
  derives; do not add another timestamp codec.

### Resource and Streaming Limits

- [ ] Add a textual encapsulation payload-allocation budget before decoding.
  Metadata-byte and metadata-depth budgets already exist and do not bound the
  complete decoded payload allocation.
- [ ] Provide caller-configurable variants for policy-free encapsulation
  utilities (`from_dc_str`, `push_binary_with_metadata`, `to_dc_string`,
  `validate_metadata`, `write_text`), which currently enforce default limits.
- [ ] Qualify raised depth limits across syntax evaluation, arena construction,
  projection, editing, and output. A configurable knob is implemented; accepting
  a deeper document must not merely move stack exhaustion into another phase.
- [ ] Measure memory on large records/journals and exercise 32-bit offsets above
  4 GiB, short reads/writes, truncated tokens, and allocation failures. Separate
  per-record parser overhead from retained snapshot memory and temporary disk
  use. Keep general hierarchical navigation usable beyond journals.
- [ ] Decide whether full grammar validation on arbitrary forward-only input is
  needed. The existing byte-stream copier validates transport/encapsulation,
  not full grammar; typed-record validation spools to a seekable anonymous file.
  See [streaming boundaries](2026-10-13-Dc-syntax-parser-refactor/streaming-dc-document-validation.md).
  Do not treat either as a missing implementation of the other.

### Database and Semantic Decisions

- [ ] Specify downstream expansion of multi-token `<equiv>` decompositions,
  such as `f350 f351`. Single-token canonical equivalence and rejection of
  conflicting single-token syntax on decompositions are already implemented.
- [ ] Resolve the retained
  [flattened name/format FIXME](../src/formats/dcstring/dc_mixed_serde.rs#L4158)
  and [routine-invocation clarification](../src/formats/dcdata/data/categories/el.csv#L38).
  Restoring a comment or adding an identifier does not resolve its semantics.
- [ ] Review Remote URI/ETag mappings, String (`f80`) versus Text (`f14`), and
  encoding formats' relationship to Text. Existing Remote fields use filepath
  and payload-checksum meanings; establish intended semantics before changing
  IDs or weakening grammar. These are inherited/approval-sensitive questions.
- [ ] Reconcile older flag-ontology reports involving Dc 342 and 422/423 against
  current derives. `FinderLabel` is already derived; no handwritten numeric
  migration is needed there. Preserve approved Dc meanings and obtain approval
  for any new concepts.

### Review and Qualification

- [ ] Finish cumulative-review closure against the original refactor baseline
  `5aafa2bf3762da8a6d719145cb71f362851c1c6e`, accounting for later changes.
  Historical line numbers and defect lists are not fresh reproductions. Audit
  fallback/error handling and unresolved FIXME provenance, not just roundtrips.
- [ ] Review adversarial test quality and consolidate duplicate challenge
  harnesses only where coverage is preserved. Use complete positive controls
  before corruption/truncation mutations; do not weaken grammar to fix fixtures
  or infer full contract coverage from a test's name.
- [ ] Run affected-crate musl tests/Clippy for implementation changes; qualify
  clean bootstrap and relevant non-Linux runtime behavior separately. No current
  full-workspace or cross-platform certification is claimed.

## Related and Deferred Work

- Journal crash recovery, integrity diagnostics, completion/resume, archive
  qualification, and unknown Apple entry preservation belong to
  [CSC and archives](csc.md), sharing the parser/derive work above.
- EITE armor, UTF-8/UUID conversion, ELAD, and editor RPC reproduction remain in
  [EITE](eite.md); they are not duplicate parser implementation tasks.
- Detection-confidence follow-ups belong to [format detection](format-detection.md).
- Unsupported temporal-format conversion (including Panorama), GUI editor
  development, zero-allocation static table residency, and a database-backed
  streaming journal snapshot remain deferred, not completed parser work.

## Verification

Documentation consolidation does not certify parser correctness. Historical
test counts are not current test results; focused regressions and affected-crate
Linux musl checks remain required when implementing the pending work.

Extend existing tests in [dcdata](../src/formats/dcdata/syntax/resolver.rs),
[Dc syntax](../src/formats/dcstring/tests/e2e_dc_syntax.rs),
[projection](../src/formats/dcstring/tests/challenge_m2_projection.rs), and
[journals](../src/io/csc/tests/e2e_journal_v4.rs). Cover text/mixed/file parity,
strict rejection and ordinary forgiving recovery of the unchanged development
fixtures, retained diagnostics, and no scope bleed. Empty framed values must
still satisfy database-required headers; historical test prose is not grammar.

Use focused commands first, then the affected crates' `--lib --tests` and
Clippy checks, with explicit `--target x86_64-unknown-linux-musl`. Keep at least
25 GB free, do not add dependencies or listening ports, and do not suppress
Clippy errors. Documentation-only cleanup needs link and whitespace checks,
not a broad Rust rebuild.