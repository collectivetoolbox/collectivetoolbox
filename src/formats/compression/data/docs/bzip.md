# Specification of the `bzip` 0.21 Format

This document details the binary container structure, compression pipeline, and decompression algorithms for the original `bzip` file format (version 0.21, designed by Julian Seward in 1996, utilizing the Burrows-Wheeler block-sorting transform paired with adaptive arithmetic coding).

---

## 1. Pipeline Overview

The compression and decompression pipelines operate through multiple well-defined transformation layers:

```
[Uncompressed Input Stream]
           │
           ▼
┌─────────────────────────────────────────┐
│ 1. Run-Length Encoding Stage 1 (RLE1)   │ ── Contiguous byte runs (≥4) folded into 4-byte prefixes + count
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 2. Stream Sentinel (Last Block Only)    │ ── Appends byte 0x2A ('*') to the final block data
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 3. Deterministic Block Perturbation     │ ── Stepped pseudo-random ±1 byte adjustments ("Spotting")
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 4. Burrows-Wheeler Transform (BWT)      │ ── Lexicographical cyclic sort producing L-column and origPtr
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 5. Move-to-Front Transform (MTF)        │ ── Dynamic 256-symbol ranking with Wheeler 0-run coding (RUNA/RUNB)
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 6. Adaptive Arithmetic Coding (DCC95)   │ ── Moffat-Neal-Witten entropy coder using hierarchical probability models
└─────────────────────────────────────────┘
           │
           ▼
[bzip 0.21 Compressed Bitstream]
```

---

## 2. Stream Layout and Container Framing

A valid `bzip` 0.21 stream consists of a 4-byte plaintext header, followed by an arithmetic-coded payload consisting of zero or more blocks, terminated by an arithmetic-coded 32-bit CRC checksum.

```text
+-------------------+-------------------+-------------------+-------------------+
|     Byte 0        |     Byte 1        |     Byte 2        |     Byte 3        |
|    'B' (0x42)     |    'Z' (0x5A)     |    '0' (0x30)     |    '1' - '9'      |
+-------------------+-------------------+-------------------+-------------------+
|                                                                               |
|                      Arithmetic-Coded Stream Payload                          |
|                                                                               |
|   ┌───────────────────────────────────────────────────────────────────────┐   |
|   │ Block 1: origPtr (32-bit int) + MTF / RLE Symbols + EOB               │   |
|   ├───────────────────────────────────────────────────────────────────────┤   |
|   │ Block 2: origPtr (32-bit int) + MTF / RLE Symbols + EOB               │   |
|   ├───────────────────────────────────────────────────────────────────────┤   |
|   │ ...                                                                   │   |
|   ├───────────────────────────────────────────────────────────────────────┤   |
|   │ Block N (Last Block): -origPtr (32-bit int) + MTF / RLE Symbols + EOB │   |
|   └───────────────────────────────────────────────────────────────────────┘   |
|                                                                               |
|   Stream CRC-32 (32-bit uint, sent via uniform 256-symbol byte model)         |
|                                                                               |
+-------------------------------------------------------------------------------+
```

### 2.1 Preamble Header (Bytes 0–3)

1. **Magic Bytes (Bytes 0–1)**:
   * Byte 0: `0x42` (ASCII `'B'`)
   * Byte 1: `0x5A` (ASCII `'Z'`)
2. **Version Indicator (Byte 2)**:
   * Byte 2: `0x30` (ASCII `'0'`). Identifies the stream as version 0 format.
3. **Block Size Indicator (Byte 3)**:
   * Byte 3: ASCII character between `'1'` (`0x31`) and `'9'` (`0x39`).
   * Defines the maximum buffer capacity per block in units of $100{,}000$ bytes:
     $$\text{BlockSizeLimit} = 100{,}000 \times (\text{Byte}_3 - \text{'0'})$$
   * Valid block limits range from $100{,}000$ bytes (`'1'`) up to $900{,}000$ bytes (`'9'`).

---

