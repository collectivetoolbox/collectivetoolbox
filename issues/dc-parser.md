# Dc Parser

**Partially Implemented.** This is the canonical backlog for Dc syntax parsing,
validation, projection, and derive work.

Reviewed against source at `1314c0dace0d5137d3c2717d3bc318b01cb24a88` on
2026-10-22. Checked items below describe bounded implementation evidence, not
full feature completion. Unchecked review leads require a focused reproduction
before fixes.

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

- [ ] Validate ambiguous database grammar and ambiguous derive mappings at
  build time. Equivalence parity is already checked; establish which other
  ambiguity classes remain unsupported.
- [ ] Evaluate adding `@ident` for Dcs used in struct/enum derive attributes as well, to make compiler/diagnostic errors more obvious (note that this is different from the current accepted style guide, which forbids @ident only for enums/structs).
  - [ ] Alternatively: Instead of annotating structs/enums directly in code with Dcs, maybe use a new @struct()/@enum() note in the database and generate the structs from there?
- [ ] Audit remaining handwritten structured codecs and grammar-independent
  encoding paths. Writer-based derives exist; do not equate this with all
  serialization being generated from syntax specifications.

### Resource and Streaming Limits

- [ ] Measure memory on large records/journals and exercise 32-bit offsets above
  4 GiB, short reads/writes, truncated tokens, and allocation failures. Separate
  per-record parser overhead from retained snapshot memory and temporary disk
  use. Keep general hierarchical navigation usable beyond journals.
- [ ] Decide whether full grammar validation on arbitrary forward-only input is
  needed. Seekable documents use [`DcDocumentNavigator::from_dcmm_file`](../src/formats/dcstring/navigator.rs)
  (`validate_document` / `step_validate_construct`) for full grammar validation
  with bounded container skipping; [`validate_and_copy_dcmm_stream`](../src/formats/dcstring/navigator.rs)
  validates transport and encapsulation framing on forward-only `Read -> Write`
  streams without full grammar rules; and [`validate_and_write_dc_record`](../src/formats/dcstring/navigator.rs)
  spools typed records to an anonymous seekable temporary file before validated
  copy-out. Do not treat any of these three entrypoints as a missing
  implementation of another.
- /// FIXME: Will break on `csc archive` of large files. Finding a streaming validation strategy will be important.
pub fn validate_and_write_dc_record(

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