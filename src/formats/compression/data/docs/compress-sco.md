# Specification of SCO `compress -H` (LZH Compression Format)

This document provides a comprehensive technical specification of the SCO `compress -H` file format (also known as the `lzh` compression method in `gzip`), as implemented and decompressed by historical Unix tools such as `gzip` 1.2.4 (`old/unix-tools/gzip124/unlzh.c`).

---

## 1. Overview & Historical Background

In SCO Unix (Santa Cruz Operation System V/386), the standard Unix `compress` utility was extended with an optional `-H` command-line switch. While standard `compress` uses adaptive Lempel-Ziv-Welch (LZW) coding (`0x1F 0x9D`), passing `-H` invokes an alternative compression algorithm based on **LZSS (Lempel-Ziv-Storer-Szymanski) sliding window dictionary coding combined with static Huffman trees** (often designated as the `lzh` or LHarc `-lh1-` format).

The decompressor code in [`gzip124/unlzh.c`](file:///workspaces/ctoolbox/old/unix-tools/gzip124/unlzh.c) was authored by Jean-loup Gailly, adapted directly from Haruhiko Okumura's public domain `ar002` archiver (1988–1989).

---

## 2. Command-Line Interface (CLI) Specification

### 2.1 SCO `compress -H` Utility

#### Synopsis
```sh
compress -H [-f] [-c] [-v] [file ...]
```

#### Options & Behavior
* **`-H` (LZH Algorithm Selection)**:
  * Selects the LZH (LZSS + Static Huffman) algorithm instead of default LZW.
  * Outputs a stream prefixed with the magic header `0x1F 0xA0`.
* **`-c` (Standard Output Mode)**:
  * Writes compressed LZH output stream to `stdout`.
* **`-f` (Force Overwrite)**:
  * Forces compression even if no space savings are realized.
* **`-v` (Verbose Mode)**:
  * Prints compression ratio and statistics to `stderr`.

---

### 2.2 `gzip` / `gunzip` Decompression Support

#### Synopsis
```sh
gunzip [file.Z ...]
gzip -d [file.Z ...]
```

#### Auto-Detection
When `gzip` inspects an input file or stream:
1. Reads the 2-byte magic identifier.
2. If `magic[0] == 0x1F` and `magic[1] == 0xA0` (`LZH_MAGIC`), `gzip` identifies the file as SCO `compress -H` (`method = LZHED = 3`).
3. Hands off bitstream decoding to the [`unlzh`](file:///workspaces/ctoolbox/old/unix-tools/gzip124/unlzh.c#L385) engine.

---

## 3. Binary File Format Specification

### 3.1 Binary Layout Overview

An SCO `compress -H` file consists of a 2-byte magic identifier followed immediately by a sequence of bit-packed compressed data blocks:

```text
+-----------------------+-----------------------+----------------------------------+
|  Magic Header Byte 0  |  Magic Header Byte 1  |                                  |
|         0x1F          |         0xA0          |  Packed LZH Data Blocks...       |
|       (octal 037)     |      (octal 0240)     |  (Block Headers & Huffman Data)  |
+-----------------------+-----------------------+----------------------------------+
```

---

### 3.2 Header Specification

| Field | Offset (Bytes) | Size | Value / Description |
| :--- | :---: | :---: | :--- |
| **Magic Header** | `0x00` | 2 Bytes | `0x1F 0xA0` (`LZH_MAGIC`). Identifies SCO `compress -H` format. |
| **Data Bitstream** | `0x02` | End of file | Packed MSB-first bitstream containing sequential compressed blocks. |

* **CRC / Checksum**: None.
* **Uncompressed File Size**: Not stored in header or footer. End of stream is signaled by a block with `blocksize == 0`.

---

### 3.3 Bitstream Architecture & Bit I/O

Bits are packed into bytes from Most Significant Bit (MSB) to Least Significant Bit (LSB).

* **Bit Buffer (`bitbuf`)**: 16-bit unsigned register.
* **Bit Extraction (`getbits(n)`)**:
  * Extracts top $n$ bits: `x = bitbuf >> (16 - n)` ($1 \le n \le 16$).
  * Shifts buffer left by $n$ bits and replenishes from input byte stream.
* **Initialization (`init_getbits()`)**:
  * Clears `bitbuf` and pre-loads 16 bits from the input file immediately after the magic header.

---

### 3.4 Data Block Structure

The bitstream is divided into variable-length **blocks**. Each block decodes up to `blocksize` literal characters and match tokens:

```text
+-----------------------+-----------------------+-----------------------+-----------------------+-----------------------+
| Block Size (16 bits)  | Pre-Tree Lengths (PT) |  Literal/Length Tree  |     Position Tree     |  Encoded Symbols...   |
| (Symbol Count N)      | (PT Bit-Length Table) |  (C-Tree Bit-Lengths) |  (P-Tree Bit-Lengths) |  (N Decoded Tokens)   |
+-----------------------+-----------------------+-----------------------+-----------------------+-----------------------+
```

1. **Block Size Field (`blocksize`)**:
   * 16-bit unsigned integer read via `getbits(16)`.
   * If `blocksize == 0`: Signals **End-of-File (EOF)** and halts decompression.
   * If `blocksize > 0`: Specifies the number of LZSS symbols (literals + match references) to decode in this block.
2. **Pre-Tree Definition (PT Table)**:
   * Encodes code length table for the Literal/Length Tree.
3. **Literal/Length Code Tree (C-Tree)**:
   * Encodes literal bytes ($0 \dots 255$) and match lengths ($3 \dots 256$).
4. **Position Code Tree (P-Tree)**:
   * Encodes sliding-window match offsets ($0 \dots 8191$).

---

### 3.5 Alphabet & Tree Specifications

#### 3.5.1 Parameters & Constants

| Constant | Value | Description |
| :--- | :---: | :--- |
| `DICBIT` | `13` | Sliding window size bit width ($2^{13} = 8192$ bytes / 8 KB). |
| `DICSIZ` | `8192` | Sliding window buffer size in bytes. |
| `THRESHOLD` | `3` | Minimum LZSS match length (bytes). |
| `MAXMATCH` | `256` | Maximum LZSS match length (bytes). |
| `NC` | `510` | Literal/Length symbol alphabet size ($255 + 256 + 2 - 3$). |
| `CBIT` | `9` | Bit width for $NC$ alphabet size ($\lfloor \log_2 510 \rfloor + 1$). |
| `NP` | `14` | Position symbol alphabet size (`DICBIT + 1`). |
| `PBIT` | `4` | Bit width for $NP$ table size. |
| `NT` | `19` | Pre-Tree ($PT$) symbol alphabet size (`CODE_BIT + 3` where `CODE_BIT = 16`). |
| `TBIT` | `5` | Bit width for $NT$ table size. |

---

#### 3.5.2 Symbol Interpretations

1. **Literal/Length Symbol Code ($j \in [0, 509]$)**:
   * **$0 \le j \le 255$**: Literal byte value ($0 \dots 255$).
   * **$256 \le j \le 509$**: LZ77 Match Length token.
     * Decoded Match Length: $L = j - 253$ bytes (range $3 \dots 256$).
   * **$j = 510$ ($NC$)**: End-of-File marker.

2. **Position/Distance Code ($P \in [0, 13]$)**:
   * **$P = 0$**: Offset distance $D = 0$ (1 byte back).
   * **$1 \le P \le 13$**: Offset distance $D = 2^{P-1} + \text{getbits}(P - 1)$.
   * Window position index: $\text{src\_idx} = (\text{dest\_idx} - D - 1) \bmod 8192$.

---

### 3.6 Huffman Tree Decoding Algorithms

#### 3.6.1 Pre-Tree Bit-Length Table Reading (`read_pt_len`)

Reads bit lengths for small tables ($NT = 19$ for C-Tree header, or $NP = 14$ for P-Tree header):

1. Read count $n$ ($TBIT = 5$ or $PBIT = 4$ bits).
2. If $n == 0$:
   * Read single symbol $c$ ($TBIT$ or $PBIT$ bits).
   * Set all element lengths to 0; populate 8-bit lookup table directly with symbol $c$.
3. If $n > 0$:
   * Loop $i$ from $0$ to $n-1$:
     * Peek top 3 bits of bit buffer.
     * If peek value $< 7$: Code length $c =$ peek value (consume 3 bits).
     * If peek value $== 7$: Count extra set bits (1s). Code length $c = 7 + \text{extra\_ones}$ (consume $3 + \text{extra\_ones}$ bits).
     * Store length `pt_len[i] = c`.
     * **Special Zero Insertion**: If $i == 3$ (when reading $NT$ pre-tree):
       * Read 2 bits ($getbits(2)$).
       * Append that count of zero lengths (`pt_len[i++] = 0`).
   * Pad remaining `pt_len[n..nn-1] = 0`.
   * Build 8-bit canonical lookup table `pt_table[256]`.

---

#### 3.6.2 Literal/Length Tree Reading (`read_c_len`)

Reads the 510-entry `c_len` table using the Pre-Tree `pt_table`:

1. Read count $n$ (`CBIT = 9` bits).
2. If $n == 0$:
   * Read single symbol $c$ (`CBIT = 9` bits).
   * Set all `c_len` entries to 0; populate 12-bit lookup table `c_table[4096]` directly with $c$.
3. If $n > 0$:
   * Loop $i$ from $0$ to $n-1$:
     * Decode code $c$ using Pre-Tree `pt_table`.
     * **Run-Length Zero Decoding ($c \le 2$)**:
       * $c == 0$: Insert 1 zero length.
       * $c == 1$: Read 4 bits; insert $\text{getbits}(4) + 3$ zero lengths.
       * $c == 2$: Read 9 bits (`CBIT`); insert $\text{getbits}(9) + 20$ zero lengths.
     * **Non-Zero Code Lengths ($c > 2$)**:
       * Store bit length `c_len[i++] = c - 2` (range $1 \dots 16$ bits).
   * Pad remaining `c_len[n..509] = 0`.
   * Build 12-bit canonical lookup table `c_table[4096]`.

---

### 3.7 LZSS Decompression Loop

1. Initialize bit reader and set `blocksize = 0`.
2. **Block Loop**:
   * If `blocksize == 0`:
     * Read `blocksize = getbits(16)`.
     * If `blocksize == 0`: Decompression complete (EOF).
     * Read Pre-Tree ($NT=19, TBIT=5, i\_special=3$).
     * Read Literal/Length Tree (`read_c_len`).
     * Read Position Tree ($NP=14, PBIT=4, i\_special=-1$).
   * Decrement `blocksize--`.
   * Decode symbol $j$ using `c_table[4096]`.
   * **Literal Byte ($j \le 255$)**:
     * Write $j$ to output buffer and update sliding window.
   * **LZ77 Match Reference ($256 \le j \le 509$)**:
     * Match Length $L = j - 253$ bytes.
     * Decode Position Code $P$ using `pt_table` for $NP$.
     * Calculate Distance $D$:
       * If $P == 0$: $D = 0$.
       * If $P > 0$: $D = 2^{P-1} + \text{getbits}(P - 1)$.
     * Copy $L$ bytes from circular window offset $\text{src\_idx} = (\text{dest\_idx} - D - 1) \bmod 8192$ to output buffer byte-by-byte.

---

## 4. Reference Code & Mapping

| Component | Source File | Key Functions / Constants |
| :--- | :--- | :--- |
| **Format Detection** | (gzip124/gzip.c#L1290) | `memcmp(magic, LZH_MAGIC, 2) == 0` |
| **Magic Header** | (gzip124/gzip.h#L158) | `#define LZH_MAGIC "\037\240"` (`0x1F 0xA0`) |
| **Main Decompressor** | (gzip124/unlzh.c#L385) | `unlzh(int in, int out)` |
| **Block & Huffman Decoder** | (gzip124/unlzh.c#L273) | `decode_c()`, `decode_p()`, `read_c_len()`, `read_pt_len()` |
| **Table Construction** | (gzip124/unlzh.c#L140) | `make_table()` |
