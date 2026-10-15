# Review & Remediation Plan: `2026-10-13-Dc-syntax-parser-refactor`

## 1. Executive Summary: What Is Broken, Incomplete, or Non-Compliant

Per `AGENTS.md`, this review leads with **what is broken, left unfinished, or in violation of repository rules and the initial prompt intent** in commits `5aafa2bf3..HEAD`.

### A. Critical Bugs & Logic Defects Found in `HEAD`
1. **Factual Bug in `DCMM_HEADER_LEN` Fallback (`navigator.rs:731-734`)**:
   - [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L731-L734) writes `let start = u64::try_from(DCMM_HEADER_LEN).unwrap_or(10);`. Not only is `unwrap_or` on a compile-time constant an `AGENTS.md` antipattern, **`DCMM_HEADER_LEN` is `21` (5-byte magic + 16-byte UUID), not `10`**.
2. **Broken Batch Corruption Rollback in `read_v4_journal_snapshot` (`journal.rs:1028-1188`)**:
   - In [journal.rs](file:///workspaces/ctoolbox/src/io/csc/journal.rs#L1028-L1188), `let mut batch_corrupted = false;` is declared *inside* the inner record `loop` (line 1029), set to `true` before `break;` at lines 1034, 1084, 1114, 1152, and 1168, and **never checked after the loop**. If a record partway through a batch fails to decode, the loop breaks and lines 1176–1187 **commit the partially decoded corrupt batch** instead of rolling it back.
3. **Strict-Mode Error Suppression in `read_v4_journal_snapshot` (`journal.rs:953-988`)**:
   - [journal.rs](file:///workspaces/ctoolbox/src/io/csc/journal.rs#L954) scans top-level stubs using `DcProjectionPolicy::forgiving_quiet()` even when `policy` is `strict()`.
   - Even worse, at lines 964–988, when decoding `JournalSessionRecord` (`547..548`) fails under `strict()` (`!policy.is_forgiving()`), `read_v4_journal_snapshot` **catches the strict error and retries with `forgiving_quiet()`**!
4. **On-Disk `BinaryPayloadRef::FileSpan` (> 4 KiB) Fails to Decode (`navigator.rs` / `dc_mixed_serde.rs`)**:
   - When `DcDocumentNavigator` reads from `SeekableFile` (as in `read_v4_journal_snapshot`), binary payloads $> 4\text{ KiB}$ (`INLINE_BINARY_THRESHOLD`) are stored in `DcSyntaxArena` as `BinaryPayloadRef::FileSpan { offset, byte_len }`.
   - However, `DcProjectionContext` has no access to the underlying seekable reader, and every `DcMixedDecode` implementation calls `payload.materialize_bytes()`, which calls `self.materialize(None)` ([navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L490-L492)) and **unconditionally errors** (`"Cannot materialize on-disk FileSpan without a seekable reader"`). Furthermore, `BinaryPayloadRef::as_slice()` ([navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L429-L433)) silently returns `&[]` for unmaterialized `FileSpan`s.
5. **Pre-1970 `SystemTime` Silently Clobbered to `UNIX_EPOCH` (`dc_mixed_serde.rs:1762`)**:
   - [dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L1762) uses `self.duration_since(std::time::UNIX_EPOCH).unwrap_or_default()`, silently turning any pre-1970 timestamp into `1970-01-01T00:00:00Z` on encode even though `SystemTime::decode_dc_node` supports negative signed seconds (`i64`).
6. **`(A, B)` and `Vec<T>` Fail Transparent Optional (`315`) Unwrapping + Removed Test (`to-review:127-135`)**:
   - In [challenge_m5_adversarial.rs](file:///workspaces/ctoolbox/src/formats/dcstring/tests/challenge_m5_adversarial.rs#L495-L590), the `(u32, bool)` test wrapped in `397` -> `315` was removed because:
     1. `(A, B)::decode_dc_node` ([dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L925)) and `Vec<T>::decode_dc_node` ([dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L1034)) call `ctx.unwrap_equivalent_or_self(node_id)` instead of `ctx.unwrap_transparent(node_id)`, so they fail whenever wrapped in `Dc 315` (`DC_OPTIONAL_PRESENT`).
     2. `(A, B)::encode_dc_mixed` ([dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L913-L923)) emits two unframed sibling values without `514..515` (`DC_BEGIN_KV_PAIR..DC_END_KV_PAIR`), whereas `(A, B)::decode_dc_node` expects a `514` or `310` container!
   - Also, [dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs) is missing `impl<A: DcForgivingDefault, B: DcForgivingDefault> DcForgivingDefault for (A, B)` (it was placed in `navigator.rs`, or needs to cover all tuple sizes used by `#[derive(DcMixed)]`).

---

### B. Violations of `to-review`, `plan.md`, and `AGENTS.md`

| Issue in `to-review` / `AGENTS.md` | Current State in `HEAD` | Root Cause & Required Fix |
| :--- | :--- | :--- |
| **1. Audit of `"Reason for fallback"` notes (`to-review:1-15`)** | **40+ bogus fallbacks added in `5aafa2bf3..HEAD`** across `resolver.rs`, `validation.rs`, `navigator.rs`, `dc_source.rs`, `encapsulation.rs`, `ctb_dc_macro/src/lib.rs`, `dc_mixed_serde.rs`, `journal.rs`, `metadata.rs`, `streams.rs`. | Includes `unwrap_or(0)` on bitwise masks ([journal.rs:1124](file:///workspaces/ctoolbox/src/io/csc/journal.rs#L1124)), provably in-bounds slices ([resolver.rs:255](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs#L255), [metadata.rs:952](file:///workspaces/ctoolbox/src/io/file/metadata.rs#L952)), `usize::try_from(offset).unwrap_or(0)` on 32-bit targets (19 sites), swallowing proc-macro attribute syntax errors ([ctb_dc_macro/src/lib.rs:697,734](file:///workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs#L697)), and passing a tag node as its own argument when `args` is empty ([ctb_dc_macro/src/lib.rs:984,1705,2868](file:///workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs#L984)). |
| **2. Deleted `FIXME` on `SampleFlattenedFormatsStruct` (`to-review:18-20`)** | **Deleted without resolving** at [dc_mixed_serde.rs:3076](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L3076). | The container was changed from `422..423` to `317..318`, but the design note (*"Name and format list should probably be alternate representations, and 342 the overarching type for this, maybe?"*) was not addressed and the `FIXME` was deleted. |
| **3. Parser vs Macro Layer Separation `FIXME` (`to-review:22-24`)** | **Still unresolved** at [ctb_dc_macro/src/lib.rs:1153](file:///workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs#L1153) and expanded. | `DcSyntaxKind` lacks normalized CST variants (`KvMap`, `Equivalents`, `Optional`, `FormatRepresentation`). As a result, `559` vs legacy `313` KV-map parsing is triplicated across `ctb_dc_macro`, `dc_mixed_serde.rs`, and `source_edit.rs`; `ctb_dc_macro` also contains `type_name.contains("PathBuf")` hacks and manual flat-atom lookahead. |
| **4. Prohibition on Ad-Hoc `DcMixedEncode` / `DcMixedDecode` (`to-review:31-73`)** | **Violated in `src/io/file/`**: `#[derive(DcMixed)]` was **removed** from `PlatformRawFlags`, `NativeMetadata`, and `AttachedStream` and replaced with ~400 lines of manual impls; `AppleMetadata`, `FileTimestamps`, `StreamName`, and `LinkPath` also use manual impls. | Fix `ctb_dc_macro` (`raw = (field, tag)` arg unwrapping, `Option<T>` with `omit_default` not emitting `315`, general `#[dc(flatten)]` delegating to `DcMixedFields`, and timestamp/paired field projection) and restore/apply `#[derive(DcMixed)]`. |
| **5. Hardcoded `310` alongside `397`/`561` in `dc_mixed_serde.rs` (`to-review:78-90`)** | Present and duplicated twice in [dc_mixed_serde.rs:1549-1603](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L1549-L1603), plus hardcoded `is_opaque_text_container` in [navigator.rs:3377](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L3377) and bare integer Dc literals across `dc_mixed_serde.rs` & `navigator.rs`. | Normalize `397`/`561` (`:~ [list]`) in Stage 1 (`DcSyntaxArena`) using CSV rules and use `ctb_formats_dcdata::dc::*` constants everywhere. |
| **6. Unauthorized `Dc 556` (`Log message`) CSV syntax change (`to-review:93-99`)** | [semantic.csv:242](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv#L242) changed `556` from `:~ [string]` to `:~ (([string] [number:Integer]) \| [string])` so `JournalErrorRecordPayload` could stuff `(error_message, timestamp_sec)` into `#[dc(556)]`. | Revert `556` to `:~ [string]` in `semantic.csv` & `DcList.generated.csv`. Encode the incident timestamp as its own semantic field on `552..553` and `554..555`. |
| **7. Hardcoded Depth Limits vs Caller Knob (`to-review:102-118`)** | Hardcoded `MAX_SYNTAX_EXPANSION_DEPTH = 32` in [navigator.rs:64](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L64), `0..32` in `unwrap_transparent` ([dc_mixed_serde.rs:705](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L705)), and call-stack recursion in `build_arena_node_from_stub` ([navigator.rs:1327](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L1327)) & `validate_span_strict` ([navigator.rs:3146](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs#L3146)). | Add configurable depth limit on `DcProjectionPolicy` (with a diagnostic action so callers/users can permit deeper documents) and convert call-stack recursion in `navigator.rs` and `unwrap_transparent` to heap-allocated stacks with cycle/progress guards. |
| **8. Duplicated `b64_digit_to_char` (`to-review:120-124`)** | [dc_mixed_serde.rs:278-323](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs#L278-L323) duplicates `digit_to_base64_char` and Base64-to-`malachite::Integer` parsing from [dc_number.rs:80-101, 320-335](file:///workspaces/ctoolbox/src/formats/dcstring/dc_number.rs#L80-L101). | Deduplicate into a single shared function in `dc_number.rs` (or `ctb_formats_base64`). |
| **9. Bare Integer Format IDs & Duplicate Dc Consts (`AGENTS.md`)** | `CharTarget::Format(usize)` in [ast.rs:36](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs#L36), `format_rules: HashMap<u32, DcSyntaxRule>` in [resolver.rs:146](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs#L146); local `GID_*` / `SHORT_*` int consts in [kv_map.rs:28](file:///workspaces/ctoolbox/src/formats/dcstring/kv_map.rs#L28) & [shared.rs:29](file:///workspaces/ctoolbox/src/formats/dcstring/shared.rs#L29). | Replace bare integer format IDs with `FormatId`, and move/use `DcChar` constants in `dcdata/dc.rs`. |
| **10. Test Sprawl & Top-Level `#![allow(...)]` (`AGENTS.md`)** | 15 new test binaries (9,876 lines) with duplicate subagent names (`challenger_m1_tests.rs`, `empirical_challenge_m4.rs`, `challenge_m3_challenger3_empirical.rs`, etc.), 5 files with prohibited top-level `#![allow(...)]`, and leftover `eprintln!("PART 1 PASSED")` debug prints. | Consolidate into clean per-crate test suites, move `#[allow(...)]` to item level, and remove debug `eprintln!`s. |

---

## 2. Open Questions & Design Decisions for Confirmation

> [!IMPORTANT]
> Before executing the changes, please confirm your preferences on these three design details:
>
> 1. **Timestamp & Warning Code Fields in `552..553` (Error Record) and `554..555` (Warning Record)**:
>    - Reverting `Dc 556` (`Log message`) to `:~ [string]` means `#[dc(556)]` holds only the `String` message.
>    - For the incident timestamp on `JournalErrorRecordPayload` (`552..553`) and `JournalWarningRecordPayload` (`554..555`), we can use existing `Dc 390` (`Begin time moment description`, i.e., `#[dc(nested = 390)] pub timestamp: SystemTime` or Unix seconds inside `390 f486 <secs> 391`), which is already a self-framing `[date]` construct in `semantic.csv`. Does using `390..391` (`[date]`) inside `552..553` and `554..555` for the record timestamp look right to you?
> 2. **Resource Limit Knob on `DcProjectionPolicy`**:
>    - We propose adding `pub max_depth: usize` (default `64`) and `pub depth_limit_exceeded: DiagnosticAction` (`Error` in `strict()`, `Warn` in `forgiving()`, or `Ignore` when a caller explicitly wants to continue without warnings) to `DcProjectionPolicy`, while converting all parser/arena/unwrapping traversals to explicit heap stacks (`Vec`) with span-progress guards so continuing past `max_depth` is 100% safe against stack overflow and infinite loops. Does this match what you had in mind?
> 3. **Generalizing `FileTimestamps`, `StreamName`, and `LinkPath` into `#[derive(DcMixed)]`**:
>    - `PlatformRawFlags`, `NativeMetadata`, `AttachedStream`, and `AppleMetadata` can be immediately converted to `#[derive(DcMixed)]` with small, general fixes to `ctb_dc_macro`.
>    - For `FileTimestamps`, storing or projecting `atime`/`mtime`/`ctime`/`birthtime` via `#[derive(DcMixed)]` can be done either by supporting `#[dc(flatten)]` on a struct with `#[dc(330, sec_nsec_pair = "atime_nsec")]` (or changing `FileTimestamps` fields to `UnixTimestampNanos` / `(i64, u32)` with helper accessors `atime_sec()`/`atime_nsec()`). Would you prefer keeping the struct fields `atime_sec: i64, atime_nsec: u32` via a general macro attribute, or simplifying `FileTimestamps` fields?

---

## 3. Phased Implementation Plan

To avoid context window compaction issues (as noted in `plan.md`), the remediation is divided into **4 self-contained phases**, each leaving the workspace compiling and passing tests.

### Phase 1: Stage 1 & Stage 2 Core Cleanup — CST Normalization, Heap-Stack Depth Knob, `u64` Offsets, & Fallback Audit (`ctb-formats-dcdata` + `ctb-formats-dcstring`)

#### [MODIFY] [ast.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/ast.rs) & [resolver.rs](file:///workspaces/ctoolbox/src/formats/dcdata/syntax/resolver.rs) & [validation.rs](file:///workspaces/ctoolbox/src/formats/dcdata/validation.rs) & [dc_number_minimal.rs](file:///workspaces/ctoolbox/src/formats/dcdata/dc_number_minimal.rs)
- Change `SyntaxDiagnostic::token_offset` from `usize` to `u64` so 32-bit `x86` targets never truncate $> 4\text{ GiB}$ file offsets or require `.unwrap_or(0)`.
- Replace `CharTarget::Format(usize)` and `format_rules: HashMap<u32, DcSyntaxRule>` with strongly typed `FormatId` (`AGENTS.md` rule: Format IDs must never be bare integers).
- Remove bogus `// Reason for fallback` + `.map_or_else(Vec::new, ...)` / `.unwrap_or(...)` in `resolver.rs:255,274`, `validation.rs:400`, and `dc_number_minimal.rs`.

#### [MODIFY] [navigator.rs](file:///workspaces/ctoolbox/src/formats/dcstring/navigator.rs)
- Fix `DCMM_HEADER_LEN` offset handling (`DCMM_HEADER_LEN = 21`, eliminating the buggy `.unwrap_or(10)` and `.unwrap_or(0)` calls).
- Remove `BinaryPayloadRef::as_slice()` (which silently returned `&[]` for on-disk `FileSpan`s) and thread a `&mut dyn DcBinaryResolver` (implemented by `DcDocumentNavigator`) into `DcProjectionContext` so `BinaryPayloadRef::FileSpan` payloads $> 4\text{ KiB}$ can be lazily read on demand when projecting `BinaryPayload` / `Vec<u8>`.
- Normalize structural wrappers in `DcSyntaxKind` at the Stage 1 parser layer:
  - Parse `559..560` (with `514..515` pairs) and legacy `313..314` into a normalized `DcSyntaxKind::KvMap { entries: Vec<(NodeId, NodeId)>, is_legacy: bool }` (emitting a `deprecated_syntax` diagnostic for `313`), eliminating the triplicated KV-map parser in `ctb_dc_macro`, `dc_mixed_serde.rs`, and `source_edit.rs`.
  - Parse `397` / `561` (`:~ [list]`) so the inner `310..311` list items are directly exposed as equivalent candidates without conflating bare `310` lists with `397` equivalents.
- Replace call-stack recursion in `build_arena_node_from_stub` and `validate_span_strict` with explicit heap-allocated work stacks (`Vec`), and wire depth checking to `DcProjectionPolicy { max_depth, depth_limit_exceeded }`.
- Replace hardcoded short-Dc integer literals (`is_opaque_text_container`, etc.) with data-driven `SyntaxRuntimeRegistry` queries and `ctb_formats_dcdata::dc::*` constants.

#### [MODIFY] [dc_number.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_number.rs) & [dc_mixed_serde.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_mixed_serde.rs) & [dc_source.rs](file:///workspaces/ctoolbox/src/formats/dcstring/dc_source.rs) & [source_edit.rs](file:///workspaces/ctoolbox/src/formats/dcstring/source_edit.rs) & [kv_map.rs](file:///workspaces/ctoolbox/src/formats/dcstring/kv_map.rs) & [shared.rs](file:///workspaces/ctoolbox/src/formats/dcstring/shared.rs)
- Delete duplicated `b64_digit_to_char` and `parse_dc_number_digits` from `dc_mixed_serde.rs`, reusing `dc_number.rs`.
- Fix `SystemTime::encode_dc_mixed` so pre-1970 timestamps encode signed negative seconds instead of `.unwrap_or_default()` clamping to 1970.
- Fix `(A, B)::encode_dc_mixed` and `(A, B)::decode_dc_node` (and `Vec<T>::decode_dc_node`) to use `ctx.unwrap_transparent(node_id)` and consistent pair framing (`514..515`).
- Make `DcProjectionContext::unwrap_transparent` respect `policy.max_depth` and `policy.depth_limit_exceeded` (with cycle detection via visited `NodeId` tracking so even unbounded unwrapping on a malformed graph cannot loop infinitely).
- Remove duplicated `decode_filename_bytes` match arms and bare `310` container opener check in `dc_mixed_serde.rs`.
- Restore the deleted `FIXME` comment on `SampleFlattenedFormatsStruct` in `dc_mixed_serde.rs`.
- Remove local integer Dc constants in `kv_map.rs` and `shared.rs` in favor of `ctb_formats_dcdata::dc::*`.

---

### Phase 2: Eliminate Ad-Hoc `DcMixedEncode` / `DcMixedDecode` Impls (`ctb-dc-macro` + `ctb-io`)

#### [MODIFY] [lib.rs](file:///workspaces/ctoolbox/src/formats/dcstring/ctb_dc_macro/src/lib.rs)
- Fix the `FIXME` at line 1153: remove legacy `313` vs `559` syntax branching, `type_name.contains("PathBuf")` special-casing, and flat-atom lookahead from `ctb_dc_macro`, relying on Stage 1's normalized `DcSyntaxKind::KvMap` and `PathBuf::decode_dc_node`.
- Fix `parse_field_attrs(f)` in `expand_dc_forgiving_default` to propagate `syn::Result` with `?` instead of `.unwrap_or(false)`.
- Fix empty `Tagged { tag, args }` handling to report a malformed field via `ctx.policy` rather than falling back to passing the tag node `cid` as its own argument.
- Fix named-field enum variant decoding to honor `ctx.policy.missing_required_field` using `DcForgivingDefault`.
- Fix `raw = (raw_ident, raw_tag)` to pass the unwrapped argument `NodeId` to `<#raw_ty as DcMixedDecode>::decode_dc_node`.
- For `Option<T>` fields with an explicit field tag and `omit_default`, encode `Some(v)` directly under the field tag without wrapping in `Dc 315` (`DC_OPTIONAL_PRESENT`).
- Generalize `#[dc(flatten)]` on `T` and `Option<T>` to delegate to `DcMixedFields::encode_fields_with_context` and `DcMixedFields::decode_field`.

#### [MODIFY] [metadata.rs](file:///workspaces/ctoolbox/src/io/file/metadata.rs), [streams.rs](file:///workspaces/ctoolbox/src/io/file/streams.rs), [link_path.rs](file:///workspaces/ctoolbox/src/io/file/link_path.rs)
- Restore `#[derive(ctb_formats_dcstring::DcMixed)] #[dc(binary)]` on `PlatformRawFlags` and delete its manual `DcForgivingDefault` / `DcMixedEncode` / `DcMixedDecode` impls.
- Restore `#[derive(ctb_formats_dcstring::DcMixed)] #[dc(key_value)]` on `NativeMetadata` and delete its 252-line manual impl.
- Restore `#[derive(ctb_formats_dcstring::DcMixed)] #[dc(begin = 323, end = 324)]` on `AttachedStream` and delete its manual impl.
- Replace manual `DcMixedEncode` / `DcMixedDecode` / `DcForgivingDefault` on `AppleMetadata`, `FileTimestamps`, `StreamName`, and `LinkPath` with `#[derive(ctb_formats_dcstring::DcMixed)]`.

---

### Phase 3: Fix `csc::journal` Strict/Forgiving Semantics, Batch Rollback, & Revert `Dc 556` (`ctb-io-csc` + `ctb-formats-dcdata`)

#### [MODIFY] [semantic.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/categories/semantic.csv) & [DcList.generated.csv](file:///workspaces/ctoolbox/src/formats/dcdata/data/DcList.generated.csv)
- Revert `Dc 556` (`Log message`) syntax from `:~ (([string] [number:Integer]) | [string])` back to `:~ [string]`.

#### [MODIFY] [journal.rs](file:///workspaces/ctoolbox/src/io/csc/journal.rs)
- Update `JournalErrorRecordPayload` (`552..553`) and `JournalWarningRecordPayload` (`554..555`) so `#[dc(556)]` is `pub message: String` (the log message) and the timestamp is encoded under its own semantic field (`390..391`).
- Fix `read_v4_journal_snapshot`:
  1. Remove the fallback that retries `JournalSessionRecord` with `forgiving_quiet()` when `policy` is `strict()`. In `strict()` mode, invalid syntax in the session header or records must fail or roll back strictly according to `policy`.
  2. Move `let mut batch_corrupted = false;` *outside* the inner batch record loop and check `if !batch_corrupted` before committing `pending_entities`, `pending_errors`, `pending_warnings`, and `snapshot.last_batch_id = seq_id`.
  3. Pass the seekable file source into `decode_frame_with_arena` so `BinaryPayloadRef::FileSpan` payloads $> 4\text{ KiB}$ materialize properly from disk.
  4. Replace `u32::try_from(payload.packed_status & 0xFFFF_FFFF).unwrap_or(0)` with an infallible conversion/expect per `AGENTS.md`.

---

### Phase 4: Test Suite Consolidation, Restored `(u32, bool)` Test, & Lint Cleanup

#### [MODIFY] Test suites in `src/formats/dcstring/tests/`, `src/io/csc/tests/`, `src/io/file/tests/`, `src/formats/apple_single_double/tests/`
- Restore the removed `(u32, bool)` and `Vec<u32>` transparent `397` -> `315` unwrapping test in `challenge_m5_adversarial.rs` and remove the leftover `eprintln!("PART N PASSED")` statements.
- Add a test verifying that callers can configure `DcProjectionPolicy::max_depth` / `depth_limit_exceeded` to continue parsing deeply nested documents (> 500 levels) without stack overflow or infinite loops.
- Add a test in `ctb-io-csc` for an on-disk v4 journal containing a `FileEntity` stream payload $> 4\text{ KiB}$ (`INLINE_BINARY_THRESHOLD`) to verify `BinaryPayloadRef::FileSpan` materialization.
- Add a test in `ctb-io-csc` verifying that a batch with a valid first entity and a corrupted second entity rolls back the *entire* batch (testing the `batch_corrupted` fix).
- Consolidate the redundant subagent-named challenger test files (`challenger_m1_tests.rs`, `empirical_challenge_m4.rs`, `challenge_m5_empirical.rs`, `challenge_m3_challenger3_empirical.rs`, `challenge_m3_challenger_4.rs`), remove all top-level `#![allow(...)]` attributes, and fix all uncommented `.unwrap_or` calls in tests.

---

## 4. Verification Plan

### Automated Tests & Lints
- Phase 1:
  - `cargo test -p ctb-formats-dcdata`
  - `cargo test -p ctb-formats-dcstring`
- Phase 2:
  - `cargo test -p ctb-dc-macro`
  - `cargo test -p ctb-formats-apple-single-double`
  - `cargo test -p ctb-io`
- Phase 3 & 4:
  - `cargo test -p ctb-io-csc`
  - `cargo test -p ctb-formats-conversion`
  - `RUSTFLAGS="-A warnings" cargo clippy --no-deps -p ctb-formats-dcdata -p ctb-formats-dcstring -p ctb-dc-macro -p ctb-formats-apple-single-double -p ctb-io -p ctb-io-csc --all-targets`
  - `./lint --quick`



# Feedback given to worker in phase 2:

Please recalibrate; you've gone off track with introducing excessively detailed awareness of parser strictness into the serde module. The serde module should be relatively simple and independent of the parser, which should own all logic related to the actual parsing. *All* the serde module should do is take the already-parsed document (which will have been parsed with whatever strictness the caller requested) and then map it onto the structs based on the annotations, and return back to the caller whether any Dcs were left over after doing that. This should be a *general* implementation; there should never be anything like ```
       if let DcSyntaxKind::Container { opener, .. } = &node.kind {
            if *opener == DC_BEGIN_LIST {
                if !ctx.policy.is_forgiving() {
                    ctx.policy.handle(
                        ctx.policy.syntax_error,
                        "Byte list representation of PathBuf is not allowed under strict policy; expected Dc 511 + f494",
                    )?;
                }
                let bytes = <Vec<u8> as DcMixedDecode>::decode_dc_node(ctx, node_id)?;
                return path_from_native_bytes(&bytes);
            }
        }
``` at all. It should just see "Did the struct use up all the parsed Dc blocks that we had? Yes? OK, projection succeeded cleanly. No? Check strictness, if permissive, return warning, if strict, return error." This should be a *single* area of the logic and should not be repeated or referenced in more than one place.

