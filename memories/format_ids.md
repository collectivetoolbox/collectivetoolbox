# Format IDs and Identifiers

- **No "Short Format ID" concept**: There is no such thing as a "short format ID".
- **Never expose or process Format IDs as bare integers**: Bare integer IDs are strictly prohibited because they are confusing and unbranded—they look identical to short Dc IDs.
- **Do not add integer conversion APIs**: Never add `short_id()` or `from_short_id()` methods to `FormatId`.
- **Use typed representations**: Always use the strongly typed `FormatId` enum, `FormatId::ident()` (e.g. `"Gzip"`, `"Brotli"`), or format shorthand strings (`FormatId::shorthand()`, `FormatId::from_shorthand("f34")`).

## Block Reservations

- `data-src/README.dc-blocks.csv` and `README.format-blocks.csv` are authoritative allocation registries, not generated reports. They store inclusive ranges and allocation high-water marks as hexadecimal long IDs.
- `README.scripts.csv` increment changes affect future blocks only. Existing reservations must never be reconstructed from current increments or automatically shrunk. Manual tail shrinking must retain the high-water mark and every assigned ID.
- `updater::validate_category_allocations` validates both namespaces without writes; normal table validation and write-mode preflight use it. The updater rejects duplicate IDs, overlapping reservations, wrong-script membership, and region overflow.
- `ctb_build_support::is_unassigned_id` treats nonnumeric text (including prefixed format IDs and oversized numbers) as unassigned. Category allocation uses `assigned_category_id` to preserve `f`-prefixed IDs and reject oversized numeric IDs before considering placeholders.
- Validation: `cargo test -p ctb-formats-dcdata --target x86_64-unknown-linux-musl --lib` passed all 120 tests; production Clippy for the library and `validate-data-tables` passed with existing workspace warnings (2026-11-07).
