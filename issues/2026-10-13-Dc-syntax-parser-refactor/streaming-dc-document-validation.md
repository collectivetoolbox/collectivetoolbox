
There is no single 1:1 streaming replacement that works over an arbitrary forward-only `std::io::Read` / `Write` while still performing full Document Character grammar validation. However, depending on what level of validation and what type of stream you have, the codebase provides two different streaming equivalents in [`ctb_formats_dcstring`](file:///workspaces/ctoolbox/src/formats/dcstring/dcstring.rs):

---

### 1. File-Backed Streaming Syntax Validation: [`DcDocumentNavigator::from_dcmm_file`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L1087-L1127)

[`validate_dc_mixed_strict`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L4480-L4483) is actually a thin wrapper:
```rust
pub fn validate_dc_mixed_strict(mstr: &DcMstr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    nav.validate_document_strict()
}
```

If you have a file on disk (such as a `.cscjournal` or `.dcmm` file) and want to validate full Dc grammar, container delimiter pairing, and syntax rules without loading multi-gigabyte contents into an in-memory [`DcMstring`](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed.rs#L540), you can use [`DcDocumentNavigator::from_dcmm_file`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L1087):

```rust
let mut nav = ctb_formats_dcstring::DcDocumentNavigator::from_dcmm_file(file, base_offset)?;
nav.validate_document_strict()?;
```

* **How it streams**: Backed by a `BufReader<File>`, reading chunks on demand.
* **Limitation**: Requires a seekable `std::fs::File` (seeking is necessary to resolve container spans and delimiters).

---

### 2. Forward-Only `Read`/`Write` Streaming Validator: [`validate_and_copy_dcmm_stream`](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed.rs#L1329-L1339)

For arbitrary `R: std::io::Read` and `W: std::io::Write` streams (e.g. pipes, sockets, or non-seekable streams):

```rust
pub fn validate_and_copy_dcmm_stream<R: std::io::Read, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64>
```
(or with custom options via [`validate_and_copy_dcmm_stream_with_options`](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed.rs#L1353-L1360)).

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

`validate_and_write_dc_record` in `src/formats/dcstring/dc_writer.rs` accepts
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
source-aware file encoder in `src/io/file/encoding.rs`. `BodyPolicy::Omit`
explicitly omits primary and attached bodies recursively while retaining
identity metadata. `BodyPolicy::Include` fails on missing attached sources.
Do not use empty readers or rewrite filesystem origins to request omission.

Validation finishes before the checksummed-section footer is emitted. Encoding
failures may leave a partial section; existing rollback and failed-writer rules
still apply. `DcBinarySourceError` classifies source failures, while an observed
sink failure always prohibits source-only rollback.