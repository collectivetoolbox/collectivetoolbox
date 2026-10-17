## Plan: Cumulative Dc Refactor Review And Remediation

### Implementation Progress: 17 October 2026

**Partially Implemented.** The planning-only account below is historical.
Implementation started at `ac3e1a7bdb61c224a71e8f297c5de1da4f42aaf4`;
the operator committed and edited concurrently, so both committed changes and
the working tree were included in validation without reverting unrelated work.

The bounded integrity, resource-depth, opaque-framing, generic projection,
inspection, journal replay/policy, grammar-generation, and detection-confidence
repairs are summarized in [PROJECT.md](PROJECT.md). Final affected-crate Linux
musl validation passed **852 tests across 19 harnesses, zero failed or ignored**,
plus the separately passing build-script resource-locator regression.
Affected-crate Clippy passed with warnings and no errors. Canonical data
regeneration reported no diagnostics.

The operator explicitly approved two new semantic concepts during this pass:
Dc 563, File data format (`[format]`), and Dc 564, Archive member position
(`[number:Natural0]`). `FileOrigin::Archive` now uses a typed `FormatId` and
these roles rather than application-bundle-format and inode tags. Existing
Dc meanings were not reassigned. Remote-resource semantics were not changed.

### Remaining Gaps vs Original Request

- Numeric/list subtype enforcement and lossless escaped non-Unicode payloads.
- Fully data-driven shape normalization and centralized retained diagnostics,
	including the original strict-deprecation contract.
- Unsupported converted/optional flattened derive combinations.
- Remote URI/ETag semantics, unknown Apple entries, and inherited binary
	type-insensitive source reconciliation.
- Textual payload-allocation budgets and total retained journal snapshot size.
- Complete cumulative-review closure, clean-bootstrap, full workspace, and
	non-Linux runtime validation. Unsupported Panorama conversion and editor
	development remain explicitly deferred.

The FIXME ledger remains explicit: the restored flattened-format note and
inherited routine-invocation note are open; parser-vs-macro legacy/deprecation
ownership is only partially addressed. No historical comment deletion is being
counted as a completed fix without supporting implementation evidence.

### Status And Boundaries

- Planning only: no repository source edits, commits, branches, builds, Rust tests, or Clippy executions were performed in this session.
- Requested baseline: 5aafa2bf3762da8a6d719145cb71f362851c1c6e. Recorded endpoint: c9f913d53bdd4a6c802b36f240aec2561f78ce31. Initial worktree clean. Final endpoint/worktree recheck is required before handoff because the operator commits during work.
- Scope: evaluate the cumulative effect of all 95 changed paths (31,168 insertions, 7,097 deletions), completion against the initial prompt and subsequent feedback at the start of issues/2026-10-13-Dc-syntax-parser-refactor/plan.md, correctness, and applicable repository guidelines. Per-commit inspection is reserved for the requested FIXME provenance audit and attribution where needed.
- This is a preliminary source/diff review and an actionable continuation plan, not certification that every changed line was reviewed or that runtime failures were reproduced. Main agent has working Git/terminal tools. Three Explore agents using GPT-6.1 Sol were read-only; they lacked terminal/Git access and supplied current-source leads, which the main agent triangulated against Git selectively. Their inability to inspect deltas must not be presented as a limitation of the main session.
- The original refactor remains Partially Implemented. Current source substantially consolidates navigation and projection, and now has a generated syntax table. Do not repeat stale claims that static codegen, file-backed payload checksums, required list headers, unsupported-date rejection, or nested document-root consumption are wholly unimplemented: current fixes/regressions exist.

### Verification Already Performed

- Recorded Git HEAD and clean worktree; inspected cumulative stat, selected full file diffs, semantic CSV deltas, small build/lock/submodule deltas, and current owning functions.
- git diff --check 5aafa2bf..c9f913d FAILED: blank EOF lines in AGENTS.md, project/review documents, challenge_m2_projection.rs, and e2e_journal_v4.rs; trailing-whitespace reports on changed format CSV rows. Inspect CSV line-ending conventions before modifying data to satisfy whitespace checks.
- bash -n build PASSED, run separately after the failed diff check short-circuited an earlier command chain.
- At inspection, filesystem had 32 GiB available against the required 25 GB reserve; no cargo/rustc/qemu processes were shown. Do not start a broad build or switch targets without checking again.
- Rust tests, Clippy, native non-Linux checks, and memory measurements were not run. Historical PROJECT.md counts are not current-session validation.

