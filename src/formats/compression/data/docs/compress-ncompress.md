# Specification of `compress`, `uncompress`, `zcat`, and `ncompress` Utilities

This document provides a comprehensive technical specification of the command-line interface (CLI), operational behaviors, binary file formats, and historical revisions for the Unix `compress`, `uncompress`, `zcat`, `zcmp`, `zdiff`, and `zmore` utilities (originating from Spencer W. Thomas, James A. Woods, Joe Orost et al., 1984–1985, maintained as `ncompress` by Peter Jannesen and Mike Frysinger, and including proposed POSIX 2024 updates from `46.diff`).

---

## 1. Command-Line Interface (CLI) Specification

The `ncompress` suite provides standard file compression and decompression utilities:
* `compress`: Compresses files or standard input using adaptive LZW coding or POSIX 2024 DEFLATE/gzip coding.
* `uncompress`: Restores compressed `.Z` or `.gz` files or standard input to uncompressed form.
* `zcat`: Decompresses compressed files or standard input directly to standard output.
* `zcmp`: Compares compressed files using `cmp`.
* `zdiff`: Compares compressed files using `diff`.
* `zmore`: Interactive terminal file perusal filter for viewing compressed text.

---

### 1.1 `compress` Utility

#### Synopsis
```sh
compress [-dfghvcVr] [-b maxbits] [-m algo] [-n] [-s] [-q] [-h] [--] [path ...]
```

#### Options & Flags

* **`-b maxbits | level`**:
  * Dual-purpose parameter depending on the selected algorithm:
    * **LZW Mode (`-m lzw`)**: Sets the maximum code width in bits ($9 \le \text{maxbits} \le 16$, default $16$). If missing when `-b` is passed, prints `Missing maxbits` to `stderr` and exits with status 1.
    * **Gzip/Deflate Mode (`-m gzip` / `-m deflate` / `-g`)**: Sets the gzip compression level ($1 \le \text{level} \le 9$, default $6$). If a value outside $[1, 9]$ is specified, prints `gzip compression level must be between 1 and 9` to `stderr` and exits with status 1.
* **`-g` (Gzip Algorithm Shortcut - POSIX 2024)**:
  * Shortcut option equivalent to `-m gzip`. Selects DEFLATE compression in gzip format.
* **`-m algo` (Algorithm Selection - POSIX 2024)**:
  * Selects the compression algorithm. Supported values for `algo`:
    * `lzw`: Adaptive Lempel-Ziv coding (default). Output suffix: `.Z`.
    * `gzip`: DEFLATE data wrapped in gzip format. Output suffix: `.gz`.
    * `deflate`: Synonym for `gzip`. Output suffix: `.gz`.
  * If multiple `-g` or `-m` options are specified, the last option on the command line takes precedence.
  * If an invalid algorithm name is provided, prints `Unknown algorithm: <name>` and exits with status 1.
  * If the selected algorithm requires zlib support but the binary was built without zlib (`NO_ZLIB=1`), outputs `<algo> algorithm not supported in this build` to `stderr` and exits with status **3**.
* **`-f` / `-F` (Force Overwrite & Compression)**:
  * Overwrites existing destination files without prompting.
  * Forces compression output even if the compressed size is equal to or larger than the original uncompressed file size.
  * Forces compression of regular files with multiple hard links (`st_nlink > 1`).
* **`-c` (Standard Output Mode)**:
  * Writes compressed output data directly to standard output (`stdout`).
  * Leaves the original source file(s) untouched.
* **`-d` (Decompress Mode)**:
  * Switches mode from compression to decompression (behaves identically to invoking `uncompress`).
* **`-k` (Keep Input Files)**:
  * Retains input files on disk after successful compression or decompression, rather than unlinking them by default.
* **`-v` (Verbose Statistics)**:
  * Writes compression statistics (percentage of data saved) to `stderr` for each successfully compressed file.
* **`-V` (Version & Build Configuration)**:
  * Prints program version identifier (e.g. `Compress version: 5.1`) and compilation parameters (`BITS`, `IBUFSIZ`, `OBUFSIZ`, `USE_ZLIB`, compilation flags) to `stdout`/`stderr`, then exits with status 0.
* **`-r` / `-R` (Recursive Directory Traversal)**:
  * Operates recursively. If a path argument is a directory, `compress` recurses into all subdirectories and compresses all regular files found inside.
  * When compressing recursively, files already carrying the active compression suffix (`.Z` for LZW, `.gz` for gzip) are ignored. When decompressing recursively, non-compressed files are ignored.
* **`-s` (Silent Mode)**:
  * Suppresses error messages and compression statistics (`silent = 1`, `quiet = 1`).
