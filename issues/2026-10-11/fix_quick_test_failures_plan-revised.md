# Implementation Plan: Fix `./test --quick` Test Failures (Revised)

## Goal Description
Resolve all 27 test failures identified during `./test --quick` across `ctb-formats-conversion` (3 tests) and `ctb-io-csc` (24 tests), incorporating operator feedback:
1. **Isolate Legacy Deserialization in `legacy.rs`**: Clean legacy JSON representations (e.g. `{"source_os": "linux", "raw_value": 0, "has_unparsed_flags": false}`) entirely within `src/io/csc/journal/legacy.rs` before decoding into `FileMetadata`, avoiding any pollution or legacy handling in `src/io/file/metadata.rs`.
2. **Cross-Platform Lossless Link Path Representation**: Rather than forcing a host `PathBuf` (which can alter or reject non-host paths when moving between Linux and Windows), introduce a lossless `LinkPath` representation that stores raw path bytes/segments cross-platform, provides methods to adapt to `PathBuf` on materialization, and implements `DcMixed` serialization adhering strictly to Dc 339 `:~ [filepath]`.
3. **Constrained Finder Codes**: Dc 403 and Dc 404 accept exactly four printable
   ASCII characters or exactly four decoded bytes:
   `:~ ([string:[asciiprintable]{4}] | [data:bytes=4])`.
4. **Hardlink Relative Path Correction**: Fix `hardlink_map` in `copy_engine.rs` to store destination-relative paths (`rel`) instead of full host paths (`dest_dir.root_path().join(rel)`), preventing sandboxing rejections.
5. **StreamName Dc 338 Syntax Compliance**: Encode Windows UTF-16 units as
   WTF-8-typed `[data]`, satisfying `:~ ([string] | [data])` without losing the
   source encoding. A bare string would decode as `StreamName::Bytes`.

## Review Corrections and Validation — 2026-10-12

The original fix commit `e52ce7867` passed the existing affected suites but did
not fully satisfy the losslessness and legacy-isolation requirements.

### Corrections

- Preserve exact link bytes, including repeated/trailing separators, UNC
  prefixes, drive-relative paths, unpaired Windows surrogates, and arbitrary
  Unix bytes. A source convention plus an absolute flag is not enough if empty
  segments or separator spelling are discarded.
- Pair a typed filepath and exact bytes under Dc 561 (partially equivalent
  representations). The raw representation supplements, not replaces, the
  typed filepath. Decode and JSON roundtrips do not depend on the host OS;
  incompatible materialization fails explicitly.
- Share filepath segment codecs with `PathBuf`; reject malformed filenames,
  missing delimiters, and conflicting alternative representations.
- Keep CSC compatibility out of the generic derive macro. Adapt legacy JSON
  flags and historical v4 flag maps in `journal/legacy.rs`; preserve original
  source positions for checksum validation.
- Preserve the generic binary-field trait contract: historical byte lists are
  decoded into a byte buffer before field conversion, without requiring the
  field type to implement `DcMixedDecode`. Regression coverage uses a binary
  newtype with only `AsRef<[u8]>` and `TryFrom<&[u8]>`.
- Correct Windows stream metadata: `f2` is Dc Integer List, not UTF-16. Reuse
  the existing WTF-8 codec for lossless units. Historical incorrectly tagged
  little-endian units are adapted only at the journal boundary; odd byte
  lengths remain opaque with their original declared format and bytes.
  Malformed or noncanonical WTF-8 is likewise retained without normalization.
  Interpretation and filesystem use fail explicitly, not archival reads.
- Keep hardlink targets destination-relative without rebasing them through a
  canonical absolute root. Do not treat two canonicalization failures as the
  same file.
- Reject malformed string subtype syntax and depth-limit violations, propagate
  diagnostics, and enforce exact ASCII/length constraints even without a
  caller-provided resolver.
- Replace the ambiguous text/Base64 JSON helper with text/byte-array encoding.
  Previously non-UTF-8 bytes became Base64 text that deserialization treated as
  literal filename bytes.
- Remove leftover journal debug output.

### Verification Status

Verified on the Linux host:

- 464 library tests across conversion, DcData, DcString, CSC, and file crates:
  no failures or skipped tests.
- Targeted Clippy for the affected Rust crates and derive macro: passed.
- Authentic v1, v2, v3, and v4 journal fixtures remain readable.
- Regression coverage includes exact cross-source path roundtrips, strict Dc
  339/338 syntax, relative-destination hardlinks, malformed encodings,
  conflicting historical flags, and constrained-string rejection cases.

### Pending Work / Remaining Verification

- Full Finder-record strict validation still has a pre-existing Dc 405
  label-representation failure. The filename and rectangle regression checks
  validate those fields independently; ordinary full-record roundtrips remain
  covered. The label issue is not fixed by broadening its accepted syntax.
- The full `./test --quick` workspace/platform run has not been rerun. Available
  disk space is approximately 18 GiB, with a required 15 GiB reserve; a separate
  full target build would risk violating that reserve.
