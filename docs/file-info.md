# File Information & Format Architecture: Master Checklist

## Implementation Progress Snapshot
- **Core Format Specification DSL & Parser:** Completed (`src/formats/dcdata/format_spec/`)
- **Prefix Dc Stream Encoder / Decoder:** Completed (`dc_stream.rs`)
- **Data Migration to `@chain(...)`, `@implies(...)` & `@based_on(...)`:** Completed in CSVs and column spec parser (`src/formats/dcdata/column_spec.rs`)
- **Declarative File Type Detection Engine & Universal Source Trait:** In Progress (Prototype, source abstraction, format catalog, and confidence tiers completed; modular refactoring and full `file` parity underway)
- **Graph Triples & Relation Predicates:** Pending
- **Lossless Archive Format Model:** Pending
- **Parametric Formats (EITE Base Numerals & Line Conventions):** Pending

---

### Phase 1: Format Identity, Ontological Boundaries & Data Cleanup
- [x] **Global Graph Identity Authority:** Establish global graph ID as authoritative (short Dcs offset 1114112, format-local offset 2228224; e.g., `f542` = `l2228766`).
- [x] **Preserve Existing Dc Identities:** Ensure existing Dc numbers retain their semantic identity without reassignments or destructive merges.
- [x] **Clean Up Math Dimension Equivalences:** Remove false `<equiv>` claims between fixed-dimensional (`Point1D`) and variable-dimensional formats.
- [x] **Relocate Math Origin/Axis/Ordering Metadata:** Move invalid concatenated `<semantic>` decompositions into descriptive metadata pending graph predicates.
- [x] **Line Convention Disambiguation:** Differentiate terminated-line vs. separated-line identities across character encodings (ASCII, EBCDIC NEL) without silent heuristic conflation.
- [x] **Calendar Identity Boundary:** Preserve Dc 9 strictly as the Gregorian calendar identifier without merging or assuming equivalence with other calendar concepts.

### Phase 2: Format Specification DSL & Dc Token Stream
- [x] **Recursive AST Design:** Implement `FormatExpr` AST supporting `TypeRef`, `NamedType`, `Union` (`&`, Dc 300), `Intersection` (`|`, Dc 516), `Transform` (`:`, Dc 301), `Convert` (`>`, Dc 302), and `Reinterpret` (`!`, Dc 303) in `src/formats/dcdata/format_spec/ast.rs`.
- [x] **Format Expression Parser:** Implement recursive descent parser supporting infix notation, prefix Polish notation, explicit grouping, and `@chain(...)` wrapping in `parser.rs`.
- [x] **Semantic Validator:** Enforce AST depth limit (32), node count limit (256), valid Dc ranges, and registered transformation target restrictions in `validator.rs`.
- [x] **Expression Formatter:** Format AST into standard readable infix strings in `formatter.rs`.
- [x] **Bidirectional Dc Token Stream:** Implement prefix Polish notation encoder/decoder with tail-position optimization and delimiter `299` disambiguation in `dc_stream.rs`.

### Phase 3: CSV Format Data Migration
- [x] **Unified Column Specification Parser:** Add unified parsing for column 6 (formats) and column 9 (characters) supporting `@chain(...)`, `@implies(...)`, and `@based_on(...)` directives (`src/formats/dcdata/column_spec.rs`).
- [x] **Disallow Ambiguous Bare Identifiers:** Reject bare format references in format CSVs to ensure unambiguous expression parsing.
- [x] **CSV Data Migration:** Convert existing entries to valid `@chain(...)`, `@implies(...)`, and `@based_on(...)` directives across format CSV files (`src/formats/dcdata/data/`).
- [x] **Format Details Integration:** Expose validated `format_spec: Option<FormatExpr>` on `FormatDetails` and `DcDef`.

### Phase 4: Syntax Framing, Grammar & Safe Evaluator
- [x] **Syntactic Roles Specification:** Define distinct named-type roles for `string`, `identifier`, `value`, and `statement` in named-type definitions.
- [x] **Framing & Escaping Rules:** Document literal delimiter syntax (`260 <header> <payload> 261`) with escape character `255` protecting terminators and inner escapes.
- [x] **Strict Grammar Matcher:** Upgrade syntax matcher from token placeholder consumption to full, namespace-safe, bounded recursive rule expansion.
- [x] **Strict Framing Validation:** Reject truncated structures or dangling escapes instead of falling back to warning recovery during execution validation.
- [x] **Non-Evaluating Parser & Safe Evaluator:** Ensure parsing, indexing, and deserialization never execute quoted payloads; implement explicit execution contexts.
- [x] **Bit-for-Bit Representation Preservation:** Preserve original escaped spellings separately where round-trip verbatim reconstruction is required.
- [x] **Extended Grammar Constructs:** Formalize explicit syntax for routine arguments, list/map element framing, and nested executable blocks before inclusion in `statement` / `value`.
- [x] **Typed Literal Headers:** Extend literal type headers beyond String marker 264.

### Phase 5: Declarative File Type Detection Engine & Parity with `file`/polyfile/DROID
- [x] **Basic Multi-Signal Detection Prototype:** Initial prototype combining static magic byte matching (`MAGIC_REGISTRY`), preliminary extension rules (`EXTENSION_REGISTRY`), and `FormatCategory` domain filtering (`ctb_formats_utilities::detection`).
- [x] **Probabilistic Multipart Extension Parsing & Candidate Chains; old/filedetect/ ports:**
  - Note: Reusing/porting algorithms from the packages in old/filedetect/ (which should all be compatibly licensed), or using standard Rust crates, is encouraged, rather than reinventing things wholesale. We'll want to reuse existing databases, perhaps mapping them to Dcs, so that the data maintained directly in this crate (`src/formats/dcdata/data/categories/formats/` and as-yet-unimplemented `src/formats/dcdata/data/categories/formats/magic/`) can be relatively limited.
  - [x] Replace hardcoded extension lists with data-driven extension resolution sourced directly from the Dc format dataset (preferred extensions, alternate extensions, and MIME mappings).
  - [x] Support probabilistic multi-candidate extension parsing (`Vec<ProbableFormatChain>` or `Vec<FormatCandidate>`) rather than collapsing ambiguities into a single deterministic `FormatChain`.
  - [x] Account for ambiguous extensions (e.g., `.as` for ActionScript vs. AppleSingle vs. AngelScript; `.m` for Objective-C vs. MATLAB vs. Mathematica; `.doc` for Word vs. FrameMaker vs. plain documentation) with ranked likelihood.
  - [x] Add format prevalence/commonness metadata (frequency weights overall and relative likelihood among formats sharing the same extension).
  - [x] Add platform association and environment priors (e.g., AppleSingle / `.as` or `.app` on macOS / classic Mac OS; `.exe` / `.bat` on Windows; `.sh` on POSIX; web/cross-platform formats).
  - [x] Support recursive layer peeling that preserves branch probabilities across stages (e.g., `archive.as.gz` peels outer Gzip with high confidence, while the peeled inner `.as` branches into ranked ActionScript vs. AppleSingle candidates).
