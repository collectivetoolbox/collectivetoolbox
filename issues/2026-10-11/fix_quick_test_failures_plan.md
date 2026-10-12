# Implementation Plan: Fix `./test --quick` Test Failures

## Goal Description
During a full run of `./test --quick`, two crates exhibited test failures:
1. `ctb-formats-conversion` (3 tests failed)
2. `ctb-io-csc` (24 tests failed)

All 27 failures stem from five specific root causes:
- **Root Cause A (`PlatformRawFlags` JSON Deserialization)**: Legacy CSC journal fixtures (`v1`, `v2`, `v3`) encode `platform_raw_flags` in metadata JSON as an object (e.g. `{"source_os": "linux", "raw_value": 0, "has_unparsed_flags": false}`). `PlatformRawFlags::deserialize` currently expects an integer `u64`, causing `invalid type: map, expected u64` when reading legacy journals.
- **Root Cause B (Hardlink Relative Path Storage)**: `hardlink_map` in `copy_engine.rs` stored absolute paths (`dest_dir.root_path().join(rel)`) instead of relative paths (`rel`). Under `PathTraversalPolicy::StrictSandboxed`, `materialize_entity` rejects paths containing `RootDir` (`/`) with `"Hardlink target path rejected: target escapes destination root or contains root/parent components"`.
- **Root Cause C (Dc 339 `File target link path` Strict Syntax)**: `FileEntityKind::Symlink` and `FileEntityKind::Hardlink` had `#[dc(short = 339, binary)]` on `target` and `target_relative_path`. This serialized link paths as opaque binary blobs (`[data]`, Dc 203) instead of `[filepath]`. Under strict Dc syntax validation, Dc 339 requires `:~ [filepath]`. Furthermore, `AGENTS.md` explicitly mandates: *"Path values must follow the declared typed filepath syntax rather than substituting an undelimited string or opaque byte buffer."* The failure during journal batch validation also caused `FS_CACHE_TEST_MUTEX` poisoning that broke 5 move-engine and verifier tests.
- **Root Cause D (Dc 338 `File stream name` Strict Syntax)**: `StreamName::WindowsUtf16` encoded `units: Vec<u16>` as a typed list (`Dc 310 ... Dc 311`), but Dc 338's syntax rule in `semantic.csv` is `:~ ([string] | [data])`.
- **Root Cause E (Dc 403 & 404 Finder File Type and Creator Syntax)**: `FinderInfo.file_type` and `FinderInfo.file_creator` are `String`s serialized as typed string literals (`[string]`, `Dc 260 ... Dc 261`), but Dc 403 and Dc 404's syntax in `semantic.csv` was `:~ ([asciiprintable:4] | [data])`, rejecting valid typed strings.

---

## User Review Required

> [!IMPORTANT]
> **Semantic Syntax Update for Dc 403 and Dc 404**:
> In `src/formats/dcdata/data/categories/semantic.csv`, Dc 403 (`Mac OS Finder file type`) and Dc 404 (`Mac OS Finder file creator`) are updated from `:~ ([asciiprintable:4] | [data])` to `:~ ([asciiprintable:4] | [string] | [data])`. This allows four-character Finder Ostype codes to be represented both as raw printable ASCII tokens or typed string literals (`[string]`), as emitted by `FinderInfo.file_type` and `file_creator`. `DcList.generated.csv` is regenerated via `cargo run -p ctb-formats-dcdata --bin validate_data_tables`.
>
> **Link Target Serialization as Typed Filepaths**:
> In `src/io/file/entity.rs`, `FileEntityKind::Symlink.target` and `FileEntityKind::Hardlink.target_relative_path` are transitioned to typed filepath serialization matching Dc 339 `:~ [filepath]` rather than binary blobs (`[data]`), strictly adhering to `AGENTS.md`'s rule on path syntax.

---

## Proposed Changes

### Component 1: `ctb_io_file` (`src/io/file`)

#### [MODIFY] [`src/io/file/metadata.rs`](file:///workspaces/ctoolbox/src/io/file/metadata.rs)
- Update `Deserialize` for `PlatformRawFlags` to accept both raw `u64` and the legacy map format (`{"raw_value": u64, ...}`):
```rust
impl<'de> Deserialize<'de> for PlatformRawFlags {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum RawFlagsInput {
            Raw(u64),
            Map { raw_value: u64 },
        }

        let input = RawFlagsInput::deserialize(deserializer)?;
        let raw = match input {
            RawFlagsInput::Raw(v) => v,
            RawFlagsInput::Map { raw_value } => raw_value,
        };
        Ok(Self::from_raw(raw))
    }
}
```