### Required Behavior From Original Prompt

- One database-driven parsing/framing/validation owner reused by projection, inspection/editing, journal replay, and serialization verification. Generated Rust derives project already parsed nodes; they must not independently rediscover legacy syntax or field framing.
- Open containers preserve unknown/new child syntax without scope bleed. Sacred openers cannot be discarded because an interior grammar check failed.
- Callers explicitly choose strict/forgiving policy. Original strict specification errors on deprecated syntax as well as syntax errors, unknown fields, malformed fields, and missing required fields. Diagnostics must be available to callers, not only emitted or silently discarded.
- General hierarchical streaming/seek API, lazy file-backed binary spans, reusable per-record arena, and controlled resource budgets usable by future editors as well as journals.
- No ad hoc rewrites for malformed development v4 fixtures; strict rejection and ordinary forgiving salvage. Preserve the original fixture bytes.
- Generic derives/attributes instead of per-struct codecs; no duplicated Dc constants, bare integer FormatId interfaces, silently swallowed errors, undocumented fallbacks, new unapproved semantic IDs, or new dependencies without approval.

### Historical FIXME Audit: Executed

A read-only Python subprocess script iterated git rev-list --reverse baseline..endpoint, obtained each git show --format= --unified=3 diff, selected added non-header lines containing FIXME, and printed filename/hunk plus three lines of surrounding context. It covered 49 commits and 13 added FIXME-containing lines. Follow-up git log -S and baseline/endpoint source comparisons established:

1. Macro-layer note: d0fc1e921 added `FIXME: This file should not know about legacy versus non-legacy syntax...` in ctb_dc_macro/src/lib.rs; 3ba6ec818 removed it. Endpoint uses ctx.extract_kv_map_entries at lib.rs:1204 backed by DcSyntaxArena::extract_kv_map_operand_entries (navigator.rs:829 onward), but lib.rs:1205 still branches on Legacy to handle deprecation and names Dc 313. Classify Partially Addressed, not silently resolved: duplicated entry extraction moved to shared arena code, but parser/projector separation and centralized diagnostic ownership still require review. Do not restore an obsolete comment verbatim; preserve an explicit unresolved tracking item until the boundary is actually resolved.
2. Flattened-format note: `FIXME doesn't look right. 422 is File flag... Name and format list should...` already existed at baseline dc_mixed_serde.rs:2661. d0fc1e921 temporarily removed it; e378fd1bc restored it. It remains at endpoint :3526 above SampleFlattenedFormatsStruct (:3529). Container is 317..318, but the semantic name/format equivalence question remains unresolved. This is retained inherited debt, not a permanently deleted new FIXME.
3. Routine-invocation clarification: el.csv:38 already contained `FIXME: clarify this??` at baseline. e378fd1bc changed the row to add @ident, causing it and generated CSV/Rust copies to appear in the added-line scan. The note remains; no new omission was established. Preserve it and its generated provenance.
4. Other matches are quoted copies of those notes in to-review/subplan-2.md or the guideline sentence forbidding unresolved FIXME deletion in AGENTS.md and the then-duplicated Copilot instructions. They are not independent code tasks. The redundant instruction file was subsequently deleted, with the guideline retained in AGENTS.md.

### Findings And Triage

These are source-traced findings, not runtime reproductions. Preserve that distinction in review output. Add focused regressions before remediation, and drop or downgrade a lead if the regression disproves it.

High-priority current paths:

