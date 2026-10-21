# EITE and Dc Follow-Ups

## Encoding and Editor

Historical reports below are not all reproduced on current builds. Conversion
and escape handling already exist; these are correctness and coverage gaps,
not instructions to implement a second codec.

- [ ] Make Basenb armor independently movable: emit maximal unmappable runs
    with their own start/end markers, ending before the next mappable character.
    The EITE UTF-8 converter still closes its armored region at end-of-stream.
- [ ] Preserve private-use characters overlapping Basenb ranges, including bytes
    decoded from base64 encapsulation; avoid interpreting literal text as armor.
- [ ] Cover an escaped, otherwise-valid base64 encapsulation and an escaped
    escape Dc. Existing escape state is not proof of correct roundtrips.
- [ ] Audit UTF-8 UUID conversion in both directions and the reported all-zero
    UUIDs for general binary encapsulation. Reuse existing binary conversion and
    tests; establish what remains missing rather than assuming it is absent.
- [ ] Implement ELAD beyond its current ASCII placeholder.
- [ ] Reconcile execution fields still in `EiteState` with the existing
    `ctb-runtime` crate. Do not create another runtime crate. Reduce the large
    UTF-8 conversion routines as part of correctness work, not a parallel rewrite.
- [ ] Reproduce the EITE import RPC failure below and retain a regression.
    The original report returned `from_code_point: unknown code point [63489]`.

`POST /api/rpc/eite`:

```json
{"function":"importDocument","args":["utf8",[244,143,191,187,239,160,129,244,143,191,182,239,160,129,244,131,157,169,239,159,185]]}
```

## Dc Syntax Merge Checklist

Pending reconciliation with the active
[Dc syntax project](2026-10-13-Dc-syntax-parser-refactor/). These preserve
operator requirements from the inbox; they do not assert that the current
parser still fails every case. Merge against that project's results before
opening duplicate implementation work.

- [ ] Reject ambiguous database syntax during validation and ambiguous derive
    mappings at build time. Consider generated Dc identifiers for struct
    annotations when that improves diagnostics without duplicating metadata.
- [ ] Resolve semantic roles through generated tables and `<equiv>` relations,
    not bare Dc-number allowlists in readers or derive output. Equivalent tokens
    with conflicting syntax must fail validation, not inherit an arbitrary rule.
- [ ] Accept equivalent/partially equivalent representations wherever the value
    permits them, not only in specially named fields. Preserve escape-Dc handling
    when values collide with surrounding syntax. Cover timestamp resolution and
    extent fields as regression examples.
- [ ] Reject negative values for `Natural0`.
- [ ] Reconcile the `f632` format-reference cap introduced in `e378fd1bc`:
    distinguish recognized formats from allocated reference space. If the cap
    tracks defined formats, generate it from the database rather than hardcode it.
- [ ] Reconcile remaining reader migration in `dc_char.rs`,
    `dc_number_minimal.rs`, `format_spec/dc_stream.rs`, and the navigator. Replace
    semantic token comparisons with resolver/role queries; do not repeat already
    migrated parsing paths.
- [ ] Audit remaining handwritten encapsulation/serialization paths for shared
    generated grammar use. Arena parsing and derive projection already exist;
    fully generated serialization was not established by the earlier update.
- [ ] Verify runtime-produced document streams through projection and splicing.
    The earlier design splices evaluated streams rather than mutating the borrowed
    CST arena; do not revive synthetic-node insertion without an actual gap.
- [ ] Recheck the reported
    `test_rule_and_delimiter_pairing_when_non_canonical_or_target_owns_rule`
    failure: the fixture cloned syntax closing with Dc 311 while declaring a
    different `@closes` target (960). This is a historical report, not a fresh
    failing-test claim.
- [ ] Clarify downstream handling of multi-token `<equiv>` decompositions
    (for example `f350 f351`), separately from the single-token equivalence set
    and the ban on attaching single-token syntax/closers to decompositions.
- [ ] Review ontology questions separately from parser bugs: encoding formats'
    relationship to Text (`f14`), and the distinction between String (`f80`) and
    Text. Do not change stable Dc meanings during cleanup.
- [ ] Reconcile old file metadata derive concerns: flag representation under
    Dc 342 and the reported read-only/reparse tag confusion (422/423); replace
    handwritten `FinderLabel` numeric serialization with a general derive when
    supported. Coordinate with the file API timestamp/unknown-field follow-ups
    in [csc.md](csc.md).
