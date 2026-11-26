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
import os
import pathlib
import stat
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

    def test_setup_permissions_groups_help_without_root(self):
        res = subprocess.run(
            ["./scripts/setup-permissions-groups", "--help"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0, res.stderr)
        self.assertIn("--server-user", res.stdout)
        self.assertIn("--deployer-user", res.stdout)

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

    def test_deploy_rejects_existing_version_tag_when_not_dry_run(self):
        # Verify Cargo.toml version lookup + early tag collision guard logic
        meta = subprocess.check_output(
            ["cargo", "metadata", "--format-version=1", "--no-deps"],
            cwd=REPO_ROOT,
            text=True,
        )
        pkgs = json.loads(meta)["packages"]
        ctb_ver = next(p["version"] for p in pkgs if p["name"] == "ctoolbox")
        tag_name = f"v{ctb_ver}"
        tag_exists = (
            subprocess.run(
                ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag_name}"],
                cwd=REPO_ROOT,
                capture_output=True,
                check=False,
            ).returncode
            == 0
        )
        if tag_exists:
            with tempfile.TemporaryDirectory() as cfg_dir:
                cfg = pathlib.Path(cfg_dir)
                for name, val in (
                    ("username", "deployer"),
                    ("sudoer-username", "sudoer"),
                    ("server-username", "server"),
                    ("ip", "127.0.0.1"),
                    ("port", "2224"),
                    ("global-graph-pass", "pass123"),
                    ("key", "dummy-key"),
                ):
                    (cfg / f"ctb-deploy.{name}").write_text(f"{val}\n", encoding="utf-8")
                res = subprocess.run(
                    [
                        "./deploy",
                        "quick",
                        "--target",
                        cfg_dir,
                        "--no-build",
                        "--allow-dirty",
                    ],
                    cwd=REPO_ROOT,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(res.returncode, 1)
                self.assertIn(f"Git tag '{tag_name}' already exists", res.stderr)

    def test_backup_server_help_and_flag_validation(self):
        res = subprocess.run(
            ["./scripts/backup-server", "--help"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(res.returncode, 0, res.stderr)
        self.assertIn("--dest", res.stdout)
        self.assertIn("--exclude-releases", res.stdout)
        self.assertIn("--stop-service", res.stdout)
        self.assertIn("/var", res.stdout)

        # Reject destination inside git repository without creating it
        inside_repo_dest = REPO_ROOT / "tmp_should_not_be_created_backup_dir"
        with tempfile.TemporaryDirectory() as cfg_dir:
            cfg = pathlib.Path(cfg_dir)
            for name, val in (
                ("username", "deployer"),
                ("sudoer-username", "sudoer"),
                ("server-username", "server"),
                ("ip", "127.0.0.1"),
                ("port", "2224"),
            ):
                (cfg / f"ctb-deploy.{name}").write_text(f"{val}\n", encoding="utf-8")
            key_path = cfg / "ctb-deploy.key"
            key_path.write_text("dummy-key\n", encoding="utf-8")

            res_inside = subprocess.run(
                [
                    "./scripts/backup-server",
                    "--target",
                    cfg_dir,
                    "--dest",
                    str(inside_repo_dest),
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res_inside.returncode, 1)
            self.assertIn("must be outside the git repository", res_inside.stderr)
            self.assertFalse(inside_repo_dest.exists())

    def test_backup_server_syncs_home_and_excludes_os_dirs_and_ephemeral_cache(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = pathlib.Path(tmpdir)
            remote_root = tmp / "mock_remote_fs"
            # Populate mock remote /home and mock remote /var (which must NEVER be backed up)
            server_home = remote_root / "home" / "ctb-server"
            deployer_home = remote_root / "home" / "ctb-deployer"
            (server_home / ".local" / "share" / "collectivetoolbox" / "graphs" / "1").mkdir(
                parents=True
            )
            (
                server_home
                / ".local"
                / "share"
                / "collectivetoolbox"
                / "graphs"
                / "1"
                / "node.bin"
            ).write_bytes(b"important-graph-bytes")
            users_db = (
                server_home / ".local" / "share" / "collectivetoolbox" / "users.db"
            )
            users_db.write_bytes(b"users-db-v1-00")
            fixed_mtime = (1700000000, 1700000000)
            os.utime(users_db, fixed_mtime)
            (
                server_home / ".local" / "share" / "collectivetoolbox" / "setup_token"
            ).write_text("ephemeral-token\n", encoding="utf-8")
            (
                server_home / ".local" / "share" / "collectivetoolbox" / "releases" / "bh"
            ).mkdir(parents=True)
            (
                server_home
                / ".local"
                / "share"
                / "collectivetoolbox"
                / "releases"
                / "bh"
                / "chunk.br"
            ).write_bytes(b"release-chunk")
            (
                server_home / ".cache" / "collectivetoolbox" / "downloads_cache"
            ).mkdir(parents=True)
            (
                server_home
                / ".cache"
                / "collectivetoolbox"
                / "downloads_cache"
                / "stale.tar.gz"
            ).write_bytes(b"disposable-cache")
            deployer_home.mkdir(parents=True)
            (deployer_home / ".bashrc").write_text("# deployer rc\n", encoding="utf-8")

            (remote_root / "var" / "log").mkdir(parents=True)
            (remote_root / "var" / "log" / "syslog").write_text(
                "os-log-do-not-backup\n", encoding="utf-8"
            )

            # Create mock ssh + rsync wrapper bin directory (no network ports used!)
            bin_dir = tmp / "mock_bin"
            bin_dir.mkdir()
            real_rsync = subprocess.check_output(["which", "rsync"], text=True).strip()

            mock_ssh = bin_dir / "ssh"
            mock_ssh.write_text(
                f"""#!/usr/bin/env bash
set -euo pipefail
# Skip ssh flags to get user@host and command
while [[ $# -gt 0 ]]; do
    case "$1" in
        -i|-p|-o) shift 2 ;;
        -*) shift ;;
        *) break ;;
    esac
done
shift # user@host
cmd="${{*:-}}"
if [[ "$cmd" == "sudo -n true" ]]; then
    exit 0
fi
if [[ "$cmd" == sudo\\ -n\\ test\\ -d\\ * ]]; then
    rel_path="${{cmd#sudo -n test -d }}"
    rel_path="${{rel_path#\'}}"
    rel_path="${{rel_path%\'}}"
    [[ -d "{remote_root}${{rel_path}}" ]]
    exit $?
fi
exit 0
""",
                encoding="utf-8",
            )
            mock_ssh.chmod(mock_ssh.stat().st_mode | stat.S_IXUSR)

            rsync_log = tmp / "rsync_invocations.jsonl"
            mock_rsync = bin_dir / "rsync"
            mock_rsync.write_text(
                f"""#!/usr/bin/env python3
import json, os, subprocess, sys
with open({str(rsync_log)!r}, "a", encoding="utf-8") as logf:
    logf.write(json.dumps(sys.argv[1:]) + "\\n")
args = []
i = 1
while i < len(sys.argv):
    a = sys.argv[i]
    if a == "-e":
        i += 2
        continue
    if a.startswith("--rsync-path="):
        i += 1
        continue
    if ":" in a and not a.startswith("-"):
        _, remote_path = a.split(":", 1)
        args.append({str(remote_root)!r} + remote_path)
    else:
        args.append(a)
    i += 1
rc = subprocess.call([{real_rsync!r}] + args)
# Simulate a live same-size, same-mtime edit right after Pass 1 (e.g. mmap/same-second DB write)
if "--checksum" not in sys.argv[1:]:
    db_path = {str(users_db)!r}
    with open(db_path, "wb") as f:
        f.write(b"users-db-v2-99")
    os.utime(db_path, (1700000000, 1700000000))
sys.exit(rc)
""",
                encoding="utf-8",
            )
            mock_rsync.chmod(mock_rsync.stat().st_mode | stat.S_IXUSR)

            cfg = tmp / "cfg"
            cfg.mkdir()
            (cfg / "ctb-deploy.username").write_text("ctb-deployer\n", encoding="utf-8")
            (cfg / "ctb-deploy.sudoer-username").write_text("ctb-sudoer\n", encoding="utf-8")
            (cfg / "ctb-deploy.server-username").write_text("ctb-server\n", encoding="utf-8")
            (cfg / "ctb-deploy.ip").write_text("198.51.100.10\n", encoding="utf-8")
            (cfg / "ctb-deploy.port").write_text("22\n", encoding="utf-8")
            (cfg / "ctb-deploy.key").write_text("mock-key\n", encoding="utf-8")

            backup_dest = tmp / "backup_out"
            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env['PATH']}"

            res = subprocess.run(
                [
                    "./scripts/backup-server",
                    "--target",
                    str(cfg),
                    "--exclude-releases",
                    str(backup_dest),
                ],
                cwd=REPO_ROOT,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 0, res.stderr)

            # Verify two full passes over /home/: Pass 1 without --checksum, Pass 2 with --checksum
            invocations = [
                json.loads(line)
                for line in rsync_log.read_text(encoding="utf-8").splitlines()
                if line.strip()
            ]
            self.assertEqual(len(invocations), 2, invocations)
            self.assertNotIn("--checksum", invocations[0])
            self.assertIn("--checksum", invocations[1])
            for inv in invocations:
                self.assertNotIn("-avz", inv)
                self.assertNotIn("-a", inv)

            # Same-size, same-mtime edit to users.db after Pass 1 must be caught by Pass 2 --checksum
            self.assertEqual(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".local"
                    / "share"
                    / "collectivetoolbox"
                    / "users.db"
                ).read_bytes(),
                b"users-db-v2-99",
            )

            # /home accounts & graph data backed up
            self.assertEqual(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".local"
                    / "share"
                    / "collectivetoolbox"
                    / "graphs"
                    / "1"
                    / "node.bin"
                ).read_bytes(),
                b"important-graph-bytes",
            )
            self.assertTrue((backup_dest / "home" / "ctb-deployer" / ".bashrc").is_file())

            # /var and OS folders NEVER backed up
            self.assertFalse((backup_dest / "var").exists())

            # Ephemeral cache/tokens and --exclude-releases respected
            self.assertFalse(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".cache"
                    / "collectivetoolbox"
                    / "downloads_cache"
                ).exists()
            )
            self.assertFalse(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".local"
                    / "share"
                    / "collectivetoolbox"
                    / "setup_token"
                ).exists()
            )
            self.assertFalse(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".local"
                    / "share"
                    / "collectivetoolbox"
                    / "releases"
                ).exists()
            )

            # Metadata manifest recorded
            meta = json.loads((backup_dest / "backup-metadata.json").read_text(encoding="utf-8"))
            self.assertEqual(meta["scope"], "/home")
            self.assertIn("/var", meta["excluded_os_dirs"])
            self.assertTrue(meta["exclude_releases"])

    def test_backup_server_stop_service_reunlocks_global_user_and_handles_spaces_in_key_and_root_sudoer(
        self,
    ):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = pathlib.Path(tmpdir)
            remote_root = tmp / "mock_remote"
            server_home = remote_root / "home" / "ctb-server"
            deployer_home = remote_root / "home" / "ctb-deployer"
            (server_home / ".local" / "share" / "collectivetoolbox").mkdir(parents=True)
            (server_home / ".local" / "share" / "collectivetoolbox" / "users.db").write_bytes(
                b"cold-db"
            )
            deployer_home.mkdir(parents=True)
            (deployer_home / "note.txt").write_text("ok\n", encoding="utf-8")
            # Note: sudoer_user is 'root', so /home/root intentionally does NOT exist!

            ssh_events_log = tmp / "ssh_events.jsonl"
            bin_dir = tmp / "mock_bin"
            bin_dir.mkdir()
            real_rsync = subprocess.check_output(["which", "rsync"], text=True).strip()

            mock_ssh = bin_dir / "ssh"
            mock_ssh.write_text(
                f"""#!/usr/bin/env python3
import json, os, pathlib, sys

args = sys.argv[1:]
i = 0
while i < len(args):
    a = args[i]
    if a in ("-i", "-p", "-o"):
        i += 2
    elif a.startswith("-"):
        i += 1
    else:
        break

target = args[i]
remote_cmd = " ".join(args[i + 1:])
stdin_data = ""
if "setup-global-user" in remote_cmd or remote_cmd == "bash":
    stdin_data = sys.stdin.read()

with open({str(ssh_events_log)!r}, "a", encoding="utf-8") as logf:
    logf.write(json.dumps({{"target": target, "cmd": remote_cmd, "stdin": stdin_data}}) + "\\n")

if remote_cmd == "sudo -n true":
    # Simulate non-NOPASSWD sudo so fallback per-user sync is exercised
    sys.exit(1)

if remote_cmd.startswith("test -d "):
    rel = remote_cmd[len("test -d "):].strip().strip("'")
    exists = pathlib.Path({str(remote_root)!r} + rel).is_dir()
    sys.exit(0 if exists else 1)

if "setup-global-user" in remote_cmd:
    payload = json.loads(stdin_data)
    assert payload == {{"password": "secret-global-pass-xyz"}}, payload
    print("ctoolbox.service restarted and global graph user re-unlocked successfully.")
    sys.exit(0)

sys.exit(0)
""",
                encoding="utf-8",
            )
            mock_ssh.chmod(mock_ssh.stat().st_mode | stat.S_IXUSR)

            mock_rsync = bin_dir / "rsync"
            mock_rsync.write_text(
                f"""#!/usr/bin/env python3
import subprocess, sys
args = []
i = 1
while i < len(sys.argv):
    a = sys.argv[i]
    if a == "-e":
        rsh = sys.argv[i + 1]
        assert "'/" in rsh and "key with spaces" in rsh, f"Expected quoted key path in -e: {{rsh}}"
        i += 2
        continue
    if ":" in a and not a.startswith("-"):
        _, remote_path = a.split(":", 1)
        args.append({str(remote_root)!r} + remote_path)
    else:
        args.append(a)
    i += 1
sys.exit(subprocess.call([{real_rsync!r}] + args))
""",
                encoding="utf-8",
            )
            mock_rsync.chmod(mock_rsync.stat().st_mode | stat.S_IXUSR)

            cfg = tmp / "cfg"
            cfg.mkdir()
            (cfg / "ctb-deploy.username").write_text("ctb-deployer\n", encoding="utf-8")
            (cfg / "ctb-deploy.sudoer-username").write_text("root\n", encoding="utf-8")
            (cfg / "ctb-deploy.server-username").write_text("ctb-server\n", encoding="utf-8")
            (cfg / "ctb-deploy.ip").write_text("198.51.100.11\n", encoding="utf-8")
            (cfg / "ctb-deploy.port").write_text("22\n", encoding="utf-8")
            (cfg / "ctb-deploy.global-graph-pass").write_text(
                "secret-global-pass-xyz\n", encoding="utf-8"
            )

            key_dir = tmp / "dir with spaces"
            key_dir.mkdir()
            spaced_key = key_dir / "key with spaces"
            spaced_key.write_text("mock-private-key\n", encoding="utf-8")

            backup_dest = tmp / "cold_backup_out"
            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env['PATH']}"

            res = subprocess.run(
                [
                    "./scripts/backup-server",
                    "--target",
                    str(cfg),
                    "--ssh-key",
                    str(spaced_key),
                    "--stop-service",
                    str(backup_dest),
                ],
                cwd=REPO_ROOT,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res.returncode, 0, f"stdout={res.stdout}\nstderr={res.stderr}")
            self.assertIn("Skipping /home/root", res.stdout)
            self.assertTrue(
                (
                    backup_dest
                    / "home"
                    / "ctb-server"
                    / ".local"
                    / "share"
                    / "collectivetoolbox"
                    / "users.db"
                ).is_file()
            )
            self.assertFalse((backup_dest / "home" / "root").exists())

            events = [
                json.loads(ln)
                for ln in ssh_events_log.read_text(encoding="utf-8").splitlines()
                if ln.strip()
            ]
            # Service must be stopped before sync and re-unlocked via stdin (never in cmd) on exit
            self.assertTrue(
                any("systemctl --user stop ctoolbox" in ev["stdin"] for ev in events),
                events,
            )
            unlock_events = [ev for ev in events if "setup-global-user" in ev["cmd"]]
            self.assertEqual(len(unlock_events), 1, events)
            self.assertNotIn("secret-global-pass-xyz", unlock_events[0]["cmd"])
            self.assertIn("secret-global-pass-xyz", unlock_events[0]["stdin"])

    def test_deploy_version_bump_locks_cargo_lock_skips_stale_noise_and_passes_clean_targets(
        self,
    ):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = pathlib.Path(tmpdir)
            repo = tmp / "repo"
            (repo / "scripts").mkdir(parents=True)
            (repo / "built" / "linux-x64").mkdir(parents=True)

            # Copy deploy and run-dev-ctb into isolated temp git repo
            deploy_dst = repo / "deploy"
            deploy_dst.write_text(
                (REPO_ROOT / "deploy").read_text(encoding="utf-8"), encoding="utf-8"
            )
            deploy_dst.chmod(0o755)
            (repo / "scripts" / "run-dev-ctb").write_text(
                (REPO_ROOT / "scripts" / "run-dev-ctb").read_text(encoding="utf-8"),
                encoding="utf-8",
            )

            # Seed stale/unversioned binaries in built/linux-x64 to verify no noise by default
            (repo / "built" / "linux-x64" / "ctoolbox").write_bytes(
                b"\x7fELF-legacy-no-build-marker"
            )
            (repo / "built" / "linux-x64" / "ctoolbox-installer").write_bytes(
                b"\x7fELF-legacy-installer-no-marker"
            )
            (repo / "built" / "ctoolbox.rsrc").write_bytes(b"rsrc-stub")

            (repo / "Cargo.toml").write_text(
                '[package]\nname = "ctoolbox"\nversion = "0.1.49"\nedition = "2024"\n',
                encoding="utf-8",
            )
            (repo / "Cargo.lock").write_text(
                'version = 4\n\n[[package]]\nname = "ctoolbox"\nversion = "0.1.49"\n',
                encoding="utf-8",
            )
            (repo / ".gitignore").write_text("/built/\n", encoding="utf-8")

            build_args_log = tmp / "build_args.jsonl"
            cargo_args_log = tmp / "cargo_args.jsonl"
            mock_build = repo / "build"
            mock_build.write_text(
                f"""#!/usr/bin/env python3
import json, sys
with open({str(build_args_log)!r}, "a", encoding="utf-8") as f:
    f.write(json.dumps(sys.argv[1:]) + "\\n")
# Exit cleanly with sentinel error so deploy stops right after ./build invocation
sys.stderr.write("SENTINEL_STOP_AFTER_BUILD\\n")
sys.exit(42)
""",
                encoding="utf-8",
            )
            mock_build.chmod(0o755)

            bin_dir = tmp / "mock_bin"
            bin_dir.mkdir()
            mock_cargo = bin_dir / "cargo"
            mock_cargo.write_text(
                f"""#!/usr/bin/env python3
import json, pathlib, re, sys
args = sys.argv[1:]
with open({str(cargo_args_log)!r}, "a", encoding="utf-8") as f:
    f.write(json.dumps(args) + "\\n")
if args and args[0] == "metadata":
    toml_txt = pathlib.Path("Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r'^version\\s*=\\s*"([^"]+)"', toml_txt, re.M)
    ver = m.group(1) if m else "0.0.0"
    # Real cargo metadata ONLY updates Cargo.lock when --no-deps is NOT passed!
    if "--no-deps" not in args:
        lock_path = pathlib.Path("Cargo.lock")
        lock_txt = lock_path.read_text(encoding="utf-8")
        lock_txt = re.sub(
            r'(name = "ctoolbox"\\nversion = ")[^"]+(")',
            r"\\g<1>" + ver + r"\\g<2>",
            lock_txt,
        )
        lock_path.write_text(lock_txt, encoding="utf-8")
    print(json.dumps({{"packages": [{{"name": "ctoolbox", "version": ver}}]}}))
    sys.exit(0)
if args and args[0] == "clean":
    sys.exit(0)
sys.exit(0)
""",
                encoding="utf-8",
            )
            mock_cargo.chmod(0o755)

            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            subprocess.run(
                ["git", "config", "user.email", "test@example.com"], cwd=repo, check=True
            )
            subprocess.run(["git", "config", "user.name", "Test"], cwd=repo, check=True)
            subprocess.run(
                ["git", "add", "Cargo.toml", "Cargo.lock", ".gitignore", "deploy", "build", "scripts"],
                cwd=repo,
                check=True,
            )
            subprocess.run(["git", "commit", "-q", "-m", "init"], cwd=repo, check=True)

            cfg = tmp / "cfg"
            cfg.mkdir()
            for name, val in (
                ("username", "deployer"),
                ("sudoer-username", "sudoer"),
                ("server-username", "server"),
                ("ip", "127.0.0.1"),
                ("port", "2224"),
                ("global-graph-pass", "pass123"),
                ("key", "dummy-key"),
            ):
                (cfg / f"ctb-deploy.{name}").write_text(f"{val}\n", encoding="utf-8")

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env['PATH']}"

            # 1) Default ./deploy quick --clean: bumps Cargo.toml + Cargo.lock in one commit,
            #    emits ZERO stale/metadata noise, and passes --clean-targets to ./build.
            res_clean = subprocess.run(
                ["./deploy", "quick", "--clean", "--target", str(cfg)],
                cwd=repo,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res_clean.returncode, 42, res_clean.stderr)
            self.assertIn("SENTINEL_STOP_AFTER_BUILD", res_clean.stderr)
            self.assertNotIn("Cannot read build metadata", res_clean.stderr)
            self.assertNotIn("binaries are stale", res_clean.stdout)
            self.assertIn('version = "0.1.50"', (repo / "Cargo.toml").read_text(encoding="utf-8"))
            self.assertIn('version = "0.1.50"', (repo / "Cargo.lock").read_text(encoding="utf-8"))
            git_status = subprocess.check_output(
                ["git", "status", "--porcelain"], cwd=repo, text=True
            ).strip()
            self.assertEqual(git_status, "", "Cargo.lock must be clean/committed after version bump")

            build_calls = [
                json.loads(ln)
                for ln in build_args_log.read_text(encoding="utf-8").splitlines()
                if ln.strip()
            ]
            self.assertEqual(len(build_calls), 1)
            self.assertIn("--clean-targets", build_calls[0])

            # 2) ./deploy quick --reuse-built: probes binaries cleanly WITHOUT raw
            #    'Cannot read build metadata' stderr noise, while reporting structured staleness.
            res_reuse = subprocess.run(
                ["./deploy", "quick", "--reuse-built", "--dry-run", "--target", str(cfg)],
                cwd=repo,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res_reuse.returncode, 42, res_reuse.stderr)
            self.assertNotIn("Cannot read build metadata", res_reuse.stderr)
            self.assertIn("Target 'linux-x64' binaries are stale", res_reuse.stdout)
            build_calls = [
                json.loads(ln)
                for ln in build_args_log.read_text(encoding="utf-8").splitlines()
                if ln.strip()
            ]
            self.assertEqual(len(build_calls), 2)
            self.assertNotIn("--clean-targets", build_calls[1])

    def test_build_per_target_clean_and_low_disk_guard(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = pathlib.Path(tmpdir)
            repo = tmp / "repo"
            (repo / ".cargo").mkdir(parents=True)
            (repo / ".cargo" / "config.default.toml").write_text("", encoding="utf-8")
            (repo / "built" / "v86_out" / "build").mkdir(parents=True)
            (repo / "built" / "v86_out" / "bios").mkdir(parents=True)
            (repo / "built" / "v86_out" / "build" / "v86.wasm").write_bytes(b"wasm")
            (repo / "built" / "v86_out" / "bios" / "seabios.bin").write_bytes(b"bios")
            (repo / "built" / "v86_out" / "bios" / "vgabios.bin").write_bytes(b"vga")
            (repo / "built" / "ctoolbox.rsrc").write_bytes(b"rsrc")

            build_dst = repo / "build"
            build_dst.write_text(
                (REPO_ROOT / "build").read_text(encoding="utf-8"), encoding="utf-8"
            )
            build_dst.chmod(0o755)

            cargo_log = tmp / "cargo_clean_calls.jsonl"
            bin_dir = tmp / "mock_bin"
            bin_dir.mkdir()

            mock_cargo = bin_dir / "cargo"
            mock_cargo.write_text(
                f"""#!/usr/bin/env python3
import json, pathlib, shutil, sys
args = sys.argv[1:]
with open({str(cargo_log)!r}, "a", encoding="utf-8") as f:
    f.write(json.dumps(args) + "\\n")
if args and args[0] == "build":
    target = args[args.index("--target") + 1]
    bin_name = args[args.index("--bin") + 1]
    out_dir = pathlib.Path("target") / target / "release"
    out_dir.mkdir(parents=True, exist_ok=True)
    # Also create shared host build dir to verify per-target clean preserves it!
    (pathlib.Path("target") / "release" / "build").mkdir(parents=True, exist_ok=True)
    (out_dir / bin_name).write_bytes(b"\\x7fELF-static")
    sys.exit(0)
if args and args[0] == "clean":
    if "--target" in args:
        triple = args[args.index("--target") + 1]
        assert (pathlib.Path("target") / "release" / "build").is_dir()
        tdir = pathlib.Path("target") / triple
        if tdir.is_dir():
            shutil.rmtree(tdir)
    else:
        tdir = pathlib.Path("target")
        if tdir.is_dir():
            shutil.rmtree(tdir)
    sys.exit(0)
sys.exit(0)
""",
                encoding="utf-8",
            )
            mock_cargo.chmod(0o755)

            for tool, out_txt in (
                ("ldd", "statically linked\n"),
                ("file", "ELF 64-bit LSB executable, statically linked\n"),
            ):
                p = bin_dir / tool
                p.write_text(
                    f"#!/usr/bin/env bash\nprintf '%s' {out_txt!r}\n", encoding="utf-8"
                )
                p.chmod(0o755)

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env['PATH']}"

            # Case 1: --clean-targets cleans per-target triple between targets, then full cargo clean at end
            res1 = subprocess.run(
                [
                    "./build",
                    "--release",
                    "--no-docs",
                    "--skip-codegen",
                    "--no-lints",
                    "--no-tests",
                    "--clean-targets",
                    "linux-x64",
                    "linux-x86",
                ],
                cwd=repo,
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res1.returncode, 0, res1.stderr)
            calls1 = [
                json.loads(ln)
                for ln in cargo_log.read_text(encoding="utf-8").splitlines()
                if ln.strip()
            ]
            clean_calls1 = [c for c in calls1 if c[0] == "clean"]
            self.assertEqual(
                clean_calls1,
                [
                    ["clean", "--target", "x86_64-unknown-linux-musl"],
                    ["clean", "--target", "i586-unknown-linux-musl"],
                    ["clean"],
                ],
            )
            self.assertTrue((repo / "built" / "linux-x64" / "ctoolbox").is_file())
            self.assertTrue((repo / "built" / "linux-x86" / "ctoolbox").is_file())

            # Case 2: Without --clean-targets, when free disk space < CTB_MIN_FREE_KIB,
            # stale other-target dirs AND completed intermediate target dirs are reclaimed automatically!
            cargo_log.unlink()
            (repo / "target" / "x86_64-win7-windows-gnu").mkdir(parents=True)
            env_low_disk = env.copy()
            # Set threshold above any realistic disk so low-disk eviction triggers deterministically
            env_low_disk["CTB_MIN_FREE_KIB"] = str(10**15)
            res2 = subprocess.run(
                [
                    "./build",
                    "--release",
                    "--no-docs",
                    "--skip-codegen",
                    "--no-lints",
                    "--no-tests",
                    "linux-x64",
                    "linux-x86",
                ],
                cwd=repo,
                env=env_low_disk,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(res2.returncode, 0, res2.stderr)
            calls2 = [
                json.loads(ln)
                for ln in cargo_log.read_text(encoding="utf-8").splitlines()
                if ln.strip()
            ]
            clean_calls2 = [c for c in calls2 if c[0] == "clean"]
            self.assertIn(
                ["clean", "--target", "x86_64-win7-windows-gnu"], clean_calls2
            )
            self.assertIn(
                ["clean", "--target", "x86_64-unknown-linux-musl"], clean_calls2
            )


if __name__ == "__main__":
    unittest.main()
