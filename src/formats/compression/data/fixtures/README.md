# Compression Data Fixtures

The 34 compressed fixtures in this directory expand to the 2,086-byte Panorama
database [example2 with lemurs.pan](example2%20with%20lemurs.pan).
Historical sources live in [old/unix-tools](../../../../../old/unix-tools).
Names distinguish historical versions, synthetic in-repository encoders
(`synthetic`), wrapped Rust libraries (`ctblib`), and independent encoders
(`external`). ZPAQ is intentionally absent.

## Fixture Inventory

Each suffix below follows `example2 with lemurs.pan`.

| Suffix | Encoder / format | Header | Bytes |
| --- | --- | --- | ---: |
| `.1.0.Z` | compress 1.0, headerless LZW | None | 958 |
| `.1.6.Z` | compress 1.6, VAX-layout port, early initial width transition | None | 1022 |
| `.2.0.Z` | compress 2.0, non-block LZW | `1f 9d 10` | 993 |
| `.3.0.Z` | compress 3.0, block LZW | `1f 9d 90` | 953 |
| `.12.Z` | compress 4.0, 12-bit block LZW | `1f 9d 8c` | 953 |
| `.Z` | ncompress, 16-bit block LZW | `1f 9d 90` | 953 |
| `.z` | System III pack, canonical Huffman | `1f 1e` | 1057 |
| `.old.z` | Zucker old-pack, PDP-11 tree representation | `1f 1f` | 1404 |
| `.C` | BSD compact, adaptive Huffman | `ff 1f` | 998 |
| `.bz` | bzip 0.21, original bzip1 format | `42 5a 30` | 769 |
| `.2.0.F` | freeze 2.5, Freeze 2 format | `1f 9f` | 883 |
| `.synthetic.1.0.F` | ctoolbox Freeze 1, fixed position table | `1f 9e` | 866 |
| `.synthetic.sco` | ctoolbox SCO compress -H, LZH | `1f a0` | 850 |
| `.rz` | rzip 2.1, long-range matching and bzip2 blocks | `52 5a 49 50` | 1007 |
| `.gz` | gzip, RFC 1952 | `1f 8b` | 850 |
| `.bz2` | bzip2 | `42 5a 68` | 879 |
| `.br` | Brotli | No fixed magic | 738 |
| `.xz` | XZ, LZMA2 container | `fd 37 7a 58 5a 00` | 856 |

The following formats have **both** library-generated and independently
generated fixtures. Every file is decoded by both implementations during
generation and read-only verification.

An external oracle checks interoperability, not complete algorithmic
independence: for example, the zstd CLI and Rust wrapper can share upstream
libzstd code.

| Format | Library suffix / bytes | External suffix / bytes | Independent oracle |
| --- | --- | --- | --- |
| Raw Deflate, RFC 1951 | `.ctblib.deflate` / 828 | `.external.deflate` / 832 | Python zlib, `wbits=-15` |
| Zlib, RFC 1950 | `.ctblib.zz` / 834 | `.external.zz` / 838 | Python zlib |
| LZ4 frame | `.ctblib.lz4` / 1120 | `.external.lz4` / 1107 | lz4 CLI |
| LZMA-alone | `.ctblib.lzma` / 803 | `.external.lzma` / 803 | xz `--format=lzma` |
| Raw LZMA2 | `.ctblib.lzma2` / 792 | `.external.lzma2` / 792 | xz, explicit 8-MiB dictionary |
| Lzip | `.ctblib.lz` / 816 | `.external.lz` / 817 | lzip CLI |
| Zstandard frame | `.ctblib.zst` / 879 | `.external.zst` / 883 | zstd CLI |
| Raw LZO1X | `.ctblib.lzo` / 948 | `.external.lzo` / 1064 | reference liblzo2 |

Raw LZO is not an lzop container. It has neither the lzop magic header nor an
embedded output length; the test oracle receives the expected raw length.

## Generation and Verification

[build-historic-compressors](build-historic-compressors) builds disposable
copies of the historical sources. Source substitutions fail if their anchors
are absent. The current build recipes are tested on little-endian LP64 Linux;
their native-word assumptions are not a cross-platform historical emulator.

Test-only prerequisites: Bash, GCC, Python 3, gzip, bzip2, Brotli, XZ, lzip,
lz4, zstd, the bzip2 development library (for historical rzip), and liblzo2.
These are external test tools, not application dependencies.

Build a current CLI before testing. `CTB_BIN` is an executable path, not a
shell command; without it the scripts may reuse an existing binary.

```bash
cargo build --release -p ctoolbox --bin ctoolbox
export CTB_BIN="$PWD/target/release/ctoolbox"

# Read committed bytes without regenerating or changing them.
src/formats/compression/data/fixtures/generate-compression-fixtures --verify-only

# Stage all outputs, verify every file, then publish them.
src/formats/compression/data/fixtures/generate-compression-fixtures

# Generate into a separate directory for comparison.
src/formats/compression/data/fixtures/generate-compression-fixtures --output-dir /tmp/compression-fixtures
```