- Native Windows materialization and the Windows-only test branches have not
  been executed. Windows-source representations are tested on Linux.
- Historical writers' already-discarded link bytes cannot be reconstructed.
  Old bare JSON targets have no separator convention and retain the previous
  host-native interpretation with an explicit warning. New JSON includes the
  convention and preserves exact bytes.

### Operator Corrections — 2026-10-13

- Malformed/noncanonical Windows WTF-8 and odd historical stream-name payloads
  survive Dc and JSON storage without truncation or normalization. Filesystem
  conversion rejects uninterpretable names. Interpretable historical UTF-16
  remains adapted only at the journal boundary.
- Dc 403/404 binary alternatives now require `[data:bytes=4]`; format and byte
  constraints can be combined as `[data:f84;bytes=4]`. Length is measured from
  the actual decoded payload, not its optional declared length or checksum.
  Equivalent-representation wrappers cannot bypass the payload constraints.
- Dc 496 requires `[filename]`; the writer emits its typed format boundaries.
- Dc 510 requires `511 [f524:] 512`; the derive macro's general
  `representation = f524` attribute emits those boundaries. The decoder also
  recognizes window bounds when merging Apple metadata.
- Format macro expansion preserves the identifier for rules beginning with
  `~`. Declared closing delimiters take precedence over inferred closers, so
  fields ending in a nested format's closer are not mistaken for containers.
- Verification: 478 library tests across AppleSingle/Double, conversion,
  DcData, DcString, file, and CSC passed with no failures or ignored tests.
  Targeted Clippy passed for changed crates and the derive macro.
- Remaining verification and the unrelated Finder-label gap are listed above;
  no full-workspace or native Windows run is claimed.

---

## User Review Required

> [!IMPORTANT]
> **Key Architecture Decisions**:
> 1. **Legacy Isolation**: `PlatformRawFlags` in `src/io/file/metadata.rs` will NOT be modified. In `src/io/csc/journal/legacy.rs:read_entity_payload`, we inspect `record.metadata` and normalize `platform_raw_flags` from an object `{"raw_value": n, ..}` to `n` before decoding.
> 2. **Lossless `LinkPath`**: Defined in `src/io/file/link_path.rs`. Stores exact
> bytes and the source separator convention, computing segments on demand.
> Dc 561 pairs a typed `FilePath` (`511 [f229:] 512`) with the raw bytes because
> the typed representation alone omits slash/backslash spelling. Windows bytes
> use WTF-8 metadata. Roots, prefixes, and encoding are not reinterpreted until
> materialization, where incompatible paths produce errors.
> 3. **Syntax Rule & Matcher Extension**: In `matcher.rs`, when matching `SyntaxTerm::NamedConstruct { name: "string", subtype: Some(sub), .. }`, parse and validate the literal's `decoded_payload` against the inner pattern (e.g. `[asciiprintable]{4}`). Update Dc 403 and 404 in `semantic.csv` to `:~ ([string:[asciiprintable]{4}] | [data])` and regenerate `DcList.generated.csv`.

---

## Proposed Changes

### Component 1: `ctb_io_csc` (`src/io/csc`)

#### [MODIFY] [`src/io/csc/journal/legacy.rs`](file:///workspaces/ctoolbox/src/io/csc/journal/legacy.rs)
- In `read_entity_payload`, normalize `record.metadata` before passing to `serde_json::from_value` / `FileMetadata::decode_json_with_context`:
```rust
        if let Some(flags) = record.metadata.get_mut("platform_raw_flags") {
            if flags.is_object() {
                let raw = flags.get("raw_value").and_then(serde_json::Value::as_u64)
                    .context("Legacy platform_raw_flags must contain an unsigned raw_value")?;
                *flags = serde_json::Value::from(raw);
            }
        }
```
This resolves the 3 failures in `ctb-formats-conversion` and 2 failures in `ctb-io-csc::journal::tests` without touching `metadata.rs`.
- Historical v4 Dc maps and incorrectly tagged stream names are normalized in
  `decode_v4_entity` only when direct current decoding fails. Normalization
  must change a known historical representation, reject conflicting values,
  and leave persisted source bytes and checksum offsets unchanged.

#### [MODIFY] [`src/io/csc/copy_engine.rs`](file:///workspaces/ctoolbox/src/io/csc/copy_engine.rs)
- Fix `hardlink_map` to store destination-relative paths:
  - Line 168: `hardlink_map.insert(key.clone(), rel.clone());`
  - Line 1360: `hardlink_map.insert(key.clone(), rel.clone());`
  - Compare paths in the same destination-relative/root coordinate system.
    Canonical paths count as equal only when canonicalization succeeds.
  - `target_relative_path: LinkPath::from_relative_path(first_target_rel)?,`
- Do not rebase already-relative targets in `record_journal_entry`.

---

