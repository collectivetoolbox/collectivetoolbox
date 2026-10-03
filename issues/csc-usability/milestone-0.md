Milestone 0 is feasible only as a one-week decision and probe, not as the start of the archive, the graph, or a Windows fix. The two-month plan is a usable scope governor. It is not a credible promise that both a Dc workspace and a preservation-qualified backup of the real disk will be done in 120–160 hours.

## Feasibility

The originating prompts locked the product direction. Those decisions should not be reopened in week 1:

- Personal use on Linux and Windows, about 15–20 hours a week, one to two months as a budget rather than a deadline.
- Two separate outcomes: a Dc workspace you can open and navigate, and a preservation tool you can trust on a named filesystem. Neither one counts as the other.
- Stop expanding format detection unless it blocks those two outcomes.
- Global nodes are permanent and append-only. Local and team graphs may actually discard deleted content, but the ID stays reserved. Deleting a revision reveals the remaining trusted revision. Hiding a global node locally is reversible. Distrust is reversible. Deleting the only retained copy is not.
- Trust, scopes, and most settings live in the local graph. Safe mode must ignore local overrides. A signature is not the same thing as trust, and trust is not permission to read a file.
- The browser may be the first screen, but the application must not depend on page loads, routes, or request/response.
- `ctoolbox csc` stays the copy tool. A separate `ctoolbox archive` command will make and extract a local, unencrypted Dc archive. Compression is outside the archive format.
- NTFS, btrfs, and ZFS matter first. Filenames are exact byte or UTF-16 sequences, never normalized text. Inode numbers, ctime, and physical layout are recorded facts, not things extraction must recreate.
- The first big backup may assume a mostly quiet source plus a final repeat sync. That is not a snapshot.
- Displaying a Dc document runs it, with only a small amount of pure computation allowed unless you grant more. Looking at raw bytes must not run the document.
- Encryption, MLS, and metadata-private sync stay out of this MVP. Week 1 only writes down the boundaries.

What does not fit the week-1 budget of 6–8 hours, or this machine:

- This workspace is btrfs. There is no ZFS tool here, and no Windows host. A cross-compiled Windows test would not qualify NTFS. The roadmap already says that. Those two captures are operator-machine tasks, not something week 1 can finish here.
- There is no filename-collision test yet. The requirement exists only in the docs.
- Windows code is ahead of the safety note, and it fails open. If opening a file handle fails, metadata capture returns success with that information missing. If reading a reparse point fails, it logs a warning and continues. Alternate data streams are read entirely into memory. Strict sandboxed directory creation is explicitly rejected on non-Unix. The safety note still says those Windows features are unimplemented and return errors. Both documents cannot be treated as current.
- A field-by-field NTFS/btrfs/ZFS matrix, a million-entry trial, and the 100-million-file backup are later evidence. Even 1 KiB of extra data per file is about 100 GB at that count. Week 1 only needs a number you are willing to live with.
- The later milestones depend on contracts that are not written yet. Starting the node builder or the archive format this week would freeze guesses.

The eight-week total of 98–136 hours is arithmetically consistent and leaves too little room for the known gaps: no archive format, a renderer that still passes strings through, Windows capture that can succeed while dropping data, and no measured resume cost. A credible four-week checkpoint is native node artifacts plus a plainly limited copy/archive preview. Full qualification of the real disk is calendar time on top of coding time.

## Week 1 task list

Do these in order. Stop when the timebox is gone and write down what was not reached. Do not fix the bugs you find unless one of them makes the measurements meaningless.

**1. Write the decision sheet, about two hours.**

One short note, not a new design. Record only what the next milestone must not guess.

Already decided, so copy them down as settled: identity and deletion rules above; Git is the ledger for published global nodes, and a node body does not change after it is first committed; new assertions are new nodes; archives are local and unencrypted; compression is not part of the format; filenames stay exact; source observations are not restoration requirements.

