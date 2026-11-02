# Walkthrough: DCEUtils Port Implementation

## Correctness Audit (2026-11-02): Partially Implemented

This section supersedes the older completeness claims below. The legacy PHP
port remains bug-compatible; `dce_convert_new` and `dce_convert_new_with_log`
expose the independent converter. Both paths now run the same PHP fixture body
without maintaining duplicate fixtures.

### Remaining Gaps vs Upstream

- The 3.02a spreadsheet still shows the 3.01a header example and leaves its
  version-selector assignment blank, while `Dc 205` in the Dc database is
  `DCE Version Selector 3.03a`. To avoid colliding with 3.03a on
  `DCE_Version_Selectors:03`, DCE 3.02a (`3_02a`, `3_02a_raw`, `hex_3_02a`,
  `hex_3_02a_raw`) is marked unimplemented and returns an error when invoked,
  with its `Core:C9..=CB` (`Dc 200..=202`) mappings left stubbed in
  `converter.rs` and its version selector marked as TBD.
- Legacy one-step translators and HTML output still delegate to the
  PHP-compatible implementation. They have not been independently rewritten or
  established as correct general-purpose HTML converters.
- Keyed embeddings are preserved as marker/payload sequences, not interpreted
  or validated against an external key registry.
- Processing-only `FF` is retained by the control-preserving API. Its support
  should not be read as permission to interchange processing-only streams.
- Tests establish the documented byte mappings and the cases listed below,
  not exhaustive correctness for every malformed stream or modern semantic
  interpretation. No cross-platform or full-workspace validation is implied.

### Confirmed Defects Addressed

- Seven whitespace/control byte assignments disagreed with both the original
  ODS sheets and their Markdown copies. Independent tests now read all byte
  rows of each version's mapping sheets, rather than relying solely on
  encoder/decoder round trips. Missing selector rows in the byte sheets are
  explicitly reconciled with the character list; the 3.02a extension is explicit.
- Newline's Unicode projection incorrectly used NEXT LINE; decimal-point
  projection was missing. Unicode hyphen-minus is now preserved by UTF-8
  encapsulation on capable targets, including exports from modern Dc documents.
- Empty text, UTF-32, and encapsulated Base64 now round-trip. Malformed Base64,
  invalid encapsulated UTF-8, unterminated text payloads, malformed identifier
  lists, and malformed file envelopes return errors.
- Strict CDCE now returns an error instead of returning a successful byte
  payload containing an error message. The PHP-compatible API is unchanged.
- Identity conversions no longer bypass input-format validation. CDCE text
  output escapes literal at signs, including those decoded from encapsulation.

### Shared UTF-8 Conversion

The corrected converter now builds a `DcMstring` and uses the shared UTF-8
APIs instead of privately decoding encapsulated Base64 or Unicode. Strict
conversion rejects malformed payloads. The boolean permissive option preserves
valid text, substitutes U+FFFD for damaged sequences, and returns diagnostics;
it is independent of Serde projection policies.

Unicode equivalence and approximation mappings come from the modern Dc
resolver, not EITE tables. All 134 legacy Unicode mapping rows were accounted
for in modern declarations, and resolver tests verify runtime-graph/source
mapping parity. No metadata or graph append was necessary.

The modern DCE document bridge lives in `document.rs` within `ctb-formats-dceutils`,
allowing DCE conversion to depend on shared document APIs without a dependency
cycle. Import validates encapsulated UTF-8 through the same shared repair API.
The remaining gaps above still apply; this is not full DCE conformance.

### Test Expectation Classification

Six PHP fixtures intentionally expected punctuation or non-BMP Unicode to be
dropped on import. Those expectations remain valid compatibility assertions for
the legacy port, but are invalid conformance expectations for the corrected
converter. Its comparison uses explicit preserved sequences and independent
text round trips. The strict-CDCE fixture likewise has a corrected error
expectation. All other shared fixtures retain their original expectations.

Additional regressions cover exact mixed-sheet sequences, reserved bytes,
truncated headers/trailers, malformed payloads, and modern Dc export/import.

### Validation

