# Quick-test regression verification

## Pending Work

The October 11 report concerned 27 failures in conversion (3) and CSC (24):
legacy journal flags, destination-relative hardlinks, lossless link paths,
stream-name serialization, and Finder metadata syntax. Fixes and follow-ups
were recorded as passing 490 affected library tests and targeted Clippy by
October 13. These are historical results, not validation of current HEAD.

The original implementation plans are superseded. Remaining verification
was not recorded as completed:

- Run the full `./test --quick` suite on the current tree. The earlier run was
  deferred for disk space; keep the current required 25 GB reserve.
- Exercise native Windows link/stream materialization and Windows-only test
  branches. Windows-source roundtrips on Linux do not establish native support.
- Exercise encoding-restrictive filesystems directly, including strict refusal
  and explicit best-effort replacement. Existing checks covered conversion
  logic and Unix raw-byte replication on the host filesystem.

## Scope

Do not revive the old implementation proposals or generic compatibility
fallbacks. Historical writers' discarded path bytes cannot be reconstructed;
legacy journal adaptation belongs at the journal boundary. Dc syntax parser
work is tracked separately in [../2026-10-13-Dc-syntax-parser-refactor/](../2026-10-13-Dc-syntax-parser-refactor/).