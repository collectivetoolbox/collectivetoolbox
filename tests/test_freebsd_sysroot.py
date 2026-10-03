# SPDX-License-Identifier: AGPL-3.0-or-later

import copy
import contextlib
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/download-freebsd-sysroot.py"
SPEC = importlib.util.spec_from_file_location("freebsd_sysroot", SCRIPT)
SYSROOT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SYSROOT)


class FreeBSDSysrootTests(unittest.TestCase):
    def setUp(self):
        self.raw = (
            b"/* SPDX-License-Identifier: BSD-2-Clause */\n"
            b"/* Copyright First Author; full first notice. */\n"
            b"int first(void);\n"
            b"/* Copyright Second Author; full second notice. */\n"
            b"int second(void);\n"
        )
        self.entry = {
            "repository": "freebsd", "path": "include/test.h",
            "destination": "usr/include/test.h",
            "sha256": hashlib.sha256(self.raw).hexdigest(), "kind": "source",
            "licenses": ["BSD-2-Clause"], "review": "Fixture review",
            "source_only_reviewed": True, "notices": [],
        }
        self.manifest = {
            "version": 1,
            "commits": {name: commit for name, (_, commit) in SYSROOT.REPOSITORIES.items()},
            "files": [self.entry],
        }

    def load(self, manifest):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            return SYSROOT.load_manifest(path)

    def test_manifest_requires_review_and_exact_commits(self):
        self.load(self.manifest)
        for field, value in (
            ("licenses", ["BSD-4-Clause"]), ("licenses", ["LicenseRef-Unknown"]),
            ("licenses", ["GPL-3.0-only"]), ("licenses", ["AGPL-3.0-only"]),
            ("review", ""), ("source_only_reviewed", False),
            ("sha256", ""), ("notices", ["COPYING"]),
            ("destination", "../escape.h"), ("destination", "usr/lib/test.o"),
            ("destination", "/absolute.h"), ("path", "include/../test.h"),
        ):
            with self.subTest(field=field, value=value):
                manifest = copy.deepcopy(self.manifest)
                manifest["files"][0][field] = value
                with self.assertRaises(ValueError):
                    self.load(manifest)
        manifest = copy.deepcopy(self.manifest)
        manifest["commits"]["freebsd"] = "0" * 40
        with self.assertRaises(ValueError):
            self.load(manifest)

    def test_duplicate_and_overlapping_destinations_rejected(self):
        for destination in ("usr/include/test.h", "usr/include/test.h/child.h"):
            manifest = copy.deepcopy(self.manifest)
            entry = copy.deepcopy(self.entry)
            entry["destination"] = destination
            manifest["files"].append(entry)
            with self.assertRaises(ValueError):
                self.load(manifest)

    def test_all_spdx_not_just_first_checked(self):
        for tail in (
            b"/* SPDX-License-Identifier: BSD-4-Clause */\n",
            b"/* SPDX-License-Identifier: LicenseRef-Proprietary */\n",
            b"/* SPDX-License-Identifier: BSD-2-Clause OR LicenseRef-Unknown */\n",
        ):
            raw = self.raw + tail
            entry = dict(self.entry, sha256=hashlib.sha256(raw).hexdigest())
            with self.assertRaises(ValueError):
                SYSROOT.validate_bytes(entry, raw)

    def test_binary_blobs_rejected_even_with_review_and_spdx(self):
        for prefix in (b"\x7fELF", b"!<arch>\n", b"MZ", b"GIF89a", b"\0", b"\xff"):
            raw = prefix + self.raw
            entry = dict(self.entry, sha256=hashlib.sha256(raw).hexdigest())
            with self.assertRaises((ValueError, UnicodeError)):
                SYSROOT.validate_bytes(entry, raw)

    def test_digest_mismatch_rejected(self):
        with self.assertRaises(ValueError):
            SYSROOT.validate_bytes(self.entry, self.raw + b"/* new terms */")

    def test_notices_and_entire_multi_author_source_preserved(self):
        notice_raw = b"Companion license: full terms and copyright notice.\n"
        notice = dict(
            self.entry, path="COPYING", destination="share/licenses/freebsd/COPYING",
            kind="notice", sha256=hashlib.sha256(notice_raw).hexdigest(),
        )
        manifest = copy.deepcopy(self.manifest)
        manifest["files"][0]["notices"] = ["COPYING"]
        manifest["files"].append(notice)
        self.load(manifest)
        with tempfile.TemporaryDirectory() as temporary:
            staged = Path(temporary) / "staged"
            staged.mkdir()
            with patch.object(SYSROOT, "read_blob", side_effect=[self.raw, notice_raw]):
                SYSROOT.stage_sysroot(manifest, {"freebsd": Path("unused")}, staged)
            self.assertEqual((staged / self.entry["destination"]).read_bytes(), self.raw)
            self.assertEqual((staged / notice["destination"]).read_bytes(), notice_raw)
            self.assertFalse((staged / "usr/lib").exists())

    def test_symlinks_and_executables_rejected(self):
        for mode in (b"120000", b"100755", b"160000"):
            record = mode + b" blob abc\tinclude/test.h\0"
            with patch.object(SYSROOT, "git", return_value=record):
                with self.assertRaises(ValueError):
                    SYSROOT.read_blob(Path("unused"), "commit", "include/test.h")

    def generator_fixture(self, destination):
        inputs = {
            ("freebsd", "sys/sys/param.h"): '#define __FreeBSD_version 1304000\n',
            ("freebsd", "sys/sys/types.h"): 'typedef int fixture_type;\n',
            ("freebsd", "lib/libthr/thread/thr_private.h"): 'struct pthread;\n',
            ("freebsd", "lib/libthr/thread/thr_create.c"): 'int\npthread_create(void) {}\n',
            ("freebsd", "lib/libthr/thread/thr_join.c"): 'int\npthread_join(void) {}\n',
            ("freebsd", "sys/arm64/include/_align.h"): '#define _ALIGNBYTES 7\n',
            ("freebsd", "sys/arm64/include/param.h"): '#define MACHINE "arm64"\n',
            ("freebsd", "include/xlocale/_strings.h"): 'int fixture_strings(void);\n',
            ("llvm", "libc/include/sys/syscall.h.def"): '#define SYS_fixture 42\n',
            ("llvm", "libc/include/llvm-libc-macros/pthread-macros.h"): '#define PTHREAD_FIXTURE 1\n',
            ("llvm", "libc/include/llvm-libc-macros/sched-macros.h"): '#define SCHED_FIXTURE 2\n',
            ("llvm", "libc/include/llvm-libc-types/struct_sched_param.h"): (
                'struct sched_param {\n int sched_priority;\n};\n'
            ),
            ("llvm", "libc/include/llvm-libc-macros/stdint-macros.h"): '#define INT_FIXTURE 3\n',
            ("llvm", "clang/lib/Headers/stdint.h"): 'typedef int int32_t;\n',
            ("llvm", "libc/include/pthread.yaml"): (
                '  - name: pthread_yaml\n    return_type: int\n'
            ),
            ("llvm", "libc/include/sched.yaml"): '  - name: sched_yaml\n    return_type: int\n',
        }
        entries = []
        for (repository, path), body in inputs.items():
            raw = (
                f'/* SPDX-License-Identifier: BSD-2-Clause */\n'
                f'/* Copyright first {path} */\n{body}'
                f'/* Copyright second {path}; retain this later notice. */\n'
            ).encode("utf-8")
            entry = dict(
                self.entry, repository=repository, path=path,
                destination=f'src/{repository}/{path}', sha256=hashlib.sha256(raw).hexdigest(),
            )
            target = destination / entry["destination"]
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(raw)
            entries.append(entry)
        return dict(self.manifest, files=entries)

    def test_restored_generator_preserves_every_contributor_and_later_notice(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            manifest = self.generator_fixture(destination)
            SYSROOT.generate_reviewed_headers(manifest, destination)
            include = destination / "usr/include"
            pthread = (include / "pthread.h").read_text()
            for path in (
                'lib/libthr/thread/thr_create.c', 'lib/libthr/thread/thr_join.c',
                'libc/include/pthread.yaml', 'libc/include/llvm-libc-macros/pthread-macros.h',
            ):
                self.assertIn(f'Copyright first {path}', pthread)
                self.assertIn(f'Copyright second {path}; retain this later notice.', pthread)
            self.assertIn('int\tpthread_join(void);', pthread)
            self.assertIn('int\tpthread_yaml(void);', pthread)
            sched = (include / 'sys/sched.h').read_text()
            self.assertIn('Copyright second libc/include/sched.yaml', sched)
            self.assertIn('int\tsched_yaml(void);', sched)
            self.assertIn('#define MACHINE "amd64"', (include / 'machine/param.h').read_text())
            self.assertIn('#define SYS_fixture 42', (include / 'sys/syscall.h').read_text())
            self.assertIn('Copyright second include/xlocale/_strings.h', (
                include / 'xlocale/_strings.h'
            ).read_text())
            self.assertIn('typedef int\t__int32_t;', (include / 'x86/_types.h').read_text())
            provenance = json.loads((destination / 'header-provenance.json').read_text())
            self.assertIn('lib/libthr/thread/thr_join.c', {
                entry['path'] for entry in provenance['usr/include/pthread.h']
            })

    def test_missing_required_generator_input_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary)
            manifest = self.generator_fixture(destination)
            manifest['files'] = [
                entry for entry in manifest['files'] if entry['path'] != 'libc/include/sched.yaml'
            ]
            with self.assertRaisesRegex(ValueError, 'Required generation input'):
                SYSROOT.generate_reviewed_headers(manifest, destination)

    def test_missing_manifest_has_no_download_or_deletion(self):
        with patch.object(SYSROOT, "git") as git, patch.object(SYSROOT, "publish") as publish:
            with self.assertRaises(SystemExit), contextlib.redirect_stderr(io.StringIO()):
                SYSROOT.main([])
            git.assert_not_called()
            publish.assert_not_called()

    def test_publish_preserves_existing_output_when_copy_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "output"
            output.mkdir()
            sentinel = output / "existing.h"
            sentinel.write_bytes(self.raw)
            with patch.object(SYSROOT.shutil, "copytree", side_effect=OSError("copy failed")):
                with self.assertRaises(OSError):
                    SYSROOT.publish(root / "staged", [output])
            self.assertEqual(sentinel.read_bytes(), self.raw)

    def test_real_git_blob_read(self):
        with tempfile.TemporaryDirectory() as temporary:
            repository = Path(temporary)
            subprocess.run(["git", "init", "--quiet", str(repository)], check=True)
            (repository / "include").mkdir()
            (repository / self.entry["path"]).write_bytes(self.raw)
            subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
            subprocess.run([
                "git", "-C", str(repository), "-c", "user.name=Test", "-c",
                "user.email=test@example.invalid", "-c", "commit.gpgsign=false",
                "commit", "--quiet", "-m", "fixture",
            ], check=True)
            self.assertEqual(
                SYSROOT.read_blob(repository / ".git", "HEAD", self.entry["path"]), self.raw,
            )

    def test_end_to_end_local_download_and_failed_review(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repository = root / "upstream"
            repository.mkdir()
            subprocess.run(["git", "init", "--quiet", str(repository)], check=True)
            subprocess.run([
                "git", "-C", str(repository), "config", "uploadpack.allowFilter", "true",
            ], check=True)
            (repository / "include").mkdir()
            (repository / self.entry["path"]).write_bytes(self.raw)
            subprocess.run(["git", "-C", str(repository), "add", "."], check=True)
            subprocess.run([
                "git", "-C", str(repository), "-c", "user.name=Test", "-c",
                "user.email=test@example.invalid", "-c", "commit.gpgsign=false",
                "commit", "--quiet", "-m", "fixture",
            ], check=True)
            commit = subprocess.check_output([
                "git", "-C", str(repository), "rev-parse", "HEAD",
            ], text=True).strip()
            repositories = {"freebsd": (repository.as_uri(), commit)}
            outputs = (root / "reference/sysroot", root / "final/sysroot")
            manifest = copy.deepcopy(self.manifest)
            manifest["commits"] = {"freebsd": commit}
            path = root / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with (
                patch.object(SYSROOT, "WORKSPACE_ROOT", root),
                patch.object(SYSROOT, "REPOSITORIES", repositories),
                patch.object(SYSROOT, "OUTPUTS", outputs),
                patch.object(SYSROOT, "generate_reviewed_headers") as generate,
            ):
                SYSROOT.main(["--manifest", str(path)])
                generate.assert_called_once()
                for output in outputs:
                    self.assertEqual((output / self.entry["destination"]).read_bytes(), self.raw)
                    self.assertEqual(
                        {str(item.relative_to(output)) for item in output.rglob("*") if item.is_file()},
                        {
                            self.entry["destination"], "reviewed-manifest.json",
                            "src/freebsd/include/test.h",
                        },
                    )
                manifest["files"][0]["sha256"] = "0" * 64
                path.write_text(json.dumps(manifest), encoding="utf-8")
                with self.assertRaises(ValueError):
                    SYSROOT.main(["--manifest", str(path)])
                for output in outputs:
                    self.assertEqual((output / self.entry["destination"]).read_bytes(), self.raw)


if __name__ == "__main__":
    unittest.main()