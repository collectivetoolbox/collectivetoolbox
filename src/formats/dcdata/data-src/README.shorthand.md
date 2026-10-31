Dcs may be referenced using different formats.

Besides the bare-integer long Dc and short Dc formats, some tools also accept shorthands as follows:

- bare integer: short Dc
- `u` prefix, like `u12a`: Unicode character, hexadecimal (`u12a` = `U+012A`)
- `f` prefix: Format Dc (f0 = UTF-8)
- `l` prefix: Long Dc
- `L` prefix: Local graph ID (not accepted/relevant in all contexts); equivalent to short Dc 296 followed by a Dc number of the integer following the `L`

These shorthands are case-sensitive.

Valid shorthand identifiers match `[flL]?\d+` or `u[0-9a-f]+`; non-matching strings should be rejected by parsers.
