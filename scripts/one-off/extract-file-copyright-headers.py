#!/usr/bin/env python3
"""
Extract licensing headers and standalone COPYING/COPYRIGHT/LICENSE files
from old/filedetect/file, sorting files with the clause
"immediately at the beginning of the file" all together at the top.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path

# Phrases that identify actual copyright or license grant statements
LICENSE_PATTERNS = [
    r"copyright\s+(?:\([c-zC-Z0-9]\)|[0-9]|by|the|author|\xa9)",
    r"\([cC]\)\s*[0-9]",
    r"redistribution and use",
    r"spdx-license-identifier",
    r"all rights reserved",
    r"permission is hereby granted",
    r"licensed under",
    r"public domain",
    r"file_public domain",
    r"free software foundation",
    r"disclaims all warranties",
    r"not subject to any license",
]
LICENSE_RE = re.compile("|".join(LICENSE_PATTERNS), re.IGNORECASE)

PRIORITY_CLAUSE = "immediately at the beginning of the file"


def is_standalone_license_file(filename: str) -> bool:
    """Check if the file is an entire license / copying / copyright document."""
    upper = filename.upper()
    return "COPYING" in upper or "COPYRIGHT" in upper or upper in ("LICENSE", "LICENSE.TXT")


def extract_c_style_headers(text: str) -> str | None:
    """Extract C block comments (/* ... */) that contain licensing terms."""
    blocks: list[str] = []
    for match in re.finditer(r"/\*.*?\*/", text, re.DOTALL):
        comment = match.group(0)
        if LICENSE_RE.search(comment):
            blocks.append(comment.strip())
        # Do not search beyond the top preamble / headers of the file
        if match.start() > 4000:
            break
    return "\n\n".join(blocks) if blocks else None


def extract_line_comment_headers(text: str, prefixes: tuple[str, ...]) -> str | None:
    """Extract leading line comments starting with any prefix in prefixes."""
    lines = text.splitlines()
    blocks: list[str] = []
    cur_block: list[str] = []

    for line in lines[:100]:
        stripped = line.strip()
        # Skip shebang before first comment block
        if stripped.startswith("#!") and not cur_block:
            continue

        if any(stripped.startswith(p) for p in prefixes):
            cur_block.append(line)
        else:
            if cur_block:
                block_text = "\n".join(cur_block)
                if LICENSE_RE.search(block_text):
                    blocks.append(block_text.strip())
                cur_block = []

    if cur_block:
        block_text = "\n".join(cur_block)
        if LICENSE_RE.search(block_text):
            blocks.append(block_text.strip())

    return "\n\n".join(blocks) if blocks else None


def extract_license_header(path: str, text: str) -> str | None:
    """Extract licensing text or header from a file."""
    basename = os.path.basename(path)

    # Standalone license file: return whole content
    if is_standalone_license_file(basename):
        return text.strip()

    ext = os.path.splitext(path)[1].lower()

    # C / C++ / header files / configure template headers
    if ext in (".c", ".h", ".in") or basename == "magic.h.in":
        return extract_c_style_headers(text)

    # Shell scripts / Dockerfile
    if ext in (".sh", ".bash") or basename == "Dockerfile":
        return extract_line_comment_headers(text, ("#",))

    # Man pages
    if ext in (".man", ".1", ".2", ".3", ".4", ".5", ".6", ".7", ".8"):
        return extract_line_comment_headers(text, ('.\\"', r'.\"', '.\\" ', ". "))

    # M4 macros
    if ext == ".m4":
        return extract_line_comment_headers(text, ("dnl", "#"))

    return None


def collect_license_headers(
    root_dir: str,
    path_style: str = "relative",
) -> tuple[list[tuple[str, str, bool]], list[str]]:
    """
    Collect license headers across root_dir.
    Returns:
        (licensed_items, unlicensed_file_paths)
        where licensed_items is a list of tuples:
        (display_name, extracted_header, has_priority_clause).
    """
    items: list[tuple[str, str, bool]] = []
    unlicensed: list[str] = []

    for dirpath, dirnames, filenames in os.walk(root_dir):

        dirnames.sort()
        for filename in sorted(filenames):
            filepath = os.path.join(dirpath, filename)
            relpath = os.path.relpath(filepath, root_dir)

            # Skip test files, build artifacts, git
            if any(relpath.endswith(ext) for ext in (".testfile", ".result", ".mgc", ".cvsignore")):
                continue
            if ".git" in relpath.split(os.sep):
                continue

            try:
                with open(filepath, "rb") as fp:
                    raw = fp.read()
                # Skip binary files
                if b"\x00" in raw[:512]:
                    continue
                text = raw.decode("utf-8", errors="ignore")
            except OSError:
                continue

            extracted = extract_license_header(filepath, text)
            if not extracted:
                unlicensed.append(relpath)
                continue

            has_clause = PRIORITY_CLAUSE in extracted

            if path_style == "relative":
                display_name = relpath
            elif path_style == "full":
                display_name = os.path.abspath(filepath)
            else:
                display_name = filename

            items.append((display_name, extracted, has_clause))

    # Sort items:
    # 1. Headers containing "immediately at the beginning of the file" at the top
    # 2. Within each group, sort by display name alphabetically (case-insensitive)
    items.sort(key=lambda item: (not item[2], item[0].lower()))
    unlicensed.sort(key=str.lower)
    return items, unlicensed


def format_file_list(files: list[str]) -> str:
    """Format a list of files with commas and 'and'."""
    if len(files) == 1:
        return files[0]
    if len(files) == 2:
        return f"{files[0]} and {files[1]}"
    return f"{', '.join(files[:-1])}, and {files[-1]}"


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract licensing headers and COPYING/COPYRIGHT files from old/filedetect/file."
    )
    default_dir = os.path.normpath(
        os.path.join(os.path.dirname(__file__), "..", "..", "old", "filedetect", "file")
    )
    parser.add_argument(
        "--dir",
        default=default_dir,
        help=f"Directory to scan (default: {default_dir})",
    )
    parser.add_argument(
        "-o",
        "--output",
        default=None,
        help="Output file path (default: stdout)",
    )
    parser.add_argument(
        "--path-style",
        choices=["relative", "basename", "full"],
        default="relative",
        help="How to format the filename in header labels (default: relative)",
    )

    args = parser.parse_args()

    if not os.path.isdir(args.dir):
        sys.stderr.write(f"Error: Directory not found: {args.dir}\n")
        sys.exit(1)

    headers, unlicensed = collect_license_headers(args.dir, path_style=args.path_style)

    # Group files that have bit-for-bit identical license notice blocks
    grouped: dict[str, list[str]] = {}
    for display_name, content, _ in headers:
        if content not in grouped:
            grouped[content] = []
        grouped[content].append(display_name)

    # Sort groups:
    # 1. Blocks containing "immediately at the beginning of the file" at the top
    # 2. Within each partition, sort by the first file in the group
    group_list: list[tuple[bool, str, str, list[str]]] = []
    for content, files in grouped.items():
        sorted_files = sorted(files, key=str.lower)
        has_clause = PRIORITY_CLAUSE in content
        group_list.append((not has_clause, sorted_files[0].lower(), content, sorted_files))

    group_list.sort(key=lambda x: (x[0], x[1]))

    chunks: list[str] = []
    for _, _, content, sorted_files in group_list:
        file_list_str = format_file_list(sorted_files)
        chunks.append(f"{content}\n\nThe preceding notice is from {file_list_str} in `file`.\n\n")

    if unlicensed:
        unlicensed_block = (
            "================================================================================\n"
            f"Files without detected license headers ({len(unlicensed)}):\n"
            "================================================================================\n"
            + "\n".join(f"- {p}" for p in unlicensed)
        )
        chunks.append(unlicensed_block)

    output_text = "\n\n".join(chunks) + "\n"

    if args.output:
        out_path = Path(args.output)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(output_text, encoding="utf-8")
    else:
        sys.stdout.write(output_text)




if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        # Python flushes standard streams on exit; redirect remaining to devnull
        devnull = os.open(os.devnull, os.O_WRONLY)
        os.dup2(devnull, sys.stdout.fileno())
        sys.exit(0)

