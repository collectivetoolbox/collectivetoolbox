Searched for "guid"
Viewed magic_parser.rs:455-520
Searched for "elif"
Searched for "COND_"
Searched for "strength"
Searched for "parse_extra"
Viewed apprentice.c:2530-2640
Searched for "MagicTest::Regex"
Viewed magic.rs:1160-1210
Viewed is_csv.c:1-100
Viewed is_csv.c:100-180
Viewed is_tar.c:1-100
Viewed memtest.c:1-100
Viewed magic.h.in:1-100
Viewed extra_rules.rs:1-100
Viewed extra_rules.rs:450-520
Viewed issues.md:1-100
Viewed file.c:100-160
Viewed file.c:161-210
Listed directory file
Viewed file.c:230-350
Searched for "guess_format"
Searched for "ctb_formats_detection"
Searched for "file"
Listed directory tests

Here is the audited breakdown of features, subsystems, data types, and heuristics present in upstream `file` ([`old/filedetect/file/src/`](file:///workspaces/ctoolbox/old/filedetect/file/src/)) that are **not replicated** (or only partially replicated) in `ctoolbox`'s format detection implementation ([`src/formats/detection/`](file:///workspaces/ctoolbox/src/formats/detection/)).

---

## 1. Upstream Detection Subsystems

### A. SIM-H Magnetic Tape Parser ([`src/is_simh.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/is_simh.c))
* **Upstream feature:** `file_is_simh` and `simh_parse` detect SIMH magnetic tape image files. It verifies 32-bit record headers with forward and backward size checks (`getlen`), handles automatic endianness detection (`simh_bo`), skips tapemarks (`0x00000000`), handles End-of-Medium (`0xFFFFFFFF`), and outputs `"SIMH tape data"` with MIME type `"application/SIMH-tape-data"`.
* **Ctoolbox state:** **Implemented** in [`container.rs`](file:///workspaces/ctoolbox/src/formats/detection/container.rs) (`inspect_simh`). Supports both Little-Endian and Big-Endian SIMH record framing independently of host endianness, checks record padding and trailer lengths, handles tapemarks and EOM, and integrates into `detect_container_candidates`.

### B. ASN.1 / DER Parser & Comparator ([`src/der.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/der.c), [`src/der.h`](file:///workspaces/ctoolbox/old/filedetect/file/src/der.h))
* **Upstream feature:** Upstream implements a dedicated ASN.1 DER (Distinguished Encoding Rules) tag scanner and evaluator (`der_offs`, `der_cmp`, `gettag`, `getlength`, `der_tag`, `der_data`). This backs the `der` rule type used in [`Magdir/der`](file:///workspaces/ctoolbox/src/formats/dcdata/data/magic/upstream/magic/Magdir/der) and certificate/key signatures (e.g., `der seq`, `der set`, `der obj_id3=...`, `der utf8_str=...`, `der prt_str=...`, `der int1=...`, `der null`).
* **Ctoolbox state:** **Implemented** in [`der.rs`](file:///workspaces/ctoolbox/src/formats/detection/der.rs), [`magic_parser.rs`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) (`MagicTest::Der`), and [`magic.rs`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs). Traverses hierarchical constructed containers (SEQUENCE/SET) for child rules and advances sibling offsets across sequential TLV elements.

### C. OS/2 Apptype Resolution ([`src/apptype.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/apptype.c))
* **Upstream feature:** `file_os2_apptype` identifies OS/2 and early Windows application header types using `DosQueryAppType`.
* **Ctoolbox state:** **Intentionally omitted (non-portable)**. In upstream `file`, `file_os2_apptype` is entirely enclosed in `#ifdef __EMX__` and calls the proprietary OS/2 kernel system call `DosQueryAppType`. It is dead code outside OS/2 build environments. PE and NE binaries are handled portably via static and Magdir patterns.

---

## 2. Magic Rule Data Types Missing from the Engine

Upstream [`src/file.h`](file:///workspaces/ctoolbox/old/filedetect/file/src/file.h#L253-L315) and [`src/softmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/softmagic.c) support several comparison types. All 7 previously missing types have been implemented in [`MagicTest`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) and evaluated in [`magic.rs`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs):

| Upstream Type (`file.h`) | Syntax / Meaning | Upstream Responsibility | Ctoolbox Status |
| :--- | :--- | :--- | :--- |
| `FILE_FLOAT`, `FILE_BEFLOAT`, `FILE_LEFLOAT` | `float`, `befloat`, `lefloat` | IEEE-754 32-bit single-precision float comparison with relational operators. | **Implemented** in `MagicTest::FloatLe` / `FloatBe` and `magic.rs`. Decodes raw IEEE bits with relational IEEE semantics (`<`, `>`, `<=`, `>=`, `=`, `!`, `x`). |
| `FILE_DOUBLE`, `FILE_BEDOUBLE`, `FILE_LEDOUBLE` | `double`, `bedouble`, `ledouble` | IEEE-754 64-bit double-precision float comparison. | **Implemented** in `MagicTest::DoubleLe` / `DoubleBe` and `magic.rs`. Decodes 64-bit IEEE bits with float format description specifiers (`%f`, `%g`, `%e`). |
| `FILE_BESTRING16`, `FILE_LESTRING16` | `bestring16`, `lestring16` | Fixed-endian 16-bit wide character string comparisons. | **Implemented** in `MagicTest::String16Le` / `String16Be` and `magic.rs`. Supports case-insensitive matching (`/c`), wildcard matching (`x`), and description substitution. |
| `FILE_BEVARINT`, `FILE_LEVARINT` | `bevarint`, `levarint` | Variable-length integer encoding (e.g. MIDI / Protobuf). | **Implemented** in `MagicTest::VarintLe` / `VarintBe` and `magic.rs`. Decodes 7-bit continuation byte sequences with bitmasking and relational comparison. |
| `FILE_QWDATE`, `FILE_LEQWDATE`, `FILE_BEQWDATE` | `qwdate`, `leqwdate`, `beqwdate` | Windows 64-bit `FILETIME` timestamp (100ns increments since 1601-01-01). | **Implemented** in `MagicTest::DateWindows64Le` / `DateWindows64Be` and `magic.rs`. Converts Windows epoch to Unix timestamp and formats via standard date representation. |
| `FILE_OCTAL` | `octal` | Direct comparison against ASCII octal digit strings. | **Implemented** in `MagicTest::Octal` and `magic.rs`. Compares ASCII octal digits, parses numeric value, and formats as decimal or raw string. |
| `FILE_LEGUID`, `FILE_BEGUID` | `guid`, `leguid`, `beguid` | Distinguishes Microsoft mixed-endian GUIDs from standard big-endian RFC 4122 UUIDs. | **Implemented** in `MagicTest::GuidLe` / `GuidBe` and `magic.rs`. Supports little/mixed-endian (`guid`, `leguid`) and big-endian (`beguid`), wildcards (`x`), and uppercase standard formatted GUID string substitution. |

---

## 3. Magic Engine Evaluation & Syntax Limitations

### A. Missing Bitwise and Arithmetic Indirect Pointer Operations
* **Upstream:** In `softmagic.c` and `file.h` (`FILE_OPS`), indirect offsets `(<offset>[.,]<type>[~][op][val]+<adj>)` support arbitrary operators on the dereferenced pointer before applying relative adjustments:
  * Bitwise AND (`&`), OR (`|`), XOR (`^`) (e.g. `(0x10.l&0x00FFFFFF)` to mask pointer addresses)
  * Multiplication (`*`), Division (`/`), and Modulo (`%`)
  * Inverse operations (`~` / `FILE_OPINVERSE`) and signed dereference (`FILE_OPSIGNED`, `,` vs `.`)
* **Ctoolbox Status:** **Implemented**. [`IndirectOp`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) supports `Mul`, `Div`, `Mod`, `And`, `Or`, `Xor`, `Add`, and `Sub`. [`Offset::Indirect`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) and [`Offset::RelativeIndirect`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) store `is_signed`, `is_inverse`, `op`, and `adjustment`. Evaluated in [`resolve_offset`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs) and [`read_indirect_val`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs) with safe arithmetic and sign-extension.

### B. Conditionals in Magic Rules (`ENABLE_CONDITIONALS`)
* **Upstream:** Upstream supports rule branching via `>if`, `>elif`, `>else`, and `>clear` (`COND_IF`, `COND_ELIF`, `COND_ELSE`).
* **Ctoolbox Status:** **Implemented**. [`MagicCond`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) enum (`None`, `If`, `Elif`, `Else`) and `MagicTest::Clear` are parsed by [`parse_magic_line`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) and evaluated across child hierarchies in [`evaluate_children`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs), correctly tracking branch outcomes and skipping alternative arms when an `if` or `elif` arm succeeds.

### C. String Matching Modifiers
* **Upstream:** Upstream supports modifier flags on string tests:
  * `/w`: Compact *optional* whitespace (differs from `/W` mandatory whitespace compression).
  * `/f`: Full-word boundary matching.
  * `/t`: Text-only test (only evaluated if the buffer is deemed text).
  * `/b`: Binary-only test (only evaluated if the buffer is deemed binary).
* **Ctoolbox Status:** **Implemented**. [`StringFlags`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) parses `/w` (`optional_whitespace`), `/W` (`compact_whitespace`), `/f` (`full_word`), `/t` (`text_only`), and `/b` (`binary_only`). Evaluated in [`MagicTest::String`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs) and [`MagicTest::Search`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs) with word boundary verification and binary/text buffer classification.

### D. Regex Search Limits
* **Upstream:** Upstream regex flags accept `/l<count>` to constrain regex evaluation to the first `<count>` lines of the input.
* **Ctoolbox Status:** **Implemented**. [`parse_regex_flags`](file:///workspaces/ctoolbox/src/formats/detection/magic_parser.rs) parses optional line limits (`line_limit: Option<usize>`), and [`MagicTest::Regex`](file:///workspaces/ctoolbox/src/formats/detection/magic.rs) evaluates candidates line-by-line according to POSIX `REG_NEWLINE` semantics, respecting line count constraints and multiline pattern fallbacks.

---

## 4. Decompression / Peeking Scope ([`src/compress.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/compress.c))

* **Upstream:**
  1. Supports transparent decompression for 15+ formats via built-in libraries or external tools: compress/uncompress (`.Z`, LZW), gzip (`.gz`), bzip2 (`.bz2`), xz (`.xz`), lzip (`.lz`), lrzip (`.lrz`), lz4 (`.lz4`), zstd (`.zst`), lzma (`.lzma`), pack (`\037\036`), frozen (`\037\236`), SCO LZH (`\037\240`), and raw zlib deflate.
  2. When decompressed, it recursively invokes `file_buffer` on the decompressed stream, running **all** detection tests (identifying inner ELF binaries, shell scripts, XML, source code, etc.).
* **Ctoolbox Status:** **Partially Implemented**.
  1. **Working:** [`probe_decompression_candidates`](file:///workspaces/ctoolbox/src/formats/detection/decompression.rs) implements transparent and recursive bounded decompression inspection across currently supported formats (`Gzip`, `Bzip2`, `Bzip`, `Xz`, `Lzip`, `Lz4`, `Zstd`, `Lzma`, `CompressLzw` 1/2/16/modern, `Pack`, `OldPack`, `Compact`, `Freeze1/2`, `SCO Compress`, `Deflate`/`Zlib`, `Rzip`, `Lzo`, `Szip`). Supports `--compat` mode (`-z` and `-Z`) as well as rich compound resolution (`TarGz`, `TarBz2`, `TarZ`, `TarLzo`) and format chain directives (`@chain(...)`).
  2. **Remaining Gaps vs Upstream:**
     - **`lrzip` (`.lrz`):** Not implemented. Blocked on `ZPAQ` decompression support and multi-file archive streaming facilities.
     - **External helper fallbacks:** Upstream can delegate to system decompression binaries (`gzip -cd`, `uncompress -c`, etc.) if built-in decoders fail; ctoolbox relies solely on in-process decoders.



---

## 5. In-Depth Container Introspection Details

### A. ELF Introspection ([`src/readelf.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/readelf.c) vs [`src/formats/detection/readelf.rs`](file:///workspaces/ctoolbox/src/formats/detection/readelf.rs))
* **Status:** `Partially Implemented`
* **Upstream:** `readelf.c` (1,981 lines) performs comprehensive structural analysis:
  * Program headers: Extracts dynamic linker interpreter path (`PT_INTERP`), and distinguishes statically linked vs. dynamically linked binaries.
  * Section headers: Identifies stripped vs. not stripped binaries, debug symbols, and section layout.
  * Notes: Parses OS ABI tags (e.g. `for GNU/Linux 3.2.0`, `FreeBSD 13.0`), GNU Build-ID (`BuildID[sha1]=...`), Go build IDs, Intel CET shadow stack / IBT properties, AArch64 BTI/PAC, and core dump process/register data.
* **Ctoolbox Implementation:**
  * Added dedicated [`readelf.rs`](file:///workspaces/ctoolbox/src/formats/detection/readelf.rs) mirroring upstream `readelf.c`: parses 32-bit and 64-bit ELF headers, program headers (`PT_INTERP`, `PT_DYNAMIC`, `PT_NOTE`), section headers (`.debug_info`, `SHT_SYMTAB`, `SHT_NOTE`), and note segments (`NT_GNU_BUILD_ID`, `NT_GO_BUILD_ID`, `NT_GNU_ABI_TAG`, BSD tags).
  * Added `${x?then:else}` variable expansion (`varexpand`) in `magic.rs` driven by execution mode bit and `DF_1_PIE` flag, resolving `pie executable` vs `shared object` and MIME types.
  * Tied ELF compatibility suffix formatting (`static-pie linked`, `dynamically linked`, interpreter, BuildID, ABI version, `stripped` / `not stripped`) into detection reporting.
* **Remaining Gaps vs Upstream:**
  * Intel CET shadow stack and indirect branch tracking (`GNU_PROPERTY_X86_FEATURE_1_AND`).
  * AArch64 PAC / BTI properties (`GNU_PROPERTY_AARCH64_FEATURE_1_AND`).
  * Core dump register & process data extraction (`NT_PRSTATUS`, `NT_PRPSINFO`).
  * PaX / hardened Linux flag note extraction.


### B. OLE2 Compound Document Files ([`src/readcdf.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/readcdf.c), [`src/cdf.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/cdf.c) vs [`inspect_ole2_cdf`](file:///workspaces/ctoolbox/src/formats/detection/container.rs#L1190))
* **Upstream:** `cdf.c` (1,200 lines) traverses the entire directory SAT/FAT/MiniFAT chain and parses property streams (`\x05SummaryInformation` and `\x05DocumentSummaryInformation`):
  * Extracts Title, Subject, Author, Keywords, Comments, Template name, Last Saved By, Revision Number, Editing Time, Create/Saved Timestamps, Word/Page/Slide Counts, and Application Name.
* **Ctoolbox:** [`inspect_ole2_cdf`](file:///workspaces/ctoolbox/src/formats/detection/container.rs#L1190) only scans the first directory sector for static UTF-16 stream names (`WordDocument`, `Book`, `PowerPoint Document`). It does not follow multi-sector directory chains or extract document metadata/timestamps.

### C. Filesystem & Inode Special Details ([`src/fsmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/fsmagic.c) vs [`special.rs`](file:///workspaces/ctoolbox/src/formats/detection/special.rs))
* **Upstream:**
  * Symlinks: Resolves targets and explicitly reports broken links (`broken symbolic link to <target>`).
  * Devices: Extracts and formats major/minor numbers (`character special (major/minor)`).
  * Permissions/Attributes: Reports setuid, setgid, sticky bits, and BSD file flags (`UF_NODUMP`, `UF_IMMUTABLE`, `SF_APPEND`, etc.).
  * Solaris Doors: Detects `S_IFDOOR`.
* **Ctoolbox:** [`detect_special_entity`](file:///workspaces/ctoolbox/src/formats/detection/special.rs#L467) only reflects passed-in hint strings; it does not read live file descriptors, inspect symlink targets, or format major/minor numbers.

---

## 6. Text & Encoding Detection Pipeline ([`src/encoding.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/encoding.c), [`src/ascmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/ascmagic.c))

* **UTF-7 Encoding:** Upstream implements `looks_utf7`. Missing from [`src/formats/detection/text.rs`](file:///workspaces/ctoolbox/src/formats/detection/text.rs).
* **Decoded Softmagic on Encoded Buffers:** Upstream's `file_ascmagic_with_encoding` converts decoded Unicode text (`ubuf` from UTF-16, UTF-32, or EBCDIC) into a temporary UTF-8 buffer and re-runs `file_softmagic` with `TEXTTEST`. In ctoolbox, magic evaluation runs strictly against raw bytes; an XML or script file encoded in UTF-16 (without BOM) or EBCDIC will not match text magic rules.
* **Text Formatting Sub-attributes:** Upstream reports detailed layout qualifiers:
  * `, with escape sequences` (ANSI escapes)
  * `, with overstriking` (backspace sequences)
  * `, with no line terminators`
  * Complex line terminator combinations (e.g. `CRLF, CR, NEL line terminators`).

---

## 7. Command-Line Options and Dynamic Tuning ([`src/file.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/file.c))

Upstream `file` provides extensive runtime tuning parameters and filtering flags that are not yet exposed:
* **Exclusion Flags (`-e` / `--exclude`):** Excludes individual tests at runtime: `apptype`, `ascii`/`text`, `cdf`, `compress`, `csv`, `elf`, `encoding`, `soft`, `tar`, `json`, `simh`.
* **Parameter Tuning (`-P` / `--parameter`):** Configures limits for `bytes`, `elf_notes`, `elf_phnum`, `elf_shnum`, `elf_shsize`, `encoding`, `indir`, `name`, `regex`, and `magwarn`.
* **Execution Options:** `-k` (`--keep-going` / continue after first match), `-z` / `-Z` (uncompress and report / transparent uncompress), `-b` (`--brief`), `-N` (no padding), and `-s` (inspect device nodes).