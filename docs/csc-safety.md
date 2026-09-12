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

## Metadata Capture and Retention

Capture and reproduction are separate operations. Every captured field is stored
in the journal, including fields the destination cannot reproduce. Copy records
are committed before materializing each entry, so a strict metadata failure does
not discard that entry's original metadata. This is incremental capture, not a
preflight archive of the entire source tree.

The journal's `CTBMETA1` preservation record retains:

- Complete portable metadata, including birth time when reported by the OS,
  nanosecond timestamps, inspection time, ownership, mode, semantic flags, and
  raw platform flag bits with their originating OS.
- Extensible native records with signed integers, unsigned integers, and opaque
  byte values, without converting them through display strings.
- Complete discovered logical extent maps and each attached stream's original
  name, encoding, classification, descriptor, and payload bytes.

Linux capture includes Unix stat fields, the requested `statx` fields and support
masks, `FS_IOC_GETFLAGS`, and, where supported, `FS_IOC_FSGETXATTR` fields (including
project ID, extent hints, and raw extended flags) and inode generation. Enumerated
xattrs are read in full, including Linux POSIX ACL and security-label xattrs.
Metadata-only inspection skips the main payload hash, not streams or extent maps.
Unexpected native query errors abort capture instead of manufacturing empty
metadata. An API reporting that a feature is unsupported is distinct from a
permission error or a failed read.

Birth times and native fields without a reproduction implementation are compared
with the destination. Strict mode fails on differences; best-effort mode warns.
Malformed metadata and native read errors fail in either mode. Flag application
includes readback even when the separate copy verification pass is disabled.
Identity and allocation observations (for example inode number, mount ID, extent
count, and inode generation) are retained, not assigned to the destination.
Portable fields have their own audit policies, including the optional ctime check.

Linux has no general birth-time setter here. Consequently, strict copies and
copy-based moves commonly refuse new entries whose birth time differs. They must
not report such a destination as a fully reproduced original.

Journals containing native metadata, birth times, raw flags, or attached streams
are retained even with `--delete-manifest-after`, and after copy-based moves.
The result reports the retained journal path. Keep it with the copy: it holds
original metadata that the destination alone may not preserve. Journals contain
potentially sensitive security metadata and stream contents. They do not contain
the main file payload and are not standalone backup archives. Legacy journals
cannot recover fields that their writers never stored. A checksummed record that
cannot be decoded is an error, not permission to truncate it during resume.

## Stream Names

Stream names are tagged raw bytes or raw Windows UTF-16 code units. The journal
preserves non-UTF-8 byte names and unpaired UTF-16 surrogates on either host;
decoding a stream name does not require constructing a host filesystem path.
Lossy string conversion is for diagnostics only.

Native recreation converts names only at the filesystem boundary. Exact Unicode
transcoding is allowed; unrepresentable names and names colliding after conversion
are rejected before stream writes. Verification compares destination-native names
without changing the original journal name. Recreating a foreign stream also
requires the target's namespace to accept it: storing an NTFS name in a journal
does not make that name a valid Linux xattr. Stream write or payload failures
remain fatal even in best-effort metadata mode.

## Platform Status

Windows named data stream enumeration uses the native wide-character API and
retains its UTF-16 names and payloads. Full Windows filesystem capture remains
blocked: security descriptors, reparse data, complete identity capture, stream
recreation, and descriptor-relative sandboxing are not implemented. Operations
requiring those guarantees return errors rather than fabricated metadata. This
prevents lossless Windows filesystem copies and cross-device moves; same-device
OS rename remains available. Cross-compilation is not a Windows runtime test.

Universal native metadata capture supports BSD/macOS ACLs outside xattrs:
Darwin extended ACLs (macOS) and FreeBSD NFSv4/POSIX.1e ACLs are captured with
dual binary (acl_copy_ext) and canonical text representations, preserving
granular permissions, inheritance flags, and trivial ACL distinctions using
symlink-safe link APIs. The extensible journal retains their values, and
restoration applies them with strict validation. Do not treat current coverage
as proof that every metadata class on every source filesystem has been
enumerated.

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
- POSIX ctime is captured but is not generally assignable. Sparse verification
  checks hole presence, not identical physical allocation.

## Validation

Run `cargo test -p ctb-io-file -p ctb-io-csc --lib` for regression coverage.
Compile Windows file tests with
`cargo test -p ctb-io-file --lib --no-run --target x86_64-pc-windows-gnu`.