- P1 bounded allocation: /workspaces/ctoolbox/src/formats/dcstring/navigator.rs:2891 materializes a section digest's entire payload using read_span_bytes, whose seekable-file branch allocates declared length at :1216. Hash-type matching does not first enforce digest length. A sparse multi-GiB checksum payload can force huge allocation. Validate algorithm-specific digest size before reading and use bounded buffers. The same section verification hardcodes SHA-256 at :2904 although the grammar accepts general checksum types; either implement already-supported algorithms with StreamingHasher or explicitly reject unsupported ones without guessing.
- P1 recursion budget: navigator.rs:3515 recursively evaluates type union/intersection rules before reaching the generic depth/call-stack checks at :3558 onward. Nested type constructors can bypass the configured budget. Make evaluation iterative or fully budgeted, including raised-budget cases; shallow scan_delimited_container also grows closer_stack without a budget check (:2815).
- P1 required flattened fields: /workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs:2026 initializes Option<Self> from DcForgivingDefault, and finish_flattened returns the instance at :2035 without per-field presence validation. New flattening on FileOrigin::Filesystem.key can turn a missing inode into zero under strict policy. Preserve per-field state and finalize required fields through shared generic code for structs and enum variants.
- P1 journal error classification: /workspaces/ctoolbox/src/io/csc/journal.rs:1148, :1195 and :1222 swallow malformed/unknown records by breaking and returning a successful prefix; open_for_resume truncates to valid_length at :456. Distinguish intended incomplete crash-tail rollback from syntax/policy/I/O errors and unsupported well-framed interior records. Original strict behavior must not silently authorize destructive truncation of a valid later suffix.
- P1 journal completion: navigator.rs:1383 returns escaped token as stub.head_dc, so journal.rs:1129 can treat `255 551` as OperationCompleted. Later batches also do not reset is_completed. Dispatch must use structural record classification rather than an escaped head value, and completion must reflect the last valid logical operation state.
- P1 journal integrity diagnostics: journal.rs:996 uses forgiving_quiet for scans; :1086 ignores digest mismatch in forgiving mode and commits records without caller-requested warning/structured integrity status. Preserve crash recovery but distinguish verified batches from salvage; do not call a checksum-invalid batch checksum-verified.

Additional concrete leads to reproduce:

- navigator.rs:3576 / syntax/resolver.rs:64 substitute only $type for named type arguments. Numeric constraints such as Dc 405's Natural0&<8 and list element subtype constraints can be ignored. Build cross-backend validation/projection regressions for encoded value 8 and a filepath list containing a boolean. Check baseline behavior before describing either as newly introduced.
- navigator.rs:3069 and :3108 convert literal/identifier payloads to Unicode String during Stage 1. Escaped non-Unicode Dc values allowed by CSV grammar should remain lossless Dc-native data until Rust String projection. Test escaped 261/271, not just escaped Unicode ASCII.
- navigator.rs:2815 recognizes payload openers inside opaque comment framing. Test valid `246 317 248`, escaped closers, and mixed/file/text backends; use generated opaque classification rather than a new hardcoded list.
- /workspaces/ctoolbox/src/formats/dcstring/dc_source.rs:326 always parses with forgiving() and discards successful-recovery diagnostics even when inspection requested strict/quiet policy. :355/:369 also assume one root and call decode_dc_node rather than document-level decode; valid flattened FileTimestamps inspection can fail while whole-document decoding succeeds.
- Macro named enum field decode at lib.rs:3853 propagates conversion errors directly rather than applying malformed_field policy like ordinary structs. Unify shared FieldProjectionSpec behavior and completion validation, not another local catch.
- /workspaces/ctoolbox/src/io/csc/index_engine.rs:794 uses strict journal wrapper in the flush-deleted flow despite allow-journal-errors; /workspaces/ctoolbox/src/io/csc/journal/legacy.rs:1032 unconditionally uses forgiving for embedded identity. Thread caller policy consistently without fixture-specific exceptions.
- /workspaces/ctoolbox/src/io/csc/tests/e2e_journal_v4.rs:510/:649 and challenge_m4_empirical.rs:1276/:1329 append DcMstring.as_ref(), including another DCMM header. Replay may stop before the intended corrupt/truncated batch. Correct fragments to payload_bytes, calculate real hashed spans, and prove the complete untampered control commits before asserting mutation/truncation behavior. New tests must not merely accept an empty strict snapshot for an invalid fixture.
- /workspaces/ctoolbox/src/formats/detection/detection.rs:1221 promotes extension-only structured formats to HighestConfidence using generic compatible text encoding evidence. A random ASCII .cscdesc file is not proof of descriptor syntax. Assess content signature and scoring together; require a negative unrelated-text case. Keep ranking a specific compatible candidate separate from claiming structural certainty.
- navigator.rs:237 sets strict().deprecated_syntax=Warn, contrary to the initial strict policy agreement. Check subsequent operator-approved decisions before changing; otherwise restore the original contract and update all legitimate deprecated-syntax tests explicitly.
- DiagnosticAction handling currently logs/ignores without a comprehensive caller-owned diagnostic sink. Review parser/projection/inspection/journal diagnostic retention as one contract.
- /workspaces/ctoolbox/src/formats/dcdata/syntax/resolver/codegen.rs:684 and resolver.rs:778 may drop invalid grammar during authoritative loading/generation. Test malformed named/category grammar; fail generation contextually before writing a reduced table. The DSL malformed quantifier fallback at syntax/parser.rs:603 appears inherited; verify baseline and report separately rather than treating it as a new regression.