- [x] **Unified Source Interface via `ctb_io_file` (`FileEntity` & `PayloadSource`):**
  - [x] Ground detection `Source` abstraction directly in the universal file representation from `src/io/file/` (`FileEntity` and `PayloadSource` trait).
  - [x] Expose bounded range reads, known length, and sparse extent maps (`Extent::Data` / `Extent::Hole`) without eager in-memory buffering.
  - [x] Support in-memory byte slices (`MemoryPayloadSource`), disk files (`DiskPayloadSource`), and archive member streams through uniform `PayloadSource` handles.
  - [x] Implement bounded immediate-child probing for directory packages / application bundles (`FileEntityKind::Bundle` or `Directory`, e.g., macOS `.app`) using `SandboxedDir` / `read_dir_safe`, restricted to bounded depth (1-2) and strict entry/byte quotas without following arbitrary symlinks.
  - [x] Leverage attached streams / forks (`AttachedStream`, e.g., AppleDouble `._` companion metadata and resource forks) as rich detection signals.
  - [x] Explicitly distinguish between insufficient data / quota exhaustion, read errors, and true negative matches.
- [x] **Priority/Weight Mechanics & MIME Inheritance:**
  - [x] Implement an explicit 0–100 priority/weight scale for resolving rule conflicts.
  - [x] Implement MIME inheritance graphs (`sub-class-of`) and parent/child candidate subsumption in `resolve_candidate_conflicts`.
  - [x] Port point-based evidence weighting to produce calibrated multi-candidate confidence tiers (`HighestConfidence`, `Strong`, `Moderate`, `Weak/Heuristic`, `Conflicted`).

- [x] **Sub-Phase 5A: Modularization of `detection.rs` (Decoupling Monolith into Clean Submodules):**
  - [x] Extract `src/formats/utilities/detection/types.rs`: isolate `ConfidenceTier`, `DetectionHint`, `DetectionQuotaType`, `DetectionOutcome`, `DetectionEvidence`, `DetectionCandidate`, and `DetectionReport`.
  - [x] Extract `src/formats/utilities/detection/source.rs`: isolate `DetectionSource` trait and implementations for byte slices (`&[u8]`), `Vec<u8>`, `Cursor<T>`, and `EmptySource`.
  - [x] Extract `src/formats/utilities/detection/platform.rs`: isolate `is_os_match`, `current_platform_os`, and OS platform prior scoring logic.
  - [x] Extract `src/formats/utilities/detection/conflict.rs`: isolate `resolve_candidate_conflicts`, ancestor subsumption, MIME specialization boosts, and multi-candidate demotion.
  - [x] Extract `src/formats/utilities/detection/chain.rs`: isolate `ProbableFormatChain`, `FormatChain`, `guess_format_chains`, and `parse_format_chain`.
  - [x] Refactor `src/formats/utilities/detection.rs` to serve as a clean, concise pipeline coordinator (~200-250 lines) re-exporting all submodules for backward compatibility.

- [ ] **Sub-Phase 5B: Text & Character Encoding Detection Subsystem (`detection/text.rs` ported from `ascmagic.c` & `encoding.c`):**
  - [ ] Character set identification engine:
    - [ ] UTF-8 detection (with and without BOM, strict sequence validation).
    - [ ] UTF-16LE and UTF-16BE detection (with and without BOM, surrogate pair checking).
    - [ ] UTF-32LE and UTF-32BE detection (with and without BOM).
    - [ ] 7-bit ASCII text validation (printable ASCII + standard whitespace / C0 controls).
    - [ ] ISO-8859 series detection (ISO-8859-1 through ISO-8859-15).
    - [ ] Non-ISO 8-bit extended ASCII encodings (CP437, MacRoman, Windows-1252) using existing `CharEncoding`.
    - [ ] EBCDIC detection (standard IBM US / international EBCDIC codepages).
  - [ ] Text line convention & layout profiling:
    - [ ] Line terminator counting and convention profiling (POSIX LF, Classic Mac CR, Windows CRLF, EBCDIC NEL) via `LineEndingKind`.
    - [ ] Line length analysis and long line tracking (>300 characters).
    - [ ] Control character and ANSI escape sequence detection.
  - [ ] Text language & syntax heuristics:
    - [ ] Shebang (`#!`) interpreter extraction (e.g. `#!/bin/sh`, `#!/usr/bin/env python3`, `#!/usr/bin/perl`).
    - [ ] Programming language heuristics (C/C++, Python, Shell, Perl, Ruby, Lisp, Assembler).
    - [ ] Markup heuristics (HTML tags, XML declaration `<?xml`, SGML, roff/troff commands `.TH`/`.so`, TeX `\documentclass`).
    - [ ] Mail and news header heuristics (RFC 822 `From:`, `Subject:`, `Date:`).
  - [ ] Pipeline integration:
    - [ ] Integrate text detection pass in `guess_format_report` as fallback when binary magic does not match, eliminating false `TrueNegative` results on plain text files.
    - [ ] Output character set encoding and line ending style in candidate evidence (`DetectionEvidence::Encoding`, `DetectionEvidence::TextProperties`).

- [ ] **Sub-Phase 5C: Hierarchical Magic Engine Feature Parity & Magdir Compilation (`magic_parser.rs` & `magic.rs`):**
  - [ ] Relative offset support: parse and evaluate relative offsets (`&<offset>`) relative to the end of the previous match level.
  - [ ] Indirect offset pointer dereferencing: parse and evaluate `(<offset>.<type>+<adjustment>)` (e.g., `(0x3c.l)` for MS-DOS PE headers, `(&4.s)` relative indirect pointers).
  - [ ] Comprehensive data type parity:
    - [ ] Fix 64-bit quad integers (`quad`, `lequad`, `bequad`, `ulequad`, `ubequad`) in `MagicTest` (distinguish from 32-bit `long`).
    - [ ] Date types (`date`, `ldate`, `qdate`, `medate`, `bedate`, `ledate`).
    - [ ] Regex patterns (`regex` with flags `/c`, `/s`, `/l`).
    - [ ] Pascal strings (`pstring` with length variants `/B`, `/H`, `/h`, `/L`, `/l`, `/J`).
    - [ ] String matching flags (`/c` case-insensitive, `/b` blank-insensitive, `/t` trim whitespace, `/W` compact whitespace).
  - [ ] Formatted description strings:
    - [ ] Parse printf-style format specifiers (`%s`, `%d`, `%u`, `%x`, etc.) in rule descriptions and format extracted values dynamically.
    - [ ] Handle backspace `\b` space-suppression and punctuation formatting in child rules.
  - [ ] Macro subroutines: parse and execute named rule templates (`name` declaration and `use` invocation).
  - [ ] Ingestion & compilation pipeline for full upstream `Magdir/` database:
    - [ ] Compile all 359 Magdir files (`src/formats/dcdata/data/magic/upstream/magic/Magdir/`, >15,000 rules) into an offline precompiled binary cache or code-generated lookup tables.
    - [ ] Map upstream MIME types and descriptions systematically to authoritative Dc format IDs.

