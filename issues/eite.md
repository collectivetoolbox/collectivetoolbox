# EITE

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