* **`-q` (Quiet Mode)**:
  * Suppresses compression ratio statistics printed to `stderr` (`quiet = 1`).
* **`-n` (No Magic Header Mode)**:
  * Omits the 2-byte magic header prefix (legacy mode for LZW compatibility with very early `compress` versions).
* **`-h` (Help Display)**:
  * Prints a concise summary of command line usage to `stdout` and exits with status 0.
* **`--` (End of Options)**:
  * Terminating flag. Halts option parsing so all subsequent command-line arguments are interpreted strictly as file or directory paths.

---

#### Behavior & Execution Workflow

`compress` operates in two distinct modes depending on whether path arguments are provided:

1. **Filter Mode (0 Path Arguments)**:
   * Reads uncompressed binary or text data from standard input (`stdin`).
   * Writes compressed stream (LZW `.Z` or gzip `.gz` bitstream) to standard output (`stdout`).
   * Writes compression statistics to `stderr` upon completion, unless `-q` or `-c` is specified:
     ```text
     Compression: XX.XX%
     ```
   * If output size $\ge$ input size (`bytes_out >= bytes_in`) and `-f` is not specified, `compress` exits with exit status **2**.

2. **In-Place File Mode (1 or More Path Arguments)**:
   Processes each path argument sequentially in command-line order:

   * **File Path & Suffix Validation**:
     * Appends the algorithm suffix (`.Z` for LZW, `.gz` for gzip/deflate) to construct the target filename (e.g., `doc.txt` $\to$ `doc.txt.Z` or `doc.txt.gz`).
     * If the input path already ends with the active algorithm suffix, `compress` skips the file and outputs to `stderr`:
       ```text
       <path>: already has <suffix> suffix -- no change
       ```
     * In recursive mode (`-r`), files already having the target suffix are skipped silently.

   * **File System & Type Checks**:
     * **Directory Check**: If target path is a directory and `-r` is **not** specified, prints (`stderr`):
       ```text
       <path> is a directory -- ignored
       ```
     * **Non-Regular File Check**: Symbolic links, sockets, FIFOs, and device nodes are ignored (`stderr`):
       ```text
       <path> is not a directory or a regular file - ignored
       ```
     * **Hard Link Check**: If file has hard link count $> 1$ (`st_nlink > 1`) and `-f` is **not** specified, prints (`stderr`):
       ```text
       <path> has N other links: unchanged
       ```

   * **Target File Conflict & User Prompt**:
     * If destination file (`<path><suffix>`) exists and `-f` is not specified:
       * If `compress` is running interactively (foreground tty), prompts user on `stderr`:
         ```text
         <path><suffix> already exists.
         Do you wish to overwrite <path><suffix> (y or n)?
         ```
       * If user response does not start with `y` or `Y`, cancels operation:
         ```text
         <path><suffix> not overwritten
         ```

   * **Compression Threshold & In-Place Replacement**:
     * Destination file is created with permissions `0600`.
     * **Successful Compression (`bytes_out < bytes_in` or `-f` specified)**:
       * Copies permissions (`chmod`), ownership (`chown`), and timestamps (`utime`) from source file to destination file.
       * If `-v` is active, prints replacement confirmation and reduction percentage (`stderr`):
         ```text
         <path>: -- replaced with <path><suffix> Compression: XX.XX%
         ```
       * Unlinks (deletes) original source file unless `-k` (keep) or `-c` (stdout) is specified.
     * **Non-Saving Compression (`bytes_out >= bytes_in` and `-f` omitted)**:
       * Prints warning message (`stderr`):
         ```text
         No compression -- <path> unchanged
         ```
       * Unlinks partially written output file, retains original source file untouched, and sets exit status to **2**.

---

### 1.2 `uncompress` Utility

#### Synopsis
```sh
uncompress [-fkvcV] [-n] [-s] [-q] [-h] [--] [path ...]
```

#### Behavior & Operations

* **Filter Mode (0 Arguments)**:
  * Reads compressed stream from `stdin`.
  * Auto-detects stream format from the first 2 magic header bytes:
    * `0x1F 0x9D`: LZW format.
    * `0x1F 0x8B`: Gzip format (POSIX 2024 extension).
  * Writes decompressed stream to `stdout`. If header magic does not match any supported format, prints (`stderr`) and exits with status 1:
    ```text
    stdin: not in compressed format
    ```