- [ ] **Sub-Phase 5D: Specialized Deep Parsers & Container Inspection (`detection/container.rs`):**
  - [ ] TAR archive verification: validate octal checksums across the 512-byte header block (V7, ustar, GNU, pax) without full extraction (`is_tar.c`).
  - [ ] Fast JSON heuristic parser: state-machine scanner validating balanced objects/arrays/literals without eager allocation (`is_json.c`).
  - [ ] Tabular CSV/TSV validator: column consistency scoring across sample lines (`is_csv.c`).
  - [ ] OLE2 Compound Document File (CDF) inspector: traverse internal directory streams to classify Word (`.doc`), Excel (`.xls`), PowerPoint (`.ppt`), and MSI installers (`readcdf.c`).
  - [ ] ELF binary inspector: parse ELF header, 32/64-bit, endianness, machine architecture, dynamic linker interpreter (`/lib64/ld-linux-x86-64.so.2`), OS ABI, and notes (`readelf.c`).
  - [ ] Transparent payload decompression: probe inside gzip, bzip2, xz, and zstd byte streams to inspect inner payload magic (`compress.c`).

- [ ] **Sub-Phase 5E: Filesystem & Inode Special File Detection (`detection/special.rs` from `fsmagic.c`):**
  - [ ] Explicit detection and candidate generation for 0-byte `Empty` files and 1-3 byte `VeryShort` files.
  - [ ] Reporting special filesystem entities: directory packages/bundles, symlinks, FIFOs, sockets, block and character devices.

- [ ] **Sub-Phase 5F: DROID / PRONOM Signatures & PolyFile Attribution:**
  - [ ] Container signatures for ZIP-based formats (DOCX, XLSX, PPTX, EPUB, JAR, APK, ODF) via central directory inspection without extraction.
  - [ ] Dual-anchored BOF / EOF signatures with variable offset windows.
  - [ ] PolyFile-style byte-range attribution and polyglot container detection.
  - [ ] Nested file parsing like and/or ported from polyfile.
  - [ ] Integrate DROID database.

### Phase 6: Parameterized Formats & Comprehensive Format Catalog
- [ ] **Parametric Application Syntax:** Design and implement typed application expressions (e.g., `base-numeral(radix=16, alphabet=f359)`) using BaseNNumeral (`f350`) and Base (`f354`).
- [ ] **EITE Number Base Catalog:** Inventory and register all supported EITE number bases, alphabets, digit orderings, case rules, and padding conventions in the format dataset.
- [ ] **Line-Ending Formats Inventory:** Comprehensively specify terminated vs. separated conventions across line-ending formats (CRLF, LF, CR, NEL).
- [ ] **Parameterized Equivalence Migration:** Replace temporary `[number:'...']` syntax placeholders in `f371`–`f375` `<equiv>` entries with parameter bindings.

### Phase 7: Semantic Graph Triples & Relation Predicates
- [ ] **Relation Instance Model:** Represent semantic graph relations with dedicated node IDs carrying `(subject, predicate, object)` fields plus qualified statement attachments.
- [ ] **Predicate Allocation:** Formally define and allocate Dcs for relation predicates (axes, origins, component ordering, subtypes, conversions) once domains and cardinality are fixed.
- [ ] **Migrate Math Metadata from Prose:** Convert interim math coordinate/vector metadata from descriptions into structured graph relation statements.
- [ ] **Split Overloaded Base/Chain/Syntax:** Fully separate subtype inheritance, conversion pipelines, and payload byte layouts into distinct graph edges.
- [ ] **Dc Document Triple Indexing:** Build pipeline to compile the Dc CSV dataset into native Dc semantic triple documents, retaining CSVs solely for bootstrapping/regeneration.

### Phase 8: Lossless Archive & Container Representation
- [ ] **Lossless Archive Data Model:** Design a Dc document schema capable of losslessly representing archive contents (tar/zip containers).
- [ ] **Preserve Low-Level Metadata:** Retain raw byte paths, nanosecond timestamp precision, sparse extents, permission bits, alternate data streams, and unrecognized header fields.
- [ ] **Reconstruction vs. Round-Trip:** Distinguish logical content extraction from bit-for-bit archive reconstruction (header padding, duplicate entries, compression parameters).
- [ ] **Archive Detection without Unpacking:** Ensure container classification and package detection operate via bounded header inspection without full archive extraction.

### Phase 9: Metadata, CLI Nicknames & Public API
- [ ] **Format Nicknames Registry:** Define authoritative CLI nicknames and preferred UI display nicknames.
- [ ] **Extension & MIME Registry Integration:** Consolidate preferred extension lists, MIME types, Apple UTIs, and creator codes into the unified format lookup service.
- [ ] **Public Formats API:** Provide a clean, idiomatic formats query API for CLI tools and GUI components (in-memory, no filesystem requirement).

---

This repository currently has some support in formats/utilities for information about files and file formats. It uses data from the Dc database (which is in effect Unicode extended with additional characters, most of which serve to represent semantic data - Dcs are meant as an implementation-independent encoding for documents and semantic data).

The current formats/utilities implementation is not maintainable or robust, so I would like to rework it.

I have provided several other applications that implement file type detection for reference of their features and approaches for use in planning the data sets necessary for the replacement, in old/filedetect. They're probably not relevant for the immediate task, though, as the file type detection project is primarily background information for the current clean-up efforts.

The following types of features are relevant to the file detection:

- Detecting a file type given a file or array of bytes - it should return multiple candidates if relevant, with an indication of the confidence in its guesses.
  - It should primarily use the format of the file as its source.
  - This should optionally take into account file name patterns.
  - It should be able to work on a directory (to detect package formats like Mac .app), but needs to run quickly and not traverse the whole directory.
  - It should be able to work without a file system - other libraries should be able to call it as an in-memory utility. A common set of types for representing files, filesystem objects, and compressed archive entries will likely be relevant.
    - An upcoming project is going to be implementing support for representing and unpacking archives like tar files, including metadata.
- Providing file format "nicknames" for command-line utilities, and a most-preferred nickname for use in UIs.
- Providing a most-preferred extension.
- Providing Rust struct identifiers.
- Global graph ID references for file formats - these are the IDs in the current spreadsheet, with the offset to locate them in the formats graph block (see the graph layout documentation).
- Implementing a readable format description DSL that can also be represented in Dcs (=global graph IDs). I'd like you to plan out a syntax for this. Examples of the general idea (not sure about this syntax): `directory > tar > bz2` (a bz2 tarball), `bz2` (any bz2 stream), `((english > iso8859-1) ! utf8) > utf8` (English text encoded as latin1, misinterpreted/mojibaked as UTF-8, stored as UTF-8), `pan > (((macroman | altura-mac-to-win) > utf8) & CRLF & CSV) | (hexdump & xxd & utf8)` (I think this is the format that `ctoolbox pan2csv --encoding win-utf8 'example2 with lemurs.pan' | xxd` returns), `(pan > (json & utf8)) | jq['.prelude'] & json & utf8` (that last one is a lot more ambitious, but it would be really cool to be able to declaratively build a pipeline like that - for the moment it's blocked until I figure out how I'd like to represent it in Dcs). I have added Dcs 298 through 303 for semantic grouping and for operations "union" (which I used & for here), "transform" (which I used : for here), "convert" (which I used a > for here), and "transmute" (which I used a ! for here). Example: `((english > iso8859-1) ! utf8) > utf8` I'm thinking would be:
    - 302 conversion
    - 303 transmutation
    - 302 conversion
    - f15 English
    - f542 ISO 8859-1
    - 299 end associativity group
    - f0 UTF-8
    - 299 end associativity group
    - f0 UTF-8
