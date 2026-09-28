import gzip
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


RUNNER = Path(__file__).resolve().parents[1] / "src/formats/compression/data/fixtures/test-compressors"
SOURCE = RUNNER.read_text()
CTB_HELPERS = SOURCE[SOURCE.index("ctb_compress() {"):SOURCE.index("tool_compress() {")]
WORKERS = SOURCE[SOURCE.index("record_result() {"):SOURCE.index('RUN_DIR="$TMP_DIR"')]
DISPATCH = SOURCE[SOURCE.index('RUN_DIR="$TMP_DIR"'):]

MOCKS = r'''
mock_ctb() {
    [[ "$CTB_TEST_STORAGE_DIR" == "$TMP_DIR/storage" ]]
    case "$1" in
        compress) mock_compress "$2" "$3" "$5" ;;
        decompress) mock_decompress "$2" "$3" "$5" ;;
        *) return 26 ;;
    esac
}
mock_compress() {
    printf 'start %s\n' "$BASHPID" >> "$RUN_DIR/events"
    if [[ "$MODE" == compress-fail ]]; then return 23; fi
    mkdir -p "$CTB_TEST_STORAGE_DIR"
    cp "$2" "$CTB_TEST_STORAGE_DIR/scratch"
    gzip -n -c < "$2" > "$3"
    printf 'end %s\n' "$BASHPID" >> "$RUN_DIR/events"
}
tool_decompress() {
    if [[ "$MODE" == decompress-fail ]]; then return 24; fi
    gzip -d -c < "$2" > "$3"
    if [[ "$MODE" == forward-corrupt ]]; then printf corrupt > "$3"; fi
    cmp "$in_path" "$TMP_DIR/storage/scratch"
}
tool_compress() {
    if [[ "$MODE" == reverse-fail ]]; then return 25; fi
    if [[ "$MODE" == missing-output ]]; then return; fi
    if [[ "$MODE" == skip ]]; then
        REVERSE_SKIP_REASON="explicit oracle refusal"
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

    def run_runner(self, jobs=4, mode="pass", datasets=8, fmt="gzip", archives=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            work = root / "work"
            work.mkdir()
            (work / "unused-tool").touch()
            for index in range(datasets):
                dataset_dir = root / f"input_{index}"
                dataset_dir.mkdir()
                (dataset_dir / "same name.bin").write_bytes(
                    b"" if fmt == "compact" else bytes([index]) * 65536
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
                     "FORMAT": fmt, "DATASET_FILTER": "", "DATASET_COUNT": str(datasets)},
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
            return [line.split(" | ")[:4] + line.split(" | ")[5:8]
                    for line in output.splitlines() if line.startswith("PASS |")]
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
        for line in parallel.stdout.splitlines():
            if line.startswith("PASS |"):
                fields = line.split(" | ")
                self.assertAlmostEqual(float(fields[7]), int(fields[6]) / int(fields[5]), places=4)
                self.assertGreaterEqual(float(fields[8]), 0)

    def test_failures_cannot_be_reported_as_success(self):
        for mode in ("compress-fail", "decompress-fail", "forward-corrupt",
                     "reverse-fail", "reverse-corrupt", "missing-output"):
            with self.subTest(mode=mode):
                result, _ = self.run_runner(mode=mode)
                self.assertNotEqual(result.returncode, 0, result.stdout)
                self.assertEqual(sum(line.startswith("FAIL |") for line in result.stdout.splitlines()), 8, result.stdout)
                self.assertIn("8 failed cases", result.stdout)

    def test_skip_survives_timing_wrapper(self):
        result, _ = self.run_runner(mode="skip")
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertIn("8 COMPRESSION COMPATIBILITY TESTS PASSED; 8 explicit skips; 0 failed cases", result.stdout)

    def test_empty_historical_inputs_still_skip(self):
        result, _ = self.run_runner(fmt="compact", datasets=2)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("0 COMPRESSION COMPATIBILITY TESTS PASSED; 2 explicit skips", result.stdout)

    def test_no_matching_cases_fails(self):
        result, _ = self.run_runner(datasets=0)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn("No compression compatibility tests ran", result.stdout)

    def test_archived_comparisons_and_missing_corpus(self):
        result, _ = self.run_runner(fmt="sco-compress", archives=True)
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(sum(line.startswith("PASS | Archived") for line in result.stdout.splitlines()), 4)
        self.assertIn("20 COMPRESSION COMPATIBILITY TESTS PASSED", result.stdout)
        missing, _ = self.run_runner(fmt="sco-compress")
        self.assertNotEqual(missing.returncode, 0, missing.stdout)
        self.assertIn("16 COMPRESSION COMPATIBILITY TESTS PASSED; 0 explicit skips; 1 failed cases", missing.stdout)


if __name__ == "__main__":
    unittest.main()