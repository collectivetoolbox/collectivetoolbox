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

"""
Fixes and ensures the required multi-license headers and footers for files in the
format detection module (src/formats/utilities/extension_rule.rs, src/formats/detection/detection.rs and
src/formats/detection/*.rs).

The detection module combines elements from `file` (libmagic), polyfile, binwalk,
fileid, and DROID, and is subject to strict multi-license requirements verified
by the header validator linter (scripts/lint-headers):
  - Leading HASH_BSD_DARWIN_HEADER
  - SPDX_HEADERS_DETECTION
  - AGPL_COPYRIGHT_BLOCK
  - DESCRIPTION_DETECTION
  - Module docblock (//!)
  - Trailing FILE_ADDITIONAL_LICENSES
  - Trailing DETECTION_LICENSES_OTHER
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


def find_repo_root() -> Path:
    """Locate the root directory of the repository."""
    start = Path(__file__).resolve().parent
    for parent in [start, *start.parents]:
        if (parent / "Cargo.toml").is_file() and (parent / "src").is_dir():
            return parent
    return start.parent


def extract_const(source: str, name: str) -> str:
    """Extract a string constant definition from license_consts.rs."""
    pattern = rf"pub const {name}: &str = (r#\"(.*?)\"#|r\"(.*?)\"|\"(.*?)\");"
    match = re.search(pattern, source, re.DOTALL)
    if not match:
        raise ValueError(f"Could not find constant `{name}` in license_consts.rs")
    val = match.group(2) or match.group(3) or match.group(4)
    return val


def load_detection_license_blocks(repo_root: Path) -> tuple[str, str]:
    """Load the required detection header and footer blocks from license_consts.rs."""
    license_consts_path = repo_root / "src/build_support/license_consts.rs"
    if not license_consts_path.is_file():
        raise FileNotFoundError(f"Missing license_consts.rs at {license_consts_path}")

    source = license_consts_path.read_text(encoding="utf-8")
    darwin = extract_const(source, "HASH_BSD_DARWIN_HEADER")
    spdx = extract_const(source, "SPDX_HEADERS_DETECTION")
    agpl = extract_const(source, "AGPL_COPYRIGHT_BLOCK")
    desc = extract_const(source, "DESCRIPTION_DETECTION")
    addl = extract_const(source, "FILE_ADDITIONAL_LICENSES")
    other = extract_const(source, "DETECTION_LICENSES_OTHER")

    header = f"{darwin}\n\n{spdx}\n{agpl}\n\n{desc}\n"
    footer = f"{addl}\n{other}\n"
    return header, footer


def find_docblock_start(content: str) -> int:
    """Find the starting character offset of the module docblock (//! or /*!)."""
    for match in re.finditer(r"^(?://!|/\*!).*", content, re.MULTILINE):
        return match.start()
    return 0


def strip_existing_licenses(content: str, addl_block: str, other_block: str) -> str:
    """Strip any existing license header and footer, returning only the code content."""
    # Find module docblock start
    doc_start = find_docblock_start(content)
    code_part = content[doc_start:]

    # Remove trailing license footer if present
    pos_addl = code_part.find(addl_block)
    if pos_addl != -1:
        code_part = code_part[:pos_addl]
    else:
        pos_other = code_part.find(other_block)
        if pos_other != -1:
            code_part = code_part[:pos_other]

    return code_part.strip()


def fix_detection_file(
    file_path: Path,
    header: str,
    footer: str,
    addl_block: str,
    other_block: str,
    check_only: bool,
) -> bool:
    """Fix license header and footer for a single detection file.

    Returns True if the file was modified or would be modified.
    """
    content = file_path.read_text(encoding="utf-8")
    code = strip_existing_licenses(content, addl_block, other_block)

    new_content = f"{header}\n{code}\n\n{footer}"
    # If the file already matches exactly, no change needed
    if content == new_content or content == f"{header}\n{code}\n{footer}":
        return False

    if not check_only:
        file_path.write_text(f"{header}\n{code}\n{footer}", encoding="utf-8")

    return True


def collect_detection_files(repo_root: Path) -> list[Path]:
    """Find all Rust files belonging to the detection module."""
    files: list[Path] = []
    extension_rule_rs = repo_root / "src/formats/utilities/extension_rule.rs"
    if extension_rule_rs.is_file():
        files.append(extension_rule_rs)

    detection_dir = repo_root / "src/formats/detection"

    if detection_dir.is_dir():
        for path in sorted(detection_dir.glob("*.rs")):
            if path.is_file():
                files.append(path)

    return files


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fix multi-license headers and footers for the detection module."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check only; return exit code 1 if any files require changes.",
    )
    parser.add_argument(
        "files",
        nargs="*",
        type=Path,
        help="Optional list of specific files to fix (defaults to all detection files).",
    )
    args = parser.parse_args()

    repo_root = find_repo_root()
    try:
        header, footer = load_detection_license_blocks(repo_root)
        source = (repo_root / "src/build_support/license_consts.rs").read_text(
            encoding="utf-8"
        )
        addl = extract_const(source, "FILE_ADDITIONAL_LICENSES")
        other = extract_const(source, "DETECTION_LICENSES_OTHER")
    except Exception as exc:
        print(f"Error loading license templates: {exc}", file=sys.stderr)
        return 1

    targets = [p.resolve() for p in args.files] if args.files else collect_detection_files(repo_root)
    if not targets:
        print("No detection files found.", file=sys.stderr)
        return 0

    modified: list[Path] = []
    for file_path in targets:
        if not file_path.is_file():
            print(f"Warning: file not found: {file_path}", file=sys.stderr)
            continue
        rel = file_path.relative_to(repo_root) if file_path.is_relative_to(repo_root) else file_path
        changed = fix_detection_file(
            file_path=file_path,
            header=header,
            footer=footer,
            addl_block=addl,
            other_block=other,
            check_only=args.check,
        )
        if changed:
            modified.append(rel)
            action = "Needs update" if args.check else "Updated"
            print(f"{action}: {rel}")
        else:
            print(f"Valid: {rel}")

    if args.check and modified:
        print(f"\n{len(modified)} file(s) require license updates.", file=sys.stderr)
        return 1

    action_word = "would be updated" if args.check else "updated"
    print(f"\nDone: {len(modified)} file(s) {action_word}, {len(targets) - len(modified)} already valid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
