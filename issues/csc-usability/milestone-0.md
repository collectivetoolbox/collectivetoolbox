# Milestone 0: csc and archive probes

Updated 2026-10-03. This note is the current work queue. It narrows
[the roadmap](../../docs/mvp-roadmap.md): the Dc document runtime, ontology
explorer, and graph-stored trust are future work. They start after `csc` and
the archiver have a usable MVP. Do not spend this milestone on them.

A usable preservation MVP is copy, archive, and extract on btrfs and NTFS,
with the contract below, plus an honest ZFS gap. It is not a qualified backup
of the 100-million-file disk, and it is not a Dc workspace.

## Still missing

- No archive command and no stable archive format. Do not invent that format
  until a small btrfs copy and a Windows copy have been run, and the data-loss
  bugs below are either reproduced or cleared.
- ZFS is unqualified. A BSD or Illumos VM can be prepared later. It does not
  exist, and the application has not been run on those systems. Nothing in
  this workspace is a ZFS result.
- Windows capture can succeed while dropping data. Opening the metadata handle
  can return success without that metadata. A failed reparse read is a
  warning, then success. Each alternate stream is read with a whole-file read.
  Strict sandboxed directory creation is rejected on non-Unix. These are code
  readings, not VM results.
- [csc-safety.md](../../docs/csc-safety.md) still says those Windows features
  are unimplemented and return errors. That section is stale. Correct it
  after a VM run, not before.
- No filename-collision test exists. The requirement is only in the docs.
- Journal growth per entry has no budget. At 100 million files, 1 KiB of
  extra data per file is about 100 GB. That risk remains.
- btrfs subvolumes and ZFS datasets are not in this milestone. File kinds and
  their metadata are. Volume send/receive is a different tool; do not treat
  silence as support.

## Settled

These are closed. Do not reopen them while writing fixtures.

- Work is `csc` and the archiver only, on NTFS, btrfs, and ZFS. Detection
  catalogs, the renderer, and graph editing are out of this MVP.
- btrfs work can run on this machine. NTFS work uses the Windows 7 QEMU guest
  driven by [scripts/win-vm](../../scripts/win-vm). Agents may run guest
  commands or build NTFS fixtures in that VM. Cross-compilation does not
  qualify NTFS. Windows 7 is a real NTFS host, not a stand-in for every later
  NTFS feature. A feature the guest cannot create stays untested.
- ZFS waits for a BSD or Illumos VM. Do not prepare that VM in this
  milestone, and do not substitute Linux ZFS or a mock.
- Resume does not have to continue inside a file. On restart, rewind may
  discard at most 500 files, and at most the larger of the most recent file
  and 1 GB. A multi-gigabyte file may be recopied whole, because stopping
  inside it is not required. That cap is not permission to recopy the rest of
  the tree. A changed live source is still not a snapshot.
- The three filesystems need a comprehensive, robust implementation. Not
  knowing which NTFS features are on one disk does not shrink that. This
  milestone inventories the gap. It does not finish the implementation.
- Archives stay local and unencrypted. Compression is outside the archive
  format. Security boundaries need no further decision here.
- Later graph deletion, recorded so it is not re-decided: the node type
  becomes Deleted, the content is discarded, and the ID stays reserved.
  Indexes are garbage-collected on demand or within a time bound. The bound
  was not specified and is not needed for `csc`.
- Global ID collisions across Git branches are not a problem: one developer,
  branches not used that way.
- Filenames stay exact native sequences. Normalization is not identity.
  Inode numbers, ctime, and physical layout are source facts, not things
  extraction must recreate.

## Task list

Do these in order. Stop when the session is gone and write down what was not
reached. Do not fix a bug unless the measurement is meaningless without the
fix. Do not start the archive format, the Dc runtime, or a ZFS VM here.

**1. Write the filesystem gap list.**

One table for NTFS, one for btrfs, one for ZFS. Rows are file kinds and
metadata the platform can store, not the features someone remembers using.
For each row, mark captured, restored, failed closed, failed open, or
untested. Use the current file and `csc` code plus the platform's own
behavior. On NTFS include streams, security descriptors, reparse points,
timestamps, attributes, hard links, sparse files, short names, and names
that differ only by case or by Unicode normalization. On btrfs include
xattrs, ACLs, inode flags, project and extent flags, sparse files, and
special file kinds. On ZFS include ACLs, xattr modes, and dataset filename
properties (`casesensitivity`, `normalization`, `utf8only`), because a
normalizing dataset cannot honestly store this corpus. Mark every ZFS row
untested.

Done when a later session can pick one failing row without guessing what
"comprehensive" means.

**2. Build the small hostile tree on this btrfs volume.**

A few dozen entries, not a copy of the disk. Include composed and decomposed
café names, as files and as directories, with different contents; a name
that is not valid UTF-8; an empty file; a small file; one file large enough
that a whole-file read is obvious; a hard link; a symlink; a sparse file; a
case-only pair; and one unreadable file. Keep a manifest of the raw name
bytes and payload hashes. Do not normalize or rename while creating it.

**3. Run today's `csc` copy against that tree.**

Linux to Linux only. Record kept, changed, skipped, or crashed for every
case, after a first copy, a second copy, and an interrupted copy. Record
journal bytes per entry and peak memory. This catches a whole-file buffer.
It does not prove the 100-million-file case.

**4. Repeat the fixture on the Windows 7 guest.**

Use [scripts/win-vm](../../scripts/win-vm). Create the NTFS objects in the
guest, including an alternate stream, a reparse point, an unreadable file,
and two names Windows treats as the same. Copy them with the Windows build.
Record the same kept, changed, skipped, or crashed result, and whether a
metadata or reparse failure still exits 0. If the VM is not usable, stop
and write that down. Do not replace it with a cross-compile or with
`mkfs.ntfs` on Linux.

**5. Check resume against the rewind cap.**

Interrupt a copy. Record how many files and how many bytes are redone. The
run meets the new rule if the redo stays within 500 files and the larger of
the most recent file and 1 GB, or if the tool stops and leaves a usable
journal instead of a silently shortened tree. It fails if it rewinds the
whole tree, drops a name, or reports success after a partial write. Also
note whether already committed entries are rechecked. The current journal
does recheck them. That is not by itself a violation of the rewind cap. It
is a scale risk if the recheck has to load the whole tree.

**6. Close ZFS in one paragraph.**

No host, no run, not validated on BSD or Illumos. Name VM preparation as a
later ticket. Do not start it here.

**7. Go or no-go.**

Three lists: what failed or was not run; the next single `csc` or archiver
ticket; what this milestone left alone. The next ticket is a data-loss fix
if task 3 or 4 found one. Archive framing comes after that, not before. The
Dc runtime stays closed.

This milestone is done when those lists exist and the btrfs run has a
result. A Windows or ZFS blank is a remaining gap, not a pass.