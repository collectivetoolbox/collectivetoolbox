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

import pathlib
import subprocess
import tempfile
import unittest

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]


class DeployFlagsTests(unittest.TestCase):
    def test_deploy_help(self):
        res = subprocess.run(
            ["./deploy", "--help"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("--target", res.stdout)
        self.assertIn("--no-build", res.stdout)
        self.assertIn("--reuse-built", res.stdout)
        self.assertIn("--dry-run", res.stdout)
        self.assertIn("--mock-artifacts", res.stdout)
        self.assertIn("--allow-dirty", res.stdout)

    def test_linux_vm_help(self):
        res = subprocess.run(
            ["./scripts/linux-vm", "--help"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("test-deploy", res.stdout)
        self.assertIn("start", res.stdout)
        self.assertIn("snapshot", res.stdout)
        self.assertIn("ssh", res.stdout)

    def test_setup_linux_vm_image_help(self):
        res = subprocess.run(
            ["./scripts/setup-linux-vm-image", "--help"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0)
        self.assertIn("--image", res.stdout)
        self.assertIn("--url", res.stdout)

    def test_missing_vm_ssh_key_error(self):
        res = subprocess.run(
            ["./deploy", "--target", "vm", "--ssh-key", "/tmp/nonexistent-key-12345", "--allow-dirty"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("VM SSH key not found at", res.stderr)

    def test_no_build_rejects_missing_binary(self):
        with tempfile.NamedTemporaryFile() as key_file:
            res = subprocess.run(
                [
                    "./deploy",
                    "--target", "vm",
                    "--ssh-key", key_file.name,
                    "--no-build",
                    "--allow-dirty",
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(res.returncode, 0)
            self.assertTrue(
                "Cannot use --no-build" in res.stderr
                or "Stale binary rejected" in res.stderr
                or "Error: Binary for target" in res.stderr,
                res.stderr,
            )

    def test_dirty_working_tree_rejected(self):
        with tempfile.NamedTemporaryFile(dir=REPO_ROOT, prefix="test_dirty_") as tmp_file:
            res = subprocess.run(
                ["./deploy", "--target", "vm"],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 1)
            self.assertIn("unstaged or uncommitted changes", res.stderr)

    def test_dirty_working_tree_allowed_with_flag(self):
        with tempfile.NamedTemporaryFile(dir=REPO_ROOT, prefix="test_dirty_") as tmp_file:
            res = subprocess.run(
                ["./deploy", "--target", "vm", "--allow-dirty", "--ssh-key", "/tmp/nonexistent-key-12345"],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 1)
            self.assertNotIn("unstaged or uncommitted changes", res.stderr)
            self.assertIn("VM SSH key not found at", res.stderr)


if __name__ == "__main__":
    unittest.main()