* **In-Place File Mode (1 or More Arguments)**:
  * **Suffix Check & Resolution**:
    * Accepts files ending in `.Z` or `.gz`.
    * If specified filename does not end in `.Z` or `.gz` and exact path does not exist, `uncompress` attempts appending `.Z` for backward compatibility.
    * If file cannot be found or lacks a recognized compressed suffix (and not in recursive mode), prints (`stderr`):
      ```text
      <path> - no .Z or .gz suffix
      ```
  * **Target Filename Generation**:
    * Strips the trailing `.Z` or `.gz` suffix to determine the output filename (e.g., `archive.tar.gz` $\to$ `archive.tar`).
  * **Format & Header Validation**:
    * Inspects header bytes to confirm stream format.
    * For LZW files: Validates header magic (`0x1F 0x9D`) and `maxbits`. If `maxbits > BITS`, prints `<path>: compressed with X bits, can only handle Y bits` and exits with status **4**.
    * For Gzip files: Validates header magic (`0x1F 0x8B`). If zlib support is disabled in current build, prints `gzip algorithm not supported in this build` and exits with status **3**.
  * **Extraction & Replacement**:
    * Creates uncompressed file, preserving mode (`chmod`), ownership (`chown`), and timestamps (`utime`).
    * On successful extraction, unlinks source file (unless `-k` or `-c` is specified).

---

### 1.3 `zcat` Utility

#### Synopsis
```sh
zcat [-V] [--] [path ...]
```

#### Behavior
* Identical to `uncompress -c`.
* Reads compressed `.Z` or `.gz` files (or standard input if no file arguments specified) and decompresses the bitstream directly to `stdout`.
* Auto-detects LZW vs. gzip format from magic header bytes (`0x1F 0x9D` or `0x1F 0x8B`).
* Leaves all input files untouched on disk.

---

### 1.4 Auxiliary Comparison & Viewing Utilities

#### `zcmp` Utility
* **Synopsis**: `zcmp [cmp_options] file1 [file2]`
* **Behavior**:
  * Wrapper script that invokes `cmp` on compressed or uncompressed files.
  * Option flags starting with `-` are passed directly to `cmp`.
  * If file arguments end in `.Z` or `.gz`, decompresses them (via `zcat` or temporary files in `/tmp`) prior to comparison.
  * Preserves and returns the exact exit status of `cmp`.

#### `zdiff` Utility
* **Synopsis**: `zdiff [diff_options] file1 [file2]`
* **Behavior**:
  * Wrapper script that invokes `diff` on compressed or uncompressed files.
  * Follows identical file resolution and decompression logic as `zcmp`.
  * Preserves and returns the exact exit status of `diff`.

#### `zmore` Utility
* **Synopsis**: `zmore [path ...]`
* **Behavior**:
  * CRT screen paging filter for compressed text files using `more`.
  * Displays file banner before each file: `------> <path> <------`
  * Interactive prompt key commands:
    | Command | Action |
    | :--- | :--- |
    | `<space>` / `i<space>` | Display next screenful (or `i` lines). |
    | `<return>` | Display 1 additional line. |
    | `^D` / `d` / `i^D` | Scroll 11 lines (or `i` lines). |
    | `iz` | Set window size to `i` lines and display screenful. |
    | `is` | Skip `i` lines and display screenful. |
    | `if` | Skip `i` screenfuls and display screenful. |
    | `q` / `Q` / `:q` / `:Q` | Quit reading current file; proceed to next file. |
    | `e` | Exit `zmore` completely. |
    | `=` | Display current line number. |
    | `i/expr` | Search forward for `i`-th occurrence of regular expression `expr`. |
    | `in` | Repeat search for `i`-th occurrence of previous regular expression. |
    | `!command` | Execute shell command (`!` replaced by previous command). |
    | `.` | Repeat previous command. |

---

### 1.5 Exit Status Codes

| Exit Code | Description |
| :---: | :--- |
| **`0`** | Successful completion; all files processed without error. |
| **`1`** | Fatal error encountered (file open/read/write failure, invalid option flag, corrupt compressed stream, missing `.Z`/`.gz` suffix). |
| **`2`** | File size was not reduced by compression (`bytes_out >= bytes_in`) when `-f` was omitted. Original input file retained. |
| **`3`** | Selected compression algorithm (`gzip` / `deflate`) is not supported in this build (compiled with `NO_ZLIB=1`). **[POSIX 2024 Extension]** |
| **`4`** | Decompression failed because input file's encoded `maxbits` parameter exceeds executable capacity (`maxbits > BITS`). |

---

## 2. Binary File Format Specifications

---

### 2.1 `.Z` LZW Format Specification

