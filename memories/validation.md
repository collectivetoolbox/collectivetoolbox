- File/copy crate tests: `cargo test -p ctb-io-file -p ctb-io-csc --lib` (hyphenated package names).
- Do not run scripts/format or scripts/format-quick for routine formatting: they auto-stage/commit and destructively clean vendor. Use rustfmt directly on touched files/lines.
- Operator commits during work; blank git diff does not mean edits vanished. Inspect commit history and task baseline.
- rustix 0.38 IFlags is not Copy or PartialEq: use .bits() for repeated masking/comparison and from_bits_retain for syscall values.
- Native Windows stream capture can be cross-compiled with `cargo test -p ctb-io-file --lib --no-run --target x86_64-pc-windows-gnu`; this does not run Windows tests.

- Panorama debug CLI is `pan2parsejson`. Procedure payloads can contain tokenized code plus a separate source copy; source-only mutation tests the current AST runtime, not necessarily original Panorama execution.

- Dc data validator CLI only recognizes `--write`/`-w`; other flags are silently ignored. Default validates CSVs embedded directly from src/formats/dcdata/data, not the general asset bundle. For read-only live-file validation use validate_all_data_tables_from_repo (covered by test_math_format_metadata); avoid --write when IDs must remain untouched.