### Component 2: `ctb_io_file` (`src/io/file`)

#### [MODIFY] [`src/io/file/entity.rs`](file:///workspaces/ctoolbox/src/io/file/entity.rs)
- Define `LinkPath`:
```rust
/// Exact source bytes of a link target, independent of the reader's host OS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct LinkPath {
    #[serde(with = "crate::serde_helpers::text_or_bytes")]
    bytes: Vec<u8>,
    is_windows: bool,
}
```
- Implement `LinkPath`:
  - `from_bytes(bytes: &[u8], is_windows: bool) -> Self`
  - `to_bytes(&self) -> Vec<u8>`
  - `segments(&self) -> Vec<Vec<u8>>`: retains empty segments.
  - `to_path_buf(&self) -> Result<PathBuf>`: adapts only for the current host.
  - `from_relative_path(&Path) -> Result<Self>`: rejects roots/prefixes rather
    than silently stripping them.
  - `impl DcMixedEncode for LinkPath`: pairs typed filepath and exact bytes
    under Dc 561; filenames and delimiters use the existing filepath syntax.
  - `impl DcMixedDecode for LinkPath`: validates both alternatives and their
    agreement without constructing host paths.
  - JSON stores bytes and the convention; UTF-8 bytes are readable strings and
    other bytes are arrays. Historical strings/arrays remain readable with a
    warning about absent source metadata.
- Update `FileEntityKind::Symlink` and `FileEntityKind::Hardlink`:
```rust
    #[dc(short = 360)]
    Symlink {
        /// Exact target link path.
        #[dc(short = 339)]
        target: LinkPath,
    },
    /// A hardlink to an existing path or inode in the session.
    #[dc(short = 361)]
    Hardlink {
        /// Relative path to the original linked file.
        #[dc(short = 339)]
        target_relative_path: LinkPath,
    },
```
- Update callers in `entity.rs`, `materializer.rs`, and `verifier.rs` to use `LinkPath`.

#### [MODIFY] [`src/io/file/streams.rs`](file:///workspaces/ctoolbox/src/io/file/streams.rs)
- Update `StreamName::encode_dc_mixed` and `decode_dc_mixed`:
  - Encode valid and invalid UTF-16 as WTF-8-typed `[data]` using the existing
    workspace codec, preserving every unit and the Windows variant.
  - Preserve invalid/noncanonical WTF-8 losslessly; reject it on filesystem use.
  - Never label UTF-16 bytes as `f2`; historical instances are adapted at the
    journal boundary; preserve odd byte lengths without truncation and reject
    their use as native names.
  - Both strictly satisfy rule 338 `:~ ([string] | [data])`.

---

### Component 3: `ctb_formats_dcdata` (`src/formats/dcdata`)

#### [MODIFY] [`src/formats/dcdata/syntax/matcher.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/matcher.rs)
- Validate the entire decoded literal payload against the parsed subtype.
- Invalid subtype syntax must produce diagnostics and mismatch, not silently
  become an unconstrained string.
- Enforce depth limits and propagate inner diagnostics. Use the dataset
  resolver for constraints when the caller omitted a resolver.
- Do not let unframed placeholder consumption bypass constraints.
- Support Dc 561 wrappers alongside Dc 397 for partially equivalent values.

#### [MODIFY] [`src/formats/dcdata/data/categories/semantic.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv)
- Update lines 108 and 109 to replace `[asciiprintable:4]` with `[string:[asciiprintable]{4}]`:
```csv
1114515,403,Mac OS Finder file type,0,BN,,!Cx,.Semantic,:~ ([string:[asciiprintable]{4}] | [data:bytes=4]),Four-character Mac OS Finder file type code.
1114516,404,Mac OS Finder file creator,0,BN,,!Cx,.Semantic,:~ ([string:[asciiprintable]{4}] | [data:bytes=4]),Four-character Mac OS Finder creator code.
```
- Regenerate `DcList.generated.csv`:
```bash
cargo run -p ctb-formats-dcdata --bin validate_data_tables
```

---

## Verification Plan

### Automated Tests
1. **Conversion Crate Unit Tests**:
   ```bash
   cargo test -p ctb-formats-conversion --lib
   ```
2. **CSC Crate Unit Tests**:
   ```bash
   cargo test -p ctb-io-csc --lib
   ```
3. **DcData Crate Tests (including Syntax Tests)**:
   ```bash
   cargo test -p ctb-formats-dcdata
   ```
4. **Full Quick Test Suite**:
   ```bash
   ./test --quick
   ```
   Must complete cleanly with exit code 0.

### Manual Verification
- Verify that legacy journal fixtures (`v1`, `v2`, `v3`) are successfully read and converted to v4 without modifying `metadata.rs`.
- Verify that `LinkPath` accurately preserves raw bytes across roundtrips and validates against strict Dc syntax rules without sandbox escape errors.
- Verify that Finder file types and creators validate under the constrained `[string:[asciiprintable]{4}]` rule.