- Linux musl library suites: `ctb-formats-dcdata`, `ctb-formats-dcstring`,
  `ctb-formats-dceutils`, and `ctb-formats-conversion` passed; zero failed,
  skipped, or ignored. Focused CLI coverage also verifies parsed
  `--permissive` recovery of malformed DCE text.
- Affected-crate Clippy (`--lib --tests --no-deps`) passed, with existing
  dependency warnings. Editor diagnostics reported no touched-file errors.
- The PHP fixture generator now emits the shared runner. Translation of all
  88 original assertions and generated-Rust syntax validation passed using
  in-memory output, without overwriting the checked-in suite.
- Full-workspace, platform-specific, and independent HTML validation were not
  run. The remaining gaps above prevent a blanket completeness claim.

We have successfully completed the LLM-assisted port of the `dceutils` (libdce 2.51) library into a faithful, bug-compatible Rust implementation. All 88 compatibility test assertions generated directly from the PHP test suite compile and pass.

## Changes Made

### 1. Cargo Dependencies
- Added format dependencies (`ctb-formats-hexdump`, `ctb-formats-base64`, `ctb-formats-encoding`) and `csv` to the dependencies block of [Cargo.toml](ctoolbox/src/formats/dceutils/Cargo.toml).

### 2. Static Map Loading
- Created [tables.rs](ctoolbox/src/formats/dceutils/tables.rs) to lazy-initialize all translation tables from the CSV data under `data/csv` using `OnceLock`.

### 3. Delimiter Split Utility
- Created [tools.rs](ctoolbox/src/formats/dceutils/tools.rs) to implement `explode_escaped` with trailing backslash merge logic matching PHP.

### 4. Format Conversion Submodules
- Created [cdce.rs](ctoolbox/src/formats/dceutils/cdce.rs) for CDCE state machine decoding and encoding.
- Created [dce3_0a.rs](ctoolbox/src/formats/dceutils/dce3_0a.rs) for DCE 3.0a formatting.
- Created [dce3_01a.rs](ctoolbox/src/formats/dceutils/dce3_01a.rs) for DCE 3.01a formatting, including the transition search.
- Created [unicode.rs](ctoolbox/src/formats/dceutils/unicode.rs) for UTF-8, UTF-32, base64, and Dc64 formats.
- Created [legacy.rs](ctoolbox/src/formats/dceutils/legacy.rs) with legacy function stubs and `onestep_3_0a_old_to_none`.

### 5. Main Library Router
- Created [dceutils.rs](ctoolbox/src/formats/dceutils/dceutils.rs) as the main routing entrypoint exposing `dce_convert` and `get_dce_version`.

### 6. Automated Compatibility Tests
- Created [test_converter.py](ctoolbox/src/formats/dceutils/test_converter.py) to read `dceutils_tests.php` and generate the Rust equivalent test file [dceutils_tests.rs](ctoolbox/src/formats/dceutils/dceutils_tests.rs).
- This ensures that no manual translation hallucinations could impact the test data.

## Faithfully Replicated Quirks
- **Undefined Parameter Bug**: Replicated PHP's `$dc` bug where `convert_dc_to_3_01a_output` always returns the header/footer sequence `b"DCEe\x02\x01\x02\xfd\xfd\x03"`.
- **Switch State Name Bug**: Replicated PHP's state mismatch bug in `3_01a` decoder switch cases, where the `Whitespace_and_Punctuation` state case label does not match the actual state `Punctuation_and_Whitespace`, thereby bypassing updating the `append` output segment.
- **Intval Cast Fallback**: Replicated PHP's integer parsing on trailing non-numeric values (e.g. `'!'` casting to `0`), preventing the `,115` suffix formatting on CDCE strict error messages.

## Verification Results