## 3. Bitstream and Arithmetic Coding Engine

All data subsequent to the initial 4-byte header is encoded using the **Moffat, Neal, and Witten (DCC95 / June 1996) finite-precision arithmetic coder**.

### 3.1 Bit Packing

* Bits are packed into bytes **most-significant bit (MSB) first** (bit 7 down to bit 0).
* When reading from the bitstream, bit 7 of the first byte is the first bit read.
* When writing, bits fill from bit 7 downward; once 8 bits are buffered, the byte is emitted.
* At stream termination, any remaining unwritten bits in the buffer are padded with `0` bits on the right and flushed.

### 3.2 Arithmetic Coder Parameters and Registers

The arithmetic coding engine operates on 32-bit unsigned integer registers with the following configuration:

* **Bit Precision ($b$)**: $26$ bits.
* **Frequency Limit Precision ($f$)**: $18$ bits (maximum total frequency $\le 2^{18} = 262{,}144$).
* **Registers**:
  * $L$ (Lower bound of current interval): 32-bit unsigned integer, initialized to $0$.
  * $R$ (Range / width of current interval): 32-bit unsigned integer, initialized to $2^{b-1} = 2^{25} = 33{,}554{,}432$ (`0x02000000`).
  * $D$ (Decoder code value): 32-bit unsigned integer (used only in decoder).
  * $bitsOutstanding$: Integer counter tracking delayed opposite-bit emissions during interval underflow.

### 3.3 Initialization

* **Encoder Start**:
  * Set $L \leftarrow 0$.
  * Set $R \leftarrow 2^{25} = 33{,}554{,}432$.
  * Set $bitsOutstanding \leftarrow 0$.
* **Decoder Start**:
  * Set $L \leftarrow 0$.
  * Set $R \leftarrow 2^{25} = 33{,}554{,}432$.
  * Set $D \leftarrow 0$.
  * Read the first $b = 26$ bits from the bitstream MSB to LSB and shift them into $D$:
    $$\text{For } i = 1 \dots 26: \quad D \leftarrow (D \ll 1) + \text{ReadBit}()$$

### 3.4 Symbol Encoding Algorithm

To encode a symbol $s \in [1, \text{numSymbols}]$ using a probability model with total frequency $T$ and cumulative frequency range $[L_s, H_s)$, where $L_s = \sum_{k=1}^{s-1} \text{freq}[k]$ and $H_s = L_s + \text{freq}[s]$:

1. **Subdivide Interval**:
   $$r = \lfloor R / T \rfloor$$
   $$L \leftarrow L + (r \times L_s)$$
   $$\text{If } H_s < T: \quad R \leftarrow r \times (H_s - L_s) \quad \text{Else: } R \leftarrow R - (r \times L_s)$$

2. **Renormalize**:
   While $R \le 2^{b-2}$ (where $2^{b-2} = 2^{24} = 16{,}777{,}216$):
   * **Case 1: $L + R \le 2^{b-1}$ ($2^{25} = 33{,}554{,}432$)**:
     * Emit bit `0`.
     * Emit $bitsOutstanding$ bits of value `1`.
     * $bitsOutstanding \leftarrow 0$.
   * **Case 2: $L \ge 2^{b-1}$ ($2^{25} = 33{,}554{,}432$)**:
     * Emit bit `1`.
     * Emit $bitsOutstanding$ bits of value `0`.
     * $bitsOutstanding \leftarrow 0$.
     * $L \leftarrow L - 2^{b-1}$.
   * **Case 3: (Underflow / Mid-range expansion)**:
     * $bitsOutstanding \leftarrow bitsOutstanding + 1$.
     * $L \leftarrow L - 2^{b-2}$.
   * Scale registers:
     $$L \leftarrow 2 \times L$$
     $$R \leftarrow 2 \times R$$

### 3.5 Symbol Decoding Algorithm

To decode a symbol from the bitstream using a model with total frequency $T$:

1. **Identify Symbol Interval**:
   $$r = \lfloor R / T \rfloor$$
   $$target = \min\left(T - 1, \lfloor D / r \rfloor\right)$$
   Find the unique 1-based symbol index $s \in [1, \text{numSymbols}]$ such that:
   $$\sum_{k=1}^{s-1} \text{freq}[k] \le target < \sum_{k=1}^{s} \text{freq}[k]$$
   Let $L_s = \sum_{k=1}^{s-1} \text{freq}[k]$ and $H_s = L_s + \text{freq}[s]$.

2. **Update Decoder State**:
   $$D \leftarrow D - (r \times L_s)$$
   $$\text{If } H_s < T: \quad R \leftarrow r \times (H_s - L_s) \quad \text{Else: } R \leftarrow R - (r \times L_s)$$

3. **Renormalize**:
   While $R \le 2^{b-2}$ ($2^{24} = 16{,}777{,}216$):
   $$R \leftarrow 2 \times R$$
   $$D \leftarrow (2 \times D) + \text{ReadBit}()$$

4. Return the decoded symbol index $s$.

### 3.6 Encoder Termination

When all stream elements have been encoded:
1. For $i = b$ ($26$) down to $1$:
   $$bit = (L \gg (i - 1)) \ \& \ 1$$
   * Emit $bit$.
   * Emit $bitsOutstanding$ bits of value $(1 - bit)$.
   * $bitsOutstanding \leftarrow 0$.
2. Flush any partially filled output byte buffer by shifting left and writing the byte.

---

## 4. Probability Models

A probability model maintains a discrete frequency distribution over a fixed alphabet of size $N = \text{numSymbols}$ (1-indexed, $1 \dots N$).

### 4.1 Model Structure & Adaptation Rules

Each model is characterized by:
* $\text{numSymbols}$: Alphabet cardinality $N$.
* $\text{freq}[1 \dots N]$: Array of symbol counts.
* $\text{totFreq}$: Sum of counts, $\sum_{i=1}^N \text{freq}[i]$.
* $\text{incValue}$: Count added to symbol frequency upon each occurrence.
* $\text{noExceed}$: Maximum permitted $\text{totFreq}$ threshold before halving.

#### Model Initialization
* If $\text{incValue} == 0$ (Static Uniform Model):
  $$\text{freq}[i] = 1 \quad (\forall i \in [1, N]), \quad \text{totFreq} = N$$
* If $\text{incValue} > 0$ (Adaptive Model):
  $$\text{freq}[i] = \text{incValue} \quad (\forall i \in [1, N]), \quad \text{totFreq} = N \times \text{incValue}$$

#### Model Update
Immediately following the encoding or decoding of symbol $s$:
1. $\text{totFreq} \leftarrow \text{totFreq} + \text{incValue}$
2. $\text{freq}[s] \leftarrow \text{freq}[s] + \text{incValue}$
3. If $\text{totFreq} > \text{noExceed}$ (Frequency Rescaling):
   $$\text{totFreq} \leftarrow 0$$
   $$\text{For } i = 1 \dots N: \quad \text{freq}[i] \leftarrow \lfloor (\text{freq}[i] + 1) / 2 \rfloor, \quad \text{totFreq} \leftarrow \text{totFreq} + \text{freq}[i]$$

### 4.2 Defined Models in `bzip` 0.21

#### 1. Static Uniform Byte Model (`bogusModel`)
Used exclusively for transmitting raw 8-bit bytes (such as block headers and stream checksums) through the arithmetic coder:
* $\text{numSymbols} = 256$
* $\text{incValue} = 0$ (never adapts)
* $\text{noExceed} = 256$
* Initial frequencies: $\text{freq}[1 \dots 256] = 1, \quad \text{totFreq} = 256$.
* A byte value $C \in [0, 255]$ corresponds directly to symbol $s = C + 1$.
* 32-bit integers are transmitted as 4 consecutive bytes in big-endian network byte order (MSB to LSB).