#### Header Layout (3 Bytes)
```text
+------------------+------------------+------------------+
|  Byte 0: MAGIC_1 |  Byte 1: MAGIC_2 |   Byte 2: MODE   |
|      0x1F        |      0x9D        | (Flags & Maxbits)|
+------------------+------------------+------------------+
```
* **Byte 0 (`MAGIC_1`)**: `0x1F` (octal `\037`, decimal 31).
* **Byte 1 (`MAGIC_2`)**: `0x9D` (octal `\235`, decimal 157).
* **Byte 2 (`MODE`)**: Bit 7 = `BLOCK_MODE` ($1 = \text{block mode with CLEAR flushes}$, $0 = \text{unblocked}$); Bits 5–6 = reserved ($0$); Bits 0–4 = `maxbits` ($9 \le \text{maxbits} \le 16$).

#### Symbol Encoding & Bitstream Rules
* **Symbol Space**: Literal bytes ($0\text{--}255$), `CLEAR` code ($256$, in block mode), dynamic dictionary codes ($257\text{--}2^{\text{maxbits}}-1$).
* **Bit Packing**: Codes packed in **little-endian bit order** (least significant bit first).
* **Re-alignment**: Bitstream position is realigned to the next byte boundary whenever code width $N$ increments to $N+1$ or a `CLEAR` code ($256$) is emitted.
* **Block Reset Threshold**: Resets string dictionary and emits `CLEAR` code ($256$) if compression ratio decreases at $10{,}000$-byte checkpoints after dictionary fills.

---

### 2.2 `.gz` Gzip DEFLATE Format Specification (POSIX 2024 Extension)

#### Header & Structure Layout
When built with zlib (`USE_ZLIB=1`), `compress` generates standard gzip containers compliant with RFC 1952 / POSIX 2024:
```text
+------------------+------------------+------------------+------------------+
|  Byte 0: ID1     |  Byte 1: ID2     |  Byte 2: CM      |  Byte 3: FLG     |
|      0x1F        |      0x8B        |      0x08        |     Flags        |
+------------------+------------------+------------------+------------------+
|               4-Byte Modification Time (MTIME)                            |
+---------------------------------------------------------------------------+
|  Extra Flags (XFL) | Operating System (OS) |   Compressed DEFLATE Payload |
+--------------------+-----------------------+                              |
|                                                                           |
+---------------------------------------------------------------------------+
|   4-Byte CRC-32 Checksum                  |   4-Byte Uncompressed ISIZE     |
+-------------------------------------------+-------------------------------+
```

* **Byte 0 (`ID1`)**: `0x1F` (octal `\037`, decimal 31). Gzip identification byte 1.
* **Byte 1 (`ID2`)**: `0x8B` (octal `\213`, decimal 139). Gzip identification byte 2.
* **Byte 2 (`CM`)**: Compression Method `0x08` (DEFLATE).
* **Payload**: Raw DEFLATE compressed data stream produced by `deflateInit2(&stream, level, Z_DEFLATED, 15 + 16, 8, Z_DEFAULT_STRATEGY)`.
* **Footer**: 4-byte CRC-32 checksum followed by 4-byte little-endian uncompressed input size modulo $2^{32}$.

---

## 3. Revisions & File Format History

