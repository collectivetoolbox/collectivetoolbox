I'd like to find a way to make the databases of formats, Dcs, and the graph layout as easy as possible to maintain: retaining uniqueness of IDs, checking validity of fields, and so on. The current CSVs structure are not too bad - and I do like working in a spreadsheet for ease of editing - but it may be worth considering if some other structure would be better. At least adding automatic validation (maybe at build time?) for the CSV would be important. There is also the issue that as the amount of records increases, it will become unwieldy.

The relevant CSV files are: formats.csv, global_graph_layout.csv, and DcData.csv. (DcData.csv is currently in the EITE crate because I've been sharing it, but it's kind of messy that I keep editing the data in that module (which is a compatibility layer and should be more or less frozen in time) with new Dcs that aren't actually supported in the original version. I should probably do what I already did with formats.csv and 'fork' DcData.csv into another module for ongoing additions.)

One other issue is that in global_graph_layout.rs and dc_number.rs, and probably various other places, there are many constants that are canonically maintained in CSV.

I don't like the duplication of defining them as constants.

Please review the current design and evaluate alternatives and ways I can improve the DX and maintainability of this. I suspect that build-time validation and codegen of constants (maybe by a flag on certain Dcs that they are important and should get a constant?) will be the ideal route, though I wouldn't want to have duplication of code for parsing things like the DSLs that the CSV uses, which would need to be validated at build time, and then also parsed for actual use at runtime.



Searched for "formats.csv"
Searched for "global_graph_layout.csv"
Ran command: `find . -name "*.csv"`
Searched for "DcData"
Ran command: `find src assets vendor -name "*layout*" -o -name "*format*" -o -name "*DcData*" -o -name "*dc_number*"`
Searched for "global_graph_layout"
Viewed global_graph_layout.rs:1-200
Searched for "get_storage_minimal_inline_data"
Viewed minimal.rs:1-60
Ran command: `find src -name "data"`
Ran command: `find src/formats/utilities/data src/formats/eite/data src/storage/minimal/data -type f`
Ran command: `head -n 20 src/storage/minimal/data/global-graph-layout.csv src/formats/utilities/data/formats.csv src/formats/eite/data/DcData.csv`
Viewed dc_number.rs:1-250
Viewed format_info.rs:1-250
Searched for "SHORT_DC_"
Viewed data.rs:1-150
Viewed dcal.rs:1-100
Ran command: `head -n 50 src/formats/eite/data/DcData.csv`
Ran command: `grep -E "[><:]" src/formats/eite/data/DcData.csv | head -n 30`
Searched for "formats.csv"
Ran command: `diff -u src/formats/eite/data/formats.csv src/formats/utilities/data/formats.csv | head -n 40`
Ran command: `ls -la src/formats`
Ran command: `ls -la src/formats/dctext`
Ran command: `ls -la src/build_support`
Ran command: `find . -name "build.rs" -not -path "*/vendor/*" -not -path "*/target/*"`
Viewed build.rs:1-58
Viewed build.rs:1-100
Viewed build_support.rs:1-27
Viewed ipc_codegen.rs:1-60

Here is a comprehensive review of the current design across [formats.csv](file:///workspaces/ctoolbox/src/formats/utilities/data/formats.csv), [global-graph-layout.csv](file:///workspaces/ctoolbox/src/storage/minimal/data/global-graph-layout.csv), and [DcData.csv](file:///workspaces/ctoolbox/src/formats/eite/data/DcData.csv), along with an evaluation of storage alternatives, validation strategies, and a code generation design that eliminates constant duplication and avoids DSL parser redundancy.

---

### 1. Assessment of the Current State & Pain Points

| Source File | Location | Current Role & Observations |
| :--- | :--- | :--- |
| **`global-graph-layout.csv`** | [`src/storage/minimal/data/`](file:///workspaces/ctoolbox/src/storage/minimal/data/global-graph-layout.csv) | Defines partition ranges (`Unicode`, `Document Characters`, `Formats and Filters`, `Reserved`, `System`). Range boundaries are duplicated in [global_graph_layout.rs](file:///workspaces/ctoolbox/src/storage/minimal/global_graph_layout.rs#L31-L36) as `const UNICODE_REGION_START`, `DC_REGION_START`, etc. |
| **`formats.csv`** | [`src/formats/utilities/data/`](file:///workspaces/ctoolbox/src/formats/utilities/data/formats.csv) | Modern registry of formats with 18 columns (`Ident`, `Category`, `Layer`, `MIME`, `Extensions`, `Support levels -1..=5`, `BaseFormat`, etc.). Parsed at runtime in [format_info.rs](file:///workspaces/ctoolbox/src/formats/utilities/format_info.rs#L56-L129) via `LazyLock<HashMap<usize, FormatInfo>>`. |
| **`DcData.csv`** | [`src/formats/eite/data/`](file:///workspaces/ctoolbox/src/formats/eite/data/DcData.csv) | Classic Dc registry (304+ rows) containing character attributes, bidi classes, combining classes, and DSL syntax expressions in the `Aliases` column (`:[246 247] [^248 255]+ ~`, `<equiv>`, `<ambiguous>`, `>32`). Currently trapped in `ctb_formats_eite` despite being edited for new Dcs. |

#### Key DX & Maintenance Bottlenecks
1. **Constant Duplication & Drift**:
   - In [dc_number.rs](file:///workspaces/ctoolbox/src/formats/dctext/dc_number.rs#L46-L81), short IDs (`SHORT_DC_BEGIN_NUMBER = 6`, `SHORT_ID_FORMAT_199 = 199`, `SHORT_DC_BASE64_START = 127`, etc.) and their calculated GIDs (`GID_BEGIN_NUMBER = 1_114_118`, `GID_FORMAT_199 = 2_228_423`) are hardcoded manually.
   - Any ID adjustment in CSV requires hunting down constants across `dc_number.rs`, `global_graph_layout.rs`, `cli_identifiers.rs`, and tests.
2. **Lack of Automated Schema & Integrity Validation**:
   - No build-time check enforcing ID uniqueness, monotonic ordering, or non-overlapping ranges.
   - No validation on enum-like columns (Bidi class, General Category, Support level scale `-1..=5`).
   - The DSL expressions in the `Aliases` column (`<equiv>240 239`, `:~ [260:] 259`, `>32`) are unverified strings; references to non-existent Dcs or syntax mistakes are silent until runtime failure.
3. **EITE Coupling**:
   - `ctb_formats_eite` is a legacy backward-compatibility layer, but editing `DcData.csv` inside it risks breaking historical EITE test fixtures while conflating modern Dc expansions with legacy semantics.

---

### 2. Storage Format Evaluation (Spreadsheet DX vs Alternatives)

| Format | Spreadsheet DX | Git Diffs & Conflicts | Schema / Typing | Scale (10k+ rows) | Verdict |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **CSV / TSV (Normalized)** | **Excellent** (LibreOffice Calc, Excel, Modern CSV, VSCode CSV extensions) | Good if line-sorted & format-normalized | Requires external validator | Runtime parsing overhead unless codegenned | **Recommended Source of Truth** |
| **TOML / JSON / YAML** | Poor (Requires conversion scripts to edit as grid) | Excellent (record-per-key) | Built-in (Serde / JSON Schema) | Moderate file sizes | Unwieldy for high-volume tabular maintenance |
| **SQLite / DuckDB in repo** | Good (GUI DB browsers) | **Terrible** (binary git blobs, impossible git merges) | Strong | Excellent | Not suitable for git-tracked sources |
| **Multi-file (1 file per entry)** | Very Poor (No overview) | Perfect per-file diffs | Strong | High inode/directory clutter | Overkill and bad DX for tables |

#### Recommendation: **Keep CSV as Canonical Source + Add Canonical Normalizer**
Spreadsheet editing is by far the fastest and least error-prone way to scan across columns, compare definitions, and perform batch operations. To prevent CSV git pitfalls (differing quoting styles between Excel / LibreOffice / editors):
- Retain `.csv` files as the human-edited source.
- Add an automated formatter/normalizer (run via `./lint` or pre-commit) that ensures standard RFC4180 quoting, Unix line endings (`\n`), and stable sorting by ID.

---

### 3. Architecture for Build-Time Validation & Zero-Duplication Codegen

To validate CSVs and generate constants **without duplicating parser logic between build-time and runtime**, we can leverage the workspace's existing build architecture.

```
                  ┌────────────────────────────────────────────────────────┐
                  │          ctb_formats_utilities (or core)               │
                  │  - DcDataRecord / FormatInfo / LayoutEntry structs    │
                  │  - Canonical CSV parsing & column schema definitions   │
                  │  - Dc DSL Parser (syntax rules, <equiv>, xrefs)        │
                  │  - Validator logic (ID uniqueness, cross-ref integrity)│
                  └──────────────────────────┬─────────────────────────────┘
                                             │
                       ┌─────────────────────┴─────────────────────┐
                       ▼                                           ▼
         ┌───────────────────────────┐               ┌───────────────────────────┐
         │       Build-Time          │               │         Runtime           │
         │  ctb_build_support /      │               │  ctb_formats_utilities    │
         │  build.rs / ./lint test   │               │  ctb_formats_dctext       │
         │                           │               │  ctb_storage_minimal      │
         ├───────────────────────────┤               ├───────────────────────────┤
         │ • Runs full validation    │               │ • Zero CSV parsing in hot │
         │ • Validates DSL trees     │               │   paths (uses static maps/│
         │ • Emits generated consts  │               │   generated slices)       │
         │   and static tables to    │               │ • Fallback dynamic lookup │
         │   `built/` or `OUT_DIR`   │               │   when needed             │
         └───────────────────────────┘               └───────────────────────────┘
```

#### Step 1: Fork `DcData.csv` into a Modern Module
- **Action**: Move modern Dc maintenance to `src/formats/dctext/data/dc_data.csv` (or `src/formats/utilities/data/dc_data.csv`).
- **EITE**: Leave `src/formats/eite/data/DcData.csv` as a frozen snapshot representing only the original legacy Dcs (up to ID 297/304).

#### Step 2: Unify Schema & DSL Parser in a Shared Location
Define standard Serde-compatible records and a single DSL parser in `ctb_formats_utilities`:
- `FormatRow`: Validates `id`, `ident` (Rust ident valid), `support_levels` (`-1..=5`), `category`, MIME, extensions.
- `DcRow`: Validates `id`, `name`, `combining_class`, `bidi_class`, `category_type`, and parses the `aliases` DSL.
- `LayoutRow`: Validates contiguous regions, ensures `count == last_id - first_id + 1`, and validates partition boundaries.

Because `ctb_build_support` or `build.rs` can depend on `ctb_formats_utilities` (or a sub-crate `ctb_formats_core`), the **exact same DSL parser is used during build validation and at runtime**.

#### Step 3: Codegen for Constants and Global Graph IDs
In the CSVs, specify which entries emit constants:
- **Approach A (Convention-based)**: Any row with a non-empty `Ident` (in `formats.csv`) or marked with a column `ConstName` / `IsStandard` (in `dc_data.csv`) automatically generates:
  ```rust
  // Auto-generated in built/dc_constants.rs:
  pub const DC_NULL: u32 = 0;
  pub const GID_DC_NULL: u128 = 1_114_112; // DC_REGION_START + 0

  pub const DC_BEGIN_NUMBER: u32 = 6;
  pub const GID_DC_BEGIN_NUMBER: u128 = 1_114_118;

  pub const DC_BASE64_START: u32 = 127;
  pub const GID_BASE64_START: u128 = 1_114_239;
  ```
- **Approach B (Layout-Aware Formulas)**:
  Because `global-graph-layout.csv` defines `DC_REGION_START = 1_114_112` and `FORMAT_REGION_START = 2_228_224`, the codegen automatically computes `GID_*` constants using `dc_to_gid` and `format_to_gid` at generation time. No developer ever has to do manual arithmetic for global IDs again.

---

### 4. Build-Time Validation Checks to Enforce

1. **Identifier & Range Invariants**:
   - Ensure all IDs are non-negative and strictly unique within each table.
   - For `global-graph-layout.csv`: Ensure no overlapping regions; verify that each region's `Last ID - First ID + 1 == Count`.
   - Verify that all `Dc` short IDs fit within `0..=(DC_REGION_END - DC_REGION_START)`.
   - Verify that all `Format` short IDs fit within `0..=(FORMAT_REGION_END - FORMAT_REGION_START)`.
2. **Schema & Field Types**:
   - `Bidi class`: Must be one of `BN, ON, WS, ES, CS, EN, ET, B, L, R, AL, ...`.
   - `General Category`: Must be valid Unicode-style category (`Cc, Po, Sm, Sc, Pd, Nd, Zs, !Cx, ...`).
   - `Support Levels`: Must be an integer in `-1..=5` or blank (0).
   - `BaseFormat or Chain`: If non-empty, must refer to an existing `Format` ID or `Ident`.
3. **DSL & Cross-Reference Integrity**:
   - Parse all `Aliases` expressions:
     - `>Target`: Check that `Target` (e.g. `>32`) is a valid Dc ID or ASCII character.
     - `<equiv> A B`: Check that all referenced tokens (`240`, `239`) are existing Dc IDs.
     - `:[246 247] [^248 255]+ ~`: Parse the syntax grammar; check that every numeric token is a known Dc ID.
4. **Rust Ident Validity**:
   - Any `Ident` column intended for constant/enum generation must match `[a-zA-Z_][a-zA-Z0-9_]*` and not clash with Rust keywords.

---

### 5. Recommended Implementation Roadmap

1. **Data Separation**:
   - Fork `DcData.csv` from `src/formats/eite/data/` to `src/formats/utilities/data/dc_data.csv` (or `dctext/data/`).
   - Keep `eite/data/DcData.csv` frozen for legacy EITE test suites.
2. **Table Validator & DSL Parser in `ctb_formats_utilities`**:
   - Add a lightweight `validator` submodule that parses `formats.csv`, `dc_data.csv`, and `global-graph-layout.csv`.
   - Implement strict checks for uniqueness, region bounds, and DSL token existence.
   - Add a test `test_validate_all_data_tables()` in `ctb_formats_utilities` that runs on `cargo test` / CI.
3. **Codegen Generator in `ctb_build_support`**:
   - Similar to the existing [ipc_codegen.rs](file:///workspaces/ctoolbox/src/build_support/ipc_codegen.rs), create `data_table_codegen.rs` that reads the validated tables and emits:
     - Generated short & GID constants for flagged Dcs/Formats.
     - Static format descriptor lookup tables (replacing runtime CSV parsing overhead).
4. **Update Consumers**:
   - Update [dc_number.rs](file:///workspaces/ctoolbox/src/formats/dctext/dc_number.rs) and [global_graph_layout.rs](file:///workspaces/ctoolbox/src/storage/minimal/global_graph_layout.rs) to use the generated constants.
   - Add a fast command to `./lint` to ensure table formatting and validation stay strictly in sync during development.

Viewed dc_number.rs:27-48



I've started by forking dc_data.csv as suggested.

One significant DX issue with the current approach is that it's difficult to spot similar Dcs or formats - they're stored and edited in numerical order by ID (which is simply the order they were added in, because similar to Unicode, I never change the *meaning* of already-assigned Dcs, besides the one big one-time break of offsetting them all by the number of Unicode codepoints so that DcText files containing only characters available in Unicode are also valid UTF-8). Since they're purely in numerical order, it's inconvenient to browse or search, and would be very easy to add duplicates by accident. What do you think of having files by category, and I could sort them in any order. Those files would hold new Dc IDs (larger numbers), which (if left empty or marked "TBD" or something) would be assigned automatically at build time by incrementing from which the largest existing one was, and then they'd be persistent for the lifetime of the Dc. This applies to formats, too, which could get split files per "Category" (column D in the current CSV).

Another upcoming issue is that adding file magic to the CSV will get very clunky once I start moving it out of being inlined in the source - maybe that should go in a separate database. It looks like the file database uses a format like:

```
<snip>
```




Searched for "magic"
Ran command: `awk -F',' '$10 != "" {print $1, $2, $10}' src/formats/utilities/data/formats.csv | head -n 30`

### 1. Categorized Files & Flexible Sorting with Auto-Assigned IDs

Splitting both **Dcs** and **Formats** by category with **auto-assigned persistent IDs** is an excellent design choice for DX, maintainability, and scalability.

#### Why this solves the current bottlenecks
1. **Semantic Locality & Deduplication**:
   - Currently, browsing a 300+ row table sorted strictly by numerical ID means `Begin number` (Dc 6), `Positive` (Dc 10), and `Negative` (Dc 11) are interspersed with control codes, while math symbols (`Sm`) and operators are scattered all over.
   - With files per category (e.g., `dc/categories/math.csv`, `dc/categories/semantic.csv`, `formats/categories/compression.csv`, `formats/categories/encoding.csv`), you can group and sort related entries logically (alphabetically, by sub-type, or by semantic similarity). Finding prior art or spotting redundant proposals becomes immediate.
2. **Virtually Zero Git Conflicts**:
   - Contributors working on compression formats won't touch the file for image formats or character encoding formats.
3. **Frictionless ID Allocation (`AUTO` / `TBD`)**:
   - You don't have to manually look up "what was the last ID allocated across all files?"
   - You simply write `AUTO`, `TBD`, or leave the `ID` column blank in the relevant category file.
   - A build/tooling step scans all category files, calculates `max(existing_ids) + 1`, replaces `AUTO` with the concrete integer directly in the CSV source file, and formats the row.
   - The ID becomes permanent and immutable.

#### Proposed Directory Structure

```
src/formats/dctext/data/
├── dc_data.csv                    # (Or split into folder below)
└── categories/
    ├── controls.csv               # Cc
    ├── syntax_semantic.csv        # !Cx
    ├── math.csv                   # Sm, Nd
    ├── punctuation.csv            # Po, Pd, Ps, Pe
    ├── symbols.csv                # Sc, So
    └── language_el.csv            # EL Miscellaneous, EL Syntax

src/formats/utilities/data/
└── formats/
    ├── archive.csv
    ├── compression.csv
    ├── document.csv
    ├── encoding.csv
    ├── programming.csv
    └── serialization.csv
```

#### How the `AUTO` ID Allocator & Normalizer Works

```
                        [ Developer edits `math.csv` ]
                        (Inserts row with ID: "AUTO" or "")
                                      │
                                      ▼
                        [ `./lint --assign-ids` or `build.rs` ]
                                      │
      ┌───────────────────────────────┴───────────────────────────────┐
      ▼                                                               ▼
1. Scan all files in categories/                                2. Validate:
   Find global `max_id` for Dcs / Formats.                         • Global ID uniqueness
   For each "AUTO" / "" row:                                       • Valid Rust Ident
     Assign ID = max_id + 1                                        • Schema check
     Update max_id                                                 • Non-overlapping ranges
                                                                      │
                                                                      ▼
3. Write back concrete ID to the source file                    4. Generate Rust Consts &
   and re-format CSV with canonical quotes                         Static Slices in `built/`
```

---

### 2. File Magic: Separating into a Dedicated Magic Database

Moving file magic out of a single CSV column into a dedicated database using the standard Unix `magic(5)` syntax is the right architectural choice.

#### Why the single CSV `Magic` column breaks down
- Real-world file detection rarely consists of a single static byte prefix:
  - Needs **variable offsets** (e.g., offset 0, 4, 512 for tar, 0x3C for PE header pointer).
  - Needs **type definitions and endianness** (`belong`, `lelong`, `beshort`, `string`, `pstring`, `search`).
  - Needs **masks and bitwise tests** (`& 0xFFFFFF00`).
  - Needs **hierarchical / nested conditions** (e.g., `>4 string ...` to differentiate zip-based formats like `.docx`, `.jar`, `.apk`).
  - Many formats have **multiple alternative magics** (e.g. different versions or container wrappers).

#### Proposed Architecture for Magic

1. **Magic Definitions Directory**:
   Store magic definitions in `src/formats/utilities/data/magic/` using standard `file(1)` syntax, split cleanly by category or format (e.g. `magic/archive`, `magic/audio`, `magic/maple`, `magic/executables`).
2. **Direct Link to CTB Formats**:
   Annotate rules with standard or custom directives linking them directly to our `FormatInfo` `ID` or `Ident`:
   ```magic
<snip>
   ```
3. **Benefits**:
   - **Upstream Compatibility**: You can import, diff, and synchronize definitions directly from TrID XML definitions, FreeDesktop `shared-mime-info`, or the standard OpenBSD/Linux `file` magic tree.
   - **Compiled Fast Matcher**: At build-time or runtime initialization, `ctb_formats_utilities` parses these rules into an optimized byte-prefix trie / decision tree matcher.
   - **Clean CSV Registry**: `formats.csv` (or category files) stays focused on high-level metadata: ID, Ident, Label, Category, Layer, MIME, and Support levels.

---

### 3. Summary of Recommended Next Steps

1. **Category Splitting**:
   - For Formats: Split `formats.csv` into `src/formats/utilities/data/formats/*.csv` by category.
   - For Dcs: Split modern `dc_data.csv` into `src/formats/dctext/data/categories/*.csv`.
2. **ID Assignment & Linter Tool**:
   - Add an ID allocator in `ctb_build_support` that scans the category files, replaces `AUTO`/blank IDs in-place with monotonically increasing persistent IDs, and verifies global uniqueness across files.
3. **Magic Tree**:
   - Create `src/formats/utilities/data/magic/` using standard `magic(5)` format with `!:ctb-ident` / `!:ctb-format` annotations, decoupling complex binary signatures from the tabular format registry.