#### 2. Structured Move-to-Front Model Suite
Move-to-Front (MTF) ranks and control tokens are encoded using a hierarchical suite of 8 adaptive models (Fenwick structured model).

> [!IMPORTANT]
> All 8 structured models are **re-initialized to their exact initial states at the beginning of each block**.

| Model Identifier | Index | $\text{numSymbols}$ | $\text{incValue}$ | $\text{noExceed}$ | Initial $\text{totFreq}$ | Represents |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| `BASIS` | 0 | 11 | 12 | 1000 | 132 | High-level MTF class tokens / control symbols |
| `MODEL_2_3` | 1 | 2 | 4 | 1000 | 8 | MTF ranks 2–3 (offset within bucket) |
| `MODEL_4_7` | 2 | 4 | 3 | 1000 | 12 | MTF ranks 4–7 (offset within bucket) |
| `MODEL_8_15` | 3 | 8 | 3 | 1000 | 24 | MTF ranks 8–15 (offset within bucket) |
| `MODEL_16_31` | 4 | 16 | 3 | 1000 | 48 | MTF ranks 16–31 (offset within bucket) |
| `MODEL_32_63` | 5 | 32 | 3 | 1000 | 96 | MTF ranks 32–63 (offset within bucket) |
| `MODEL_64_127` | 6 | 64 | 2 | 1000 | 128 | MTF ranks 64–127 (offset within bucket) |
| `MODEL_128_255` | 7 | 128 | 1 | 1000 | 128 | MTF ranks 128–255 (offset within bucket) |

---

## 5. Move-to-Front & Symbol Coding Specification

### 5.1 Symbol Alphabet & Basis Model Mapping

The primary `BASIS` model transmits 11 discrete tokens (symbol indices $1 \dots 11$):

| Basis Symbol Value | Name | Description | Secondary Extension |
| :---: | :--- | :--- | :--- |
| `1` | `VAL_RUNA` | Wheeler Zero-Run bit `0` | None |
| `2` | `VAL_RUNB` | Wheeler Zero-Run bit `1` | None |
| `3` | `VAL_ONE` | MTF rank 1 | None |
| `4` | `VAL_2_3` | MTF rank 2..3 | Sub-symbol in `MODEL_2_3` (range $1 \dots 2$) |
| `5` | `VAL_4_7` | MTF rank 4..7 | Sub-symbol in `MODEL_4_7` (range $1 \dots 4$) |
| `6` | `VAL_8_15` | MTF rank 8..15 | Sub-symbol in `MODEL_8_15` (range $1 \dots 8$) |
| `7` | `VAL_16_31` | MTF rank 16..31 | Sub-symbol in `MODEL_16_31` (range $1 \dots 16$) |
| `8` | `VAL_32_63` | MTF rank 32..63 | Sub-symbol in `MODEL_32_63` (range $1 \dots 32$) |
| `9` | `VAL_64_127` | MTF rank 64..127 | Sub-symbol in `MODEL_64_127` (range $1 \dots 64$) |
| `10` | `VAL_128_255` | MTF rank 128..255 | Sub-symbol in `MODEL_128_255` (range $1 \dots 128$) |
| `11` | `VAL_EOB` | End-of-Block sentinel | None |

### 5.2 Move-to-Front Decoding & Encoding

At the start of each block, the MTF permutation table `yy` is initialized to the identity mapping:
$$\text{yy}[i] = i \quad \text{for } i = 0 \dots 255$$

* **Non-Zero MTF Rank $R \in [1, 255]$**:
  1. Retrieve decoded byte $C = \text{yy}[R]$.
  2. Shift table entries right: $\text{yy}[k] \leftarrow \text{yy}[k-1]$ for $k = R, R-1, \dots, 1$.
  3. Move byte to front: $\text{yy}[0] \leftarrow C$.
  4. Append $C$ to the transformed block array $L$.

* **Zero MTF Rank Runs ($R = 0$)**:
  Consecutive MTF ranks of index 0 are **not** emitted as individual rank 0 symbols. Instead, runs of zero ranks are compacted using Wheeler's bijective base-2 run-length encoding.