Inherited or approval-sensitive residuals (do not silently expand implementation scope):

- source_edit.rs:164 re-encodes binary equality using only payload, ignoring data_type. Main-agent baseline comparison confirms the old source_edit::Node::write_semantic also omitted type, so this is INHERITED, not a new regression. It is relevant lossless-editing debt; reproduce and track separately, obtaining scope confirmation if not necessary to complete this refactor.
- FileOrigin::Remote uri is String under Dc 337 filepath and etag uses payload checksum semantics. These mappings predate the refactor; stricter validation may expose broken roundtrip. Verify before changing semantic identifiers; no unauthorized new Dc or weakening filepath grammar.
- FileTimestamps::decode_field currently defaults or discards failed conversions (metadata.rs:872/:878); establish production reachability and introduction before severity/ownership decisions.
- AppleMetadata.unrecognized_entries remain omitted; data preservation needs an approved generic representation if no suitable existing semantic representation exists. Do not invent a new semantic ID.
- The cumulative CSV diff includes newly added Dc 562 and semantic changes beyond @ident, including empty containers, flags, filepath, literal/reference rules. Verify operator approval/provenance and semantic invariants, not just self-roundtrip. Do not assert unapproved status merely because approval is absent from the visible chat. [Operator note: Dc 562 is approved. Changes listed in Dc-changes-17-Oct are approved. Any other changes are unknown/TBD/worth asking about.]
- Panorama temporal conversion is explicitly deferred in PROJECT.md; rejection rather than guessed units is the accepted bounded behavior. No implementation of those conversions in this plan.

### Implementation Phases And Agent Ownership

1. Evidence and review closure (main agent, before edits). Re-pin HEAD and worktree, record any new commits separately, finish cumulative review of uninspected changed paths and instructions. Turn each above lead into a confirmed finding, inherited issue, intentional/approved change, or disproved concern. Maintain a completion matrix against every original requirement and a FIXME lifecycle ledger with commit IDs, current locations, and concrete resolution evidence. Ask only about genuine semantic approval or material scope expansion. Do not consider the review complete from subagent reports alone.
2. Parser integrity and resource safety (GPT-6.1 Sol; depends on 1). Own navigator.rs and encapsulation.rs. Fix digest length/algorithm handling, type-constructor recursion/budgets, opaque/escaped framing, lossless literal payloads, and generic constraint enforcement through the owning abstractions. Stage small fixes with focused red/green checks. Do not concurrently assign navigator.rs to another editing agent.
3. Generic grammar generation and projection (two bounded GPT-6.1 Sol tasks; depends on 1, coordinate API changes with 2). Grammar task owns dcdata parser/resolver/validator/codegen and generated table. Projection task owns macro and dc_mixed_serde.rs; repairs flattened state, enum recovery, explicit policies, normalized shape consumption and retained diagnostics. The grammar task can run alongside parser integrity only if interfaces are stable; otherwise serialize. Projection's source-inspection follow-up depends on its document-decoding/diagnostic API. No duplicate helpers or file-specific handwritten codec workarounds.
4. Journal replay and test repair (GPT-6.1 Sol; depends on stable parser/projection APIs in 2-3). Own journal.rs, legacy.rs, args/callers, and existing CSC tests. Implement clear crash-tail versus fatal/recoverable error classification, trustworthy completion, policy propagation, and verified-versus-salvaged integrity reporting. Repair invalid fragment test construction first and add positive controls; retain old fixture bytes and common forgiving parsing.
5. Independent ancillary review (GPT-6 Luna for mechanical diff/FIXME/whitespace/status/dependency checks; GPT-6.1 Sol for detection scoring or semantic design; parallel with 2-3 only with distinct files). Review cumulative detection chain/decompression/CLI regressions, schema semantics/approvals, AppleSingle fidelity, build script, lockfile and vendor pointer. Current small ancillary delta upgrades existing ammonia/rustls and moves CLI compression tests to an explicitly built target binary; vendor delta advances the nested ctb-vendored submodule. Do not edit vendor/ctb-vendored. Separate intended adjacent changes from refactor findings. Detection fixes require negative content tests, not only score-order assertions.
6. Integration and independent re-review (main agent plus GPT-6.1 Sol reviewer; depends on 2-5). Reconcile interfaces and run required affected-crate checks serially on musl with disk checks. Inspect git diff starting-endpoint..HEAD, git diff HEAD, and git status --short, including operator commits. Update existing project/review/test-ready documents with explicit Remaining Gaps vs Original Request and truthful pass/fail/skipped status. No new tracking file when an existing file suffices. No commits/branches unless requested.

