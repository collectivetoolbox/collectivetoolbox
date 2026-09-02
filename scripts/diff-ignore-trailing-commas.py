#!/usr/bin/env python3
"""
Diffs the current working state against a Git commit/ref (default: HEAD~1),
ignoring changes that consist solely of trailing commas.
"""

import sys
import os
import subprocess
import difflib
import re

# ANSI Color Codes
RED = "\033[31m"
GREEN = "\033[32m"
CYAN = "\033[36m"
BOLD = "\033[1m"
RESET = "\033[0m"


def normalize_line(line: str) -> str:
    """Strip trailing commas and trailing whitespace while preserving newline."""
    stripped = line.rstrip("\r\n")
    # Remove trailing commas at end of line (e.g. `foo,bar,,` -> `foo,bar`)
    stripped = re.sub(r",+\s*$", "", stripped)
    return stripped + "\n"


def run_git(args: list[str]) -> str:
    res = subprocess.run(
        ["git"] + args,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if res.returncode == 0:
        return res.stdout
    return ""


def get_changed_files(target_ref: str) -> list[str]:
    # Handles commit, range, or working tree diff against target_ref
    out = run_git(["diff", "--name-only", target_ref])
    return [line.strip() for line in out.splitlines() if line.strip()]


def get_old_content(target_ref: str, file_path: str) -> list[str]:
    ref = target_ref.split("..")[0] if ".." in target_ref else target_ref
    out = run_git(["show", f"{ref}:{file_path}"])
    return out.splitlines(keepends=True)


def get_new_content(target_ref: str, file_path: str) -> list[str]:
    if ".." in target_ref:
        ref2 = target_ref.split("..")[1]
        out = run_git(["show", f"{ref2}:{file_path}"])
        return out.splitlines(keepends=True)
    # Read from local working tree
    if not os.path.exists(file_path):
        return []
    try:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            return f.readlines()
    except Exception as e:
        sys.stderr.write(f"Failed to read {file_path}: {e}\n")
        return []


def format_colored_diff(diff_lines: list[str]) -> str:
    use_color = sys.stdout.isatty() or "--color" in sys.argv
    colored = []
    for line in diff_lines:
        if not use_color:
            colored.append(line)
            continue
        if line.startswith("+++") or line.startswith("---"):
            colored.append(f"{BOLD}{line}{RESET}")
        elif line.startswith("+"):
            colored.append(f"{GREEN}{line}{RESET}")
        elif line.startswith("-"):
            colored.append(f"{RED}{line}{RESET}")
        elif line.startswith("@@"):
            colored.append(f"{CYAN}{line}{RESET}")
        else:
            colored.append(line)
    return "".join(colored)


def main():
    target_ref = "HEAD~1"
    args = [arg for arg in sys.argv[1:] if arg != "--color"]

    if args:
        target_ref = args[0]

    files = get_changed_files(target_ref)
    if not files:
        print(f"No changes found against {target_ref}.")
        return

    diff_found = False
    for file_path in files:
        old_lines = get_old_content(target_ref, file_path)
        new_lines = get_new_content(target_ref, file_path)

        # Normalize trailing commas on all lines
        norm_old = [normalize_line(l) for l in old_lines]
        norm_new = [normalize_line(l) for l in new_lines]

        diff = list(
            difflib.unified_diff(
                norm_old,
                norm_new,
                fromfile=f"a/{file_path}",
                tofile=f"b/{file_path}",
                lineterm="\n",
            )
        )

        if diff:
            diff_found = True
            print(format_colored_diff(diff), end="")

    if not diff_found:
        print(
            f"All changes against {target_ref} were purely trailing comma adjustments."
        )


if __name__ == "__main__":
    main()
