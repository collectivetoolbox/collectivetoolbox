# Specification of `pack`, `unpack`, and `pcat`

This document provides a comprehensive specification of the command-line interface (CLI) and binary file formats for the traditional Unix `pack`, `unpack`, and `pcat` utilities (originally authored by Steve Zucker in ~1977 and revised into the System III/V standard by Thomas G. Szymanski in ~1978–1980).

---

## 1. Command-Line Interface (CLI) Specification

The utilities `pack`, `unpack`, and `pcat` process files using Huffman coding algorithms. `pack` compresses files in-place by replacing them with `.z` files, `unpack` decompresses `.z` files back to their uncompressed forms, and `pcat` decompresses `.z` files to standard output.

---

### 1.1 `pack` Utility

#### Synopsis
```sh
pack [[ - ] file ... ] file ...
```

#### Options & Flag Control
* **`-` (Toggle Statistics Flag)**:
  * When encountered in the argument list, toggles the verbose statistics output mode (`vflag` 0 $\leftrightarrow$ 1).
  * Subsequent file arguments adopt the updated verbose setting until another `-` flag is encountered.
* **`-s` (Size Saving Override - Extension)**:
  * Force-compresses files even if no disk block savings result.
* **`-c` (Standard Output Mode - Extension)**:
  * Writes compressed output bitstream to standard output (`stdout`) instead of creating a `.z` file.
* **`-d` (Decompress Mode - Extension)**:
  * Operates in decompression mode instead of compression mode.

#### Behavior & Execution Workflow
`pack` processes each file argument sequentially in command-line order:

1. **Filename Length & Suffix Validation**:
   * **Path Length**: Total path length must be $< 77$ characters (`LNAME - 3`).
   * **Basename Length**: The filename component (excluding directory path) must not exceed $13$ characters.
   * **Existing `.z` Suffix**: If the target filename already ends with `.z`, `pack` rejects it:
     * Output (`stderr`): `<filename>: Already packed` (or `<filename>: already packed`)
   * **Length Limit Violation**:
     * Output (`stderr`): `<filename>: File name too long`

2. **File Inspection & System Checks**:
   * **File Opening**: If the source file cannot be opened for reading:
     * Output (`stderr`): `<filename>: Unable to open` (or `<filename>: cannot open`)
   * **Regular File Check**: Directories and special device files are rejected.
     * Output (`stderr`): `<filename>: Not a plain file` (or `<filename>: cannot pack a directory`)
   * **Hard Link Check**: Files with link count (`st_nlink`) $> 1$ are rejected to prevent dangling links.
     * Output (`stderr`): `'<filename>' has links` (or `<filename>: has links`)
   * **Target File Conflict**: If the output file `<filename>.z` already exists:
     * Output (`stderr`): `<filename>.z: Already exists`

3. **Content Triviality & Compression Threshold**:
   * **Trivial File Check**: If the input file contains fewer than 2 distinct byte values (e.g. 0 or 1 unique character, or 0-byte file), Huffman tree building is skipped:
     * Output (`stderr`): `<filename>: Trivial file`
     * Original file remains unchanged; no `.z` file is created.
   * **Savings Calculation**:
     * Calculates the estimated size of the output bitstream including header and tree dictionary.
     * Evaluates total 512-byte blocks:
       $$\text{input\_blocks} = \left\lfloor \frac{\text{insize} + 511}{512} \right\rfloor, \quad \text{output\_blocks} = \left\lfloor \frac{\text{outsize} + 511}{512} \right\rfloor$$
     * If $\text{output\_blocks} \ge \text{input\_blocks}$ (no block savings achieved):
       * Output (`stderr`): `<filename>: Not packed (no blocks saved)` (or `<filename>: no saving`)
       * Original file remains unchanged; partial `.z` file is unlinked.

4. **Target File Creation & Replacement**:
   * **Destination Naming**: Output filename is created by appending `.z` to the source path (`<filename>.z`).
   * **Metadata Preservation**: Inherits file mode permissions (`st_mode`), owner ID (`st_uid`), group ID (`st_gid`), and access/modification timestamps (`utime`) from the source file.
   * **Source Removal**: On successful packing, the original source file is deleted (`unlink`).

5. **Reporting & Verbose Output**:
   * **Standard Completion Report**:
     $$\text{Compression Percentage} = \left\lfloor \frac{100 \times (\text{uncompressed\_bytes} - \text{packed\_bytes})}{\text{uncompressed\_bytes}} \right\rfloor$$
     * Output (`stdout` / `stderr`): `<filename>: XX% Compression`
   * **Verbose Statistics Output** (enabled when `vflag` is active via `-` argument):
     ```text
     <filename>: N Bytes
         <freq>    <pct>% <<octal>> = <<char>> <bits>
         ...
     <filename>: Packed size: M bytes
     from N to M bytes
     Huffman tree has D levels below root
     K distinct bytes in input
     dictionary overhead = B bytes
     effective  entropy  = E.EE bits/byte
     asymptotic entropy  = A.AA bits/byte
     ```

