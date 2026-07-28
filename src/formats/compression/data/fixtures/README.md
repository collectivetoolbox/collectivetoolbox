# Compression Data Fixtures

This directory contains uncompressed and compressed test fixture files generated from [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) using the historical Unix compression utilities in [`old/unix-tools`](file:///workspaces/ctoolbox/old/unix-tools) as well as modern encoders.
NOTE: old/unix-tools is not included in this repository except temporarily to generate fixtures, so you'll have to locate them on Usenet archive, etc. to re-generate (I'd encourage that! If you don't get the same results, please let me know and I can investigate.)

---

## Fixtures Overview

| Filename | Format / Utility Variant | Magic Header Bytes | Algorithm & Features | Size | SHA-512 Prefix |
| :--- | :--- | :---: | :--- | :---: | :---: |
| [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) | Raw Input Data | None | Raw uncompressed PAN image asset | 2,086 B | `978683ef9d39...` |
| [`example2 with lemurs.pan.Z1.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z1.0) | `compress 1.0` (Headerless) | None (Headerless) | Spencer W. Thomas (July 4 1984) original headerless LZW | 953 B | `18d0b0c7e481...` |
| [`example2 with lemurs.pan.Z1.6`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z1.6) | `compress 1.6` (Sorted Chain) | None (Headerless) | Joe Orost (August 1 1984) sorted-chain headerless LZW | 993 B | `be0114bc198f...` |
| [`example2 with lemurs.pan.Z2.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z2.0) | `compress 2.0` (Non-Block) | `0x1F 0x9D 0x10` | Turkowski & Orost (Aug 28 1984) LZW without block mode | 993 B | `3d285b5c9e4b...` |
| [`example2 with lemurs.pan.Z3.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z3.0) | `compress 3.0` (Block Mode) | `0x1F 0x9D 0x90` | Woods & Orost (Jan 1985) LZW with `BLOCK_MODE` bit `0x80` & `CLEAR` code | 953 B | `44934371a4f1...` |
| [`example2 with lemurs.pan.Z12`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z12) | `compress 4.0` (`-b 12`) | `0x1F 0x9D 0x8C` | 1986 LZW with maxbits restricted to 12 (`0x80 \| 12`) | 953 B | `e38101abe904...` |
| [`example2 with lemurs.pan.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z) | `ncompress` Standard LZW | `0x1F 0x9D 0x90` | Modern ncompress 16-bit block LZW stream | 953 B | `44934371a4f1...` |
| [`example2 with lemurs.pan.br`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.br) | Brotli | None (Stream) | Brotli sliding-window LZ77 + Huffman | 823 B | `8c02e1f41be1...` |
| [`example2 with lemurs.pan.deflate`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.deflate) | Raw Deflate | None (Stream) | RFC 1951 raw DEFLATE stream | 832 B | `9119daf9075d...` |
| [`example2 with lemurs.pan.gz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.gz) | Gzip | `0x1F 0x8B` | RFC 1952 DEFLATE stream wrapped in Gzip container | 867 B | `da25f701d2c5...` |
| [`example2 with lemurs.pan.zz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.zz) | Zlib | `0x78 0x9C` | RFC 1950 Zlib-wrapped DEFLATE stream | 838 B | `9ef6eae2bd5c...` |
| [`example2 with lemurs.pan.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.z) | System III/V `pack` | `0x1F 0x1E` | Canonical Huffman coding with level leaf table | 1,057 B | `7f8319d5cf5d...` |
| [`example2 with lemurs.pan.old.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.old.z) | Early Unix `pack` | `0x1F 0x1F` | Steve Zucker ~1977 PDP-11 binary tree dictionary | 1,404 B | `3aef34365c10...` |
| [`example2 with lemurs.pan.sco`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.sco) | SCO `compress -H` | `0x1F 0xA0` | LZSS sliding window dictionary + static Huffman | 954 B |
| [`example2 with lemurs.pan.C`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.C) | `compact` (Adaptive Huffman) | `0x1F 0xFF` / `0xFF 0x1F` | McMaster's 1979 Online Adaptive Huffman Coder | 998 B |
---

## Hexdump Header Comparison of LZW Formats

```text
=== example2 with lemurs.pan.Z1.0 (compress 1.0, Spencer Thomas July 1984) ===
00000000  22 10 00 10 48 a1 0c 9e  30 6d e0 b0 29 23 03 c4

=== example2 with lemurs.pan.Z1.6 (compress 1.6, Joe Orost Aug 1 1984) ===
00000000  00 00 02 11 00 00 00 00  00 50 20 10 00 11 06 1e

=== example2 with lemurs.pan.Z2.0 (compress 2.0, Turkowski & Orost Aug 28 1984) ===
00000000  1f 9d 10 22 10 00 10 48  a1 0c 9e 30 6d e0 b0 29

=== example2 with lemurs.pan.Z12 (compress 4.0 -b 12, Thomas et al. 1986) ===
00000000  1f 9d 8c 22 10 00 18 48  a1 0c 9e 30 6d e0 b0 29

=== example2 with lemurs.pan.Z3.0 (compress 3.0, Woods & Orost Jan 1985) ===
00000000  1f 9d 90 22 10 00 18 48  a1 0c 9e 30 6d e0 b0 29

=== example2 with lemurs.pan.Z (ncompress, Jannesen & Frysinger) ===
00000000  1f 9d 90 22 10 00 18 48  a1 0c 9e 30 6d e0 b0 29
```

---

## Detailed Generation Steps

Every historical fixture in this directory is generated directly from its respective source tree under [`old/unix-tools/compress`](file:///workspaces/ctoolbox/old/unix-tools/compress) and [`old/unix-tools/ncompress`](file:///workspaces/ctoolbox/old/unix-tools/ncompress) using the automated build script [`generate-compression-fixtures`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/generate-compression-fixtures).

### Execution Command
To re-compile all historical tools and re-generate all fixtures, run:
```bash
./src/formats/compression/data/fixtures/generate-compression-fixtures
# or:
./scripts/generate-compression-fixtures
```
