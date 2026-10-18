# CSC Archives

Status: **Partially Implemented**. This is an experimental command, not yet a
production-ready backup tool. Keep original files until recovery has been
independently validated. See [the implementation tracker](../../issues/csc-usability/archive.md).

## Commands

```sh
ctb archive create backup.cscarchive source/
ctb archive create - source/ > backup.cscarchive
ctb archive create partial.cscarchive source/ --continue-on-error
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

## Capture Failures

Source inspection, traversal, opening, and read failures are retained as checked
diagnostic records whenever a valid writable archive stream can be maintained.
Semantic metadata warnings are retained too and prevent a clean capture. By
default creation stops incomplete and returns nonzero after persisting the
failure. Safety conflicts, output failures, or failure to persist diagnostics
always abort; `--continue-on-error` does not waive them.

`--continue-on-error` permits capture of other members after source failures.
File output can discard the current uncommitted member, including buffered
bytes, before truncating to its last committed boundary and recording the error.
Stdout cannot roll back: failures before member emission can be recorded, but a
source failure inside an emitted member aborts without a valid completion marker.
Broken pipes, flush/sync failures, and failed rollback always abort.

A completed archive with errors or warnings is not a lossless capture. Creation
and listing return nonzero; listing displays retained diagnostics. Verification
and extraction reject it, including before creating an extraction destination.
The library's `verify_integrity` checks completed structure and archived bytes
independently of capture success; this is not an authentication or completeness
guarantee. General salvage extraction is not provided.

File-output resume requires the original sources. It verifies committed contents
and source identities before truncating an unclosed final section. It does not
append to a completed archive or silently repair a corrupt committed section.
An incomplete archive can be listed, with nonzero status, but cannot be extracted
as though it were complete.

Resume refuses archives with committed error/warning diagnostics before
truncation, because their failures cannot yet be resolved and accounted for
durably. `--resume` and `--continue-on-error` cannot be combined. Durable source
inventory and retry/resolution semantics remain pending; a new traversal is not
a filesystem snapshot of the original operation.

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
entity, error record (Dc 552..553), or warning record (Dc 554..555). Primary
contents use existing Dc 392; attached streams retain their
existing metadata and Dc 385 data representation. Zero-size regular files may
omit the primary data field. Original sparse extents are recorded, but logical
file bytes are stored densely. Hardlink members refer to a defining regular
member rather than repeating its primary contents.

Earlier experimental readers that accept only file sections reject archives
containing diagnostic sections. Diagnostic-free v1 archives remain readable;
the magic and format IDs have not changed.

The format is uncompressed and unauthenticated. Checksums detect corruption,
not malicious replacement. Archive contents and provenance may be sensitive;
file-output creation uses private permissions on Unix.

## Pending Work

- Durable source inventory and resolution/retry of recorded capture failures.
- Lazy capture and projection for large attached streams.
- Broader interrupted-footer recovery and concurrency/security coverage.
- More replacement types, non-Unix extraction, and remaining relevant CSC flags.
- Durable extraction/publication recovery state and restart are planned as part
  of production hardening; they are not available yet.
- Compression, encryption, signatures, stdin input, and completed-archive
  updates remain outside the current scope.