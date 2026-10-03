#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# This file is part of Collective Toolbox, a database and document workspace and utilities.
# Copyright (C) 2026 Collective Toolbox Developers
# Contact: info@collectivetoolbox.com
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along
# with this program.  If not, see <https://www.gnu.org/licenses/>.

"""Copy individually reviewed upstream sources, without synthesizing ABI headers."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tempfile
import unicodedata


WORKSPACE_ROOT = Path(__file__).resolve().parents[1]
REPOSITORIES = {
    "freebsd": (
        "https://github.com/freebsd/freebsd-src.git",
        "c876eaf40915f213dc07f74e730ee6a2a9de7ba1",
    ),
    "llvm": (
        "https://github.com/llvm/llvm-project.git",
        "92ee37966563310901951b1b5a89bcbafbcea767",
    ),
}
OUTPUTS = (
    WORKSPACE_ROOT / "vendor/upstream-for-reference/freebsd-sysroot",
    WORKSPACE_ROOT / "vendor/freebsd-sysroot",
)
ALLOWED_LICENSES = frozenset({
    "BSD-1-Clause", "BSD-2-Clause", "BSD-3-Clause", "0BSD", "ISC", "MIT",
    "MIT-CMU", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "Zlib",
    "CC0-1.0", "Unlicense", "GPL-3.0-only", "GPL-3.0-or-later",
    "AGPL-3.0-only", "AGPL-3.0-or-later",
})
SOURCE_SUFFIXES = frozenset({".h", ".c", ".S", ".s", ".inc", ".def", ".yaml"})
BINARY_SIGNATURES = (
    b"\x7fELF", b"!<arch>\n", b"!<thin>\n", b"MZ", b"\x89PNG", b"GIF87a",
    b"GIF89a", b"%PDF-", b"PK\x03\x04", b"\x1f\x8b", b"\xfd7zXZ",
    b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe", b"\xfe\xed\xfa\xcf",
    b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe",
)


def relative_path(value):
    if not isinstance(value, str) or not value:
        raise ValueError("Expected a nonempty relative POSIX path")
    path = PurePosixPath(value)
    if path.is_absolute() or any(part in {".", ".."} for part in value.split("/")):
        raise ValueError(f"Unsafe path: {value!r}")
    if str(path) != value or "\\" in value or any(ord(char) < 32 for char in value):
        raise ValueError(f"Noncanonical path: {value!r}")
    return path


def unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def load_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_keys)
    if not isinstance(manifest, dict) or set(manifest) != {"version", "commits", "files"}:
        raise ValueError("Manifest must contain exactly version, commits, and files")
    if type(manifest["version"]) is not int or manifest["version"] != 1:
        raise ValueError("Unsupported manifest version")
    if manifest["commits"] != {name: commit for name, (_, commit) in REPOSITORIES.items()}:
        raise ValueError("Manifest must match both pinned upstream commits")
    entries = manifest["files"]
    if not isinstance(entries, list) or not entries:
        raise ValueError("No reviewed files: refusing to generate an empty sysroot")
    destinations = set()
    required_keys = {
        "repository", "path", "destination", "sha256", "kind", "licenses",
        "review", "source_only_reviewed", "notices",
    }
    for entry in entries:
        if not isinstance(entry, dict) or set(entry) != required_keys:
            raise ValueError(f"File entry must contain exactly {sorted(required_keys)}")
        if not isinstance(entry["repository"], str) or entry["repository"] not in REPOSITORIES:
            raise ValueError("Unknown upstream repository")
        relative_path(entry["path"])
        destination = relative_path(entry["destination"])
        if destination in destinations:
            raise ValueError(f"Duplicate destination: {destination}")
        if any(other in destination.parents or destination in other.parents for other in destinations):
            raise ValueError(f"Overlapping destinations: {destination}")
        destinations.add(destination)
        if not isinstance(entry["sha256"], str) or not re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]):
            raise ValueError("Each file requires its reviewed SHA-256 digest")
        licenses = entry["licenses"]
        if not isinstance(licenses, list) or not licenses or any(
            not isinstance(license_id, str) or license_id not in ALLOWED_LICENSES
            for license_id in licenses
        ):
            raise ValueError(f"Unknown or incompatible reviewed license: {licenses!r}")
        if not isinstance(entry["review"], str) or not entry["review"].strip():
            raise ValueError("Each file requires an explicit license review reference")
        if entry["source_only_reviewed"] is not True:
            raise ValueError("Each file must be reviewed for embedded binary blobs")
        if entry["kind"] == "source":
            if destination.suffix not in SOURCE_SUFFIXES:
                raise ValueError(f"Not a supported source filename: {destination}")
            if not (destination.is_relative_to("usr/include") or destination.is_relative_to("src")):
                raise ValueError("Source destinations must be under usr/include or src")
            if destination.is_relative_to("usr/include") and destination.suffix != ".h":
                raise ValueError("Only headers may be installed under usr/include")
        elif entry["kind"] == "notice":
            if not destination.is_relative_to("share/licenses"):
                raise ValueError("License notices must be installed under share/licenses")
        else:
            raise ValueError("Only source and notice files are permitted")
        if not isinstance(entry["notices"], list):
            raise ValueError("notices must list all required companion notice files")
        for notice in entry["notices"]:
            relative_path(notice)
    notices = {
        (entry["repository"], entry["path"])
        for entry in entries if entry["kind"] == "notice"
    }
    for entry in entries:
        if any((entry["repository"], path) not in notices for path in entry["notices"]):
            raise ValueError(f"Missing reviewed companion notice for {entry['path']}")
    return manifest


def validate_bytes(entry, raw):
    if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
        raise ValueError(f"Reviewed digest mismatch: {entry['path']}")
    if raw.startswith(BINARY_SIGNATURES):
        raise ValueError(f"Binary file: {entry['path']}")
    text = raw.decode("utf-8", errors="strict")
    if not text.strip() or any(
        unicodedata.category(char) == "Cc" and char not in "\t\r\n\f"
        for char in text
    ):
        raise ValueError(f"Empty or non-text file: {entry['path']}")
    for expression in re.findall(r"SPDX-License-Identifier:[ \t]*([^\r\n]*)", text):
        expression = expression.strip()
        if expression.endswith("*/"):
            expression = expression[:-2].rstrip()
        terms = expression.split(" AND ")
        if any(term not in entry["licenses"] for term in terms):
            raise ValueError(f"Unreviewed SPDX expression in {entry['path']}: {expression}")


def git(repository, *arguments):
    return subprocess.run(
        ["git", "--git-dir", str(repository), *arguments],
        check=True, stdout=subprocess.PIPE,
    ).stdout


def read_blob(repository, commit, path):
    records = git(repository, "ls-tree", "-z", commit, "--", f":(literal){path}").split(b"\0")
    records = [record for record in records if record]
    if len(records) != 1:
        raise ValueError(f"Expected one tracked file: {path}")
    metadata, filename = records[0].split(b"\t", 1)
    mode, kind, object_id = metadata.split()
    if mode != b"100644" or kind != b"blob" or filename.decode("utf-8") != path:
        raise ValueError(f"Not a regular, non-executable upstream source file: {path}")
    return git(repository, "cat-file", "blob", object_id.decode("ascii"))


def stage_sysroot(manifest, repositories, destination):
    for entry in manifest["files"]:
        commit = REPOSITORIES[entry["repository"]][1]
        raw = read_blob(repositories[entry["repository"]], commit, entry["path"])
        validate_bytes(entry, raw)
        target = destination / entry["destination"]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(raw)
    (destination / "reviewed-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8",
    )


def publish(staged, outputs):
    for output in outputs:
        if output.is_symlink() or (output.exists() and not output.is_dir()):
            raise ValueError(f"Refusing to replace a non-directory output: {output}")
        if any(parent.is_symlink() for parent in output.parents):
            raise ValueError(f"Refusing a symlinked output parent: {output}")
    for output in outputs:
        output.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix=".freebsd-sysroot-", dir=output.parent) as temporary:
            replacement = Path(temporary) / "sysroot"
            shutil.copytree(staged, replacement)
            backup = Path(temporary) / "previous"
            if output.exists():
                output.rename(backup)
            try:
                replacement.rename(output)
            except OSError:
                if backup.exists():
                    backup.rename(output)
                raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest", type=Path, required=True,
        help="Hash-pinned, file-level license review (see docs/freebsd-sysroot.md)",
    )
    arguments = parser.parse_args(argv)
    manifest = load_manifest(arguments.manifest)
    if not shutil.which("git"):
        raise ValueError("git is required to download reviewed upstream sources")
    build_dir = WORKSPACE_ROOT / "built"
    build_dir.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="freebsd-sysroot-", dir=build_dir) as temporary:
        temporary = Path(temporary)
        repositories = {}
        for name, (url, commit) in REPOSITORIES.items():
            repository = temporary / name
            git(repository, "init", "--bare")
            git(repository, "remote", "add", "origin", url)
            git(repository, "config", "remote.origin.promisor", "true")
            git(repository, "config", "remote.origin.partialclonefilter", "blob:none")
            git(repository, "fetch", "--depth=1", "--filter=blob:none", "origin", commit)
            resolved = git(repository, "rev-parse", "FETCH_HEAD").decode("ascii").strip()
            if resolved != commit:
                raise ValueError(f"Upstream commit mismatch: {name}")
            repositories[name] = repository
        staged = temporary / "sysroot"
        staged.mkdir()
        stage_sysroot(manifest, repositories, staged)
        publish(staged, OUTPUTS)
    print("Reviewed source files and notices copied verbatim; no binaries or derived headers generated.")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(f"error: {error}") from error