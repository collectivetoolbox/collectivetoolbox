# Implementation Plan: DcList-to-UTF-8 and UTF-8-to-DcList Conversions

Implement full bidirectional conversion between `DcList` (`&[u128]`) and UTF-8 (`&[u8]`) in the `dctext` crate's `utf8` module (`src/formats/dctext/utf8.rs`).

## User Review Required

> [!NOTE]
> - `DcList` is a superset format of UTF-8, with `0..=0x10FFFF` directly representing Unicode codepoints and `CLASSIC_DC_OFFSET` (`1_114_112`) offsetting classic Dc IDs.
> - Settings fields use the prefix `dcl_basenb_*` to avoid confusion with classic EITE `dcbasenb` formats.
> - Encapsulated runs in `DcList` use new UUID sentinels:
>   - **Start UUID**: `1880aba3-21df-42b2-9c96-e32cd647ffc5`
>   - **End UUID**: `27efca19-0439-4bec-b58f-dfff5cd8db9f`
> - In `canonicalize_equivalent_dcs` mode, when converting from UTF-8 / processing classic Dcs, any legacy base64 UTF-8 embedded sequences (classic Dc `191` start marker ... base64 Dcs ... `192` end marker) present in converted legacy documents are decoded to their raw UTF-8 bytes. Legacy base64 embeds are never emitted in output outside of lossless roundtripping.

## Open Questions

None.

## Proposed Changes

### `ctb-formats-dctext`

#### [MODIFY] [Cargo.toml](~/ctoolbox/src/formats/dctext/Cargo.toml)
- Add `ctb-formats-utf8 = { path = "../utf8" }` dependency if needed for UTF-8 helpers and replacement character constants.

#### [MODIFY] [dctext.rs](~/ctoolbox/src/formats/dctext/dctext.rs)
- Export module: `pub mod utf8;`.
- Re-export settings and primary conversion functions (`dclist_to_utf8`, `dclist_from_utf8`, `utf8_to_dclist`, `DcListUtf8Settings`).

#### [MODIFY] [utf8.rs](~/ctoolbox/src/formats/dctext/utf8.rs)
- Define `DcListUtf8Settings`:
  - `dcl_basenb_enabled: bool` (encapsulate unmappables using base17/dcBasenb encoding)
  - `dcl_basenb_fragment_enabled: bool` (encapsulate without UUID sentinels)
  - `dcl_basenb_fragment_strict: bool` (strict error handling on fragment decode)
  - `skip_unmappable: bool` (skip unmappable Dcs when encapsulation is disabled)
  - `canonicalize_equivalent_dcs: bool` (map classic Dcs to Unicode if mapping exists, and parse legacy base64 embeds if present)
  - `debug: bool`
  - Implement `Default` and `ConstDefault`.
- Define UUID Constants for `DcList`:
  - `DCL_BASENB_START_UUID_RAW`: `[0x18, 0x80, 0xab, 0xa3, 0x21, 0xdf, 0x42, 0xb2, 0x9c, 0x96, 0xe3, 0x2c, 0xd6, 0x47, 0xff, 0xc5]` (`1880aba3-21df-42b2-9c96-e32cd647ffc5`)
  - `DCL_BASENB_END_UUID_RAW`: `[0x27, 0xef, 0xca, 0x19, 0x04, 0x39, 0x4b, 0xec, 0xb5, 0x8f, 0xdf, 0xff, 0x5c, 0xd8, 0xdb, 0x9f]` (`27efca19-0439-4bec-b58f-dfff5cd8db9f`)
  - Encode sentinels into `DCL_BASENB_EMBEDDED_START_BYTES` and `DCL_BASENB_EMBEDDED_END_BYTES`.
- Implement `dclist_to_utf8(dclist: &[u128], settings: &DcListUtf8Settings) -> Result<ConversionOutput<Vec<u8>>>`:
  - Iterate `u128` values in `dclist`.
  - `0..=0x10FFFF` (excluding surrogates `0xD800..=0xDFFF`): encode as standard UTF-8 bytes.
  - `CLASSIC_DC_OFFSET..=CLASSIC_DC_OFFSET + max_classic_dc`:
    - If `canonicalize_equivalent_dcs == true`:
      - Check for legacy base64 UTF-8 embedded sequence starting with classic Dc 191 (`CLASSIC_DC_OFFSET + 191`) and ending with 192. If valid, decode base64 sequence to raw UTF-8 bytes and emit.
      - Otherwise, check `dc_to_format("utf8", classic_dc)`. If mapped to Unicode, emit UTF-8 bytes; otherwise treat as unmappable.
    - If `canonicalize_equivalent_dcs == false`: treat as unmappable (encapsulate/replace/skip) for lossless roundtripping.
  - Unmappable Dcs (`> 0x10FFFF`, surrogates, non-canonical/unmapped classic Dcs):
    - Buffer into `unmappables` if `dcl_basenb_enabled` is `true`. Flush runs using `encode_utf_8e_128` + `byte_array_to_basenb_17_utf8`, wrapped with `DCL_BASENB_EMBEDDED_START` / `END` UUID sentinels unless `dcl_basenb_fragment_enabled` is set.
    - Emit warning log and UTF-8 replacement character `U+FFFD` (`[0xEF, 0xBF, 0xBD]`) if `dcl_basenb_enabled` is `false` and `!skip_unmappable`.
    - Omit if `skip_unmappable` is `true`.
- Implement `dclist_from_utf8(utf8_bytes: &[u8], settings: &DcListUtf8Settings) -> Result<ConversionOutput<DcList>>` (and `utf8_to_dclist`):
  - Scan UTF-8 input bytes.
  - Detect `DcList` `basenb` regions using `DCL_BASENB` UUID sentinels (or fragment runs if `dcl_basenb_fragment_enabled` is set).
  - Decode `basenb` runs via `byte_array_from_basenb_17_utf8` and `decode_utf_8e_128_buf` to reconstruct `u128` Dcs.
  - Decode standard UTF-8 characters to `u128` codepoint values (`0..=0x10FFFF`).
- Add unit tests covering:
  - Basic Unicode text conversion
  - Encapsulated `dcl_basenb` with new UUID sentinels (`1880aba3-...` and `27efca19-...`) and fragment mode
  - Replacement character vs. skipping unmappables
  - Canonicalization of classic Dcs and parsing legacy base64 UTF-8 embeds
  - Roundtrip integrity checks

---

## Verification Plan

### Automated Tests
- Run quick type checking: `./lint --quick`
- Run unit tests for `ctb-formats-dctext`: `cargo test -p ctb-formats-dctext`

### Manual Verification
- Verify that `canonicalize_equivalent_dcs` maps classic Dcs (e.g. Dc 65 -> 'A') to Unicode bytes and parses legacy base64 embeds (191..base64..192) into UTF-8 bytes when enabled.
- Verify that new UUID sentinels (`1880aba3-21df-42b2-9c96-e32cd647ffc5` / `27efca19-0439-4bec-b58f-dfff5cd8db9f`) are used for `dcl_basenb` armoring.
- Verify lossless roundtrip of unmappable `u128` Dcs via `dclist_to_utf8` and `dclist_from_utf8` with `dcl_basenb_enabled = true`.
