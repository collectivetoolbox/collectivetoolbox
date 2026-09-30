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

import csv
from decimal import Decimal
import gzip
import io
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


RUNNER = Path(__file__).resolve().parents[1] / "src/formats/compression/fixtures/test-compressors"
SOURCE = RUNNER.read_text()
CTB_HELPERS = SOURCE[SOURCE.index("ctb_compress() {"):SOURCE.index("tool_compress() {")]
WORKERS = SOURCE[SOURCE.index("record_result() {"):SOURCE.index('RUN_DIR="$TMP_DIR"')]
DISPATCH = SOURCE[SOURCE.index('RUN_DIR="$TMP_DIR"'):]

MOCKS = r'''
mock_ctb() {
    case "$1" in
        compress) mock_compress "$2" "$3" "$5" ;;
        decompress) mock_decompress "$2" "$3" "$5" ;;
        *) return 26 ;;
    esac
}
mock_compress() {
    printf 'start %s\n' "$BASHPID" >> "$RUN_DIR/events"
    if [[ "$MODE" == compress-fail ]]; then return 23; fi
    mkdir -p "$TMP_DIR"
    cp "$2" "$TMP_DIR/scratch"
    gzip -n -c < "$2" > "$3"
    printf 'end %s\n' "$BASHPID" >> "$RUN_DIR/events"
}
tool_decompress() {
    if [[ "$MODE" == decompress-fail ]]; then return 24; fi
    gzip -d -c < "$2" > "$3"
    if [[ "$MODE" == forward-corrupt ]]; then printf corrupt > "$3"; fi
    if [[ "$direction" == Forward ]]; then cmp "$in_path" "$TMP_DIR/scratch"; fi
}
tool_compress() {
    if [[ "$MODE" == reverse-fail ]]; then return 25; fi
    if [[ "$MODE" == missing-output ]]; then return; fi
    if [[ "$MODE" == skip ]]; then
        REVERSE_SKIP_REASON='explicit oracle refusal, "unchanged"'
        return
    fi
    gzip -n -c < "$2" > "$3"
}
mock_decompress() {
    gzip -d -c < "$2" > "$3"
    if [[ "$MODE" == reverse-corrupt ]]; then printf corrupt > "$3"; fi
}
'''


