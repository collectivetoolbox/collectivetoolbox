# Dc Parser Refactor References

The active backlog is [issues/dc-parser.md](../dc-parser.md).
The refactor remains **Partially Implemented**. Superseded implementation plans,
duplicated review lists, agent assignments, speculative API sketches, and test
certification documents were consolidated on 2026-10-21. Their full history is
available in Git before that consolidation; old test counts are not current
verification results.

## Original Operator Requirements

- Replace independent document parsers with one database-driven parser and
  project its parsed output onto annotated Rust types. Refactor APIs rather than
  preserving redundant implementations.
- Preserve open container scope when newer/unknown children appear. An invalid
  child must not cause the enclosing opener to disappear and leak fields into
  its parent.
- Require explicit caller policy: journals default to strict, with ordinary
  forgiving recovery when requested. Strict was specified to error on deprecated
  syntax too; current behavior differs and remains tracked.
- Keep serde focused on generic projection and unused/missing-field reporting,
  not repeated grammar or legacy-format decisions.
- Use a general level-aware navigator/seeker, not a journal-only iterator.
  Future viewport editing and very large journals both need bounded parsing
  memory and lazy file-backed binary spans.
- Preserve the original malformed development fixtures unchanged. Strict mode
  should reject invalid syntax; forgiving mode may salvage only part of it.
  Do not immortalize serializer bugs with fixture-specific token rewrites.
- Dc documents have multiple representations; `.dc` is not their extension.
  `.dcmm` is the usual mixed-mode on-disk representation.
- Caller-controlled resource limits must allow safe processing of deeper
  documents when authorized, without stack overflow or infinite loops.

## Approval and FIXME Provenance

- The operator approved Dc 562 and the database changes recorded in
  [the October 17 CSV audit](Dc-changes-17-Oct). Dc 563 (File data format) and
  Dc 564 (Archive member position) were also explicitly approved. Do not infer
  approval of unrelated semantic changes from these records.
- The flattened-format FIXME predates the refactor, was temporarily removed,
  then restored. Its semantic question remains open, not resolved by restoration.
- The routine-invocation FIXME is inherited; generated/quoted copies are not
  separate tasks.
- The macro-layer FIXME was removed after shared map extraction, but remaining
  macro-owned legacy/deprecation decisions mean the boundary is only partially
  addressed. Track the behavior rather than restoring obsolete wording.
- Unsupported Panorama temporal conversion was explicitly deferred. Returning
  an unsupported-format error rather than guessing units is the accepted bounded
  behavior, not completed conversion support.

## Streaming Reference

[Streaming validation boundaries](streaming-dc-document-validation.md)
distinguishes seekable grammar validation, forward-only transport validation,
and typed writer validation with anonymous-file spooling. It is an implementation
reference, not another checklist or feature certification.