### 5.3 Wheeler Zero-Run Coding (RUNA / RUNB)

A run of $K$ consecutive zeros ($K \ge 1$) is represented as a sequence of `RUNA` and `RUNB` tokens. This encoding represents $K$ in bijective base-2 (1-2 coding), where `RUNB` carries weight 1 and `RUNA` carries weight 2 at each positional step.

#### Decoding Run Length
When encountering `RUNA` or `RUNB`:
1. Set $N \leftarrow 0$.
2. Loop:
   * $N \leftarrow (N \ll 1)$
   * If token is `RUNA`: $N \leftarrow N \ | \ 1$
   * $N \leftarrow N + 1$
   * Read next token from bitstream.
   * Continue loop while next token is `RUNA` or `RUNB`.
3. Emit $N$ copies of the current head of the MTF table ($\text{yy}[0]$) into $L$.
4. The MTF table `yy` is unchanged during zero-run emissions.
5. Process the subsequent non-run token that terminated the loop.

#### Encoding Run Length
To transmit a run of $K \ge 1$ pending zeros:
1. Deconstruct $K$ into bits (from LSB to MSB):
   * Set $bits \leftarrow 0, \ count \leftarrow 0$.
   * While $K \ne 0$:
     * $count \leftarrow count + 1$
     * $bits \leftarrow bits \ll 1$
     * $K \leftarrow K - 1$
     * If $(K \ \& \ 1) == 1$: $bits \leftarrow bits \ | \ 1$
     * $K \leftarrow K \gg 1$
2. Emit tokens in MSB to LSB order:
   * While $count > 0$:
     * If $(bits \ \& \ 1) == 1$: Emit `RUNA`
     * Else: Emit `RUNB`
     * $bits \leftarrow bits \gg 1$
     * $count \leftarrow count - 1$

---

## 6. Block Structure and Burrows-Wheeler Transform

### 6.1 Block Header: `origPtr` and Stream Termination

Each block begins with a 32-bit signed integer transmitted via `bogusModel` (4 bytes, big-endian):
* Let $V$ be the decoded 32-bit signed integer.
* **Last-Block Flag**:
  * If $V < 0$: This block is the **final block** of the stream.
  * If $V > 0$: More blocks follow this block.
* **Original Pointer Value**:
  $$\text{origPtr} = |V| - 1$$
  where $\text{origPtr}$ is the 0-based index indicating the position of the original unrotated string in the sorted cyclic permutation matrix ($0 \le \text{origPtr} < \text{BlockLength}$).

### 6.2 Inverse Burrows-Wheeler Transform (Inverse BWT)

Given the decoded $L$-column byte array $L[0 \dots N-1]$ (of length $N = \text{last} + 1$) and $\text{origPtr}$:

1. **Compute Character Frequency Frequencies**:
   * Initialize frequency table $C[0 \dots 255] \leftarrow 0$.
   * For $i = 0 \dots N-1$:
     $$C[L[i]] \leftarrow C[L[i]] + 1$$

2. **Compute Cumulative Base Offsets**:
   * Initialize $cc[0 \dots 255]$.
   * Set $sum \leftarrow 0$.
   * For $ch = 0 \dots 255$:
     $$sum \leftarrow sum + C[ch]$$
     $$cc[ch] \leftarrow sum - C[ch]$$

3. **Compute Transformation Vector $T$**:
   * Allocate integer array $T[0 \dots N-1]$.
   * For $i = 0 \dots N-1$:
     $$T[i] \leftarrow cc[L[i]]$$
     $$cc[L[i]] \leftarrow cc[L[i]] + 1$$

4. **Reconstruct Unsorted Block**:
   * Reconstruct original byte sequence $\text{block}[0 \dots N-1]$ in reverse order:
     * Set $curr \leftarrow \text{origPtr}$.
     * For $j = N-1$ down to $0$:
       $$\text{block}[j] \leftarrow L[curr]$$
       $$curr \leftarrow T[curr]$$