class CompressorRunnerTests(unittest.TestCase):
    def report_rows(self, output):
        report = output[output.index("Status,Direction,"):].split("\n\n===", 1)[0]
        rows = list(csv.reader(io.StringIO(report), strict=True))
        self.assertTrue(all(len(row) == 10 for row in rows), rows)
        return rows[1:]

    def test_concurrency_limit_with_blocked_workers(self):
        scheduler = DISPATCH[:DISPATCH.index('echo "=== Running Compression Compatibility Tests')]
        setup = r'''
set -euo pipefail
declare -A ACTIVE_JOBS=()
mkfifo "$TMP_DIR/release" "$TMP_DIR/ready"
exec 8<> "$TMP_DIR/release" 9<> "$TMP_DIR/ready"
'''
        exercise = r'''
blocked_case() {
    printf '%s\n' "$BASHPID" >&9
    IFS= read -r -n 1 -t 5 -u 8 release
}
(
    for index in 1 2 3 4; do IFS= read -r -t 5 -u 9 ready; done
    if IFS= read -r -t 0.1 -u 9 unexpected; then
        echo "More than four workers started before a slot was released" >&2
        exit 1
    fi
    printf xxxx >&8
    IFS= read -r -t 5 -u 9 ready
    printf x >&8
) &
controller_pid=$!
for index in 1 2 3 4 5; do schedule_case blocked_case; done
while [[ "${#ACTIVE_JOBS[@]}" -gt 0 ]]; do wait_for_case; done
wait "$controller_pid"
[[ "$FAILED_CASES" -eq 0 && "$CASE_COUNT" -eq 5 ]]
'''
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                ["bash"], input=setup + scheduler + exercise, text=True,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                env={**os.environ, "TMP_DIR": directory, "JOBS": "4"}, timeout=20,
            )
        self.assertEqual(result.returncode, 0, result.stdout)

    def run_runner(self, jobs=4, mode="pass", datasets=8, fmt="gzip", archives=False,
                   dataset_filter="name", fixture_state="valid"):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            work = root / "work"
            work.mkdir()
            (work / "unused-tool").touch()
            raw_fixture = root / "example2 with lemurs.pan"
            raw_fixture.write_bytes(b"fixture data")
            fixture = root / (raw_fixture.name + ".gz")
            if fixture_state != "missing":
                fixture.write_bytes(gzip.compress(b"fixture data" if fixture_state == "valid" else b"wrong data"))
            for index in range(datasets):
                dataset_dir = root / f"input_{index}"
                dataset_dir.mkdir()
                (dataset_dir / 'same, "name".bin').write_bytes(
                    b"" if fmt == "compact" else bytes([index]) * (8192 * (index + 1))
                )
            if archives:
                archive_dir = root / "old/archivers/ancient/testing/test_files"
                archive_dir.mkdir(parents=True)
                for compressed, raw in (("test_C1_sco.Z", "test_C1.raw"), ("test_C2_sco.Z", "test_C2.xm")):
                    content = raw.encode() * 10
                    (archive_dir / raw).write_bytes(content)
                    (archive_dir / compressed).write_bytes(gzip.compress(content))
            setup = r'''
set -euo pipefail
declare -A ACTIVE_JOBS=()
DATASETS=("$REPO_ROOT"/input_*/*)
if [[ "$DATASET_COUNT" == 0 ]]; then DATASETS=(); fi
HIST_BINARIES=("$TMP_DIR/unused-tool")
CTB_CMD=(mock_ctb)
TEST_MATRIX=("$FORMAT|gzip|.gz|1|1|gzip|gunzip")
'''
            result = subprocess.run(
                ["bash"], input=setup + CTB_HELPERS + WORKERS + MOCKS + DISPATCH,
                text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                env={**os.environ, "REPO_ROOT": str(root), "TMP_DIR": str(work),
                     "JOBS": str(jobs), "MODE": mode, "FILTER": fmt,
                     "FORMAT": fmt, "DATASET_FILTER": dataset_filter, "DATASET_COUNT": str(datasets),
                     "RAW_FILE": str(raw_fixture)},
                timeout=30,
            )
            events = work / "events"
            return result, events.read_text().splitlines() if events.exists() else []

    def test_serial_parallel_coverage_sizes_and_isolation(self):
        serial, serial_events = self.run_runner(jobs=1)
        parallel, parallel_events = self.run_runner(jobs=4)
        for result in (serial, parallel):
            self.assertEqual(result.returncode, 0, result.stdout)
            self.assertIn("16 COMPRESSION COMPATIBILITY TESTS PASSED; 0 explicit skips; 0 failed cases", result.stdout)
        def normalized_rows(output):
            return [row[:4] + row[5:8] for row in self.report_rows(output) if row[0] == "PASS"]
        self.assertEqual(normalized_rows(serial.stdout), normalized_rows(parallel.stdout))
        for events, limit in ((serial_events, 1), (parallel_events, 4)):
            active = set()
            for event in events:
                action, pid = event.split()
                if action == "start":
                    active.add(pid)
                else:
                    active.remove(pid)
                self.assertLessEqual(len(active), limit)
            self.assertFalse(active)
        for fields in self.report_rows(parallel.stdout):
            if fields[0] == "PASS":
                self.assertIn('same, "name".bin', fields[4])
                self.assertAlmostEqual(float(fields[7]), int(fields[6]) / int(fields[5]), places=4)
                self.assertGreaterEqual(float(fields[8]), 0)

    def test_failures_cannot_be_reported_as_success(self):
        for mode in ("compress-fail", "decompress-fail", "forward-corrupt",
                     "reverse-fail", "reverse-corrupt", "missing-output"):
            with self.subTest(mode=mode):
                result, _ = self.run_runner(mode=mode)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(sum(row[0] == "FAIL" for row in self.report_rows(result.stdout)), 8, result.stdout)
                self.assertIn("8 failed cases", result.stdout)

    def test_skip_survives_timing_wrapper(self):
        result, _ = self.run_runner(mode="skip")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("8 COMPRESSION COMPATIBILITY TESTS PASSED; 8 explicit skips; 0 failed cases", result.stdout)
        for row in self.report_rows(result.stdout):
            if row[0] == "SKIP":
                self.assertEqual(row[9], 'explicit oracle refusal, "unchanged"')
            elif row[0] == "SUMMARY" and row[3] == "gzip":
                self.assertEqual(row[5:9], ["-"] * 4)
                self.assertEqual(row[9], "0 passed; 8 skipped; 0 failed")

    def test_compressor_summaries_follow_fixture_rows(self):
        result, _ = self.run_runner(dataset_filter="")
        self.assertEqual(result.returncode, 0, result.stdout)
        rows = self.report_rows(result.stdout)
        summaries = [row for row in rows if row[0] == "SUMMARY"]
        self.assertEqual(rows[-2:], summaries)
        self.assertEqual({row[3] for row in summaries}, {"ctoolbox", "gzip"})
        for summary in summaries:
            direction = "Forward" if summary[3] == "ctoolbox" else "Reverse"
            cases = [row for row in rows if row[:2] == ["PASS", direction]]
            raw = sum(int(row[5]) for row in cases)
            compressed = sum(int(row[6]) for row in cases)
            seconds = sum(Decimal(row[8]) for row in cases)
            self.assertEqual(int(summary[5]), raw)
            self.assertEqual(int(summary[6]), compressed)
            self.assertEqual(summary[7], f"{compressed / raw:.4f}")
            self.assertEqual(Decimal(summary[8]), seconds)
            self.assertEqual(summary[9], "8 passed; 0 skipped; 0 failed")

    def test_empty_historical_inputs_still_skip(self):
        result, _ = self.run_runner(fmt="compact", datasets=2)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("0 COMPRESSION COMPATIBILITY TESTS PASSED; 2 explicit skips", result.stdout)

    def test_no_matching_cases_fails(self):
        result, _ = self.run_runner(datasets=0)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("No compression compatibility tests ran", result.stdout)

    def test_archived_comparisons_and_missing_corpus(self):
        result, _ = self.run_runner(fmt="sco-compress", archives=True, dataset_filter="")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(sum(row[:2] == ["PASS", "Archived"] for row in self.report_rows(result.stdout)), 6)
        self.assertIn("22 COMPRESSION COMPATIBILITY TESTS PASSED", result.stdout)
        missing, _ = self.run_runner(fmt="sco-compress", dataset_filter="")
        self.assertNotEqual(missing.returncode, 0, missing.stdout)
        self.assertIn("18 COMPRESSION COMPATIBILITY TESTS PASSED; 0 explicit skips; 1 failed cases", missing.stdout)

    def test_committed_fixture_decoded_by_both_tools(self):
        result, _ = self.run_runner(datasets=0, dataset_filter="lemurs")
        self.assertEqual(result.returncode, 0, result.stdout)
        rows = self.report_rows(result.stdout)
        self.assertEqual(len(rows), 2)
        self.assertEqual({row[3] for row in rows}, {"ctoolbox", "gunzip"})
        self.assertTrue(all(row[:2] == ["PASS", "Archived"] and row[8] == "-" for row in rows))

    def test_missing_or_corrupt_committed_fixture_fails(self):
        for state in ("missing", "corrupt"):
            with self.subTest(state=state):
                result, _ = self.run_runner(datasets=0, dataset_filter="lemurs", fixture_state=state)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(self.report_rows(result.stdout)[0][0], "FAIL")
                self.assertIn("1 failed cases", result.stdout)


if __name__ == "__main__":
    unittest.main()