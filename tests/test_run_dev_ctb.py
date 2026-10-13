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

import json
import pathlib
import shutil
import subprocess
import tempfile
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]

BUILD_SCRIPT = """#!/usr/bin/env python3
import json
import pathlib
import subprocess
import sys
root = pathlib.Path(__file__).resolve().parent
(root / "built").mkdir(exist_ok=True)
with (root / "built/build-calls.jsonl").open("a") as calls:
    calls.write(json.dumps(sys.argv[1:]) + "\\n")
if (root / "fail-build").exists():
    sys.exit(42)
commit = "unavailable"
if (root / ".git").exists():
    commit = subprocess.check_output(
        ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
    ).strip()
if (root / "wrong-commit").exists():
    commit = "f" * 40
binary = root / "built" / sys.argv[-1] / "ctoolbox"
binary.parent.mkdir(parents=True, exist_ok=True)
binary.write_text(
    (root / "binary-template").read_text().replace("BUILD_COMMIT", commit)
)
binary.chmod(0o755)
print("build output must not contaminate stdout")
"""

BINARY_SCRIPT = """#!/usr/bin/env python3
import json
import os
import sys
if sys.argv[1:] == ["--version-json"]:
    print(json.dumps({"name": "ctoolbox", "commit": "BUILD_COMMIT"}))
else:
    print(json.dumps({"args": sys.argv[1:], "cwd": os.getcwd()}))
    sys.exit(7 if "--fail-command" in sys.argv else 0)
"""


class RunDevCtbTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="ctb runner ")
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / "scripts").mkdir()
        self.runner = self.root / "scripts/run-dev-ctb"
        shutil.copy2(REPO_ROOT / "scripts/run-dev-ctb", self.runner)
        self.runner.chmod(0o755)
        shutil.copy2(
            REPO_ROOT / "scripts/compile-freebsd-sysroot",
            self.root / "scripts/compile-freebsd-sysroot",
        )
        build = self.root / "build"
        build.write_text(BUILD_SCRIPT)
        build.chmod(0o755)
        (self.root / "binary-template").write_text(BINARY_SCRIPT)
        (self.root / ".gitignore").write_text("/built/\n/target/\n")
        self.git("init", "-q")
        self.git("add", ".")
        self.git(
            "-c", "user.name=Runner Tests", "-c", "user.email=runner@example.invalid",
            "commit", "-qm", "fixture",
        )
        self.head = self.git("rev-parse", "HEAD").stdout.strip()

    def git(self, *args):
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            text=True, capture_output=True, check=True,
        )

    def binary(self, relative="built/linux-x64/ctoolbox", commit=None, content=None):
        binary = self.root / relative
        binary.parent.mkdir(parents=True, exist_ok=True)
        binary.write_text(
            content if content is not None else BINARY_SCRIPT.replace(
                "BUILD_COMMIT", self.head if commit is None else commit
            )
        )
        binary.chmod(0o755)
        return binary

    def run_ctb(self, *args, cwd=None):
        return subprocess.run(
            [str(self.runner), *args],
            cwd=self.root if cwd is None else cwd,
            text=True, capture_output=True, check=False, timeout=30,
        )

    def builds(self):
        calls = self.root / "built/build-calls.jsonl"
        return [json.loads(line) for line in calls.read_text().splitlines()] if calls.exists() else []

    def test_current_build_preserves_arguments_cwd_and_stdout(self):
        self.binary()
        args = ["csc", "path with spaces", "*", "", "--no-progress"]
        result = self.run_ctb(*args, cwd=self.root.parent)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), {"args": args, "cwd": str(self.root.parent)})
        self.assertEqual(self.builds(), [])

    def test_missing_build_builds_native_target_and_propagates_offline(self):
        result = self.run_ctb("--offline", "--", "help")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["args"], ["help"])
        self.assertEqual(
            self.builds(), [["--quick", "--no-test", "--release", "--offline", "linux-x64"]]
        )

    def test_stale_build_does_not_hide_current_later_candidate(self):
        self.binary(commit="f" * 40)
        current = self.binary("target/debug/ctoolbox")
        result = self.run_ctb("--resolve")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), str(current))
        self.assertEqual(self.builds(), [])
        self.assertIn("Skipping stale", result.stderr)

    def test_stale_or_unrelated_commit_rebuilds(self):
        self.binary(commit="f" * 40)
        result = self.run_ctb("--resolve")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.builds()), 1)

    def test_invalid_or_failing_metadata_rebuilds(self):
        for content in [
            "#!/bin/sh\nprintf '%s\\n' 'not JSON'\n",
            '#!/bin/sh\nprintf \'%s\\n\' \'{"name":"ctoolbox","commit":123}\'\n',
            '#!/bin/sh\nprintf \'%s\\n\' \'{"commit":"' + self.head + '"}\'\n',
            "#!/bin/sh\nexit 3\n",
        ]:
            with self.subTest(content=content):
                self.binary(content=content)
                previous = len(self.builds())
                result = self.run_ctb("--resolve")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(len(self.builds()), previous + 1)

    def test_dirty_tracked_and_untracked_files_force_build(self):
        self.binary()
        (self.root / "binary-template").write_text(BINARY_SCRIPT + "\n")
        result = self.run_ctb("--resolve")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.builds()), 1)
        (self.root / "binary-template").write_text(BINARY_SCRIPT)
        (self.root / "untracked-source").write_text("new source")
        result = self.run_ctb("--resolve")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.builds()), 2)

    def test_build_failure_never_runs_stale_binary(self):
        self.binary(commit="f" * 40)
        (self.root / "fail-build").touch()
        result = self.run_ctb("help")
        self.assertEqual(result.returncode, 42, result.stderr)
        self.assertEqual(result.stdout, "")

    def test_rebuild_must_report_current_commit(self):
        (self.root / "wrong-commit").touch()
        result = self.run_ctb("help")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("does not match", result.stderr)

    def test_command_exit_code_is_preserved(self):
        self.binary()
        result = self.run_ctb("--fail-command")
        self.assertEqual(result.returncode, 7, result.stderr)

    def test_resolve_rejects_command_arguments(self):
        result = self.run_ctb("--resolve", "help")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.builds(), [])

    def test_no_git_history_rebuilds_instead_of_trusting_binary(self):
        shutil.rmtree(self.root / ".git")
        self.binary(commit="f" * 40)
        result = self.run_ctb("help")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.builds()), 1)
        self.assertEqual(json.loads(result.stdout)["args"], ["help"])

    def test_shared_metadata_reader_can_be_sourced(self):
        binary = self.binary()
        result = subprocess.run(
            ["bash", "-c", 'source "$1"; ctb_binary_commit "$2"', "_", str(self.runner), str(binary)],
            text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), self.head)
        self.assertEqual(self.builds(), [])

    def test_freebsd_compiler_uses_shared_runner(self):
        result = subprocess.run(
            ["bash", str(self.root / "scripts/compile-freebsd-sysroot"), "--offline", "--jobs", "2"],
            cwd=self.root,
            text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            json.loads(result.stdout)["args"],
            ["python", str(self.root / "scripts/compile-freebsd-sysroot.py"), "--jobs", "2"],
        )
        self.assertEqual(
            self.builds(), [["--quick", "--no-test", "--release", "--offline", "linux-x64"]]
        )


if __name__ == "__main__":
    unittest.main()
