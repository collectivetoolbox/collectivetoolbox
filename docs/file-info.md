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
   In source CSV files, column 6 consolidates format specifications using
   `@chain(...)` (e.g., `@chain(((f15 > f542) ! f0) > f0)`), avoiding horizontal
   scrolling while permitting future annotations such as `@formalalias(...)`.
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
     (e.g., `300 1 302 ...`) and symbolic operator tokens (e.g., `& 1 > | 2 3 4 ) 5`).
   - The decoder also accepts legacy balanced grouping (`298 ... 299`) and explicit
     group terminators (`299` / `)`).
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
  entries via `ctb_formats_dcdata::format_spec`. In addition, multipart filename
  extensions (such as `.html.gz` and `.pan.Z`) are parsed into structured layers
  via `FormatChain` in `ctb_formats_utilities::detection`, generating format
  specification chains (`Html > Gzip`). Initial multi-signal detection
  (`detect_format_id`) combines magic byte signatures (`MAGIC_REGISTRY`),
  weighted extension patterns (`EXTENSION_REGISTRY`), and `FormatCategory` domain
  filtering.
- Define and allocate relation predicates only after fixing their domains,
  cardinality, ordering, and relation-instance representation. Then migrate math
  metadata out of prose and split the overloaded base/chain/syntax field.
- Inventory supported EITE numeral options and missing line conventions against
  existing identities. Add missing records without changing existing alphabets
  or conflating termination with separation. Test empty input, missing final
  terminators, mixed endings, and ambiguous interpretations.
- Put detection rules in a separate versioned dataset keyed by format IDs. Each
  rule should declare byte tests, offsets, masks, bounded structural probes,
  optional filename patterns, provenance, and applicability. Return multiple
  candidates with evidence and explicit confidence semantics; a score is not a
  calibrated probability. Names and extensions alone are weak evidence.
- Share a read-only source abstraction across byte buffers, filesystem nodes,
  and archive entries: size when known, bounded range reads, raw names, metadata,
  and bounded immediate-child lookup. Set byte/read/entry budgets; do not traverse
  arbitrary subtrees or follow links during package detection. Distinguish
  insufficient evidence and unavailable data from a negative match.
- Lossless archive representation must preserve raw path bytes, metadata,
  multiple streams, links, sparse extents, timestamp precision and unknown fields.
  Logical entry round-tripping is not necessarily byte-for-byte archive
  reconstruction: ordering, duplicate entries, original headers, padding, and
  compression representation can also matter. Detection must not require fully
  unpacking an archive merely to classify it.
