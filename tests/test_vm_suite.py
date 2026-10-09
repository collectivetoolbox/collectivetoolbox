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


class VmSuiteProtocolTests(unittest.TestCase):
    def run_stream(self, serial, expected_total=1, command=None):
        script = pathlib.Path(__file__).resolve().parents[1] / "scripts" / "vm-common.sh"
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            serial_path = root / "serial.log"
            serial_path.write_text(serial)
            return subprocess.run(
                ["bash", "-s", "--", str(script), str(serial_path),
                 str(root / "suite.log"), str(expected_total)],
                input='source "$1"\n' + (command or 'vm_stream_suite_output "$2" 0 0 "$3" test "$4"') + '\n',
                capture_output=True,
                text=True,
                check=False,
            )

    def test_valid_suite(self):
        result = self.run_stream(
            "---SUITE_START---\n=== PASS example ===\n"
            "---SUITE_END:TOTAL=1,PASSED=1,FAILED=0---\n"
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_failed_executable(self):
        result = self.run_stream(
            "---SUITE_START---\n=== FAIL example [EXIT:7] ===\n"
            "---SUITE_END:TOTAL=1,PASSED=0,FAILED=1---\n"
        )
        self.assertEqual(result.returncode, 1)
        self.assertIn("example (exit code: 7)", result.stdout)

    def test_missing_startup(self):
        result = self.run_stream("")
        self.assertEqual(result.returncode, 124)
        self.assertIn("launcher did not produce", result.stdout)

    def test_completion_without_startup(self):
        result = self.run_stream("---SUITE_END:TOTAL=1,PASSED=1,FAILED=0---\n")
        self.assertEqual(result.returncode, 1)

    def test_incomplete_completion_is_not_success(self):
        result = self.run_stream(
            "---SUITE_START---\n=== PASS example ===\n"
            "---SUITE_END:TOTAL=1,PASSED=1,FAILED=0---"
        )
        self.assertEqual(result.returncode, 124)

    def test_invalid_completion_counts(self):
        for completion in (
            "---SUITE_END:TOTAL=0,PASSED=0,FAILED=0---",
            "---SUITE_END:TOTAL=1,PASSED=1,FAILED=0---",
            "---SUITE_END:TOTAL=1,PASSED=0,FAILED=0---",
            "---SUITE_END:invalid---",
        ):
            with self.subTest(completion=completion):
                result = self.run_stream("---SUITE_START---\n" + completion + "\n")
                self.assertEqual(result.returncode, 1)

    def test_packaged_binary_count_mismatch(self):
        result = self.run_stream(
            "---SUITE_START---\n=== PASS example ===\n"
            "---SUITE_END:TOTAL=1,PASSED=1,FAILED=0---\n",
            expected_total=2,
        )
        self.assertEqual(result.returncode, 1)

    def test_startup_marker_wait(self):
        result = self.run_stream(
            "unrelated output\n---SUITE_START---\n",
            command='vm_wait_for_marker "$2" 0 ---SUITE_START--- 0',
        )
        self.assertEqual(result.returncode, 0)

    def test_startup_marker_wait_rejects_unrelated_output(self):
        result = self.run_stream(
            "unrelated output\n",
            command='vm_wait_for_marker "$2" 0 ---SUITE_START--- 0',
        )
        self.assertEqual(result.returncode, 1)

    def test_startup_marker_wait_rejects_incomplete_line(self):
        result = self.run_stream(
            "---SUITE_START---",
            command='vm_wait_for_marker "$2" 0 ---SUITE_START--- 0',
        )
        self.assertEqual(result.returncode, 1)


if __name__ == "__main__":
    unittest.main()