Verification uses ctoolbox plus the historical decoder or independent library
for every fixture. Synthetic SCO is verified by gunzip; synthetic Freeze 1 by
freeze 2.5's COMPAT decoder. Generation failures leave existing fixtures
untouched. Gzip uses `-n` so source filenames and timestamps cannot cause churn.
Compressed bytes can still change across encoder versions or settings; exact
byte equality is not a validity requirement.

## Compatibility Testing

```bash
cargo test -p ctb-formats-compression --lib
python3 -B tests/test_compressor_runner.py
src/formats/compression/data/fixtures/test-compressors --fast
src/formats/compression/data/fixtures/test-compressors --fast --jobs 1
src/formats/compression/data/fixtures/test-compressors --dataset dickens
src/formats/compression/data/fixtures/test-compressors
```

[test-compressors](test-compressors) requires Bash 5.1 or newer and runs four
cases concurrently by default. Use `--jobs N` (or `JOBS=N`) to adjust the
limit; `--jobs 1` runs serially. Each matrix/dataset pair gets isolated scratch
files and ctoolbox storage/cache via `CTB_TEST_STORAGE_DIR`. Forward and reverse
checks remain sequential within each pair. Worker failures are collected while
other cases continue, and any failure makes the script exit nonzero. Case logs
are printed together as workers finish.

The final report lists each direction's status, format, oracle, dataset, input
and compressed byte counts, compression ratio, and compression wall time.
Ratios are compressed bytes divided by input bytes, so smaller is better.
Times include process startup and I/O, but exclude decompression and comparison;
parallel timings include resource contention, so use `--jobs 1` for timing
comparisons. Empty-input ratios and unavailable measurements use `-`. Archived
fixtures show their existing size ratio but no compression time. Known skips
and failures remain visible in the report; setup failures still abort early.

The quick matrix covers empty and 1/2/3-byte inputs, repetitive data, the raw
fixture, and deterministic random inputs of 65,535/65,536/65,537 bytes. It tests
both directions against external implementations, including 12-bit LZW. Full
By default (with no arguments), `test-compressors` executes across both the
quick synthetic datasets and the full corpora from `old/corpora`. The two
genuine SCO streams in the ancient fixture corpus are checked against their
known raw files and gunzip as well (missing tools, corpora, or archive fixtures
are treated as hard failures resulting in exit code 1).

Known historical empty-input defects, one-byte Compact output refusal, and
pack's explicit trivial/no-savings refusals are reported as skips. Other tool
errors and missing outputs fail the run, as does selecting no tests. Old-pack
uses `-s` so incompressible input is exercised instead of silently skipped.
Rust tests still require empty round trips except for Compact, which explicitly
rejects empty input.

## Audit of e3ab1ce13 and 10d1a6048

- Both versions of the changed Brotli (823 -> 738), gzip (867 -> 850), XZ
  (848 -> 856), Deflate (832 -> 828), and zlib (838 -> 834) fixtures decode to
  the original raw bytes with independent decoders. These changes are valid
  encoder/metadata variations, not evidence of data corruption.
- The original 1,022-byte non-VAX compress 1.6 fixture was reproducible but
  broken: its original decoder did not reproduce the input, even for `ABC`.
  The commit's 990-byte replacement was decodable but also changed the initial
  threshold from 256 to 511, masking a format-compatibility defect. The current
  1,022-byte fixture is **different** from the original: it ports both VAX bit
  operations while preserving the historical threshold. The Rust codec now
  preserves that initial threshold too. See the
  [LZW format notes](../docs/compress-ncompress.md).
- The 1.0 bit-extraction port could read beyond its buffer. It now extracts
  only the required bits. The Compact union-copy patch previously never
  matched its source; it now applies and is checked. Other build patches
  concern pointer/union representation, capacity, CLI exits, signal callback
  types, file-type checks, or unsigned size words, rather than deliberately
  changing coding rules. Compatibility tests validate their observed output;
  this is not proof of equivalence on every historical architecture.
- The shared-test refactor removed strict Freeze empty-input checks and
  replaced mixed-pattern repetition with a single repeated byte. Both gaps
  are corrected, and missing raw fixtures now fail instead of using fallback
  data. All eight new independent fixtures are included in Rust tests.

Validation during this review: 39 compression-crate tests, 568 quick matrix
comparisons (18 explicit skips), 59 comparisons on the 10-MB Dickens corpus
(no skips), and all 34 fixtures through both decoders. The complete 416-MB
corpus was not run. Original-hardware equivalence, a genuine Freeze 1 encoder,
and a native SCO encoder remain outside this validation.