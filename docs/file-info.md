This repository currently has some support in formats/utilities for information about files and file formats.

It is not maintainable or robust, so I would like to rework it.

I have provided several other applications that implement file type detection for reference.

The following types of features are relevant:

- Detecting a file type given a file or array of bytes - it should return multiple candidates if relevant, with an indication of the confidence in its guesses.
  - It should primarily use the format of the file as its source.
  - This should optionally take into account file name patterns.
  - It should be able to work on a directory (to detect package formats like Mac .app), but needs to run quickly and not traverse the whole directory.
  - It should be able to work without a file system - other libraries should be able to call it as an in-memory utility. A common set of types for representing files, filesystem objects, and compressed archive entries will likely be relevant.
- Providing file format "nicknames" for command-line utilities, and a most-preferred nickname for use in UIs.
- Providing a most-preferred extension.
- Providing Rust struct identifiers.
- Global graph ID references for file formats - these are the IDs in the current spreadsheet, with the offset to locate them in the formats graph block (see the graph layout documentation).
- Implementing a readable format description DSL that can also be represented in Dcs (=global graph IDs). I'd like you to plan out a syntax for this. Examples of the general idea (not sure about this syntax): `directory > tar > bz2` (a bz2 tarball), `bz2` (any bz2 stream), `((english > iso8859-1) ! utf8) > utf8` (English text encoded as latin1, misinterpreted/mojibaked as UTF-8, stored as UTF-8), `((macroman | altura-mac-to-win) > utf8) & CR & CSV` (