Agents must receive exact baseline/endpoint, relevant original requirements, applicable AGENTS.md, files they own, forbiddance on editing neighboring owners' files or vendor/ctb-vendored, expected concrete findings/reproducers, verification results, and unresolved gaps. Use only GPT-6.1 Sol for difficult work and GPT-6 Luna for bounded easy work; no Anthropic models. Keep agents bounded and reports short to avoid context compaction. Planning-agent invocations remain read-only until an implementation handoff is authorized.

### Relevant Full Paths And Reuse Points

- /workspaces/ctoolbox/issues/2026-10-13-Dc-syntax-parser-refactor/plan.md: initial prompt and two feedback rounds define acceptance.
- /workspaces/ctoolbox/issues/2026-10-13-Dc-syntax-parser-refactor/{PROJECT.md,TEST_INFRA.md,TEST_READY.md,to-review,subplan-3-and-review.md}: existing status, test mapping and historical leads; update rather than duplicating, never carry stale findings forward unverified.
- /workspaces/ctoolbox/src/formats/dcdata/syntax.rs; syntax/{parser.rs,resolver.rs,validator.rs,ast.rs}; syntax/resolver/{codegen.rs,syntax_table.generated.rs}; /workspaces/ctoolbox/src/formats/dcdata/build.rs: CSV-to-generated-resolver path, SyntaxRuleResolver, named argument substitution, fail-fast generation and freshness checks.
- /workspaces/ctoolbox/src/formats/dcstring/navigator.rs: DcDocumentNavigator::next_sibling_stub, scan_delimited_container, verify_checksummed_section, eval_term_single, DcSyntaxArena normalization, binary spans and policy budgets.
- /workspaces/ctoolbox/src/formats/dcstring/encapsulation.rs: ChecksumSpec, SupportedHashFormat and StreamingHasher; reuse supported integrity codecs instead of duplicating algorithms.
- /workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs and dc_mixed_serde.rs: DcMixedFlattened, FieldProjectionSpec and DcProjectionContext; shared field state, policy handling, document root consumption and diagnostics.
- /workspaces/ctoolbox/src/formats/dcstring/{dc_source.rs,source_edit.rs}: inspection/document projection and lossless CST-backed edit reconciliation.
- /workspaces/ctoolbox/src/io/csc/{journal.rs,journal/legacy.rs,index_engine.rs,args.rs}: replay/recovery/resume and CLI policy propagation.
- /workspaces/ctoolbox/src/io/file/{identity.rs,metadata.rs,streams.rs,link_path.rs} and /workspaces/ctoolbox/src/formats/apple_single_double/apple_single_double.rs: typed metadata derives, binary/date/path fidelity, existing semantic adapters.
- /workspaces/ctoolbox/src/formats/detection/{detection.rs,chain.rs,decompression.rs,cli.rs}; /workspaces/ctoolbox/src/formats/dcdata/data/: ancillary detection and ontology review.

### Verification And Acceptance Gates

