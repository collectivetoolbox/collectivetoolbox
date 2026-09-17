This repository currently has some support in formats/utilities for information about files and file formats. It uses data from the Dc database (which is in effect Unicode extended with additional characters, most of which serve to represent semantic data).

It is not maintainable or robust, so I would like to rework it.

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
  - Defining formats this way has
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
