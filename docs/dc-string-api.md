# Source-Backed Dc String API

A Dc document is a Dc string being operated on, not a separate storage type.
Keep the original `DcString`/`DcStr` or `DcMstring`/`DcMstr` as the source of
truth. Typed values are interpretations; serialization creates new strings,
not lossless reconstructions of existing ones.

Editing or saving an existing document must never automatically upgrade
deprecated syntax, normalize unknown Dcs, or reconstruct the document from a
typed value. Syntax upgrades are separate, explicitly accepted user actions.
Any valid Dc encoding remains editable through source-range replacement,
regardless of whether its Dc sequence satisfies a recognized syntax rule.
Untouched bytes, including malformed syntax and unknown Dcs, remain verbatim.

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

## Format Conversion and Application Serialization

Document format conversion must preserve Dc identities and ordering, including
deprecated characters, without evaluating syntax or applying preference links.
Changing the transport encoding is not permission to upgrade document syntax.
For example, converting DcIntegerList to DcUtf must retain Dc 313 rather than
replace it with Dc 559. Source-format whitespace and encoding bytes need not be
identical across different formats; the decoded Dc sequence must be.

Application serialization is a different operation. Decoding metadata into
Rust structs and serializing a new value does not promise preservation of the
original document. An editor operating on that same file must use its source
bytes and replace only the selected filename range, not serialize the whole
metadata struct. Dependent checksums or encoded lengths require an explicit
enclosing-format editing contract; they cannot be silently left invalid or
claimed to remain byte-identical after changing covered content.

### Remaining Conversion Gaps

- DcIntegerList conversion preserves deprecated map and routine Dcs; regression
  coverage includes Dc 313, Dc 265, explicit pairs, and an unknown Dc.
- Mixed-mode validation already rejects ordinary top-level textual
  `203 ... 204`: Dc 203 must introduce a typed, length-delimited encapsulation.
  This does not make conversion byte-roundtrippable in both directions.
- Mixed-mode export to textual Dc formats now verifies reimport against the
  original bytes and rejects loss of checksums or run-type information.
  Checksummed runs are valid mixed-mode input, but cannot currently be exported
  losslessly to these formats. They produce an explicit conversion error.
- DcUtf import interprets `203 ... 204` regions, validates the mixed-mode
  result, and verifies exact DcUtf re-export. Unclosed Dc 203 or base64 spelling
  that would be normalized is rejected rather than silently altered.
- The low-level `DcMstr::to_dc_string()` remains a representation projection
  used by application serialization; it is not a lossless document-export
  contract. Document converters use the checked conversion path.
- Lossless representations for all validly encoded Dc sequences and all
  mixed-mode run metadata remain pending. No new encoding or semantic Dc is
  allocated here. Unsupported conversions fail explicitly; arbitrary source
  sequences remain raw-editable in their original format.

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

## Key-Value Arrays and Pairs

New `BTreeMap` and `#[dc(key_value)]` encodings use Dc 559..560. The header
declares Object key and value types (Dc 275), followed by adjacent explicitly
closed pairs:

```text
559 275 275 514 key1 312 value1 515 514 key2 312 value2 515 560
```

There is no separator between pairs. Dc 514 identifies the key-value pair,
Dc 312 separates its key and value, and Dc 515 closes it. Pair operand regions
retain their historical nonempty syntax, including nested framed data; the
tuple rule now consumes its declared closing delimiter. This does not change
the separate Rust `(A, B)` codec, which still concatenates its operands.

Readers also accept deprecated Dc 313..314 arrays with alternating keys and
values separated by Dc 312. The opening Dc selects the interpretation; broken
559 arrays are never retried as flat 313 arrays. Fresh serialization of a typed
value uses 559, but it is not a document editing or saving operation. Editors
must retain and splice the original source; they must not save by decoding and
re-encoding it. An upgrade from 313 to 559 may only be offered as a separate
operation requiring explicit user acceptance.