---

### 1.2 `unpack` Utility

#### Synopsis
```sh
unpack file ...
```

#### Behavior & Operations
`unpack` restores compressed `.z` files back to their original uncompressed data:

1. **Argument & Suffix Resolution**:
   * If the argument ends with `.z` (e.g., `doc.txt.z`), the output filename is set by stripping the trailing `.z` (`doc.txt`).
   * If the argument does not end with `.z` (e.g., `doc.txt`), `.z` is appended to locate the compressed input file (`doc.txt.z`), while the argument itself (`doc.txt`) is used as the output filename.
   * **Length Constraints**: Path length must be $< 77$ characters and output basename length $\le 13$ characters.
     * Output (`stderr`): `File name too long -- <filename>`

2. **Pre-Decompression Checks**:
   * **Input Open Error**: If the `.z` file cannot be opened:
     * Output (`stderr`): `Unable to open <filename>.z`
   * **Destination Existence Conflict**: If the target uncompressed file already exists:
     * Output (`stderr`): `<filename>: Already exists`
   * **Hard Link Warning**: If the `.z` input file has hard links (`st_nlink > 1`):
     * Output (`stderr`): `Warning: '<filename>.z' has links`

3. **Format Verification**:
   * Reads 2-byte magic identifier from the header.
   * Valid magic identifiers:
     * `0x1F 0x1E` (`037 036` octal): Standard System III/V Huffman packed format.
     * `0x1F 0x1F` (`037 037` octal): Old early Unix Huffman packed format.
   * If magic bytes do not match either signature:
     * Output (`stderr`): `Unable to unpack <filename>` (or `<filename>.z: not in packed format`)
     * Target output file creation is aborted.

4. **Decompression & Clean-Up**:
   * Target output file is created inheriting permissions (`st_mode`), owner ID, group ID, and timestamps from the `.z` file.
   * Reads tree structures and decodes stream until exact original uncompressed size is recovered.
   * On completion:
     * Output (`stdout` / `stderr`): `<filename>: unpacked`
     * Deletes (`unlink`) the source `.z` file.
   * On read/write or stream error:
     * Output (`stderr`): `<filename>.z: unpacking error` (or `<filename>: write error`)
     * Partial uncompressed output file is deleted (`unlink`); source `.z` file is retained.

---

### 1.3 `pcat` Utility

#### Synopsis
```sh
pcat file ...
```

#### Behavior & Operations
* Operates as a transparent cat for packed files: decompresses `.z` files and streams raw uncompressed data to standard output (`stdout`).
* **Argument Resolution**: Accepts file names with or without `.z` extension (appends `.z` if omitted).
* **Non-Destructive**: Does **not** unlink or modify the source `.z` files.
* **Error Handling**: Diagnostics and error messages are written to standard error (`stderr`), keeping standard output clean for piping.

---

## 2. Binary File Format Specifications

There are two historical variants of the `pack` format:
1. **System III / System V Format (`0x1F 0x1E`)**: Standard canonical Huffman coding specification developed by T.G. Szymanski.
2. **Old Format (`0x1F 0x1F`)**: Early explicit binary tree specification developed by Steve Zucker (~1977) for PDP-11 systems.

---

### 2.1 System III / System V Huffman Format Specification (`0x1F 0x1E` / `.z`)

The standard `.z` file format uses **Canonical Huffman Coding** with a compact level-count tree representation.

#### Binary Layout Overview

```text
+-------------------+-------------------+---------------------------------------+
| Magic Byte 0 (US) | Magic Byte 1 (RS) |   Original Uncompressed Size (4 B)    |
|       0x1F        |       0x1E        |          (32-bit Big-Endian)          |
+-------------------+-------------------+---------------------------------------+
|  Max Depth (1 B)  | Level Counts (L B)| Symbol Array (N B)                    |
|     maxlev        | levcount[1..L]    | Ordered byte symbols                  |
+-------------------+-------------------+---------------------------------------+
|                                                                               |
|                            Packed Bitstream Data                              |
|            (Variable-length Canonical Huffman codes, MSB to LSB)              |
|                                                                               |
+-------------------------------------------------------------------------------+
```

---

#### Detailed Header Specification

1. **Magic Header Identifier (Bytes 0–1)**:
   * **Byte 0**: `0x1F` (Octal `037`, ASCII Unit Separator `US`).
   * **Byte 1**: `0x1E` (Octal `036`, ASCII Record Separator `RS`).
   * **Purpose**: Identifies file as a System III/V Huffman packed archive.

