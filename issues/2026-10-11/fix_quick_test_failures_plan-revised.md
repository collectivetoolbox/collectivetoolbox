# Implementation Plan: Fix `./test --quick` Test Failures (Revised)

## Goal Description
Resolve all 27 test failures identified during `./test --quick` across `ctb-formats-conversion` (3 tests) and `ctb-io-csc` (24 tests), incorporating operator feedback:
1. **Isolate Legacy Deserialization in `legacy.rs`**: Clean legacy JSON representations (e.g. `{"source_os": "linux", "raw_value": 0, "has_unparsed_flags": false}`) entirely within `src/io/csc/journal/legacy.rs` before decoding into `FileMetadata`, avoiding any pollution or legacy handling in `src/io/file/metadata.rs`.
2. **Cross-Platform Lossless Link Path Representation**: Rather than forcing a host `PathBuf` (which can alter or reject non-host paths when moving between Linux and Windows), introduce a lossless `LinkPath` representation that stores raw path bytes/segments cross-platform, provides methods to adapt to `PathBuf` on materialization, and implements `DcMixed` serialization adhering strictly to Dc 339 `:~ [filepath]`.
3. **Constrained String Syntax for Dc 403 and Dc 404**: Update `semantic.csv` for Dc 403 and Dc 404 to use constrained string syntax `:~ ([string:[asciiprintable]{4}] | [data])` (replacing redundant `[asciiprintable:4]` syntax), and extend the syntax matcher in `matcher.rs` to validate the inner decoded string payload against the subtype pattern.
4. **Hardlink Relative Path Correction**: Fix `hardlink_map` in `copy_engine.rs` to store destination-relative paths (`rel`) instead of full host paths (`dest_dir.root_path().join(rel)`), preventing sandboxing rejections.
5. **StreamName Dc 338 Syntax Compliance**: Encode `StreamName::WindowsUtf16` as `[string]` (or `[data]` for invalid UTF-16) to strictly satisfy `:~ ([string] | [data])`.

---

## User Review Required

> [!IMPORTANT]
> **Key Architecture Decisions**:
> 1. **Legacy Isolation**: `PlatformRawFlags` in `src/io/file/metadata.rs` will NOT be modified. In `src/io/csc/journal/legacy.rs:read_entity_payload`, we inspect `record.metadata` and normalize `platform_raw_flags` from an object `{"raw_value": n, ..}` to `n` before decoding.
> 2. **Lossless `LinkPath`**: Defined in `src/io/file/entity.rs` (or `src/io/file/link_path.rs`). Stores raw segments `Vec<Vec<u8>>` and absolute/relative flag. Implements `DcMixedEncode` and `DcMixedDecode` as a typed `FilePath` (`511 [f229:] 512`), satisfying Dc 339 while guaranteeing that arbitrary Unix link targets deserialize losslessly on Windows and are only adapted when materializing on disk.
> 3. **Syntax Rule & Matcher Extension**: In `matcher.rs`, when matching `SyntaxTerm::NamedConstruct { name: "string", subtype: Some(sub), .. }`, parse and validate the literal's `decoded_payload` against the inner pattern (e.g. `[asciiprintable]{4}`). Update Dc 403 and 404 in `semantic.csv` to `:~ ([string:[asciiprintable]{4}] | [data])` and regenerate `DcList.generated.csv`.

---

## Proposed Changes

### Component 1: `ctb_io_csc` (`src/io/csc`)

#### [MODIFY] [`src/io/csc/journal/legacy.rs`](file:///workspaces/ctoolbox/src/io/csc/journal/legacy.rs)
- In `read_entity_payload`, normalize `record.metadata` before passing to `serde_json::from_value` / `FileMetadata::decode_json_with_context`:
```rust
        if let Some(flags) = record.metadata.get_mut("platform_raw_flags") {
            if flags.is_object() {
                if let Some(raw) = flags.get("raw_value").and_then(|v| v.as_u64()) {
                    *flags = serde_json::Value::from(raw);
                }
            }
        }
```
This resolves the 3 failures in `ctb-formats-conversion` and 2 failures in `ctb-io-csc::journal::tests` without touching `metadata.rs`.

