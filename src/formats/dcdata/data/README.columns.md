# Dc Data Columns and Column Specification Directives

This document describes the schema of columns used across Document Character (Dc) data tables, including `all.generated.csv`, `unicode.generated.csv`, `formats.generated.csv`, `DcList.generated.csv`, and category tables under `categories/`.

## 22-Column Unified Schema Overview

| Col | Header | Description |
|---|---|---|
| 1 | **Dc** | Global Document Character ID (decimal integer) or Unicode codepoint shorthand (`u<hex>`). |
| 2 | **Short** | Short Dc ID (decimal integer 0..=1114111) or format shorthand (`f<id>`). Blank or `u<hex>` for Unicode characters. |
| 3 | **Name (!=deprecated)** | Canonical character or format name. Deprecated entities are prefixed with `!`. |
| 4 | **◌** | Canonical Combining Class (0..=254). |
| 5 | **⇆** | Bidirectional class (e.g. `L`, `R`, `AL`, `EN`, `ES`, `ET`, `AN`, `CS`, `B`, `S`, `WS`, `ON`, `BN`). |
| 6 | **Aa** | Casing partner Short Dc ID or format base if in legacy layout. |
| 7 | **Type** | General Category (e.g. `Lu`, `Ll`, `Lt`, `Lm`, `Lo`, `Mn`, `Mc`, `Me`, `Nd`, `Nl`, `No`, `Zs`, `Zl`, `Zp`, `Cc`, `Cf`, `Cs`, `Co`, `Cn`, `Pd`, `Ps`, `Pe`, `Pc`, `Pi`, `Pf`, `Po`, `Sm`, `Sc`, `Sk`, `So`, or `!Cx` for extended Dcs). |
| 8 | **Script** | Script or Unicode block name (e.g. `Latin`, `Common`, `Basic Latin`). |
| 9 | **Aliases; >=xref, <=decompos., :=Dc syntax, @chain** | Composite column for aliases, directives, cross-references, decompositions, and syntax rules (detailed below). |
| 10 | **Description** | Human-readable explanatory description, clarifications, and usage notes. |
| 11 | **Ident (Rust-friendly)** | PascalCase or snake_case identifier suitable for code generation. |
| 12 | **Category** | Category folder or group name (e.g. `container`, `audio`, `controls`, `latin`). |
| 13 | **Extensions** | Primary file extension first (e.g. `.tar.gz`), followed by comma-separated alternatives. |
| 14 | **MIME** | Primary MIME type first (e.g. `application/gzip`), followed by comma-separated aliases. |
| 15 | **Apple Uniform Type Identifier (UTI)** | Apple UTI string (e.g. `org.gnu.gnu-tar-archive`). |
| 16 | **Apple Type code** | Classic Mac OS 4-character Ostype code (e.g. `TAR `). |
| 17 | **Nicknames** | Short CLI or argument aliases (e.g. `tgz`). |
| 18 | **Import support** | Status or handler for importing/decoding format. |
| 19 | **Export support** | Status or handler for exporting/encoding format. |
| 20 | **Tests** | Test cases or test identifiers. |
| 21 | **Variant Types** | Comma-separated list of variant subtypes or tags. |
| 22 | **References** | External specifications, RFCs, ISO standards, or documentation URLs. |

---

## Column 9: Aliases, Directives, and Relations

Column 9 appears as Column 9 in Dc and Unicode tables and Column 6 in Format category tables (`"Base/Related Format/Category, Chain (@), or Syntax (:)"`). Cells contain comma-separated directives (`", "`).

### Parsing and Spacing Rules
- Items must be separated by exactly a comma followed by a space: `", "`.
- No leading or trailing whitespace inside the cell.
- No space before commas.
- Directives containing commas or quotes must be enclosed in parentheses or quotes as defined below. When exported to CSV, cells containing commas or double quotes are wrapped in RFC 4180 double quotes, with internal quotes doubled (`""`). Inside directive string literals, double quotes are escaped with `\"`.

### Directives Catalog

#### `@base(...)`
- **Purpose**: Defines base or parent format/category relationships using Dc shorthand syntax.
- **Syntax**: `@base(<shorthand>)` or `@base(<shorthand> & <shorthand>)`.
- **Examples**:
  - `@base(f161)`
  - `@base(f390 & f395)`

