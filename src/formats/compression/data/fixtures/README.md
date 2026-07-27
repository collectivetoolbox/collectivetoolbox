# Compression Data Fixtures

This directory contains uncompressed and compressed test fixture files generated from [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) using the historical Unix compression utilities in [`old/unix-tools`](file:///workspaces/ctoolbox/old/unix-tools) as well as modern encoders.

---

## Fixtures Overview

| Filename | Format / Utility Variant | Magic Header Bytes | Algorithm & Features | Size |
| :--- | :--- | :---: | :--- | :---: |
| [`example2 with lemurs.pan`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan) | Raw Input Data | None | Raw uncompressed PAN image asset | 2,086 B |
| [`example2 with lemurs.pan.br`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.br) | Brotli | None (Stream) | Brotli sliding-window LZ77 + Huffman | 823 B |
| [`example2 with lemurs.pan.deflate`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.deflate) | Raw Deflate | None (Stream) | RFC 1951 raw DEFLATE stream | 832 B |
| [`example2 with lemurs.pan.gz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.gz) | Gzip | `0x1F 0x8B` | RFC 1952 DEFLATE stream wrapped in Gzip container | 867 B |
| [`example2 with lemurs.pan.zz`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.zz) | Zlib | `0x78 0x9C` | RFC 1950 Zlib-wrapped DEFLATE stream | 838 B |
| [`example2 with lemurs.pan.C`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.C) | `compact` (Adaptive Huffman) | `0x1F 0xFF` / `0xFF 0x1F` | McMaster's 1979 Online Adaptive Huffman Coder | 998 B |
| [`example2 with lemurs.pan.Z1.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z1.0) | `compress 1.0` (Headerless) | None (Headerless) | Spencer Thomas 1984 9–16 bit LZW stream | 953 B |
| [`example2 with lemurs.pan.Z2.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z2.0) | `compress 2.0` (Non-Block) | `0x1F 0x9D 0x10` | 1984 LZW without adaptive block reset bit (`0x80` is 0) | 993 B |
| [`example2 with lemurs.pan.Z3.0`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z3.0) | `compress 3.0` (Block Mode) | `0x1F 0x9D 0x90` | 1985 LZW with `BLOCK_MODE` bit `0x80` & `CLEAR` code 256 | 953 B |
| [`example2 with lemurs.pan.Z12`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z12) | `compress 4.0` (`-b 12`) | `0x1F 0x9D 0x8C` | 1986 LZW with maxbits restricted to 12 (`0x80 \| 12`) | 953 B |
| [`example2 with lemurs.pan.Z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.Z) | `ncompress` Standard LZW | `0x1F 0x9D 0x90` | Standard modern 16-bit block LZW stream | 953 B |
| [`example2 with lemurs.pan.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.z) | System III/V `pack` | `0x1F 0x1E` | Canonical Huffman coding with level leaf table | 1,057 B |
| [`example2 with lemurs.pan.old.z`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.old.z) | Early Unix `pack` | `0x1F 0x1F` | Steve Zucker ~1977 PDP-11 binary tree dictionary | 1,404 B |
| [`example2 with lemurs.pan.sco`](file:///workspaces/ctoolbox/src/formats/compression/data/fixtures/example2%20with%20lemurs.pan.sco) | SCO `compress -H` | `0x1F 0xA0` | LZSS sliding window dictionary + static Huffman | 954 B |

---

## Detailed Generation & Compilation Steps

### 1. `example2 with lemurs.pan.C` (Compact / McMaster Adaptive Huffman)
* **Source Tools**: [`old/unix-tools/compact-uncompact/`](file:///workspaces/ctoolbox/old/unix-tools/compact-uncompact) ([`4.1cBSD-compact.c`](file:///workspaces/ctoolbox/old/unix-tools/compact-uncompact/4.1cBSD-compact.c), [`4.1cBSD-tree.c`](file:///workspaces/ctoolbox/old/unix-tools/compact-uncompact/4.1cBSD-tree.c), [`4.1cBSD-compact.h`](file:///workspaces/ctoolbox/old/unix-tools/compact-uncompact/4.1cBSD-compact.h))
* **Specification Document**: [`compact.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compact.md)
* **Compilation**:
  Header `compact.h` was adjusted for 64-bit systems (`intptr_t` for `union treep` `ch` and `union cio` `integ`, and `int argc`), then compiled with `gcc -w -std=gnu89 -fcommon`.
* **Generation Command**:
  ```bash
  ./compact < "example2 with lemurs.pan" > "example2 with lemurs.pan.C"
  ```
* **Decompression Verification**:
  ```bash
  ./uncompact < "example2 with lemurs.pan.C" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 2. `example2 with lemurs.pan.Z1.0` (compress 1.0 Headerless LZW)
* **Source Tools**: [`old/unix-tools/compress/compress-1.0/`](file:///workspaces/ctoolbox/old/unix-tools/compress/compress-1.0)
* **Specification Document**: [`compress-ncompress.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compress-ncompress.md)
* **Generation**:
  The 1.0 source used VAX assembly (`insv`). Using modern `ncompress` with `-n` (No Magic Header) produces the identical headerless 9–16 bit LZW stream:
  ```bash
  ./ncompress -c -n < "example2 with lemurs.pan" > "example2 with lemurs.pan.Z1.0"
  ```