#### [MODIFY] [`src/io/csc/copy_engine.rs`](file:///workspaces/ctoolbox/src/io/csc/copy_engine.rs)
- Fix `hardlink_map` to store destination-relative paths:
  - Line 168: `hardlink_map.insert(key.clone(), rel.clone());`
  - Line 1360: `hardlink_map.insert(key.clone(), rel.clone());`
  - Line 989: `let is_self = effective_dest_rel == *first_target_rel;`
  - Line 996: `target_relative_path: LinkPath::from_relative_path(first_target_rel),`

---

### Component 2: `ctb_io_file` (`src/io/file`)

#### [MODIFY] [`src/io/file/entity.rs`](file:///workspaces/ctoolbox/src/io/file/entity.rs)
- Define `LinkPath`:
```rust
/// Cross-platform representation of a symlink or hardlink target path,
/// preserving raw segment bytes losslessly across differing host operating systems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkPath {
    pub segments: Vec<Vec<u8>>,
    pub is_absolute: bool,
}
```
- Implement `LinkPath`:
  - `from_bytes(bytes: &[u8], is_windows: bool) -> Self`
  - `to_bytes(&self) -> Vec<u8>`
  - `to_path_buf_for_os(&self, is_windows: bool) -> Result<PathBuf>`
  - `impl DcMixedEncode for LinkPath`: encodes as `[filepath]` (`511 [f229:] 512`), encoding each segment as `[filename]` (`511 [f228:] 512`) separated by `DC_FILE_PATH_DELIMITER` (`513`).
  - `impl DcMixedDecode for LinkPath`: decodes segments from `[filepath]`.
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
  - When `WindowsUtf16(units)` is valid UTF-16, encode as `[string]`.
  - When invalid UTF-16, encode as `[data]` (`mst.push_binary(&le_bytes)`).
  - Both strictly satisfy rule 338 `:~ ([string] | [data])`.

---

### Component 3: `ctb_formats_dcdata` (`src/formats/dcdata`)

#### [MODIFY] [`src/formats/dcdata/syntax/matcher.rs`](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/matcher.rs)
- In `match_term_single` for `name == "string"`:
```rust
            if name == "string" {
                if first == DC_LITERAL_BEGIN {
                    if let Some((lit, consumed)) = scan_literal_frame(
                        stream,
                        context.mode,
                        &mut context.diagnostics,
                        0,
                    ) {
                        if let Some(sub) = subtype {
                            if let Ok(rule) = super::parser::parse_dc_syntax(sub) {
                                let inner_outcome = match_pattern(&lit.decoded_payload, &rule.pattern, context);
                                if !inner_outcome.is_matched() || inner_outcome.consumed_tokens() != lit.decoded_payload.len() {
                                    return MatchOutcome::Mismatch;
                                }
                            }
                        }
                        if let Some(var) = capture_var {
                            context
                                .captured_vars
                                .entry(var.clone())
                                .or_default()
                                .extend_from_slice(&lit.raw_tokens);
                        }
                        return MatchOutcome::Matched { consumed };
                    }
                    return MatchOutcome::Mismatch;
                }
...
```

#### [MODIFY] [`src/formats/dcdata/data/categories/semantic.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv)
- Update lines 108 and 109 to replace `[asciiprintable:4]` with `[string:[asciiprintable]{4}]`:
```csv
1114515,403,Mac OS Finder file type,0,BN,,!Cx,.Semantic,:~ ([string:[asciiprintable]{4}] | [data]),Four-character Mac OS Finder file type code.
1114516,404,Mac OS Finder file creator,0,BN,,!Cx,.Semantic,:~ ([string:[asciiprintable]{4}] | [data]),Four-character Mac OS Finder creator code.
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
