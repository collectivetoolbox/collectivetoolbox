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
        self.assertIn("windows-x64", res.stdout)
        self.assertIn("freebsd-x64", res.stdout)
        self.assertIn("--rotate-global-pass", res.stdout)
        self.assertIn("--new-global-pass-file", res.stdout)

    def test_linux_vm_help(self):
        res = subprocess.run(
            ["./scripts/vm", "linux", "--help"],
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

    def test_invalid_target_rejected_instead_of_live_fallback(self):
        res = subprocess.run(
            ["./deploy", "--target", "nonexistent-target-xyz", "--skip-clean", "--allow-dirty"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("Invalid --target", res.stderr)

    def test_missing_config_dir_rejected(self):
        res = subprocess.run(
            ["./deploy", "--config-dir", "/tmp/nonexistent-ctb-cfg-99999", "--skip-clean", "--allow-dirty"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 1)
        self.assertIn("Configuration directory not found", res.stderr)

    def test_target_directory_reads_config_files(self):
        with tempfile.TemporaryDirectory() as cfg_dir:
            cfg = pathlib.Path(cfg_dir)
            (cfg / "ctb-deploy.username").write_text("deployer\n", encoding="utf-8")
            (cfg / "ctb-deploy.sudoer-username").write_text("sudoer\n", encoding="utf-8")
            (cfg / "ctb-deploy.server-username").write_text("server\n", encoding="utf-8")
            (cfg / "ctb-deploy.ip").write_text("127.0.0.1\n", encoding="utf-8")
            (cfg / "ctb-deploy.port").write_text("2224\n", encoding="utf-8")

            res = subprocess.run(
                [
                    "./deploy",
                    "--target",
                    cfg_dir,
                    "--ssh-key",
                    "/tmp/nonexistent-key-12345",
                    "--skip-clean",
                    "--allow-dirty",
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 1)
            self.assertIn("SSH key specified at /tmp/nonexistent-key-12345 not found", res.stderr)

    def test_missing_option_argument_rejected(self):
        for flag in ("--target", "--config-dir", "--ssh-key", "--new-global-pass-file"):
            res = subprocess.run(
                ["./deploy", "--allow-dirty", flag],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 1)
            self.assertIn("requires", res.stderr)

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
        with tempfile.NamedTemporaryFile(dir=REPO_ROOT, prefix="test_dirty_"):
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
        with tempfile.NamedTemporaryFile(dir=REPO_ROOT, prefix="test_dirty_"):
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

    def test_cross_target_binary_commit_extraction(self):
        expected_sha = "0123456789abcdef0123456789abcdef01234567"
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = pathlib.Path(tmpdir)

            # Non-executable mock PE/cross binary with single embedded ctb-build-info marker
            valid_pe = tmp / "ctoolbox.exe"
            valid_pe.write_bytes(
                b"MZ\x90\x00\x03\x00\x00\x00ctoolbox 0.1.0 (build "
                + expected_sha.encode("ascii")
                + b")\x00trailing"
            )
            res_ok = subprocess.run(
                [
                    "bash",
                    "-c",
                    'source ./scripts/run-dev-ctb && ctb_binary_commit "$1"',
                    "_",
                    str(valid_pe),
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res_ok.returncode, 0, res_ok.stderr)
            self.assertEqual(res_ok.stdout.strip(), expected_sha)

            # Ambiguous binary with two different markers must be rejected
            ambiguous_pe = tmp / "ambiguous.exe"
            ambiguous_pe.write_bytes(
                b"MZ (build 0123456789abcdef0123456789abcdef01234567) "
                b"(build fedcba9876543210fedcba9876543210fedcba98)"
            )
            res_bad = subprocess.run(
                [
                    "bash",
                    "-c",
                    'source ./scripts/run-dev-ctb && ctb_binary_commit "$1"',
                    "_",
                    str(ambiguous_pe),
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(res_bad.returncode, 0)


if __name__ == "__main__":
    unittest.main()
