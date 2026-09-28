# Compression Data Fixtures

This directory contains uncompressed and compressed test fixture files generated from [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) using the historical Unix compression utilities in [`old/unix-tools`](file:///workspaces/ctoolbox/old/unix-tools), modern system encoders, and `ctoolbox` implementations.

---

## Fixtures Overview

| Filename | Source / Utility Variant | Magic Header Bytes | Algorithm & Features | Size |
| :--- | :--- | :---: | :--- | :---: |
| [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) | Raw Input Data | None | Raw uncompressed PAN image asset | 2,086 B |
| [`example2 with lemurs.pan.1.0.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.1.0.Z) | `compress 1.0` (Headerless) | None (Headerless) | Spencer W. Thomas (July 4 1984) original headerless LZW | 953 B |
| [`example2 with lemurs.pan.1.6.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.1.6.Z) | `compress 1.6` (Sorted Chain) | None (Headerless) | Joe Orost (August 1 1984) sorted-chain headerless LZW | 990 B |
| [`example2 with lemurs.pan.2.0.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.2.0.Z) | `compress 2.0` (Non-Block) | `0x1F 0x9D 0x10` | Turkowski & Orost (Aug 28 1984) LZW without block mode | 993 B |
| [`example2 with lemurs.pan.3.0.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.3.0.Z) | `compress 3.0` (Block Mode) | `0x1F 0x9D 0x90` | Woods & Orost (Jan 1985) LZW with `BLOCK_MODE` bit `0x80` & `CLEAR` code | 953 B |
| [`example2 with lemurs.pan.12.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.12.Z) | `compress 4.0` (`-b 12`) | `0x1F 0x9D 0x8C` | 1986 LZW with maxbits restricted to 12 (`0x80 \| 12`) | 953 B |
| [`example2 with lemurs.pan.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z) | `ncompress` Standard LZW | `0x1F 0x9D 0x90` | Modern ncompress 16-bit block LZW stream | 953 B |
| [`example2 with lemurs.pan.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.z) | System III/V `pack` | `0x1F 0x1E` | Canonical Huffman coding with level leaf table | 1,057 B |
| [`example2 with lemurs.pan.old.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.old.z) | Early Unix `pack` | `0x1F 0x1F` | Steve Zucker ~1977 PDP-11 binary tree dictionary | 1,404 B |
| [`example2 with lemurs.pan.C`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.C) | `compact` (Adaptive Huffman) | `0x1F 0xFF` / `0xFF 0x1F` | McMaster's 1979 Online Adaptive Huffman Coder | 998 B |
| [`example2 with lemurs.pan.bz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.bz) | `bzip 0.21` | `0x42 0x5A 0x30` | Julian Seward 1996 original bzip1 format | 769 B |
| [`example2 with lemurs.pan.2.0.F`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.2.0.F) | `freeze 2.5` | `0x1F 0x9F` | Leonid Broukhis Freeze 2.X format | 883 B |
| [`example2 with lemurs.pan.synthetic.1.0.F`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.synthetic.1.0.F) | Synthetic (`ctoolbox`) | `0x1F 0x9E` | Freeze 1.0 format (LZSS + Dynamic Huffman with fixed Table 1) | 884 B |
| [`example2 with lemurs.pan.synthetic.sco`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.synthetic.sco) | Synthetic (`ctoolbox`) | `0x1F 0xA0` | SCO `compress -H` (LZSS sliding window dictionary + static Huffman) | 954 B |
| [`example2 with lemurs.pan.gz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.gz) | System `gzip` | `0x1F 0x8B` | RFC 1952 DEFLATE stream wrapped in Gzip container | 867 B |
| [`example2 with lemurs.pan.bz2`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.bz2) | System `bzip2` | `0x42 0x5A 0x68` | Bzip2 format | 879 B |
| [`example2 with lemurs.pan.br`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.br) | System `brotli` | None (Stream) | Brotli sliding-window LZ77 + Huffman | 823 B |
| [`example2 with lemurs.pan.xz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.xz) | System `xz` | `0xFD 0x37 0x7A 0x58 0x5A 0x00` | XZ container format (LZMA2) | 848 B |
| [`example2 with lemurs.pan.ctblib.deflate`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.deflate) | Wrapped library (`ctblib`) | None (Stream) | RFC 1951 raw DEFLATE stream | 832 B |
| [`example2 with lemurs.pan.ctblib.zz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.zz) | Wrapped library (`ctblib`) | `0x78 0x9C` | RFC 1950 Zlib-wrapped DEFLATE stream | 838 B |
| [`example2 with lemurs.pan.ctblib.lz4`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.lz4) | Wrapped library (`ctblib`) | `0x04 0x22 0x4D 0x18` | LZ4 Frame compression | 1,120 B |
| [`example2 with lemurs.pan.ctblib.lzma`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.lzma) | Wrapped library (`ctblib`) | `0x5D 0x00 0x00` | LZMA stream format | 803 B |
| [`example2 with lemurs.pan.ctblib.lzma2`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.lzma2) | Wrapped library (`ctblib`) | None (Stream) | LZMA2 raw stream format | 792 B |
| [`example2 with lemurs.pan.ctblib.lz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.lz) | Wrapped library (`ctblib`) | `0x4C 0x5A 0x49 0x50` | Lzip format (LZMA-based) | 816 B |
| [`example2 with lemurs.pan.ctblib.zst`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.zst) | Wrapped library (`ctblib`) | `0x28 0xB5 0x2F 0xFD` | Zstandard compressed frame | 879 B |
| [`example2 with lemurs.pan.ctblib.lzo`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.ctblib.lzo) | Wrapped library (`ctblib`) | None (Stream) | LZO byte stream | 948 B |

---

## Detailed Generation Steps

All historical fixtures in this directory are generated directly from their respective source trees under [`old/unix-tools`](file:///workspaces/ctoolbox/old/unix-tools) using the shared compiler [`build-historic-compressors`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/build-historic-compressors) and automated generator [`generate-compression-fixtures`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/generate-compression-fixtures).

### Execution Command
To re-compile all historical tools and re-generate all 23 fixtures, run:
```bash
./src/formats/compression/data/fixtures/generate-compression-fixtures
```

### Compatibility Testing
To run the full forward and reverse compatibility suite across historical tools, system tools, and corpora:
```bash
./src/formats/compression/data/fixtures/test-compressors
# or for fast synthetic dataset testing:
./src/formats/compression/data/fixtures/test-compressors --fast
```