2. **Original Uncompressed File Size (Bytes 2–5)**:
   * **Format**: 32-bit unsigned Big-Endian integer.
   * **Value**: Exact total byte count of the uncompressed data stream ($N_{\text{orig}}$).
   * **Computation**:
     $$\text{origsize} = (\text{Byte}_2 \ll 24) \mid (\text{Byte}_3 \ll 16) \mid (\text{Byte}_4 \ll 8) \mid \text{Byte}_5$$

3. **Maximum Tree Depth (Byte 6)**:
   * **Format**: 8-bit unsigned integer (`maxlev`).
   * **Constraint**: Must satisfy $1 \le \text{maxlev} \le 24$.

4. **Level Leaf Counts Array (Bytes 7 to $6 + \text{maxlev}$)**:
   * **Format**: Array of `maxlev` 8-bit unsigned integers.
   * **Levels $1 \dots \text{maxlev} - 1$**: Byte $(6 + i)$ contains `levcount[i]`, specifying the exact number of leaf nodes at tree level $i$.
   * **Level $\text{maxlev}$**: Byte $(6 + \text{maxlev})$ contains `levcount[maxlev] - 2`.
     * *Note*: Because any valid non-trivial binary tree contains at least 2 leaves at maximum depth, storing count minus 2 allows level leaf counts up to 257 to fit in an 8-bit byte.

5. **Symbol Byte Array (Leaves Array)**:
   * **Format**: Sequence of 8-bit literal byte values (`0..255`).
   * **Length**: Total count equals $\sum_{i=1}^{\text{maxlev}} \text{levcount}[i] - 1$ bytes stored in the header.
   * **Ordering**: Symbols are stored in order of increasing level depth (level 1 leaves, followed by level 2 leaves, ..., up to level `maxlev` leaves). Within the same level, symbols are ordered by canonical sequence.
   * **End-of-File (END) Symbol**: Symbol 256 (`END`) is implicitly positioned as the final leaf at level `maxlev` (or maxlev-1) and is not explicitly written in the header's symbol byte array.

---

#### Canonical Huffman Tree Reconstruction & Level Leaf Table

The tree structure is completely determined by `maxlev` and the leaf count table. The decoder calculates the number of internal nodes (`intnodes[lev]`) at each level from `maxlev` down to 1:

1. **Internal Node Boundaries Calculation**:
   ```text
   nchildren = 0
   For lev = maxlev down to 1:
       count = levcount[lev]    (with levcount[maxlev] restored by adding 2)
       intnodes[lev] = nchildren / 2
       nchildren = nchildren + count
   ```

2. **Symbol Table Assignment**:
   * Pointer array `tree[lev]` points to the slice of the Symbol Array assigned to level `lev`.

---

#### Bitstream Encoding & Decoding Rules

* **Bit Packing**:
  * Bits are packed into bytes from **Most Significant Bit (MSB, `0x80`)** to **Least Significant Bit (LSB, `0x01`)**.

* **Decoding Logic**:
  1. Initialize level `lev = 1` and current code accumulator `i = 0`.
  2. For each bit read from stream:
     $$i = (i \ll 1) \mid \text{bit}$$
  3. Compute index offset:
     $$j = i - \text{intnodes}[\text{lev}]$$
  4. **Branch Condition**:
     * If $j \ge 0$: Code matches a leaf at level `lev`.
       * Look up symbol: $S = \text{tree}[\text{lev}][j]$.
       * If $S == \text{END}$ symbol or total emitted bytes equals $\text{origsize}$:
         * **Stop Decoding**: Bitstream extraction complete.
       * Otherwise:
         * Emit byte $S$ to output stream.
         * Reset state: `lev = 1`, `i = 0`.
     * If $j < 0$: Code matches an internal node.
       * Increment depth: $\text{lev} = \text{lev} + 1$, keep accumulator $i$, read next bit.

---

### 2.2 Old / Early Unix Huffman Format Specification (`0x1F 0x1F`)

The early Unix `.z` format (Steve Zucker, ~1977) uses an **Explicit Binary Tree Dictionary** serialized with PDP-11 16-bit word semantics.

#### Binary Layout Overview

```text
+-----------------------+-----------------------+---------------------------------------+
| Magic Byte 0          | Magic Byte 1          |   Original Uncompressed Size (4 B)    |
|         0x1F          |         0x1F          |         (32-bit PDP-11 Word)          |
+-----------------------+-----------------------+---------------------------------------+
|    Tree Size (2 B)    | Compressed Tree Dictionary (Variable Length)                  |
|    keysize (words)    | (Packed word nodes, 0x3FF escape + 16-bit word)               |
+-----------------------+---------------------------------------------------------------+
|                                                                                       |
|                            Packed Bitstream Data                                      |
|            (16-bit Little-Endian Words, MSB to LSB Bit Stream)                        |
|                                                                                       |
+---------------------------------------------------------------------------------------+
```