1. For every confirmed bug, add the smallest regression in the existing nearby test module/target. Demonstrate the failing behavior before its fix where practical, then run that same narrow test immediately after the edit before reading/editing another slice. No new listening ports; use memory/socketpair mocks.
2. Parser tests: extend test_dc_syntax_resolver_and_expansion_bounds; test_tier5_checksummed_section_tamper_detection; test_strict_document_syntax_validation; test_challenge_named_types_with_format_rules; test_literal_string_escape_255_roundtrip; test_challenge_framing_rejection_across_all_modes; test_syntax_table_codegen_smoke; test_static_syntax_table_matches_csv_dataset. Include text/mixed/file parity, small explicit budgets, bounded allocation counters or sparse-file spans without actual GiB allocation, supported hash positives and mismatch negatives, and high-budget nonrecursive parsing.
3. Projection tests: extend test_challenge_flattened_document_projection_boundary; test_challenge_file_timestamps_multi_root_decode; test_source_inspection_unknown_and_incomplete; test_timestamp_validation_preserves_semantic_components; named-enum malformed-field tests. Assert strict missing-field errors, forgiving diagnostics/defaults only under chosen policy, root consumption, and quiet mode with diagnostics retained. Add binary type-change edit test only if inherited issue is accepted in scope.
4. Journal tests: positive complete batch control before corrupt/truncated variants; escaped OperationCompleted must not complete; new batch invalidates old completion; valid crash-tail rollback remains; well-framed interior policy failures cannot silently truncate later good batches; forgiving checksum salvage is explicitly diagnosed; all CLI paths honor allow-journal-errors. Old v4 fixture strict syntax rejection and ordinary forgiving partial salvage remain separate from crash recovery.
5. Detection tests: real CscDesc signature/content positive, arbitrary ASCII with .cscdesc negative for structural confidence, existing generic text/extension candidates, compression fallback and chain CLI behavior. Do not weaken production evidence requirements solely to match old expected scores.
6. Linux musl focused commands, one build platform only and at least 25 GB free: env RUSTFLAGS='' RUSTDOCFLAGS='' cargo test --locked --target x86_64-unknown-linux-musl -p ctb-formats-dcstring <focused-filter>; analogous commands for ctb-formats-dcdata, ctb-dc-macro, ctb-io-csc, ctb-io-file, ctb-formats-apple-single-double, ctb-formats-conversion, ctb-formats-detection. Existing integration target names/filters must be checked before invoking; --lib alone does not include explicit integration tests.
7. After focused fixes pass, run affected packages with --lib --tests --no-fail-fast (only the crates actually touched, plus their contract consumers). Run cargo clippy on those packages with --lib --tests --no-deps and explicit musl target, without added warning suppression. Clippy errors block; known warning-only baseline is reported distinctly. Reuse current build artifacts; do not cargo clean unless required by a confirmed stale asset failure or target switch, and never during another build.
8. Missing vendored dependencies are a hard stop: report to operator; do not add packages, revert dependency upgrades, or edit autogenerated vendored dependencies to make testing pass. No full-workspace build merely for reassurance given current disk headroom. Non-Linux behavior requires targeted native/cross-platform follow-up if code changes touch it; compile-only is not runtime evidence.
9. Final read-only gates: bash -n build if still touched; cumulative and worktree diff checks; schema generated-table equality; all FIXME ledger rows explicitly closed or retained as unresolved; truthful documentation of failing/skipped/unrun gates. Do not claim O(1) total journal memory while JournalSnapshot intentionally retains all entities: distinguish O(per-record) parser overhead from O(retained snapshot) application memory.

### Decisions And Explicit Deferrals

- Review cumulative changes, not 49 individual commits, except FIXME audit and necessary provenance checks.
- Preserve intended incomplete-tail crash recovery; fix the conflation with malformed committed/interior records rather than banning all rollback.
- Existing original-plan strict-deprecation contract is the default recommendation unless subsequent operator approval establishes otherwise.
- Resolve semantic meaning/approval questions before changing IDs or grammar; no new Dc IDs or dependencies are authorized by this plan.
- Defer unsupported Panorama conversion, GUI/editor development, global zero-allocation static residency, full workspace/platform certification, and a streaming database-backed JournalSnapshot redesign. Report these limits honestly; do not label the original feature complete while required parser/projection/diagnostic gaps remain.