* **Decompression Verification**:
  ```bash
  ./ncompress -c -d -n < "example2 with lemurs.pan.Z1.0" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 3. `example2 with lemurs.pan.Z2.0` (compress 2.0 LZW Non-Block Mode)
* **Source Tools**: [`old/unix-tools/compress/compress-2.0/compress.c`](file:///workspaces/ctoolbox/old/unix-tools/compress/compress-2.0/compress.c)
* **Specification Document**: [`compress-ncompress.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compress-ncompress.md)
* **Compilation**:
  ```bash
  gcc -w -std=gnu89 -fcommon -O2 -o compress-2.0 old/unix-tools/compress/compress-2.0/compress.c
  ```
* **Generation Command**:
  ```bash
  ./compress-2.0 -c < "example2 with lemurs.pan" > "example2 with lemurs.pan.Z2.0"
  ```
* **Decompression Verification**:
  ```bash
  ./compress-2.0 -c -d < "example2 with lemurs.pan.Z2.0" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 4. `example2 with lemurs.pan.Z3.0` (compress 3.0 LZW Block Mode)
* **Source Tools**: [`old/unix-tools/compress/compress-3.0/compress.c`](file:///workspaces/ctoolbox/old/unix-tools/compress/compress-3.0/compress.c)
* **Specification Document**: [`compress-ncompress.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compress-ncompress.md)
* **Compilation**:
  ```bash
  gcc -w -std=gnu89 -fcommon -O2 -o compress-3.0 old/unix-tools/compress/compress-3.0/compress.c
  ```
* **Generation Command**:
  ```bash
  ./compress-3.0 -c < "example2 with lemurs.pan" > "example2 with lemurs.pan.Z3.0"
  ```
* **Decompression Verification**:
  ```bash
  ./compress-3.0 -c -d < "example2 with lemurs.pan.Z3.0" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 5. `example2 with lemurs.pan.Z12` (compress 4.0 LZW `-b 12`)
* **Source Tools**: [`old/unix-tools/compress/compress-4.0/compress.c`](file:///workspaces/ctoolbox/old/unix-tools/compress/compress-4.0/compress.c)
* **Specification Document**: [`compress-ncompress.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compress-ncompress.md)
* **Compilation**:
  ```bash
  gcc -w -std=gnu89 -fcommon -O2 -o compress-4.0 old/unix-tools/compress/compress-4.0/compress.c
  ```
* **Generation Command**:
  ```bash
  ./compress-4.0 -c -b 12 < "example2 with lemurs.pan" > "example2 with lemurs.pan.Z12"
  ```
* **Decompression Verification**:
  ```bash
  ./compress-4.0 -c -d < "example2 with lemurs.pan.Z12" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 6. `example2 with lemurs.pan.Z` (ncompress Standard 16-Bit Block LZW)
* **Source Tools**: [`old/unix-tools/ncompress/`](file:///workspaces/ctoolbox/old/unix-tools/ncompress)
* **Specification Document**: [`compress-ncompress.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/compress-ncompress.md)
* **Compilation**:
  ```bash
  gcc -w -std=gnu89 -O2 -DUSE_ZLIB=1 -o ncompress old/unix-tools/ncompress/compress.c -lz
  ```
* **Generation Command**:
  ```bash
  ./ncompress -c -m lzw < "example2 with lemurs.pan" > "example2 with lemurs.pan.Z"
  ```
* **Decompression Verification**:
  ```bash
  ./ncompress -c -d < "example2 with lemurs.pan.Z" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```

---

### 7. `example2 with lemurs.pan.z` (System III/V Canonical Huffman `pack`)
* **Source Tools**: [`old/unix-tools/pack-unpack-pcat/sys3-pack.c`](file:///workspaces/ctoolbox/old/unix-tools/pack-unpack-pcat/sys3-pack.c)
* **Specification Document**: [`pack.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/pack.md)
* **Compilation**:
  ```bash
  # Remove basename length restriction (i-sep > 13) for long filenames
  sed 's/(i-sep) > 13/0/' old/unix-tools/pack-unpack-pcat/sys3-pack.c > sys3-pack.c
  gcc -w -std=gnu89 -fcommon -O2 -o sys3-pack sys3-pack.c
  ```
* **Generation Command**:
  ```bash
  cp "example2 with lemurs.pan" "example2 with lemurs.pan.tmp"
  ./sys3-pack "example2 with lemurs.pan.tmp"
  mv "example2 with lemurs.pan.tmp.z" "example2 with lemurs.pan.z"
  ```
* **Decompression Verification**:
  ```bash
  gcc -w -std=gnu89 -fcommon -O2 -o sys3-unpack old/unix-tools/pack-unpack-pcat/sys3-unpack.c
  ./sys3-unpack "example2 with lemurs.pan.z"
  cmp "example2 with lemurs.pan" "example2 with lemurs.pan.tmp"
  ```

---

### 8. `example2 with lemurs.pan.old.z` (Early Unix PDP-11 Binary Tree `pack`)
* **Source Tools**: [`old/unix-tools/pack-unpack-pcat/pts-opack-port/pack.c`](file:///workspaces/ctoolbox/old/unix-tools/pack-unpack-pcat/pts-opack-port/pack.c)
* **Specification Document**: [`pack.md`](file:///workspaces/ctoolbox/src/formats/compression/data/docs/pack.md)
* **Compilation**:
  ```bash
  gcc -O2 -o old-pack old/unix-tools/pack-unpack-pcat/pts-opack-port/pack.c
  ```
* **Generation Command**:
  ```bash
  ./old-pack -c "example2 with lemurs.pan" > "example2 with lemurs.pan.old.z"
  ```
* **Decompression Verification**:
  ```bash
  ./old-pack -c -d "example2 with lemurs.pan.old.z" > decompressed.pan
  cmp decompressed.pan "example2 with lemurs.pan"
  ```