---

#### Detailed Header Specification

1. **Magic Header Identifier (Bytes 0–1)**:
   * **Value**: `0x1F 0x1F` (16-bit octal `017437`).
   * **Byte Order**: Byte 0 = `0x1F`, Byte 1 = `0x1F`.

2. **Original Uncompressed Size (Bytes 2–5)**:
   * **Format**: 32-bit PDP-11 Word (Middle-Endian word order: High 16-bit word first, Low 16-bit word second; each 16-bit word stored in PDP-11 Little-Endian byte order `[low_byte, high_byte]`).
   * **Decoding**:
     $$\text{hi} = \text{get\_word}(), \quad \text{lo} = \text{get\_word}(), \quad \text{origsize} = (\text{hi} \ll 16) \mid \text{lo}$$

3. **Tree Size (Bytes 6–7)**:
   * **Format**: 16-bit Little-Endian integer (`keysize`).
   * **Value**: Count of words in the expanded tree dictionary array (`Tree[]`).

4. **Compressed Tree Dictionary Array**:
   * Consists of variable-length encoded dictionary nodes:
     * Read byte $B$.
     * If $B == \text{0377}$ (`0xFF`): Next two bytes form a 16-bit Little-Endian word $W$. Store $\text{Tree}[t++] = W$.
     * If $B < \text{0377}$ (`0x00 .. 0xFE`): Zero-extend byte $B$ to 16-bit word. Store $\text{Tree}[t++] = B$.
   * Repeats until `keysize` entries are decoded into `Tree[]`.

---

#### Explicit Binary Tree Structure & Pointer Representation

The decoded array `Tree[]` encodes a binary tree where each node is stored as a 2-word pair at index `tp`:

* **Internal Node (`Tree[tp] != 0`)**:
  * `Tree[tp]`: Relative word offset to left child branch (Bit `0`).
  * `Tree[tp + 1]`: Relative word offset to right child branch (Bit `1`).
  * **Navigation**:
    * If bit is `0`: $\text{tp} = \text{tp} + \text{Tree}[\text{tp}]$
    * If bit is `1`: $\text{tp} = \text{tp} + \text{Tree}[\text{tp} + 1]$

* **Terminal Leaf Node (`Tree[tp] == 0`)**:
  * `Tree[tp]`: Zero marker (`0`).
  * `Tree[tp + 1]`: Literal 8-bit character value (`0..255`).
  * **Action**: Emit character `Tree[tp + 1]`, reset root index $\text{tp} = 0$, decrement remaining byte count $\text{origsize} = \text{origsize} - 1$.
  * Terminate when $\text{origsize} == 0$.

---

#### Bitstream & Word Packing Rules

* **Word Buffering**: Bitstream is read in 16-bit Little-Endian words (`get_word()`).
* **Bit Extraction Order**: Bits within each 16-bit word are extracted from **Most Significant Bit (MSB, bit 15, `0x8000`)** to **Least Significant Bit (LSB, bit 0, `0x0001`)**.
* Shift word left by 1 bit (`word <<= 1`) after consuming each bit.

---

## 3. Summary Comparison Matrix

| Feature / Dimension | Standard System III / System V `pack` | Old / Early Unix `pack` |
| :--- | :--- | :--- |
| **Magic Header Bytes** | `0x1F 0x1E` (`037 036` octal) | `0x1F 0x1F` (`037 037` octal) |
| **File Extension** | `.z` | `.z` |
| **Huffman Coding Variant** | Canonical Huffman Coding | Explicit Binary Tree |
| **Size Encoding** | 32-bit Big-Endian Integer | 32-bit Middle-Endian (PDP-11 Word) |
| **Tree Storage Layout** | Level leaf counts array + ordered leaf symbols | Compressed node offsets array (`Tree[]`) |
| **Max Tree Depth** | Up to 24 levels | Limited by dictionary buffer (1024 words) |
| **Bit Packing Unit** | 8-bit Bytes (MSB to LSB) | 16-bit Words (MSB to LSB) |
| **EOF Handling** | Implicit END symbol (256) + origsize | Original size count down (`origsize == 0`) |
| **Default CLI Action** | Replaces file with `.z` in-place | Replaces file with `.z` in-place |
| **In-Place Preservation** | Preserves permissions (`st_mode`), uid/gid, timestamps | Preserves permissions (`st_mode`), uid/gid, timestamps |
| **Non-Saving Behavior** | Cancels compression if output blocks $\ge$ input blocks | Cancels compression if output blocks $\ge$ input blocks |
| **Trivial File Handling** | Aborts if $<2$ distinct bytes in input | Aborts if $<2$ distinct bytes in input |
