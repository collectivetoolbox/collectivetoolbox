This repository currently has some support in formats/utilities for information about files and file formats.

It is not maintainable or robust, so I would like to rework it.

I have provided several other applications that implement file type detection for reference.

The following types of features are relevant:

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
- Implementing a readable format description DSL that can also be represented in Dcs (=global graph IDs). I'd like you to plan out a syntax for this. Examples of the general idea (not sure about this syntax): `directory > tar > bz2` (a bz2 tarball), `bz2` (any bz2 stream), `((english > iso8859-1) ! utf8) > utf8` (English text encoded as latin1, misinterpreted/mojibaked as UTF-8, stored as UTF-8), `pan > (((macroman | altura-mac-to-win) > utf8) & CRLF & CSV) | (hexdump & xxd & utf8)` (I think this is the format that `ctoolbox pan2csv --encoding win-utf8 'example2 with lemurs.pan' | xxd` returns), `(pan > (json & utf8)) | jq['.prelude'] & json & utf8` (that last one is a lot more ambitious, but it would be really cool to be able to declaratively build a pipeline like that - for the moment it's blocked until I figure out how I'd like to represent it in Dcs). I have added Dcs 298 through 303 for semantic grouping and for operations "union" (which I used & for here), "transform" (which I used : for here), "convert" (which I used a > for here), and "transmute" (which I used a ! for here). Example: `((english > iso8859-1) ! utf8) > utf8` would be:
    -
  - Defining formats this way has
- I wonder if it would be useful to use Kaitai Struct and the compatibly-licensed parts of its library of format definitions for some parsing?
- Many of the formats already supported in this repository are represented in the format database and syntax, but some of the "custom formatting" ones are not - the base conversions from EITE in particular are very flexible and will need some sort of parametrized syntax.

I'd like to find a way to make the database of formats as easy as possible to maintain: retaining uniqueness of IDs, checking validity of fields, and so on. The current formats.csv structure is not too bad - and I do like working in a spreadsheet for ease of editing - but it may be worth considering if some other structure would be better. At least adding automatic validation (maybe at build time?) for the CSV would be important.

The number base formats (alphabets and so on) supported by ctoolbox are not yet really comprehensively represented in formats.csv, nor are line ending formats.