- How "Syntax" and "Chain" work for formats is not currently clearly unambiguous. "Chain" is basically what I am trying to do with this syntax, but it is half-baked.
- The normal Syntax statements possibly should be used for formats.
- There is also <equiv> and <semantic> decompositions.
- The new Math formats are revealing this: Point1D, <approx>f505, <semantic>f505 f529 f537, :~ [number:Natural0] - this doens't really make sense. f505 can't be followed by f529 according to its syntax. <semantic> is basically acting as a chain here.
- Many of the formats already supported in this repository are represented in the format database and syntax, but some of the "custom formatting" ones are not - the base conversions from EITE in particular are very flexible and will need some sort of parametrized syntax.
- There are things that aren't strictly file formats but in some ways work like them, like virtual filesystems (NFS, etc) and operating systems - they're data structures or protocols. These are also represented in the formats data.
- There's various metadata that applies to formats: magic, creator codes, filesystem timestamp resolution, etc. Some don't apply to all formats.
- Some format records represent families of formats, rather than individual well-defined data types.
- It generally does not provide a clear and coherent ontology.
- There is a mix of overlap with Dcs. Dc 9, in semantic.csv, is actually a calendar format, for instance, and there is not always a clear conceptual boundary.
- I would like to revise the existing file type detection to be declarative and data-driven using the Dc formats data.

The number base formats (alphabets and so on) supported by ctoolbox are not yet really comprehensively represented in formats.csv, nor are line ending formats.

Current and near-term goals include allowing Dc documents to be used to losslessly represent file archives (similar to tar), to add Dcs to encode graph relations (allowing the entire Dc dataset to be parsed into and indexed as semantic triples consisting of Dc statements - in other words, each Dc would become a Dc document, and then those would be the source of Dc data used by the application, and the CSV would only be used to regenerate them), and to use the Dc data and some libmagic-style data to make file format detection data-driven rather than having it all embedded in code.

To illustrate what I mean about semantic triples, a graph (with non-Dc node IDs) could look like:

-     1. type: type
-     2. type: relationship type
-     3. type: relationship
-     4. type: entity
-     5. type: place
-     6. type: year
-     7. 4(entity): x
-     8. 4: y
-     9. 5: z
-     10. 6: 20xx
-     11. 6: 20yy
-     12. 2: visited
-     13. 2: began
-     14. 2: ended
-     15. 12: 7 8
-     16. 12: 7 9
-     17. 13: 16 10
-     18. 14: 16 11

In short, there's significant confusion of thought and data here. I would like you to, focusing primarily on the Dc data, review the Dc data and work towards addressing this. Please begin by tackling low-hanging fruit in the data files.

Note that the meaning of any given Dc number that exists as of this prompt may not be changed, though their names, syntax rules, etc. can be refined. In other words, existing Dcs should retain their identity.

## Initial Data Cleanup (2026-09-17)

This first pass changes declarations and descriptions, not Dc identities or
runtime file detection. No IDs are allocated, reassigned, or merged.

- Fixed-dimensional points and vectors retain their base relationships and
  payload syntax, but no longer claim `<equiv>` to variable-dimensional formats.
  The latter require a dimension count that the former do not carry.
- Math origins, axes, ordering, and component types are described as metadata,
  with their format IDs preserved in the descriptions. They are no longer
  concatenated into `<semantic>` decompositions that cannot satisfy the referenced
  formats' payload syntax. This is an interim location pending graph predicates,
  not a new prose-based machine-readable relationship format.
- The existing terminated-line and separated-line identities remain distinct.
  Descriptions specify their character sequences and final-line interpretation.
  An OS association is not proof of either convention. The same bytes may admit
  both interpretations; detection must not silently choose one. Character
  sequences are independent of byte encoding, especially for EBCDIC NEL.
- Dc 300 still means that both descriptions apply. Its historical name, "Type
  union", does not make it a choice between alternatives. Dcs 301-303 describe
  operations, but do not authorize execution.

## Model Boundaries and Implementation Status

The following describes model boundaries and current implementation status for format identities, classifications, and compositions.

1. **Identity:** The global graph ID is authoritative. Existing short Dcs use
   offset 1114112; format-local IDs use offset 2228224. For example, `f542` is
   global ID 2228766, not short Dc 542. Names, aliases, category files, and Rust
   identifiers are mutable attributes, not identity keys. Keep Dc 9 as the
   Gregorian calendar identifier; relate overlapping calendar concepts explicitly
   rather than moving it to another number or assuming equivalence.
2. **Classification:** Distinguish concrete representations, format families,
   semantic types, encodings, transformations, parameter domains, and conventions.
   An OS, protocol, filesystem, or family can have format-like relationships
   without necessarily being a detectable byte format. Permit multiple roles;
   category-file membership alone should not determine capabilities.
3. **Payload syntax:** The existing syntax DSL describes the data following a
   typed marker. Keep it separate from a description of how representations
   compose. A base/related format is neither automatic payload inheritance nor
   proof of a valid decomposition. `<equiv>`, `<approx>`, and `<semantic>` do not
   substitute for subtype, axis, origin, or conversion relationships.
