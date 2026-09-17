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

## Proposed Model Boundaries

The following is a design proposal, not an implemented schema or DSL.

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
   Migrate legacy `Chain (=)` entries individually once this representation is
   defined. Do not reinterpret every `&` in the overloaded base column as an
   executable pipeline; `Utf8_Base64` (f110), for example, describes nested
   representations, not two independent constraints on the same bytes.
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

## Proposed Description Syntax

Use distinct operators for distinct operations:

| Text | Existing Dc | Meaning |
| --- | --- | --- |
| `A & B` | 300 | Both constraints apply to the same data |
| `A : T` | 301 | Apply registered transformation T to representation A |
| `A > B` | 302 | Convert/encode A into representation B |
| `A ! B` | 303 | Reinterpret A's unchanged representation as B |
| `(A)` | 298 / 299 | Explicit grouping |

`>`/`!`/`:` bind more tightly than `&`, with left-to-right association. Parentheses
can override this; the canonical printer should show mixed-operation grouping.
Thus `directory > tar > bz2` means `(directory > tar) > bz2`, while
`pan > (json & utf8)` applies both constraints to the output. The spelling
`directory` is schematic until it is explicitly bound to a suitable descriptor;
the existing file-kind Dc 359 must not be confused with format f359 (BaseAlphabet).

Atoms should allow unambiguous ID references (`f542` for format-local IDs and
`@2228766` for global IDs) as well as registered nicknames. Nicknames such as
`iso8859-1` need hyphens; resolve them through data and reject ambiguous aliases.
Not every graph node is a type: validate the referenced node's role. IDs provide
a spelling even when a format has no nickname. A Rust identifier is not
automatically a CLI nickname.

For the initial subset, reserve `|`: neither "alternative" nor "pipe" is
interchangeable with Dc 300. Use `:` for a named transformation, so the Altura
subexpression can be written `(macroman : altura-mac-to-win) > utf8`. A hex dump
of that output requires a registered hexdump transformation and parameters for
the xxd dialect, not a conjunction asserting that the original bytes are already
a hexdump. Query expressions such as `jq['.prelude']` remain outside this subset.
Neither parsing nor detection should invoke external commands.

One proposed canonical Dc encoding uses a balanced group for each binary
expression: `298 operator left right 299`. Leaves are type references. For
`((english > iso8859-1) ! utf8) > utf8`, the symbolic token stream would be:

```text
298 302
  298 303
    298 302 f15 f542 299
    f0
  299
  f0
299
```

Here short operator numbers stand for their short-region global IDs; `fN`
stands for a formats-region global ID. This preserves the existing identities
of group delimiters. Do not use unmatched closing groups as implicit operator
terminators. A purely arity-delimited prefix encoding would also be possible,
but should not be mixed with this balanced encoding.

**Blocked on grammar work:** `[type]` currently means `[262:]` or a bare format,
not recursive expressions containing Dcs 298 and 300-303. Before implementing
this proposal, define a recursive type-expression grammar, transformation operand
constraints, depth/size limits, and text/Dc/AST round-trip tests. Do not claim
these example streams are already accepted by the payload syntax machinery.

Parameterized descriptions could use a separate, typed application form such as
`base-numeral(radix=16, alphabet=alphabet-id)`. This spelling is illustrative and
has no assigned Dc application encoding yet. Reuse BaseNNumeral (f350), Base
(f354), and alphabet identities rather than assigning a format ID for every
combination. Validate alphabet ordering, digit uniqueness, radix compatibility,
case rules, signs, padding, and separators explicitly. The current f371-f375
`<equiv>` entries contain `[number:'...']` syntax placeholders; replacing those
with actual parameter bindings requires this model, not another textual chain.

## Next Data Work

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
