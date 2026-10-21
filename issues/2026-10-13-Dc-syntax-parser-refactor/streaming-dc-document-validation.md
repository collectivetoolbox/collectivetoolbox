# Dc Streaming Validation Boundaries

Implementation reference, not test certification. Pending work is tracked in
the [Dc parser checklist](../dc-parser.md).

Full grammar validation currently needs seekable input. Forward-only byte
transport validation and typed-record validation provide different guarantees
in [ctb_formats_dcstring](../../src/formats/dcstring/dcstring.rs).

---

### 1. File-Backed Grammar Validation

`validate_dc_mixed_strict` delegates to the navigator:
```rust
pub fn validate_dc_mixed_strict(mstr: &DcMstr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    nav.validate_document_strict()
}
```

For a seekable file containing a DcMM document at `base_offset`, use
`DcDocumentNavigator::from_dcmm_file` in
[the navigator](../../src/formats/dcstring/navigator.rs):

```rust
let mut nav = ctb_formats_dcstring::DcDocumentNavigator::from_dcmm_file(file, base_offset)?;
nav.validate_document_strict()?;
```

* **How it streams**: Backed by a `BufReader<File>`, reading chunks on demand.
* **Limitation**: Requires a seekable `std::fs::File` (seeking is necessary to resolve container spans and delimiters).

---

### 2. Forward-Only Transport Validation

For arbitrary `Read`/`Write` streams, use the copier in
[dc_mixed](../../src/formats/dcstring/dc_mixed.rs):

```rust
pub fn validate_and_copy_dcmm_stream<R: std::io::Read, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64>
```
Custom options are available via `validate_and_copy_dcmm_stream_with_options`.

* **How it streams**: Reads and writes in 64 KiB buffers with bounded memory.
* **What it validates**:
  * The 21-byte `.dcmm` file header (`DCMM_MAGIC` + `DCMM_UUID`).
  * Encapsulation framing: Dc 203 header formats, payload lengths, and SHA-3 / SHA-256 checksums.
  * UTF-8e-128 text encoding byte sequences.
* **Limitations**:
  1. Expects a full `.dcmm` stream including the 21-byte header (not raw `record.payload_bytes()`).
  2. Validates transport and encapsulation framing only—it does **not** evaluate high-level grammar or check paired container delimiters (like lists or maps).

---

### 3. Validation While Encoding Typed Records

`validate_and_write_dc_record` in [dc_writer](../../src/formats/dcstring/dc_writer.rs#L194) accepts
a callback emitting through `DcWriter`. It streams binary bodies, verifies
their framing and supplied checksums, and spools the exact emitted bytes to an
anonymous file. The existing file-backed strict navigator validates that file
before the helper returns success. Binary bodies are not expanded into Dc
characters, but temporary disk space scales with record size and disk I/O is
added. Do not replace bodies with empty encapsulations: that changes length
constraints and section checksums. This is not an arbitrary byte-stream
validator: raw writes through the validating writer are rejected.

`DcMixed` derives now generate writer-based encoding. Buffered entrypoints
delegate to it. A separate `#[dc(encode_only)]` attribute permits borrowed
encoding views without generating decoding implementations. `DcBinarySource`
supplies readers for nonresident `LazyBinaryPayload` values; missing sources
and failed length/digest checks are errors.

### CSC Integration

`JournalWriter::write_batch` and `write_entity_checked` both use the derived,
source-aware [file encoder](../../src/io/file/encoding.rs). `BodyPolicy::Omit`
explicitly omits primary and attached bodies recursively while retaining
identity metadata. `BodyPolicy::Include` fails on missing attached sources.
Do not use empty readers or rewrite filesystem origins to request omission.

Validation finishes before the checksummed-section footer is emitted. Encoding
failures may leave a partial section; existing rollback and failed-writer rules
still apply. `DcBinarySourceError` classifies source failures, while an observed
sink failure always prohibits source-only rollback.