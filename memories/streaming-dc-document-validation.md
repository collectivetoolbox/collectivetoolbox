Ran command: Search validate_dc_mixed_strict (`git grep -n "validate_dc_mixed_strict"`)
Ran command: Find definition of validate_dc_mixed_strict (`git grep -n "fn validate_dc_mixed_strict"`)
Viewed navigator.rs:4470-4520
Ran command: Search stream in dcstring (`git grep -i "stream" src/formats/dcstring/`)
Viewed stream.rs:140-165
Viewed stream.rs:40-75
Viewed stream.rs:220-275
Ran command: Search validate_and_copy_dcmm_stream (`git grep -n "validate_and_copy_dcmm_stream" src/formats/dcstring/`)
Viewed dc_mixed.rs:1320-1390
Ran command: Find validate_document_strict (`git grep -n "fn validate_document_strict" src/formats/dcstring/`)
Viewed navigator.rs:4335-4420
Ran command: Find DcDocumentNavigator impls (`git grep -n "impl.*DcDocumentNavigator" src/formats/dcstring/`)
Viewed navigator.rs:1040-1130
Ran command: Search from_dcmm_file usages (`git grep "from_dcmm_file"`)
Viewed stream.rs:1-140
Ran command: Search pub fn validate in dcstring (`git grep -n "pub fn validate" src/formats/dcstring/`)
Ran command: Search validate in src/formats (`git grep -n "validate" src/formats/`)
Viewed dcstring.rs:1-75
Ran command: Search NavBackend definition (`git grep -n "enum NavBackend" src/formats/dcstring/`)
Viewed navigator.rs:995-1040
Viewed dc_mixed.rs:1170-1230
Viewed dc_mixed.rs:1360-1460
Viewed dc_mixed.rs:1460-1530
Ran command: Search streaming and validation (`git grep -i "streaming" | grep -i "validat"`)
Viewed dc-string-api.md:1-100
Viewed dc-string-api.md:101-140
Ran command: Search for StreamingValidator (`git grep -i "streamingvalid"`)
Ran command: Search validate.*stream in src (`git grep -in "validate.*stream" src/`)
Viewed convert_stream.rs:165-190
Viewed stream.rs:135-225

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

### In the Context of [`JournalWriter`](file:///workspaces/ctoolbox/src/io/csc/journal/stream.rs#L131-L157)

In `stream.rs`:
- [`write_batch`](file:///workspaces/ctoolbox/src/io/csc/journal/stream.rs#L131-L157) buffers metadata entities into a local `record: DcMstring` and validates them in memory with [`validate_dc_mixed_strict`](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L4480) before emitting `record.payload_bytes()`.
- When streaming large entity payloads, [`write_entity_checked`](file:///workspaces/ctoolbox/src/io/csc/journal/stream.rs#L167-L221) avoids in-memory buffering by encoding directly to the writer and validating payload digests incrementally on the fly with [`Sha256Stream`](file:///workspaces/ctoolbox/src/formats/checksum/checksum.rs).