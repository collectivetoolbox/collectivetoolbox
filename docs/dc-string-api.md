# Source-Backed Dc String API

A Dc document is a Dc string being operated on, not a separate storage type.
Keep the original `DcString`/`DcStr` or `DcMstring`/`DcMstr` as the source of
truth. Typed values are interpretations; serialization creates new strings,
not lossless reconstructions of existing ones.

## Decoding and Inspection

Both borrowed string types provide:

- `decode::<T>() -> Result<T>`: interpret the entire string, rejecting unknown
  syntax, incomplete records, and trailing content.
- `decode_with_context::<T>(&context)`: the same operation with explicitly
  supplied outer reference bindings.
- `inspect::<T>() -> DcInspection<T, ...>`: borrow the source, retain successful
  field annotations, and report the first interpretation failure.
- `inspect_with_context::<T>(&context)`: inspection with outer bindings.

Owned strings expose these operations through their existing borrowed views.
Inspection returns a view even when decoding fails. `value()` is present when
the typed value decoded successfully; check `diagnostic()` separately because
trailing content can remain. `source_bytes()` always returns the original bytes.

`DcMixedEncode::to_dc_mixed()` and `to_dc_string()` explicitly create new
strings. Conversion between mixed mode and DcUtf is not implicit when saving
or editing an existing string. Direct binary payloads are borrowed when read;
textual base64 encapsulations require an owned decoded byte buffer.

## Annotations and Edits

Tagged and keyed record derives produce `DcMixedFields` implementations and
source annotations during inspection. Each annotation includes its fully
qualified Rust record type, Rust field name, nesting depth, and original byte
range including its tag or map key. Mixed-mode offsets include the format
header. Nested annotations may overlap their containing field annotations.

For example, with an environment source and a replacement typed record:

```rust
let view = source.inspect::<EnvDescription>();
let updated = EnvDescription {
    os: "revised".to_owned(),
    ..EnvDescription::default()
};
let revised = view.replace_field("os", &updated)?;
```

Only the supplied `os` field is encoded. Other values in `updated` are not used.
The source may contain unknown or incomplete syntax after the annotated field.
No field insertion is inferred when `os` is absent or could not be decoded.

`replace_field()` selects a unique outer field. `replace_annotation()` accepts
an annotation borrowed from the same inspection and a record of the annotated
type, allowing explicit nested or repeated-occurrence selection. For repeated
fields, the supplied collection replaces only the selected occurrence; other
occurrences remain intact. A field encoder that omits an absent/default value
can produce an empty replacement, deleting that occurrence.

Edits return a new owned string of the original representation and preserve
every byte outside the selected range. Reinspect the new string for fresh
annotations. Borrowed views prevent mutating their source while in use;
annotations from another inspection are rejected.

Replacement values inline shared references rather than changing their outer
declarations. Edits reject source boundaries inside mixed-mode encapsulations,
including checksummed DcUtf payloads. Whole binary fields can be replaced with
new framing and checksums. No unknown syntax is collected and moved to an
overflow bucket.

This is a byte-preservation contract, not a promise that unknown syntax retains
its meaning when a value it depends on is deliberately edited.

## Runtime Boundary

Typed inspection is non-executing source analysis. Its diagnostics are not a
string-validity verdict and must not gate normal display or execution. The
runtime remains permissive and capability-gated, as described in
[Renderer Flow](renderer-flow.md).

Environment JSON remains a strict debugging representation: unknown fields
fail deserialization. There is no JSON overflow map or embedded JSON payload.

## Pending Work

- Inspection stops at its first error; it does not infer unknown syntax
  boundaries or resume after malformed/incomplete constructs.
- There is no generic insertion of absent fields, batch edit transaction,
  persistent annotation identity, incremental reinspection, or GUI integration.
- Automatic field annotations cover tagged and keyed record derives. Custom
  handwritten codecs and enum variants do not expose editable subfields unless
  they explicitly add annotations; a successfully decoded enclosing record
  field can still be replaced as a whole.
- Editing inside encapsulated DcUtf requires a future nested source view that
  deliberately rebuilds its enclosing framing/checksum; it is refused today.
- This API is not a DOM, evaluator, or guarantee of universal semantic
  roundtripping through a typed struct.