# File and CSC Preservation Safety

The file abstraction and `csc` treat payloads and attached streams as data.
Best-effort metadata mode must not turn payload or stream corruption into success.

## Safety Checks

- Regular replacements are staged, checked for exact length and SHA-256, synced,
  and renamed. Sparse extent maps must cover the complete logical file without
  gaps, overlaps, or overflow. Strict mode audits staged data and metadata before
  replacement.
- Failed link or special-node creation leaves the existing destination intact.
- Strict directory metadata and stream failures propagate to the caller.
- Hardlinks are tracked across copy tasks; journal targets are destination-relative.
- Copy roots cannot overlap or refer to the same inode. Dangling symlinks remain
  valid source entries.
- Resume rechecks entries, including previously committed entries. New journal
  records retain filesystem identity for recovering the original directory target.
  Ambiguous older journals fail rather than guessing. Damaged, uncommitted journal
  tails are discarded at the last verified boundary before appending new records.
- Copy-based moves always require strict verification. Skipped entries prohibit
  source cleanup. Cleanup rechecks recorded identities and content, removes only
  copied entries, and uses non-recursive directory removal. Unexpected entries
  therefore prevent removal of their containing directory.

## Platform Status

Native Windows stream enumeration, security metadata, file identity capture,
symlink type preservation, and descriptor-relative sandboxing are not implemented.
Operations requiring those guarantees return errors instead of fabricated empty
metadata or successful no-ops. This deliberately prevents lossless Windows
filesystem copies and cross-device moves until native support exists. Same-device
OS rename remains available. Cross-compilation is not a Windows runtime test.

## Remaining Limits

- These operations do not take a filesystem snapshot or exclude concurrent
  writers. A writer can still race between final verification and unlink, or
  replace path components used by path-based metadata operations. Quiesce writers
  and protect source and destination directories for destructive moves.
- A tree copy is not a transaction. Earlier entries may have been replaced when
  a later entry fails. Metadata-only reuse can update an existing destination
  in place; it does not provide rollback. Flags applied after rename can also
  fail after replacement has committed.
- Crash durability depends on filesystem and storage behavior. Successful fsync
  is not proof against hardware faults; no power-failure injection was performed.
- POSIX ctime is not assignable. Birth times and all platform-specific metadata
  are not universally captured or reproduced. Journals are verification records,
  not complete backup archives: stream payloads and complete extent maps are not
  stored in them. Sparse verification checks hole presence, not identical physical
  allocation.

## Validation

Run `cargo test -p ctb-io-file -p ctb-io-csc --lib` for regression coverage.
Compile Windows file tests with
`cargo test -p ctb-io-file --lib --no-run --target x86_64-pc-windows-gnu`.