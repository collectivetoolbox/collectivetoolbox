# Walkthrough: DCEUtils Port Implementation

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
   - **DCE 3.0a (`0..=115`)**: Supports all 116 allocated short Dcs (`0..=115`), including `Dc 2..=11` on `Semantic_Records` (`Core:C6`) and `Dc 12` on `Mathematics` (`Core:C7`), `Dc 13..=17`, `Dc 112` on `DCE_Version_Selectors` (`Core:01`), and `Dc 113..=115`.
   - **DCE 3.01a (`0..=199`)**: Supports all 200 allocated short Dcs (`0..=199`), unlocking Base64 encapsulation (`Dc 191` .. `Dc 127..=190` / `Dc 195` .. `Dc 192`) for arbitrary UTF-8 and data.
   - **DCE 3.02a (`0..=202`, plus `Dc 205`)**: Supports all 203 allocated short Dcs (`0..=202`), adding keyed embed framing (`Dc 200` on `Core:C9`, `Dc 201` on `Core:CB`, `Dc 202` on `Core:CA`) and version selector `Dc 205` (`DCE_Version_Selectors:03`).
2. **Upstream Spreadsheet & PHP Table Fixes**:
   - **Base64 `'i'` (`Dc 161`, digit 34) vs `'I'` (`Dc 135`, digit 8)**: LibreOffice Calc auto-capitalized lowercase `'i'` to `'I'` in the `Base64` and `Dc to Base64` sheets of the `.ods` specification, causing `Base64_to_Dc.csv` to overwrite `'I'` -> `161` and omit `'i'`, and `Dc_to_Base64.csv` to map `161` -> `'I'`.
   - **`Dc 113` (`End DCE content`)**: Fixed the `Core:01` typo in `DCE Character List by ID` (`DcMapSend_dce3_0a.csv` and `DcMapSend_dce3_01a_All.csv`) to `Core:03`.
   - **`Dc 15` (`Begin DCE content`)**: Encodes interior `Dc 15` as `Core:02` rather than repeating the 5-byte `DCEe\x02` file magic.
   - **Complete 109 Direct Unicode Mappings**: Includes the 12 Unicode mappings added in DCE 3.01a (`0x08..=0x0D`, `0x85`, `0xFE00..=0xFE04` -> `Dc 116..=126`, `Dc 193`) that were missing from `DcMap_Unicode_Lossy.csv`.
   - **State Machine Transitions**: Emits `0xFD` (`Dc 13`) to return to `Core` before switching between two non-`Core` sheets or before emitting `0x81` (`Dc 115`), and fixes the `Punctuation_and_Whitespace` vs `Whitespace_and_Punctuation` sheet-name mismatch.
