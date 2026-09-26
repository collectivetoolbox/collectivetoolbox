# Format IDs and Identifiers

- **No "Short Format ID" concept**: There is no such thing as a "short format ID".
- **Never expose or process Format IDs as bare integers**: Bare integer IDs are strictly prohibited because they are confusing and unbranded—they look identical to short Dc IDs.
- **Do not add integer conversion APIs**: Never add `short_id()` or `from_short_id()` methods to `FormatId`.
- **Use typed representations**: Always use the strongly typed `FormatId` enum, `FormatId::ident()` (e.g. `"Gzip"`, `"Brotli"`), or format shorthand strings (`FormatId::shorthand()`, `FormatId::from_shorthand("f34")`).