| Version / Revision | Primary Features & Format Changes |
| :--- | :--- |
| **`compress 1.0`** (Jul 1984) | Initial release by Spencer W. Thomas (University of Utah) based on Welch's June 1984 IEEE Computer LZW algorithm. Filter-only operation reading `stdin` or a single input file and writing directly to `stdout`. Headerless stream with no magic bytes or block clear codes; dictionary entries started at code 256. |
| **`compress 1.6`** (Jul–Aug 1984) | Multi-platform overhaul by Joseph M. Orost and Steve Davies. Provided shell script wrappers `Pack`, `Unpack`, and `Pcat` to emulate System V `pack`/`unpack`/`pcat` commands using `.Z` extensions, enforcing 12-character filename limits (for 14-char filesystems), link checks, metadata copying, and file unlinking. |
| **`compress 2.0`** (Aug 1984) | Major functional integration by Joe Orost, Jim McKie, Steve Davies, and Ken Turkowski. Integrated in-place file replacement directly into `compress` (eliminating `Pack`/`Unpack` scripts). Introduced executable hard links (`uncompress`, `zcat`), 2-byte magic header (`0x1F 0x9D`), 3rd `maxbits` header byte, flags `-f` and `-c`, optional 1.0 headerless output (`-C` / `COMPATIBLE`), and exit codes (`0`, `1`, `2`). |
| **`compress 3.0`** (Jan 1985) | Architectural upgrade by Joe Orost and James A. Woods. Introduced **Adaptive Block Compression** (`BLOCK_MODE` bit `0x80` in 3rd header byte) and `CLEAR` code ($256$) to flush dictionary on ratio drops. Replaced character chaining with open-addressing double hashing. Added `-q` (quiet mode) and `zmore` interactive CRT paging filter. Stream format was incompatible with 2.0 output (unless `-C` specified), though 3.0+ decoders retained full 2.0 reading support. |
| **`compress 4.0`** (1985–1986) | Standard System V Release 3, 4.3BSD, and 2.11BSD world release by Thomas, McKie, Davies, Turkowski, Woods, and Orost. Added build autotuning for RAM constraints (`USERMEM`/`SACREDMEM`, restricting 16-bit systems like PDP-11 to 12 maxbits). Ported to non-Unix environments such as Apollo Aegis/Domain OS (handling object types like `uasc` and `obj`). |
| **`compress 4.1`** (1990–1991) | Added recursive directory traversal (`-r` / `-R` flags) authored by Dave Mack. |
| **`(N)compress 4.2+`** | Performance enhancement release by Peter Jannesen featuring 2-level fast prime hash table lookup algorithm for high-memory systems (`USERMEM >= 800KB`). |
| **`(N)compress 5.0` / `5.1`** | Modernization by Mike Frysinger. Standardized POSIX C prototypes, added `-k` (keep input) flag, updated recursive directory handling, and removed legacy `2.0` output generator flag (`-C`). |
| **`46.diff` (POSIX 2024 Updates)** | **Under consideration for POSIX Issue 8 (POSIX.1-2024) alignment**: Integrates POSIX `gzip`/`deflate` compression support via `zlib`. Adds `-g` and `-m algo` options, dual-purpose `-b` (maxbits vs gzip compression level 1–9), `.gz` file suffix support, magic byte auto-detection (`0x1F 0x8B`), build configuration flags (`USE_ZLIB` / `NO_ZLIB`), signal handler cleanup, and exit code **`3`** for unsupported algorithm builds. |

---

## 4. POSIX 2024 Updates Detailed Feature Summary (`46.diff`)

The patch `46.diff` introduces modifications to align `(N)compress` with the **POSIX.1-2024 (Issue 8)** standard specification for `compress`, `uncompress`, and `zcat`.

### Key Features Introduced in `46.diff`

1. **POSIX Algorithm Extensions (`-g` and `-m` Flags)**:
   - POSIX 2024 mandates that implementations support DEFLATE/gzip compression in addition to traditional LZW.
   - `-g` option: Standard shortcut for `-m gzip`.
   - `-m algo` option: Supports `lzw`, `gzip`, and `deflate` as algorithm identifiers.
   - Precedence: Command-line option parsing evaluates flags left-to-right; the last specified `-g` or `-m` flag overrides previous algorithm selections.

2. **Dual-Purpose `-b` Parameter**:
   - For `lzw` algorithm: `-b` controls maximum bit width ($9 \le \text{maxbits} \le 16$, default 16).
   - For `gzip`/`deflate` algorithm: `-b` controls compression level ($1 \le \text{level} \le 9$, default 6).

3. **Format Auto-Detection & Dual Suffix Handling**:
   - Target files generated during compression receive `.gz` extension when using `gzip`/`deflate`, or `.Z` when using `lzw`.
   - `uncompress` and `zcat` automatically detect the format of input files by reading the first 2 magic header bytes (`0x1F 0x9D` for LZW vs `0x1F 0x8B` for Gzip).

4. **Zlib Integration & Build Knobs**:
   - Implements gzip/deflate processing using `zlib` API (`deflateInit2`, `deflate`, `inflateInit2`, `inflate`).
   - Gzip mode is enabled by default in builds (`ZLIB_OPTIONS= -DUSE_ZLIB=1`, `ZLIB_LBOPT= -lz`).
   - Can be disabled at build time via `make NO_ZLIB=1`.

5. **New Exit Code 3**:
   - Defines exit status **`3`** (`unsupported_algorithm`), returned when a user requests an algorithm (`gzip`/`deflate`) that was disabled at build time (`NO_ZLIB=1`).

6. **Signal Handler Safety**:
   - Replaces cast-pointer signal handlers with clean function prototype `void abort_compress_signal(int signum)` to prevent compiler warnings and undefined signal handler behavior.

7. **Test Suite Enhancements (`tests/runtests.sh`)**:
   - Includes comprehensive test cases verifying algorithm selection (`-m lzw`, `-g`, `-m deflate`), standard stream piping, zcat format detection, option override precedence (`-m gzip -m lzw` vs `-m lzw -g`), and fallback validation when built with `NO_ZLIB=1`.
