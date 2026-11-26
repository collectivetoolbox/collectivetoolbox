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


if __name__ == "__main__":
    unittest.main()