4. **Composition:** Use an expression tree of type references and operations.
   In source CSV files, column 6 (in format categories) and column 9 (in character
   categories) share a unified column specification parser (`src/formats/dcdata/column_spec.rs`).
   Consolidated directives avoid horizontal scrolling while ensuring unambiguous parsing:
   - `@chain(...)` (e.g., `@chain(((f15 > f542) ! f0) > f0)` or `@chain(f161 > f35)`):
     expresses format composition and conversion pipelines.
   - `@implies(...)` (e.g., `@implies(f271)`, `@implies(f390 & f395 & f587)`): expresses
     logical capability entailment and property guarantees using valid Dc shorthand syntax.
   - `@based_on(...)` (e.g., `@based_on(f580)`, `@based_on(f571)`): expresses historical
     ancestry, derivation lineage, or parent format inspiration without implying runtime capabilities.
     Bare format identifiers without a directive prefix are disallowed in format category files
     and trigger validation errors to prevent ambiguous parsing.
   - `@xref(...)` (e.g., `@xref(u22ee)`): expresses cross-references to code points or Dcs.
   - `@formalAliasCorrection("...")`, `@formalAliasControl("...")`, `@formalAliasAlternate("...")`,
     `@formalAliasFigment("...")`, `@formalAliasAbbreviation("...")`: normative formal name aliases
     from Unicode Standard Annex #44.
   - `@annotation("...")`: informative character notes and annotations from Unicode `NamesList.txt`.
   See [`src/formats/dcdata/data/README.columns.md`](file:///workspaces/ctoolbox/src/formats/dcdata/data/README.columns.md)
   for the complete column specification and directive catalog.
   The format specification DSL is implemented in `src/formats/dcdata/format_spec/`
   and integrated into the format CSV loader (`format_spec: Option<FormatExpr>` on
   `FormatDetails` and `DcDef`). Persisted chains accept numeric Dc references
   (shorthands like `f542`, `l2228766`, `0`) and explicitly registered stable
   named types only; aliases, Rust identifiers, and nicknames are never resolved
   implicitly. Semantic validation (`validate_format_expr`) enforces these constraints
   along with transformation target bounds and complexity limits during data loading.
   Migrate legacy `Chain (=)` entries individually. Do not reinterpret every `&`
   in the overloaded base column as an executable pipeline; `Utf8_Base64` (f110),
   for example, describes nested representations, not two independent constraints
   on the same bytes.
5. **Metadata and evidence:** Preferred extension, preferred nickname, aliases,
   MIME/UTI identifiers, creator/type codes, timestamp resolution, and detection
   rules are separate predicates. Missing metadata is unknown or inapplicable,
   not an empty value or an implicit default. Scope version-dependent properties
   to variants. Preserve existing extension preference order; define nickname
   preference explicitly before treating its existing order as authoritative.

For graph documents, predicate names above are placeholders, not newly allocated
Dcs. A relation needs a stable identity when other statements describe it: an
edge `(subject, predicate, object)` alone cannot distinguish two visits with
different start/end times. Represent relation instances with their own IDs and
subject/predicate/object fields, then attach qualifiers to those instances. Use
ordered lists for axes and component order; unordered triples must not erase
ordering or confuse alternative frames with simultaneous constraints. The CSV
importer can eventually generate these Dc documents; indexing should consume
those documents, not re-infer semantics from comments.

## Format Specification DSL and Dc Stream Encoding

Use distinct operators for distinct operations:

| Text | Existing Dc | Meaning |
| --- | --- | --- |
| `A & B` | 300 | Both constraints apply to the same data (Type union) |
| `A \| B` | 516 | Any of the alternative constraints apply (Type intersection) |
| `A : T` | 301 | Apply registered transformation T to representation A |
| `A > B` | 302 | Convert/encode A into representation B |
| `A ! B` | 303 | Reinterpret A's unchanged representation as B |
| `(A)` | 298 / 299 | Explicit grouping in infix syntax |

`>`/`!`/`:` bind more tightly than `&` and `|`, with left-to-right association
(`f10 > f20 > f30` parses as `(f10 > f20) > f30`). Association is not automatic
when mixing `&` or `|` with operational operators — explicit grouping is
required for an expression like `A & (B : C)` or `(A > B) & C`; expressions
without parentheses like `A & B > C` are rejected.

Neither `&` nor `|` takes precedence over the other. Parentheses are required
whenever `&` and `|` are mixed without grouping (e.g., `(A & B) | C` vs
`A & (B | C)`). Ambiguous expressions without explicit parenthesization are
rejected by the parser. Homogeneous chaining of `&` (`A & B & C`) and `|`
(`A | B | C`) is permitted and represented as n-ary nodes in the AST
(`FormatExpr::Union(Vec<FormatExpr>)`, `FormatExpr::Intersection(Vec<FormatExpr>)`).

The spelling `directory` in examples is schematic until it is explicitly bound to
a suitable descriptor; the existing file-kind Dc 359 must not be confused with
format f359 (BaseAlphabet).

Persisted chains accept numeric Dc references (as defined in
[README.shorthand.md](../src/formats/dcdata/data/README.shorthand.md)) and
explicitly registered stable named types from `README-named-types.csv` only.
Global IDs use lowercase `l`, not `@`: `l2228766` and `f542` identify the same Dc.
Bare integers denote short Dcs in this shorthand; uppercase `L` denotes a local
graph reference and is not interchangeable with lowercase `l`.
Labels, Rust identifiers, nicknames, and other aliases are never resolved
implicitly. There is currently no stability guarantee for keyword references or
other human-readable identifiers for Dcs; accepting aliases or attempting to
make them prematurely stable is error-prone, risking either invalidating
persisted expressions on changes or permanently baking in suboptimal names.
This keeps permanent naming commitments infrequent and deliberate. Using the
existing named-type registry is reasonable, provided it distinguishes a named
type definition from a name bound to one specific Dc; those are not
automatically interchangeable.

A named type must be valid in the operand’s context. Existing names such as
`number` and `string` describe broad types, not necessarily concrete encodings
or conversion targets. Tools may display current human-readable labels alongside
numeric references without making those labels part of the expression. Not every
graph node is a type: validate the referenced node's role. Numeric IDs provide
an unambiguous spelling even when a format has no registered named type.

Human-readable format names in the illustrative expressions below are schematic,
not implicitly accepted aliases. For example, the numeric spelling of the
mojibake expression is `((f15 > f542) ! f0) > f0`.

The `|` operator designates Dc 516 (Type intersection, expressing alternative
constraints where any of multiple types satisfy the description). It is not
interchangeable with Dc 300 (conjunction / union of descriptions), nor should it
be confused with Unix pipelining (which is represented by conversion/encoding
`>`). Use `:` for a named transformation, so the Altura subexpression can be
written `(macroman : altura-mac-to-win) > utf8`. A hex dump of that output
requires a registered hexdump transformation and parameters for the xxd
dialect, not a conjunction asserting that the original bytes are already a
hexdump. Query expressions such as `jq['.prelude']` remain outside this subset.
Neither parsing nor detection should invoke external commands.

### Canonical Dc Token Stream Encoding

The canonical Document Character (Dc) token stream representation uses a succinct
prefix (Polish) notation:

1. **Fixed-arity binary operators** (`301` `:`, `302` `>`, `303` `!`): Because
   their arity is strictly fixed at two operands (`op left right`), they are
   emitted directly in prefix order without requiring opening (`298`) or closing
   (`299`) group delimiters. For example, `((f15 > f542) ! f0) > f0` encodes
   directly as:

   ```text
   302 303 302 f15 f542 f0 f0
   ```

2. **Variable-arity operators** (`300` `&` / Type union, `516` `|` / Type intersection):
   Take an arbitrary number of child operands (`op child1 child2 ...`). A trailing
   `299` (or `)`) delimiter is emitted **only when necessary to disambiguate**
   where the child list terminates relative to subsequent operands in an enclosing
   expression (i.e. when not in tail position). When a variable-arity operator
   occupies the tail position of the expression, no trailing delimiter is required:

   - Non-tail position with subsequent operand: `1 & ((2 | 3 | 4) > 5)` encodes as:
     ```text
     300 1 302 516 2 3 4 299 5
     ```
     Here, `516 2 3 4 299` is the left operand of `302`, so `299` disambiguates
     where the intersection ends and the right operand `5` begins.
   - Tail position: `5 > (2 | 3 | 4)` encodes as:
     ```text
     302 5 516 2 3 4
     ```
     No closing `299` is required at the tail because the stream naturally ends.

3. **Dual Syntax and Backward Compatibility:**
   - The stream decoder (`decode_dc_stream`) supports both numeric Dc tokens
     (e.g., `300 1 302 ...`) and symbolic operator tokens in either infix or prefix syntax (e.g., `1 & ((2 | 3 | 4) > 5)` or `& 1 > | 2 3 4 ) 5`). Infix syntax is mildly preferred for the CSVs, while prefix syntax is preferred for Dc encoding of them.
   - The parser (`parse_format_expr`) transparently accepts both infix notation
     and prefix notation strings, with optional `@chain(...)` directive wrapping.

### Grammar and Payload Syntax Boundary

The recursive type-expression grammar, AST (`FormatExpr`), recursive parser,
formatter, semantic validator, and bidirectional Dc stream encoder/decoder are
implemented in `src/formats/dcdata/format_spec/`. Expressions are bounded by a
maximum tree depth of 32 (`MAX_FORMAT_EXPR_DEPTH`) and maximum node count of
256 (`MAX_FORMAT_EXPR_NODES`). Transformation target operands (`A : T`) are
constrained to registered transformation format IDs (`KNOWN_TRANSFORMATION_FORMAT_IDS`,
e.g. 19, 20, 323, 324) or the named types `type`/`format`.

Format specifications describe how representations compose, layer, and
transform (e.g., in `@chain(...)` declarations and multipart extension parsing).
They remain logically distinct from the inline payload syntax DSL used after
typed markers (e.g., `[type]`, `[262:]`), which specifies the intra-record byte
structure of specific format markers.

Parameterized descriptions could use a separate, typed application form such as
`base-numeral(radix=16, alphabet=alphabet-id)`. This spelling is illustrative and
has no assigned Dc application encoding yet. Reuse BaseNNumeral (f350), Base
(f354), and alphabet identities rather than assigning a format ID for every
combination. Validate alphabet ordering, digit uniqueness, radix compatibility,
case rules, signs, padding, and separators explicitly. The current f371-f375
`<equiv>` entries contain `[number:'...']` syntax placeholders; replacing those
with actual parameter bindings requires this model, not another textual chain.

## Character Sequences and Evaluation

Every Dc document and expression is represented by a character sequence. That
does not require every such sequence to have the syntax or evaluation behavior
of a **quoted literal**. Treat character sequences as the common representation;
keep quotation, reference, and invocation distinct within that representation.
Otherwise an arbitrary filename or quoted program could execute merely because
it contains the same Dcs as an invocation.

The named-type data now distinguishes these syntactic roles:

| Named type | Current bounded forms | Interpretation |
| --- | --- | --- |
| `string` | Literal 260 through 261, with a type header | Quoted payload; empty content is allowed |
| `identifier` | Name 270 through 271 | Nonempty name, not an implicit lookup |
| `value` | Literal, reference 276, or invocation 279 | One expression usable where a value is expected |
| `statement` | Value expression or assignment 269 | One expression in statement position |

These are syntax definitions, not a complete runtime type hierarchy. In
particular, `value` describes an expression that can produce a value, not only
the already evaluated result. A literal evaluates to its own represented value;
a reference resolves its binding; an invocation evaluates according to its
routine contract. A variable and a constant need not have different reference
framing: mutability belongs to the binding. Parsing, displaying, indexing, or
deserializing any of these forms must not itself perform evaluation.

A statement need not have a special return token merely to produce its result.
However, the result of evaluating an expression is **not** implicitly parsed and
evaluated again. If an invocation returns a string containing another invocation,
that result stays data. Executing quoted source would require a separate explicit
evaluation operation and execution context; this cleanup does not assign a Dc
or an existing routine to that operation. Nor does it invent a result value for
assignment or for routines currently described as returning void.

### Framing and Escaping

Literal syntax is `260 <type-header> <quoted-payload> 261`. Within that payload,
255 protects exactly the next Dc from syntactic interpretation. Thus `255 261`
represents payload Dc 261, and `255 255` represents payload Dc 255. Identifier
payloads use the same rule with terminator 271. A dangling escape or missing
terminator is invalid, not an implicit end of the string. Scanning must locate
the unescaped boundary before decoding escapes, then decode only once at that
quotation level.

Begin/end characters belonging to another construct are ordinary quoted
content. For example, parameter-end Dc 259 inside a literal cannot terminate
its enclosing parameter. Embedding a quoted literal inside another quotation
requires escaping its terminator and escape characters at the outer level;
the outer quote scanner does not recursively interpret the embedded literal.

An identifier's terminator supplies the boundary for reference 276 and the
currently declared invocation 279. Assignment consumes a complete identifier,
269, and one complete value expression. Parameter 258/259 now accepts a value
expression rather than only a literal. Optional-present 315 consequently also
has a bounded operand. No additional statement terminator is needed for these
forms. A sequence of multiple statements still requires a separate enclosing
grammar; `statement` does not mean "consume the rest of the document".

In the existing syntax DSL, `260:` is a required rule expansion, but `[260:]`
is optional. The named `string` and `identifier` rules therefore use the required
form. The literal type header uses `[262:]{1}` to require exactly one header
without triggering the parser's bare-colon/action ambiguity. An empty literal
payload must not be confused with a missing literal frame.

### Remaining Implementation Work

This cleanup fixes data definitions, not the evaluator. The current syntax
matcher consumes rule references and named constructs as token placeholders;
it does not yet expand them into a full document grammar. Its recovery mode can
also accept truncated structures with warnings. Neither behavior is suitable
for execution validation. Regression tests expand the relevant data rules in
test code and require complete, warning-free matches; they do not establish
runtime support for these expressions.

Before execution, implement namespace-safe Dc matching, bounded recursive rule
resolution, strict framing validation, and an evaluator that never executes
quoted payloads implicitly. Preserve original escaped spelling separately when
bit-for-bit reconstruction is needed. Routine arguments, other routine markers,
list/map element framing, and nested executable blocks need their own explicit
grammar before inclusion in `statement` or `value`. The existing literal type
header currently accepts only the built-in String marker 264; extending typed
literals is separate from treating quoted payloads as executable code.

## Format Detection and Next Work

- Format specifications are now parsed and validated for persisted `@chain(...)`
  entries via `ctb_formats_dcdata::format_spec`. The preliminary multi-signal
  detection and extension parser in `ctb_formats_utilities::detection` (`FormatChain`,
  `guess_format_id`, `MAGIC_REGISTRY`, `EXTENSION_REGISTRY`) is an early prototype
  using hardcoded lists and first-match selection. It must be replaced by a
  declarative, data-driven engine using the Dc format catalog and comprehensive
  rule datasets.
- Extension parsing into format chains cannot be deterministic or rigid: deriving
  formats from extensions must support probabilistic returns (`Vec<FormatCandidate>`).
  While `.html.gz` unambiguously decomposes to `Html > Gzip`, many extensions are
  inherently ambiguous (e.g., `.as` could be ActionScript, AppleSingle, or
  AngelScript; `.m` could be Objective-C, MATLAB, or Mathematica; `.doc` could be
  Word, FrameMaker, or text documentation). To preserve probabilities:
  - Formats data must include prevalence and relative frequency metadata (how common
    a format is globally, and its relative likelihood compared to other formats
    sharing that extension).
    - Platform affinity and environment priors must be supported (e.g., AppleSingle
    `.as` or `.app` on macOS / classic Mac OS; `.exe` or `.bat` on Windows; `.sh` on
    POSIX; web/cross-platform formats). Context passed by the caller (current OS,
    MIME hints, or domain flags) shifts candidate prior probabilities.
  - Multi-layer extension peeling must preserve branch probabilities across layers:
    in `archive.as.gz`, outer `gz` resolves to Gzip with high confidence, while the
    peeled inner `.as` branches into ranked ActionScript vs. AppleSingle candidates.
- Ground the detection source abstraction directly on the universal file
  architecture in `src/io/file/` (`ctb_io_file`):
  - Do not invent an ad-hoc, isolated `Source` trait in `formats`. Instead, leverage
    `FileEntity` and `PayloadSource` (`DiskPayloadSource`, `MemoryPayloadSource`),
    which already provide uniform, zero-copy, and streamed access across disk files,
    in-memory byte buffers, and archive member streams.
  - Expose bounded range reads, known logical size, and sparse extent awareness
    (`Extent::Data` / `Extent::Hole`) without eager intermediate buffering.
  - For directory packages and application bundles (`FileEntityKind::Bundle` or
    `Directory`, such as macOS `.app` or `.pages`), implement bounded child-node
    probing using `SandboxedDir` / `read_dir_safe`. Probes must adhere to strict entry
    count and shallow depth quotas (depth 1–2; e.g. checking for `Contents/Info.plist`),
    never executing arbitrary recursive traversals or following external symlinks.
  - Alternate data streams, resource forks, and companion metadata files (e.g.,
    AppleDouble `._` files via `AttachedStream`) represent primary detection
    signals and must be accessible to detection rules directly from `FileEntity`.
  - Explicitly distinguish insufficient evidence / budget exhaustion from true
    negative matches.
- Leverage license-compatible algorithms, data structures, and datasets from
  preexisting implementations in `old/filedetect`:
  - **libmagic (`old/filedetect/file/`, BSD-2-Clause):** Port the core `softmagic`
    interpreter to safe Rust (evaluating hierarchical `>` test trees, numerical/endian
    comparisons, bitmasks, indirect pointer offsets `FILE_INDIRECT`, relative
    offsets, and string/search/regex patterns). Create an offline ingestion/compilation
    tool to compile libmagic's extensive `Magdir/` rule database (15,000+ rules)
    into binary or Dc-keyed rule tables, mapping libmagic format outputs to
    authoritative Dc format IDs.
  - **Priority Weighting & MIME Inheritance:** Implement an explicit 0–100 priority/weight
    scale for resolving signature conflicts between general and specific formats, as well
    as MIME inheritance graphs (`sub-class-of`). Implementation must be strictly
    clean-room using our own format schema; do not reuse existing implementations
    of this idea.
  - **DROID / PRONOM (`old/filedetect/droid/`, BSD-3-Clause):** Adopt dual-anchored
    byte patterns: BOF (Beginning of File) and EOF (End of File) anchored signatures
    with variable offset windows. Adopt its declarative **container signature model**
    for inspecting internal entries within archive containers (ZIP, OLE2, ISO) to
    classify formats (such as DOCX, EPUB, JAR, APK) without full extraction.
  - **PolyFile (`old/filedetect/polyfile/`, Apache-2.0 / MIT):** Adopt
    point-weighted scoring to compute calibrated confidence percentages across
    candidates, and support polyglot/composite awareness where multiple valid format
    signatures legitimately co-exist in one stream.

### Detection Engine Modularization Architecture

[`detection.rs`](file:///workspaces/ctoolbox/src/formats/utilities/detection.rs) has accumulated multiple responsibilities (data transfer types, source adapters, platform matching, candidate subsumption, extension peeling, and primary orchestration), growing to over 1,500 lines. To keep the codebase maintainable and narrowly scoped, it will be refactored into focused submodules under `src/formats/utilities/detection/`:

```
src/formats/utilities/
├── detection.rs                      <-- Public facade & staged pipeline coordinator (~200 lines)
└── detection/
    ├── types.rs                      <-- ConfidenceTier, DetectionHint, DetectionOutcome,
    │                                     DetectionEvidence, DetectionCandidate, DetectionReport
    ├── source.rs                     <-- DetectionSource trait & slice, Vec, Cursor, EmptySource impls
    ├── platform.rs                   <-- is_os_match, current_platform_os, platform prior heuristics
    ├── conflict.rs                   <-- resolve_candidate_conflicts, MIME/format subsumption & demotion
    ├── chain.rs                      <-- ProbableFormatChain, FormatChain, multi-layer extension peeling
    ├── text.rs                       <-- ascmagic & encoding.c port: charsets, line terminators, shebang, syntax
    ├── special.rs                    <-- fsmagic port: empty file, directory, symlink, device, socket
    └── container.rs                  <-- Specialized deep inspectors: TAR, OLE2/CDF, ELF, JSON, CSV, uncompress
```

The top-level `src/formats/utilities/detection.rs` will re-export all public types and functions (`pub use detection::types::*;`, etc.) ensuring complete backward compatibility with existing callers across `src/io/file/entity.rs`, `src/io/file/payload.rs`, and `src/formats/compression/`.

### Subsystem Porting Details from BSD `file` (`old/filedetect/file/`)

#### 1. Text & Character Encoding Detection Subsystem (`detection/text.rs`)
Ported from [`ascmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/ascmagic.c) and [`encoding.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/encoding.c):
- **Character set identification:**
  - UTF-8 validation (checking strict UTF-8 byte sequences and optional BOM `0xEF, 0xBB, 0xBF`).
  - UTF-16LE / UTF-16BE validation (checking BOM `0xFF, 0xFE` / `0xFE, 0xFF`, surrogate pairs `0xD800`–`0xDFFF`, and absence of null bytes in odd/even positions).
  - UTF-32LE / UTF-32BE validation (checking 4-byte BOMs and valid Unicode scalar code point bounds).
  - 7-bit ASCII validation (all bytes in `0x07`–`0x0D` or `0x20`–`0x7E`).
  - ISO-8859 series heuristics (distinguishing printable Latin-1 / 8-bit European codes from control bytes `0x80`–`0x9F`).
  - Extended 8-bit DOS/Mac encodings (using existing `CharEncoding::Cp437` and `CharEncoding::MacRoman`).
  - EBCDIC detection (checking frequency of EBCDIC space `0x40` and common alphanumeric patterns).
- **Line ending profiling & text geometry:**
  - Count occurrences of `\n` (LF), `\r` (CR), `\r\n` (CRLF), and `\u{0085}` (NEL) to determine the predominant `LineEndingKind` and whether lines are terminated or separated.
  - Track maximum line length; flag files with very long lines (>300 characters, indicating minified JS/JSON or embedded data).
  - Track control codes, ANSI terminal escapes (`\x1b`), and backspaces (`\b`).
- **Language & shebang heuristics:**
  - Shebang parser: scan initial line for `#!` and extract interpreter command (e.g., `/bin/sh`, `/usr/bin/env python3`, `/usr/bin/perl`, `/bin/bash`).
  - Syntax patterns: identify C source (`#include`, `/*`), Python (`def `, `import `, `class `), Shell (`if [`, `case `, `then`), HTML (`<!DOCTYPE html`, `<html`), XML (`<?xml`), roff/man pages (`.TH `, `.so `, `.\"`), and RFC 822 mail headers (`From:`, `Subject:`).
- **Pipeline fallback:**
  - When binary magic yields no matches, `guess_format_report` invokes `detection/text.rs` before returning `TrueNegative`. A script like `#!/bin/sh\necho hi` without an extension is identified as `FormatId::Sh` / `FormatId::Ascii` rather than an unknown binary.

#### 2. Hierarchical Magic Engine Parity (`magic_parser.rs` & `magic.rs`)
Ported from [`softmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/softmagic.c) and [`apprentice.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/apprentice.c):
- **Relative offsets (`&<offset>`):**
  - In `file`, `>&0 string ...` evaluates an offset relative to the end of the byte sequence matched by the parent test level.
  - `magic.rs` must track match end positions through the evaluation context and evaluate relative offsets accordingly.
- **Indirect offsets (`(<offset>.<type>+<adjustment>)`):**
  - Evaluates pointer dereferencing in headers (e.g. `(0x3c.l)` reads a 32-bit LE pointer at offset 60 to locate the PE header in an MS-DOS stub).
  - Supports indirect offsets with sign, endianness, and arithmetic adjustments (`+`, `-`, `*`).
- **Data type parity:**
  - Fix 64-bit integer tests (`quad`, `lequad`, `bequad`) so they operate on 64-bit words rather than truncating to 32 bits.
  - Implement date types (`date`, `ldate`, `medate`, `bedate`, `ledate`).
  - Implement regex patterns (`regex` with search bounds).
  - Implement Pascal strings (`pstring` with length variants `/B`, `/H`, `/h`, `/L`, `/l`, `/J`).
  - Implement string modifier flags (`/c` case-insensitive, `/b` ignore whitespace, `/t` trim, `/W` compact).
- **Printf format string interpolation:**
  - Support `%s`, `%d`, `%u`, `%x`, `%o` in description strings to format matched integer, string, or date values dynamically (e.g. `version %d.%d`).
  - Support backspace `\b` space suppression when concatenating child descriptions.
- **Full Magdir database compilation:**
  - Provide an ingestion tool to precompile all 359 files in `src/formats/dcdata/data/magic/upstream/magic/Magdir/` into a binary lookup structure, eliminating runtime string parsing while activating the complete 15,000+ rule catalog.

#### 3. Specialized Deep Parsers & Container Inspection (`detection/container.rs`)
Ported from specialized C inspection routines in `file`:
- **TAR archive verification ([`is_tar.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/is_tar.c)):**
  - Verify octal checksum at byte offset 148 across the 512-byte header block.
  - Distinguish POSIX ustar, GNU tar, old V7 tar, and star variants.
- **Fast JSON parser ([`is_json.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/is_json.c)):**
  - Bounded state-machine scanner that skips leading whitespace and verifies valid top-level object `{`, array `[`, or literal structures.
- **CSV/TSV consistency scoring ([`is_csv.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/is_csv.c)):**
  - Sample initial lines, count candidate delimiters (`,`, `\t`, `;`, `|`), and check column-count consistency across rows with quote escaping.
- **OLE2 Compound Document File (CDF) inspector ([`readcdf.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/readcdf.c)):**
  - Parse OLE header and traverse internal directory sector entries without full extraction.
  - Differentiate Word Document (`.doc`), Excel Spreadsheet (`.xls`), PowerPoint Presentation (`.ppt`), and Windows Installer (`.msi`).
- **ELF binary inspector ([`readelf.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/readelf.c)):**
  - Parse ELF header for machine architecture (x86, x86-64, ARM, RISC-V, etc.), 32/64-bit class, endianness, OS ABI, dynamic linker interpreter path, and `.note` sections.
- **Transparent payload decompression ([`compress.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/compress.c)):**
  - Decompress the first 4–8 KB of gzip, bzip2, xz, and zstd byte streams to inspect the encapsulated payload format, enabling reporting of outer compression and inner payload (e.g. `Tar > Gzip`).

#### 4. Filesystem & Inode Special File Magic (`detection/special.rs`)
Ported from [`fsmagic.c`](file:///workspaces/ctoolbox/old/filedetect/file/src/fsmagic.c):
- Detect 0-byte streams and emit `Empty` file format candidate.
- Detect 1–3 byte non-matching streams and emit `VeryShort` format candidate.
- Report special filesystem entities: directory packages (macOS `.app`), symlinks, FIFOs / named pipes, UNIX domain sockets, block special devices, and character devices.

### Staged Detection Pipeline Execution Order

When `guess_format_report` executes, it will process candidate signals through the following staged order:

1. **Filesystem & Inode Probing (`special.rs`):** Check for empty streams, very short streams, directory bundles, or special inode types.
2. **Attached Stream & Fork Probing:** Check AppleDouble `._` files, resource fork type codes (`extract_resource_fork_type_codes`), and creator codes.
3. **Static Fast Magic Matching (`MAGIC_REGISTRY`):** Fast byte matching for common binary signatures.
4. **Compiled Hierarchical Magic Rules (`COMPILED_MAGIC_RULES`):** Evaluate hierarchical test trees with relative/indirect offsets and value interpolation.
5. **Specialized Container & Archive Inspection (`container.rs`):** Evaluate procedural inspectors (TAR checksum, OLE2 CDF streams, ELF headers, JSON/CSV, bounded decompression).
6. **Text & Character Encoding Fallback (`text.rs`):** If binary checks yield no matches, evaluate text encoding (UTF-8, UTF-16, ASCII, ISO-8859), line endings, shebang interpreter, and language syntax.
7. **Extension Hints & Platform Priors (`chain.rs`, `platform.rs`):** Evaluate extension candidates ([resolve_extension_candidates](file:///workspaces/ctoolbox/src/formats/utilities/extension.rs#L243)) and apply platform context boosts.
8. **Candidate Subsumption & Conflict Resolution (`conflict.rs`):** Apply MIME and format inheritance hierarchy demotions, detect unresolvable conflicts, and produce the sorted [DetectionCandidate](file:///workspaces/ctoolbox/src/formats/utilities/detection.rs#L259) list with calibrated [ConfidenceTier](file:///workspaces/ctoolbox/src/formats/utilities/detection.rs#L75).

---

- Lossless archive representation must preserve raw path bytes, metadata,
  multiple streams, links, sparse extents, timestamp precision and unknown fields.
  Logical entry round-tripping is not necessarily byte-for-byte archive
  reconstruction: ordering, duplicate entries, original headers, padding, and
  compression representation can also matter. Detection must not require fully
  unpacking an archive merely to classify it.
