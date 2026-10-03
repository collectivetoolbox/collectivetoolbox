# FreeBSD Sysroot

## Commands

First, download the candidate files and create a review list:

```sh
python3 -B scripts/download-freebsd-sysroot.py --prepare-manifest built/freebsd-review.json
```

Then review them one by one:

```sh
python3 -B scripts/download-freebsd-sysroot.py --review-manifest built/freebsd-review.json
```

Finally, generate the sysroot from the approved files:

```sh
python3 -B scripts/download-freebsd-sysroot.py --manifest built/freebsd-review.json
```

`--manifest` reads an existing file. It does not create one. Every flag above
needs the filename after it. You do not need to write the JSON yourself.

## Review Prompts

For each pending file, choose:

- `v`: display the entire file.
- `a`: approve it after checking all its contents and applicable license terms.
- `r`: reject it and record why.
- `s`: leave it pending and move to the next file.
- `q`: save progress and quit. Enter also quits.

Run the same `--review-manifest` command to resume. It skips saved decisions.
Preparation never approves a file, even when it has a familiar SPDX tag.

Approval asks for all applicable license IDs, your name and review evidence,
confirmation that there are no binary blobs, and required companion notices.
For example, license IDs may be `BSD-2-Clause` or
`BSD-2-Clause, Apache-2.0 WITH LLVM-exception`. Check the full terms first;
these are input examples, not blanket approval for upstream files.

Companion notice paths are relative to that file's upstream repository, such
as `libc/LICENSE.TXT`. A notice not already in the list is added for review.
Leave this answer empty only when you have checked that no companion notices
are required. Review covers directory-level terms and third-party material too.
If uncertain about any material, do not approve it.

`GPL-3.0-only` and `AGPL-3.0-only` are prohibited here. Unsupported SPDX
expressions are also rejected rather than guessed at. Text checks do not detect
every encoded binary: the reviewer must check for firmware arrays and other
blobs disguised as source.

## Generated Files

The original FreeBSD/LLVM merging and synthetic-header transformations are
retained. Their inputs must now be individually approved and hash-pinned.
Generation stops if a required input is missing or rejected.

Original sources are preserved under `src/`, and companion notices under
`share/licenses/`. Derived headers include commented copies of every contributing
source and its companion notices. This deliberately preserves more than just
the first license comment, including later copyright statements. It also makes
derived headers larger. Generated definitions carry the generator's own
AGPL-3.0-or-later notice. `header-provenance.json` records the contributors.

No compiler runs in this script. Binary files and symlinks are excluded.
Currently only UTF-8 text is supported. The entire staged tree is validated
before either existing sysroot is replaced.

## Pending Work

The upstream files still need actual license review. These changes have not
regenerated or certified the existing vendored sysroots. The restored generator
has local fixture tests, but its FreeBSD VM tests have not been rerun here.
The separate compilation script has not been changed; build integration must
compile reviewed sources into build output, not the source-only vendor trees.

Run the local tests with:

```sh
python3 -B -m unittest discover -s tests -p test_freebsd_sysroot.py -v
```