Keyed-field annotations include both pair boundaries in new arrays. Field
replacement preserves legacy framing when editing a legacy array and leaves
all bytes outside the selected field unchanged. `encode_dc_field()` creates a
new framed pair; `encode_dc_map_field()` can explicitly encode a legacy field.
Typed replacements reconcile their proposed fragment against the selected
source range, including when replacing a whole field containing nested maps.
Existing maps retain their opening/closing Dcs and surviving entry order.
Entries are matched by their source-independent key representation; unchanged
subtrees and opaque binary payloads are copied byte-for-byte. Added entries
are appended without upgrading the containing map. Nested maps retain their
own framing independently, including mixtures of 313 and 559 arrays.
Ambiguous correspondence or unsupported map headers produces an explicit error
rather than guessing or silently canonicalizing. Use a more specific nested
annotation or an explicit source-range edit in that case.

The database grammar allows an optional length counting pairs and declared
key/value types. The typed Rust map codecs currently accept only uncounted
Object/Object headers. Dependent type checks, length validation, and recovery
of ambiguous damaged operands remain pending; explicit framing improves local
recognition but cannot reconstruct lost declarations or distinguish structural
markers from quoted payload after arbitrary damage. Copy inherited type
declarations with pairs extracted as independently interpreted fragments.

## File Paths

Derived fields with path values use the shared `PathBuf` codec. File identity
keeps its relative path under Dc 337 and its enclosing path under Dc 357;
native bytes and the final filename are obtained from those values rather
than stored in separately mutable fields.

Paths are Dc 511 representations of format f229: typed lists of filename
representations (f228) and explicit Dc 513 delimiters. Filename bytes remain
binary; when they are valid UTF-8, Dc 397 groups the equal text and binary
representations. Both lists carry type headers. Different values are never
declared equivalent. Unix backslashes inside filenames remain filename bytes,
not delimiters. Windows delimiters are reconstructed as native backslashes,
including verbatim and UNC prefixes.

Native Windows bytes use checked WTF-8 decoding so unpaired UTF-16 surrogates
are not replaced. Bytes that cannot be represented on the destination platform
produce an error rather than a lossy path. The original Dc source remains the
preservation authority for foreign representations that cannot be interpreted
as a local `PathBuf`. This codec is not a platform-independent materialization
policy or a promise to preserve the lexical spelling of Windows separators.

The decoder accepts the previous whole-string/binary path representation for
read compatibility, but rejects conflicting equivalents. The CSC journal
retains its existing path/filename byte slots; new writes derive both from the
same path and reads reject contradictory stored filenames. Attached stream
names belong to `AttachedStream.name`, not to a second file identity name.

New journal preservation records use `CTBMETA3` and include the identity's
derived semantic Dc representation. This retains absent versus empty paths,
enclosing paths, and non-filesystem origin provenance that the legacy byte
slots alone cannot express. Reads require that identity and check its primary
path against the legacy slot. Older readers reject the new marker rather than
silently losing this information. Old records remain readable, but missing
semantic identity produces both a logged warning and a retained metadata
warning; information already omitted by old writers cannot be recovered.

Identity JSON rejects unknown fields, including the removed duplicate raw
fields, rather than silently discarding them. Legacy JSON exports containing
those fields require an explicit migration; they are not accepted lossily.

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
- Map codecs accept uncounted Object/Object headers only. General operand
  grammars, dependent key/value type checks, pair-count validation, and
  ambiguity-aware recovery after damaged framing remain pending. Historical
  unclosed pairs are malformed but can still be inspected permissively with
  diagnostics; no compact pair spelling is emitted or accepted as strict syntax.
- Typed source reconciliation is not a universal semantic editor: ambiguous
  frame correspondence and unsupported map headers are rejected. Raw
  source-range edits do not require syntax recognition. No automatic syntax
  migration or user-facing upgrade command is provided.
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