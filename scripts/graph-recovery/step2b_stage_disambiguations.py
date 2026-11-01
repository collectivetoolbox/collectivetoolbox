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

"""Step 2b: Stage the 2,838 dependency-order disambiguation .dcsh files into
src/formats/dcdata/data-src/nodes/inbox/ from
src/formats/dcdata/data-src/recovery-audit/step2b_manifest.json, verify their
SHA-256 digests, run a dry-run manifest check, and execute the durable inbox
allocation transaction.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "src/formats/dcdata/data-src/recovery-audit/step2b_manifest.json"
INBOX_DIR = REPO_ROOT / "src/formats/dcdata/data-src/nodes/inbox"
ALLOCATED_DIR = REPO_ROOT / "src/formats/dcdata/data-src/nodes/allocated"


def sha256_bytes_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    assert manifest["manifest_id"] == "step2b"
    assert manifest["kind"] == "inbox_allocation"

    planned_items = manifest["planned_items"]
    pinned_inputs = manifest["pinned_input_sha256s"]
    assert len(planned_items) == 2838, f"Expected 2838 Step 2b items, got {len(planned_items)}"
    assert len(pinned_inputs) == 2838

    INBOX_DIR.mkdir(parents=True, exist_ok=True)
    existing_dcsh = sorted(p.name for p in INBOX_DIR.glob("*.dcsh"))
    if existing_dcsh and existing_dcsh != sorted(pinned_inputs.keys()):
        raise RuntimeError(
            f"Refusing to stage Step 2b into inbox with unexpected files ({len(existing_dcsh)} existing .dcsh files)"
        )

    for item in planned_items:
        inbox_name = item["inbox_name"]
        dcsh_content = item["dcsh_content"]
        expected_sha = item["inbox_sha256"]
        assert pinned_inputs[inbox_name] == expected_sha
        raw_bytes = dcsh_content.encode("utf-8")
        actual_sha = sha256_bytes_hex(raw_bytes)
        if actual_sha != expected_sha:
            raise RuntimeError(
                f"SHA-256 mismatch for {inbox_name}: expected {expected_sha}, got {actual_sha}"
            )
        (INBOX_DIR / inbox_name).write_bytes(raw_bytes)

    print(f"Staged {len(planned_items)} .dcsh files in {INBOX_DIR}")

    # 1. Dry-run verification against step2b_manifest.json
    dry_cmd = [
        "cargo",
        "run",
        "-p",
        "ctb-formats-dcdata",
        "--bin",
        "validate-data-tables",
        "--",
        "--process-inbox",
        "--dry-run",
        "--require-manifest",
        str(MANIFEST_PATH),
    ]
    print("Running dry-run verification:", " ".join(dry_cmd))
    subprocess.run(dry_cmd, cwd=REPO_ROOT, check=True)

    # 2. Execute the durable inbox allocation transaction
    exec_cmd = [
        "cargo",
        "run",
        "-p",
        "ctb-formats-dcdata",
        "--bin",
        "validate-data-tables",
        "--",
        "--process-inbox",
        "--require-manifest",
        str(MANIFEST_PATH),
    ]
    print("Executing Step 2b inbox allocation:", " ".join(exec_cmd))
    subprocess.run(exec_cmd, cwd=REPO_ROOT, check=True)

    remaining_inbox = list(INBOX_DIR.glob("*.dcsh"))
    if remaining_inbox:
        raise RuntimeError(
            f"Expected nodes/inbox/ to have 0 .dcsh files after Step 2b, found {len(remaining_inbox)}"
        )

    allocated_dcsh = list(ALLOCATED_DIR.glob("*.dcsh"))
    if len(allocated_dcsh) != 5303:
        raise RuntimeError(
            f"Expected 5303 total allocated .dcsh files in nodes/allocated/ after Step 2b, found {len(allocated_dcsh)}"
        )
    print(f"Step 2b complete: {len(allocated_dcsh)} total allocated .dcsh files in {ALLOCATED_DIR}")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        sys.exit(1)