Still needs your numbers, not more architecture:

- After a local deletion, what non-content residue must remain so the ID cannot be reused, and what must disappear from indexes.
- How two development branches avoid allocating the same new global ID.
- The restart rule you will accept: maximum extra memory, maximum journal growth per file, and how long a resume may spend before it is doing useful new work. Pick a number you would actually shut the machine down for. A million-file trial later has to meet it.
- Which NTFS features the real disk uses: encryption, compression, sparse files, alternate streams, junctions, and anything else you know is there.
- Where a native Windows test can run, and where ZFS can run. If either machine is not available this month, say so. That is a release blocker, not a footnote.
- The security boundaries already in the security note: identity is not a key, trust is not read permission, a password hash is not a vault, the server must not see private content or usage, and revocation cannot erase a copy someone already has. One or two hours. If it starts turning into a protocol, stop.

Done when a later session can implement a node or an archive without inventing an ID rule, a deletion rule, or a success rule.

**2. Build the smallest hostile sample on this btrfs machine, about two hours.**

A few dozen files, not a copy of the disk. Include:

- Two names that look the same after Unicode normalization, with different contents, both as file names and as directory names. Use the café example already in the safety note: the composed form and the decomposed form.
- A name that is not valid UTF-8.
- An empty file, a small file, and one file large enough that a full-file read would be obvious in memory.
- A hard link pair, a symlink, a sparse file if btrfs will make one, and a name that differs only by case.
- One unreadable file, so a permission error has somewhere to go.

Do not normalize, rename, or skip anything while creating it. Keep a manifest of the raw bytes of each name and a hash of each payload.

Done when that tree can be recreated and checked without remembering the cases.

**3. Run today's copy tool against that sample and write down what it actually does, about two hours.**

Use the existing `csc` copy, not a new archive command. Check:

- Linux to Linux on this btrfs volume: every name byte and every payload still distinct after copy, after a second run, and after an interrupted run that is resumed.
- Whether a journal from the current writer can be read back, and how big it is per file.
- Peak memory while copying the large file and the small tree. A single sample is enough to catch "the whole file is in RAM." It does not prove the 100-million-file case.

Done when each case is marked kept, changed, skipped, or crashed, with the command and the result. No fixes in this step.

**4. Record the Windows and scale blockers from the code you already have, about one hour.**

Do not treat the safety note or the roadmap as the measurement. The current code already shows three blockers:

- A failed Windows handle open is reported as success without the missing metadata.
- A failed reparse read is a warning, then success.
- Each alternate stream is read with a whole-file read.
- Non-Unix strict sandboxing is rejected outright.
- The safety note's Windows section contradicts that code and needs a correction later so the next session does not "fix" a problem that has moved.

Also write the scale rule in one paragraph: resume must not load the whole tree; discovery state has to sit on disk; a changed file after restart must be reported; this tool will not claim a point-in-time snapshot.

Done when the next preservation ticket is one named bug, not "make Windows work."

**5. Operator captures, only if the machines are actually available this week.**

On a real NTFS volume: one file with an alternate stream, one reparse point, one file you cannot read, and two names Windows treats as the same. On ZFS: the same normalization pair as in task 2, plus whatever native flags that pool actually has.

If the machine is not available, the task is done by writing "not run, no host" and leaving NTFS or ZFS unqualified. Do not substitute a mock.

**6. Close the week with a go or no-go, about half an hour.**

Three lists only:

- What failed or was not run.
- The next single ticket, small enough for one or two sessions. The likely candidate is either the filename-byte copy bug, if task 3 finds one, or a fail-closed Windows capture fix. Do not start the node builder while a data-loss bug is still unnamed.
- What week 1 deliberately did not do: archive format, graph nodes, renderer, encryption, detection work, and any Windows fix not required to trust the measurement.

Week 1 has succeeded if those three lists exist. It has not succeeded if the only output is a cleaner plan with no fixture result and no named blocker.