We verified that the library builds cleanly and all compatibility tests pass:
```bash
cargo test -p ctb-formats-dceutils
```
```
running 4 tests
test to_csv::tests::test_dceutils_parsing ... ok
test to_csv::tests::test_dceutils_file_writing ... ok
test tools::tests::test_explode_escaped ... ok
test tests::test_libdce_compatibility ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Summary of Completed Work

1. **Submodule Architecture**:
   - **[tables.rs](ctoolbox/src/formats/dceutils/tables.rs)**: Lazy-loads all translation tables from `data/csv/` on first access using `OnceLock` and the `csv` crate.
   - **[tools.rs](ctoolbox/src/formats/dceutils/tools.rs)**: Implements `explode_escaped` with trailing-slash retry and trim semantics.
   - **[cdce.rs](ctoolbox/src/formats/dceutils/cdce.rs)**: Implements legacy CDCE and strict CDCE parsing and serialization, replicating recover/error flow.
   - **[dce3_0a.rs](ctoolbox/src/formats/dceutils/dce3_0a.rs)**: Implements standard 3.0a formatting/decoding and raw versions.
   - **[dce3_01a.rs](ctoolbox/src/formats/dceutils/dce3_01a.rs)**: Implements standard 3.01a state-machine transitions and `DcMapSendSimple` serialization, faithfully replicating the PHP `$dc` variable bug and the `Punctuation_and_Whitespace` state-name mismatch bug.
   - **[unicode.rs](ctoolbox/src/formats/dceutils/unicode.rs)**: Translates UTF-8, UTF-32, base64, and dc64 conversion steps utilizing sibling encoding/decoding crates.
   - **[legacy.rs](ctoolbox/src/formats/dceutils/legacy.rs)**: Provides wrappers and stubs for unmaintained legacy entrypoints.
   - **[dceutils.rs](ctoolbox/src/formats/dceutils/dceutils.rs)**: Exports the public interface (`dce_convert` and `get_dce_version`).

2. **Replicated Quirks & Bug Compatibility**:
   - Replicated PHP's integer casting semantics on trailing non-numeric values (e.g. `'!'` casting to `0`), ensuring strict CDCE error strings are formatted without the `,115` suffix.
   - Bypassed the outer mapping wrapper for base64/iconv/UTF-8 decoding errors in input formats to output standard format error strings matching PHP.

3. **Automated Conversion & Verification**:
   - Developed **[test_converter.py](ctoolbox/src/formats/dceutils/test_converter.py)** to translate the original `dceutils_tests.php` suite into static compile-time Rust tests (**[dceutils_tests.rs](ctoolbox/src/formats/dceutils/dceutils_tests.rs)**) without manual intervention.
   - Executed the test suite successfully with all 88 compatibility assertions passing cleanly:
     ```
     running 4 tests
     test to_csv::tests::test_dceutils_parsing ... ok
     test to_csv::tests::test_dceutils_file_writing ... ok
     test tools::tests::test_explode_escaped ... ok
     test tests::test_libdce_compatibility ... ok

     test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     ```

## Spec-Compliant Converters (`converter.rs`) vs. Legacy PHP Port

The legacy modules (`cdce.rs`, `dce3_0a.rs`, `dce3_01a.rs`, `unicode.rs`, `legacy.rs`, and the top-level `dce_convert` in `dceutils.rs`) are retained as a faithful, bug-compatible port of `old/libdce-2.51/` for side-by-side comparison.

A separate module, `converter.rs` (`ctb_formats_dceutils::converter`), implements the full DCE specification from `docs/` (`docs/converted/*.md`), addressing the gaps and bugs present in `old/libdce-2.51/`:

1. **Full Short Dc Repertoire by Version**:
   - **DCE 3.0a (`0..=115`)**: Supports all mapped short Dcs in `0..=115` (`0`, `1`, and `13..=115`, including `Dc 13..=17`, `Dc 112` on `DCE_Version_Selectors` (`Core:01`), and `Dc 113..=115`; `Dc 2..=12` had no byte mappings in 3.0a and are logged in `FormatLog` as unmappable and omitted).
   - **DCE 3.01a (`0..=199`)**: Supports all 200 allocated short Dcs (`0..=199`), unlocking Base64 encapsulation (`Dc 191` .. `Dc 127..=190` / `Dc 195` .. `Dc 192`) for arbitrary UTF-8 and data.
   - **DCE 3.02a (`0..=202`)**: Stubbed in `converter.rs` (returns an error when invoked). Allocates short Dcs `0..=202` (adding keyed embed framing `Dc 200` on `Core:C9`, `Dc 201` on `Core:CB`, `Dc 202` on `Core:CA`), with the version selector in `DCE_Version_Selectors` marked as TBD (`Dc 205` is `DCE Version Selector 3.03a`).
2. **Upstream Spreadsheet & PHP Table Fixes**:
   - **Base64 `'i'` (`Dc 161`, digit 34) vs `'I'` (`Dc 135`, digit 8)**: LibreOffice Calc auto-capitalized lowercase `'i'` to `'I'` in the `Base64` and `Dc to Base64` sheets of the `.ods` specification, causing `Base64_to_Dc.csv` to overwrite `'I'` -> `161` and omit `'i'`, and `Dc_to_Base64.csv` to map `161` -> `'I'`.
   - **`Dc 113` (`End DCE content`)**: Fixed the `Core:01` typo in `DCE Character List by ID` (`DcMapSend_dce3_0a.csv` and `DcMapSend_dce3_01a_All.csv`) to `Core:03`.
   - **`Dc 15` (`Begin DCE content`)**: Encodes interior `Dc 15` as `Core:02` rather than repeating the 5-byte `DCEe\x02` file magic.
   - **Complete 109 Direct Unicode Mappings**: Includes the 12 Unicode mappings added in DCE 3.01a (`0x08..=0x0D`, `0x85`, `0xFE00..=0xFE04` -> `Dc 116..=126`, `Dc 193`) that were missing from `DcMap_Unicode_Lossy.csv`.
   - **State Machine Transitions**: Emits `0xFD` (`Dc 13`) to return to `Core` before switching between two non-`Core` sheets or before emitting `0x81` (`Dc 115`), and fixes the `Punctuation_and_Whitespace` vs `Whitespace_and_Punctuation` sheet-name mismatch.

### 2. Known Gaps & What's Left
- Resolved by handling differently: **DCE 3.0a (`0..=115`) sheet-switch extension for `Dc 2 "Begin separator"`..`Dc 12 "Decimal point"`**:
  - In [DCE, version 3.0a r2 (26 December 2012).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.0a%20r2%20(26%20December%202012).md), `Dc 0 "Null (short Dc)"`..`Dc 115 "End unbounded content"` were allocated in the character list, but the `Mappings` column for `Dc 2 "Begin separator"`..`Dc 12 "Decimal point"` was left blank until `DCE, version 3.01a (27 December 2012)` added `Semantic_Records` (`Core:C6`) and `Mathematics` (`Core:C7`) the next day.
  - Because `Core:C6` and `Core:C7` are unassigned in 3.0a `Core`, [`converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs) allows 3.0a to encode and decode `Dc 2 "Begin separator"`..`Dc 12 "Decimal point"` using the 3.01a `Semantic_Records` / `Mathematics` sheet switches so all short Dcs in `Dc 0 "Null (short Dc)"`..`Dc 115 "End unbounded content"` roundtrip losslessly in `3_0a`.
  > Updated DceVersion::supports_short_dc, decode_dce_preserving_controls, and encode_short_dcs_into_stream so that when exporting to DCE 3.0a (3_0a, 3_0a_raw, hex_3_0a, hex_3_0a_raw), Dc 2 "Begin separator"..Dc 12 "Decimal point" are treated as unmappable, logged via log.export_warning_unmappable(...) in FormatLog, and omitted, while all mapped short Dcs in Dc 0 "Null (short Dc)"..Dc 115 "End unbounded content" (Dc 0 "Null (short Dc)", Dc 1 "At", and Dc 13 "Full drop back"..Dc 115 "End unbounded content") roundtrip losslessly.
- Apparently not an issue: ~~**DCE 3.02a (`0..=202`) `Semantic_Records` / `Mathematics` discrepancy in the 18 Oct 2013 spreadsheet**:
  - In [DCE, version 3.02a (18 October 2013).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.02a%20(18%20October%202013).md), the `DCE Character List by ID` sheet reassigned `Dc 6 "Begin number"`, `Dc 7 "End number"`, `Dc 10 "Positive"`, `Dc 11 "Negative"`, and `Dc 12 "Decimal point"` to `Mathematics:01..=05` and shifted `Dc 8 "Unknown value"` and `Dc 9 "Gregorian calendar identifier"` to `Semantic_Records:05..=06`, while the individual `Semantic Records` and `Mathematics` sheets in the same `.ods` file were never updated and still list the 3.01a layout (`Semantic_Records:01..=0A` for `Dc 2 "Begin separator"`..`Dc 11 "Negative"` and `Mathematics:01` for `Dc 12 "Decimal point"`).
  - [`converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs) currently uses the consistent 3.01a sheet layout across versions so 3.01a and 3.02a streams remain wire-compatible on `Semantic_Records` and `Mathematics`.~~ This issue appears to not exist:
  > In both [DCE, version 3.01a r2 (28 December 2012).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.01a%20r2%20(28%20December%202012).md) and [DCE, version 3.02a (18 October 2013).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.02a%20(18%20October%202013).md), `DCE Character List by ID` and the individual `Semantic Records` / `Mathematics` sheets consistently map `Dc 2 "Begin separator"`..`Dc 11 "Negative"` to `Semantic_Records:01..=0A` and only `Dc 12 "Decimal point"` to `Mathematics:01`. Removed the spurious `0x02..=0x05` branch from `DceSheet::Mathematics` in [`converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L725-L728).


### Summary of Findings & Changes

1. **Converted Specification Spreadsheets**:
   - Converted all 7 `.ods` spreadsheets in `src/formats/dceutils/docs/` to Markdown tables in `src/formats/dceutils/docs/converted/` ([DCE, version 3.0a r2 (26 December 2012).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.0a%20r2%20(26%20December%202012).md), [DCE, version 3.01a r2 (28 December 2012).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.01a%20r2%20(28%20December%202012).md), [DCE, version 3.02a (18 October 2013).md](file:///workspaces/ctoolbox/src/formats/dceutils/docs/converted/DCE,%20version%203.02a%20(18%20October%202013).md), etc.).

2. **Bugs Found in the Spreadsheets & Legacy `libdce-2.51` Tables**:

[`converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs) handles all of those bugs directly in its const mapping functions ([`base64_char_to_short_dc`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L183-L202), [`short_dc_to_base64_char`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L208-L227), [`unicode_codepoint_to_short_dc`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L289-L307), [`short_dc_to_unicode_codepoint`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L322-L340), [`encode_short_dc_in_version`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs#L343-L407)) without reading or modifying the legacy CSV tables in `src/formats/dceutils/data/csv/`.

   - **LibreOffice Calc auto-capitalization of `'i'` in Base64 (`Dc 161 "Base 64 encapsulation digit 34"` vs `Dc 135 "Base 64 encapsulation digit 8"`)**:
     - In the `Base64` and `Dc to Base64` sheets of 3.01a/3.02a (and [dceutils_data.php-Base64_to_Dc.csv](file:///workspaces/ctoolbox/src/formats/dceutils/data/csv/dceutils_data/dceutils_data.php-Base64_to_Dc.csv) / [dceutils_data.php-Dc_to_Base64.csv](file:///workspaces/ctoolbox/src/formats/dceutils/data/csv/dceutils_data/dceutils_data.php-Dc_to_Base64.csv)), lowercase `'i'` (Base64 index 34, `Dc 161 "Base 64 encapsulation digit 34"`) was auto-capitalized to `'I'`. In the legacy tables, `'I'` overwrote `Dc 135 "Base 64 encapsulation digit 8"` with `Dc 161 "Base 64 encapsulation digit 34"`, leaving `'i'` unmapped and corrupting any Base64 stream containing `'I'` or `'i'`.
     - The same Calc auto-capitalization also affected `Dc 90 "Latin Letter Lowercase I"` in the `Character` display column of `DCE Character List by ID` (and [dce3_0a_core.csv](file:///workspaces/ctoolbox/src/formats/dceutils/data/csv/dceutils_data_3_0a/dceutils_data_3_0a.php-dce3_0a_core.csv)).
   - **Typo in `Dc 113 "End DCE content"` send mapping (`Core:01` instead of `Core:03`)**:
     - In the `Core` sheet of 3.0a, 3.01a, and 3.02a, byte `0x01` is `Dc 16 "DCE Version Selectors"` (`>DCE_Version_Selectors`) and byte `0x03` is `Dc 113 "End DCE content"`. However, in `DCE Character List by ID` (and `DcMapSend_dce3_0a.csv` / `DcMapSend_dce3_01a_All.csv`), `Dc 113 "End DCE content"` was typed as `Core:01`.
   - **Missing send mappings for `Dc 14 "Drop back"`, `Dc 17 "No mapping"`, and `Dc 112 "DCE Version Selector 3.0a"`**:
     - `Dc 14 "Drop back"` (`0xFE`), `Dc 17 "No mapping"` (`0xFF`), and `Dc 112 "DCE Version Selector 3.0a"` (`DCE_Version_Selectors:01`) were defined in the `Core` and `DCE_Version_Selectors` receive sheets across all versions, but left blank in `DcMapSend_dce3_0a.csv` (and `Dc 17 "No mapping"` was blank in `DcMapSend_dce3_01a_All.csv`).
   - **Incomplete `DcMap_Unicode_Lossy` table in `libdce-2.51`**:
     - `DcMap_Unicode_Lossy` only contained the 97 mappings from 3.0a (`Dc 0 "Null (short Dc)"`, `Dc 1 "At"`, and `Dc 18 "Space (short Dc)"`..`Dc 111 "Tilde punctuation"`), omitting the 12 Unicode mappings added in 3.01a (`Dc 116 "Backspace (short Dc)"`..`Dc 121 "Ambiguous carriage return"`, `Dc 122 "Variant Selector 1"`..`Dc 126 "Variant Selector 5"`, and `Dc 193 "Form feed (short Dc)"`).
   - **Interior `Dc 15 "Begin DCE content"` encoding in 3.0a**:
     - `DcMapSend_dce3_0a.csv` mapped `Dc 15 "Begin DCE content"` to the full 5-byte `44 43 45 65 02` (`DCEe\x02`) magic header instead of `0x02` (`Core:02`).

3. **Restored Legacy Modules & Added Spec-Compliant [`converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs)**:
   - Restored [cdce.rs](file:///workspaces/ctoolbox/src/formats/dceutils/cdce.rs), [dce3_0a.rs](file:///workspaces/ctoolbox/src/formats/dceutils/dce3_0a.rs), [dce3_01a.rs](file:///workspaces/ctoolbox/src/formats/dceutils/dce3_01a.rs), [unicode.rs](file:///workspaces/ctoolbox/src/formats/dceutils/unicode.rs), and [dceutils.rs](file:///workspaces/ctoolbox/src/formats/dceutils/dceutils.rs) to match `old/libdce-2.51/` (`test_libdce_compatibility` passes all original assertions).
   - Added [`src/formats/dceutils/converter.rs`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs) supporting:
     - **DCE 3.0a**: lossless encoding/decoding of all mapped short Dcs in `Dc 0 "Null (short Dc)"`..`Dc 115 "End unbounded content"` (`0`, `1`, `13..=115`, with `Dc 2 "Begin separator"`..`Dc 12 "Decimal point"` logged as unmappable in `FormatLog` and omitted).
     - **DCE 3.01a**: lossless encoding/decoding of `Dc 0 "Null (short Dc)"`..`Dc 199 "Punctuation and Whitespace"`, plus Base64 UTF-8 encapsulation (`Dc 191 "Begin encapsulated UTF-8"` .. `Dc 127 "Base 64 encapsulation digit 0"`..`Dc 190 "Base 64 encapsulation digit 63"` / `Dc 195 "Base 64 encapsulation padding character"` .. `Dc 192 "End encapsulated UTF-8"`).
     - **DCE 3.02a**: lossless encoding/decoding of `Dc 0 "Null (short Dc)"`..`Dc 202 "End keyed embed ID sequence"` (adding `Dc 200 "Begin keyed embed fragment"`, `Dc 201 "End keyed embed fragment"`, and `Dc 202 "End keyed embed ID sequence"`, and version selector `0x03`).
   - Wired [src/formats/dcstring/converters/dceutils.rs](file:///workspaces/ctoolbox/src/formats/dcstring/converters/dceutils.rs) and [src/formats/conversion/convert_bytes.rs](file:///workspaces/ctoolbox/src/formats/conversion/convert_bytes.rs) to [`ctb_formats_dceutils::converter`](file:///workspaces/ctoolbox/src/formats/dceutils/converter.rs) while rejecting `Dc 308 "Next number is a long (global graph) Dc"` long Dcs with a diagnostic warning.
