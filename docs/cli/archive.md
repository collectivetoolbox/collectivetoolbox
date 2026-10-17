# CSC Archives

Status: **Partially Implemented**. This is an experimental command, not yet a
production-ready backup tool. Keep original files until recovery has been
independently validated. See [the implementation tracker](../../issues/csc-usability/archive.md).

## Commands

```sh
ctb archive create backup.cscarchive source/
ctb archive create - source/ > backup.cscarchive
ctb archive create backup.cscarchive --resume
ctb archive list backup.cscarchive
ctb archive verify backup.cscarchive
ctb archive extract backup.cscarchive restored/
```

Creation refuses to replace an existing archive. Sources without a trailing
slash retain their basename; directory sources with a trailing slash contribute
their contents and root metadata. Duplicate member paths are errors. The archive
must not be included in its source tree. Shell redirection can truncate an
existing file before the command starts; the command cannot undo that damage.

Stdout creation uses the same forward-only record encoder as file creation.
Stdout contains binary archive data only; creation summaries and errors go to
stderr. Broken pipes and final flush failures are errors. Stdout cannot resume,
discard failed output, sync an archive to disk, or reread output for verification.
Verify the saved archive independently.

File-output resume requires the original sources. It verifies committed contents
and source identities before truncating an unclosed final section. It does not
append to a completed archive or silently repair a corrupt committed section.
An incomplete archive can be listed, with nonzero status, but cannot be extracted
as though it were complete.

Extraction and verification require only the archive, not the source paths.
Extraction is currently Unix-only. Destination-relative paths and hardlink
targets are validated; archive writes never intentionally traverse a symlink
ancestor. Existing entries are rejected by default. `--overwrite` permits
atomic regular-file replacement and directory merging, not replacement of every
entry type. `--trust-symlink-targets` preserves raw symlink targets, including
targets outside the destination; it does not authorize archive writes outside it.
`--copy-specials-as-specials` opts into restoring FIFOs and device nodes.

Existing directory metadata and attached streams are protected too. In
particular, extracting a contents-only root into an existing directory requires
`--overwrite` to restore that directory's metadata. Freshly created directories
are finalized only after the live sandbox revalidates their identities.
Publication enforces no-clobber again after preflight. Non-Linux Unix platforms
currently reject regular-file no-replace publication as unsupported rather than
falling back to an unsafe replacing rename. Native Windows helpers compile, but
archive extraction remains Unix-gated and Windows runtime safety is unqualified.

Strict metadata restoration is the default and may fail for metadata the target
cannot reproduce. `--best-effort-metadata` reports such losses while retaining
the original metadata in the archive. Payload or stream corruption remains an
error. Do not concurrently modify archive, source, or destination trees.

## Format

Version 1 starts with the eight bytes `CTBCSCA\x01`, followed by a normal
DcMixed document header. Format identities are f633 (`CscArchive`) and f634
(`CscArchive1`). The suggested extension is `.cscarchive`.

The body uses the CSC journal's operation-session representation, sequenced
checksummed sections, and completion marker. Each section contains one file
entity. Primary contents use existing Dc 392; attached streams retain their
existing metadata and Dc 385 data representation. Zero-size regular files may
omit the primary data field. Original sparse extents are recorded, but logical
file bytes are stored densely. Hardlink members refer to a defining regular
member rather than repeating its primary contents.

The format is uncompressed and unauthenticated. Checksums detect corruption,
not malicious replacement. Archive contents and provenance may be sensitive;
file-output creation uses private permissions on Unix.

## Pending Work

- Persistent operational diagnostics and continue-on-error creation.
- Lazy capture and projection for large attached streams.
- Broader interrupted-footer recovery and concurrency/security coverage.
- More replacement types, non-Unix extraction, and remaining relevant CSC flags.
- Durable extraction/publication recovery state and restart are planned as part
  of production hardening; they are not available yet.
- Compression, encryption, signatures, stdin input, and completed-archive
  updates remain outside the current scope.