#### `@chain(...)`
- **Purpose**: Defines format composition pipelines and transformation chains using the format specification DSL.
- **Syntax**: `@chain(<expr>)`
- **Examples**:
  - `@chain(f161 > f35)`
  - `@chain(((f15 > f542) ! f0) > f0)`

#### `@xref(...)`
- **Purpose**: Cross-reference to another character or Dc entity (from Unicode `NamesList.txt` `x` lines or related cross-references).
- **Syntax**: `@xref(<target>)` where `<target>` is a valid Dc shorthand (`u<hex>`, short Dc integer, or format `f<id>`).
- **Examples**:
  - `@xref(u22ee)`
  - `@xref(u00df)`
  - `@xref(u00a0), @xref(u200b)`

#### Formal Name Aliases
Normative formal aliases from Unicode Standard Annex #44 (`NameAliases.txt`) and `NamesList.txt` (`%` lines):

- **`@formalAliasCorrection("...")`**:
  Corrections for serious errors or misspellings in canonical character names.
  - *Example*: `@formalAliasCorrection("PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET")` (for U+FE18, named `... BRAKCET`).
  - *Example*: `@formalAliasCorrection("LATIN CAPITAL LETTER GHA")` (for U+01A2).
- **`@formalAliasControl("...")`**:
  ISO 6429 and standard control function names.
  - *Example*: `@formalAliasControl("NULL")` (for U+0000).
  - *Example*: `@formalAliasControl("LINE FEED")` (for U+000A).
- **`@formalAliasAlternate("...")`**:
  Widely accepted alternate names for format or control characters.
  - *Example*: `@formalAliasAlternate("BYTE ORDER MARK")` (for U+FEFF).
- **`@formalAliasFigment("...")`**:
  Documented historical or unapproved control names.
- **`@formalAliasAbbreviation("...")`**:
  Standard abbreviations and acronyms for control codes and format markers.
  - *Example*: `@formalAliasAbbreviation("NUL")` (for U+0000).
  - *Example*: `@formalAliasAbbreviation("BOM")`, `@formalAliasAbbreviation("ZWNBSP")` (for U+FEFF).

#### `@annotation("...")`
- **Purpose**: Informative notes and annotations from Unicode `NamesList.txt` (lines starting with `* `).
- **Syntax**: `@annotation("<text>")` with internal double quotes escaped as `\"`.
- **Examples**:
  - `@annotation("German")`
  - `@annotation("misspelling of \"BRACKET\" in character name is a known defect")`
  - `@annotation("also used for inches, seconds of arc")`

#### Bare Aliases (Informal Aliases)
- **Purpose**: Informal aliases, secondary names, or common synonyms (e.g. from `NamesList.txt` `= ` lines).
- **Syntax**: Bare strings (permitted in character category tables).
- **Examples**:
  - `double quote`
  - `Eszett`
  - `tab`

#### Decomposition Tags (`<tag>...`)
- **Purpose**: Decomposition and semantic correspondence tags.
- **Syntax**: `<tag><target>` where tag is `<equiv>`, `<approx>`, `<ambiguous>`, `<prefer>`, etc.
- **Examples**:
  - `<equiv>u20`
  - `<approx>u2d`
  - `<ambiguous>257 119`

#### Syntax Rules (`:...`)
- **Purpose**: Document Character syntax DSL declarations.
- **Syntax**: `:<syntax>` (e.g. `:~ [format:base_alphabet]`).

---

### Canonical Ordering in Column 9
When records are generated or serialized, directives and items in Column 9 are arranged in the following deterministic order:
1. **Formal Name Aliases**: `@formalAliasCorrection(...)`, `@formalAliasControl(...)`, `@formalAliasAlternate(...)`, `@formalAliasFigment(...)`, `@formalAliasAbbreviation(...)`
2. **Informal Aliases**: Bare alias strings
3. **Cross-References**: `@xref(...)`
4. **Decompositions**: `<tag>...`
5. **Annotations**: `@annotation(...)`
6. **Syntax Rules**: `:<syntax>`