---

## 7. Deterministic Block Perturbation ("Spotting")

To prevent degenerate performance on repetitive inputs, `bzip` 0.21 applies a deterministic modular increment/decrement step across every block.

### 7.1 Perturbation Algorithm

Given the block array $\text{block}[0 \dots N-1]$ (with $N = \text{last} + 1$):

1. Initialize cursor and delta:
   $$\text{pos} = 8000, \quad \text{delta} = 1$$

2. While $\text{pos} < N - 1$:
   * **During Compression**:
     $$\text{block}[\text{pos}] \leftarrow (\text{block}[\text{pos}] + 1) \pmod{256}$$
   * **During Decompression**:
     $$\text{block}[\text{pos}] \leftarrow (\text{block}[\text{pos}] - 1 + 256) \pmod{256}$$
   * **Advance Delta State**:
     The delta transition table is defined as:
     $$\text{newdelta} = \begin{cases}
       1 & \text{if } \text{delta} = 3 \\
       4 & \text{if } \text{delta} = 1 \\
       5 & \text{if } \text{delta} = 4 \\
       9 & \text{if } \text{delta} = 5 \\
       2 & \text{if } \text{delta} = 9 \\
       6 & \text{if } \text{delta} = 2 \\
       7 & \text{if } \text{delta} = 6 \\
       8 & \text{if } \text{delta} = 8 \\
       3 & \text{if } \text{delta} = 7 \\
       1 & \text{otherwise}
     \end{cases}$$
     $$\text{delta} \leftarrow \text{newdelta}$$
   * **Advance Position**:
     $$\text{pos} \leftarrow \text{pos} + 8000 + 17 \times (\text{delta} - 5)$$

> [!NOTE]
> When $N \le 8001$, no bytes in the block are perturbed.

---

## 8. Run-Length Encoding Stage 1 (RLE1) and Sentinel

### 8.1 Input Stream Run-Length Folding (RLE1)

The uncompressed input stream is preprocessed by replacing runs of identical consecutive bytes ($1 \dots 255$ bytes):

* **Run of 1 byte ($c$)**: Emitted as literal $c$.
* **Run of 2 identical bytes ($c, c$)**: Emitted as literals $c, c$.
* **Run of 3 identical bytes ($c, c, c$)**: Emitted as literals $c, c, c$.
* **Run of $K \ge 4$ identical bytes**: Emitted as 4 literal copies of $c$, followed by a single count byte $(K - 4) \in [0, 251]$:
  $$\underbrace{c, \ c, \ c, \ c}_{4 \text{ bytes}}, \ (K - 4)$$

### 8.2 Stream Sentinel Byte (Last Block Only)

When the uncompressed input reaches End-of-File (EOF):
1. A single sentinel byte `0x2A` (ASCII `'*'`) is appended to the RLE1 byte stream of the final block.
2. The block is marked as the last block ($V = -(\text{origPtr} + 1)$).
3. Upon decompression of the last block, the decoder verifies that the final byte after inverse BWT and un-spotting satisfies:
   $$\text{block}[N-1] == \text{0x2A}$$
   If this check fails, the stream is corrupted.
4. The sentinel byte `0x2A` is **discarded** and not passed to RLE1 decoding.

### 8.3 Inverting RLE1

To extract the final uncompressed stream from the un-spotted block:
1. Let $\text{limit} = (N - 2)$ if this is the last block, or $(N - 1)$ if not the last block.
2. Maintain previous character $chPrev$ and run counter $count \leftarrow 0$.
3. For $i = 0 \dots \text{limit}$:
   * $ch \leftarrow \text{block}[i]$
   * Emit $ch$ to uncompressed stream and update CRC.
   * If $ch \ne chPrev$:
     * $count \leftarrow 1$
     * $chPrev \leftarrow ch$
   * Else:
     * $count \leftarrow count + 1$
     * If $count == 4$:
       * Read repetition byte $rep = \text{block}[i+1]$.
       * $i \leftarrow i + 1$.
       * Emit $rep$ additional copies of $ch$ and update CRC for each copy.
       * $count \leftarrow 0$.