#### [MODIFY] [`src/io/file/entity.rs`](file:///workspaces/ctoolbox/src/io/file/entity.rs)
- Update `FileEntityKind::Symlink` and `FileEntityKind::Hardlink` to use `PathBuf` (or a helper wrapping `PathBuf`) without the `binary` tag on Dc 339, ensuring it encodes and decodes as `[filepath]` (`511 [f229:] 512`):
```rust
    #[dc(short = 360)]
    Symlink {
        /// Exact target path.
        #[dc(short = 339)]
        target: PathBuf,
    },
    /// A hardlink to an existing path or inode in the session.
    #[dc(short = 361)]
    Hardlink {
        /// Relative path to the original linked file.
        #[dc(short = 339)]
        target_relative_path: PathBuf,
    },
```
- Update callers in `src/io/file/entity.rs`, `materializer.rs`, and `verifier.rs` to work directly with `PathBuf` for `target` and `target_relative_path`.

#### [MODIFY] [`src/io/file/streams.rs`](file:///workspaces/ctoolbox/src/io/file/streams.rs)
- Update `StreamName::encode_dc_mixed` and `decode_dc_mixed` for `StreamName::WindowsUtf16` to serialize as `[string]` when valid UTF-16, and as `[data]` (`mst.push_binary`) when containing unpaired surrogates / invalid UTF-16, satisfying rule 338 `:~ ([string] | [data])`:
```rust
impl ctb_formats_dcstring::DcMixedEncode for StreamName {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMstring) -> Result<()> {
        match self {
            Self::Bytes(bytes) => {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&s.to_string(), mst)
                } else {
                    mst.push_binary(bytes);
                    Ok(())
                }
            }
            Self::WindowsUtf16(units) => {
                if let Ok(s) = String::from_utf16(units) {
                    ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&s, mst)
                } else {
                    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
                    mst.push_binary(&bytes);
                    Ok(())
                }
            }
        }
    }
}
```

---

### Component 2: `ctb_io_csc` (`src/io/csc`)

#### [MODIFY] [`src/io/csc/copy_engine.rs`](file:///workspaces/ctoolbox/src/io/csc/copy_engine.rs)
- In `copy_engine.rs`, fix `hardlink_map` to store destination-relative paths:
  - Line 168: `hardlink_map.insert(key.clone(), rel.clone());`
  - Line 1360: `hardlink_map.insert(key.clone(), rel.clone());`
  - Line 989: `let is_self = effective_dest_rel == *first_target_rel;`
  - Line 996: `target_relative_path: first_target_rel.clone(),`

#### [MODIFY] [`src/io/csc/verifier.rs`](file:///workspaces/ctoolbox/src/io/csc/verifier.rs) & [`src/io/csc/journal.rs`](file:///workspaces/ctoolbox/src/io/csc/journal.rs)
- Update test cases where `FileEntityKind::Hardlink` or `FileEntityKind::Symlink` was instantiated with byte slices to pass `PathBuf` (e.g. `PathBuf::from("hello.txt")`).

---

### Component 3: `ctb_formats_dcdata` (`src/formats/dcdata`)

#### [MODIFY] [`src/formats/dcdata/data/categories/semantic.csv`](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv)
- Update Dc 403 and Dc 404 syntax to accept `[string]` alongside `[asciiprintable:4]` and `[data]`:
```csv
1114515,403,Mac OS Finder file type,0,BN,,!Cx,.Semantic,:~ ([asciiprintable:4] | [string] | [data]),Four-character Mac OS Finder file type code.
1114516,404,Mac OS Finder file creator,0,BN,,!Cx,.Semantic,:~ ([asciiprintable:4] | [string] | [data]),Four-character Mac OS Finder creator code.
```
- Regenerate data tables via:
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
3. **Full Quick Test Suite**:
   ```bash
   ./test --quick
   ```
   Must pass with exit code 0 and all crate suites passing cleanly.

### Manual Verification
- Confirm that `test_all_journal_revision_fixtures` decodes all legacy `v1`, `v2`, and `v3` fixture files without metadata loss or deserialization errors.
- Confirm that journals created during copy operations pass `validate_segment` strict syntax validation without panicking.
- Confirm that hardlinks are materialized properly under `StrictSandboxed` without triggering target escaping rejection.