---

## 9. Stream Checksum (CRC-32)

A single 32-bit CRC checksum protects the entire uncompressed stream. It is computed across all raw uncompressed bytes in their original order.

### 9.1 CRC-32 Definition

* **Polynomial**: `0x04C11DB7` (MSB-first / standard left-shifting IEEE 802.3 / bzip polynomial).
* **Initial Register Value**: `0xFFFFFFFF`.
* **Byte Update Function**:
  $$\text{CRC} \leftarrow (\text{CRC} \ll 8) \oplus \text{Table}[(\text{CRC} \gg 24) \oplus C]$$
  where $\text{Table}[k]$ for $k \in [0, 255]$ is generated by left-shifting $k \ll 24$ modulo polynomial `0x04C11DB7`.
* **Final Checksum Output**:
  $$\text{FinalCRC} = \sim \text{CRC} \quad (\text{bitwise NOT})$$

### 9.2 Checksum Placement

The 32-bit $\text{FinalCRC}$ is written immediately after the last block's `EOB` symbol via `bogusModel` as 4 consecutive big-endian bytes (MSB to LSB).

---

## 10. Complete Decompression Algorithm

An independent implementation can decompress a compliant `bzip` 0.21 stream by following these steps:

1. **Read & Validate Preamble**:
   * Read 4 bytes from input stream.
   * Verify Byte 0 == `'B'` (`0x42`), Byte 1 == `'Z'` (`0x5A`), Byte 2 == `'0'` (`0x30`).
   * Extract $\text{blockSize100k} = \text{Byte 3} - \text{'0'}$. Verify $1 \le \text{blockSize100k} \le 9$.
   * Set maximum block capacity $\text{limit} = 100{,}000 \times \text{blockSize100k}$.

2. **Initialize Stream State**:
   * Initialize Global CRC register: $\text{CRC} \leftarrow \text{0xFFFFFFFF}$.
   * Initialize `bogusModel` (static uniform 256 symbols).
   * Start Arithmetic Decoder (read first 26 bits from bitstream into $D$).

3. **Block Processing Loop**:
   Repeat for each block:
   1. Read 32-bit signed integer $V$ from bitstream via `bogusModel`.
   2. Determine last block status: $\text{isLastBlock} = (V < 0)$.
   3. Compute $\text{origPtr} = |V| - 1$.
   4. Initialize the 8 Move-to-Front models (`BASIS` and subsidiary models) to their initial states.
   5. Initialize MTF table $\text{yy}[i] = i$ for $i = 0 \dots 255$.
   6. Decode MTF and Wheeler zero-run tokens into byte array $L$ until `EOB` (`VAL_EOB`) is encountered.
   7. Let $N$ be the number of decoded bytes in $L$. Verify $N \le \text{limit}$ and $0 \le \text{origPtr} < N$.
   8. Execute **Inverse BWT** on $L$ and $\text{origPtr}$ to reconstruct $\text{block}[0 \dots N-1]$.
   9. Execute **Decompression Spotting** on $\text{block}[0 \dots N-1]$.
   10. If $\text{isLastBlock}$:
       * Verify $\text{block}[N-1] == \text{0x2A}$ (`'*'`).
       * Set un-RLE length to $N - 1$.
   11. Execute **Inverting RLE1** to emit raw uncompressed bytes and accumulate $\text{CRC}$.
   12. If $\text{isLastBlock}$, exit the block loop.

4. **Verify Checksum**:
   * Read 32-bit unsigned integer $\text{StoredCRC}$ from bitstream via `bogusModel`.
   * Compute $\text{ComputedCRC} = \sim \text{CRC}$.
   * Verify $\text{StoredCRC} == \text{ComputedCRC}$.

5. **Finalize**:
   * Complete arithmetic decoder and close bitstream.
