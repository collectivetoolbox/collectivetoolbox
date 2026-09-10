#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""List short Document Characters (Dcs) that are duplicative of Unicode characters.

Categorizes characters into four groups based on deprecation status and
<equiv> equivalency matching with EITE data mappings. Uses `ctb character_description`
to provide descriptions of the corresponding Unicode characters.
"""

from __future__ import annotations

import argparse
import csv
import glob
import json
import os
import re
import shutil
import signal
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

# Gracefully handle SIGPIPE when piping output to head, less, etc.
if hasattr(signal, "SIGPIPE"):
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)



@dataclass
class DuplicativeDcEntry:
    short_id: int
    dc_id: int
    name: str
    category: str
    is_deprecated: bool
    expected_codepoint: Optional[int]
    expected_hex: str
    equiv_tokens: List[str]
    equiv_codepoint: Optional[int]
    equiv_hex: Optional[str]
    is_matching: bool
    description: str
    source_file: str


def find_workspace_root() -> Path:
    """Find repository workspace root from script location."""
    current = Path(__file__).resolve().parent
    if (current.parent / "Cargo.toml").is_file():
        return current.parent
    return Path.cwd()


def discover_ctb_binary(custom_path: Optional[str] = None) -> Optional[str]:
    """Find the ctb / ctoolbox executable."""
    if custom_path:
        if os.path.isfile(custom_path) and os.access(custom_path, os.X_OK):
            return custom_path
        print(f"Warning: Specified ctb path '{custom_path}' not executable", file=sys.stderr)

    # 1. Search PATH for 'ctb'
    which_ctb = shutil.which("ctb")
    if which_ctb:
        return which_ctb

    # 2. Search PATH for 'ctoolbox'
    which_ctoolbox = shutil.which("ctoolbox")
    if which_ctoolbox:
        return which_ctoolbox

    # 3. Check known workspace build targets
    workspace_root = find_workspace_root()
    candidates = [
        workspace_root / "target" / "x86_64-unknown-linux-musl" / "release" / "ctoolbox",
        workspace_root / "built" / "linux-x64" / "ctoolbox",
        workspace_root / "target" / "release" / "ctoolbox",
        workspace_root / "target" / "x86_64-unknown-linux-musl" / "debug" / "ctoolbox",
        workspace_root / "target" / "debug" / "ctoolbox",
    ]
    for candidate in candidates:
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)

    return None


def fetch_character_descriptions(
    codepoints: Set[int], ctb_binary: Optional[str]
) -> Dict[int, str]:
    """Batch-query character descriptions using `ctb character_description`."""
    descriptions: Dict[int, str] = {}
    if not ctb_binary or not codepoints:
        return descriptions

    # Batch query via stdin text
    sorted_cps = sorted(codepoints)
    input_chars = "".join(chr(cp) for cp in sorted_cps if 0 <= cp <= 0x10FFFF)

    try:
        proc = subprocess.run(
            [ctb_binary, "character_description"],
            input=input_chars.encode("utf-8"),
            capture_output=True,
            check=False,
        )
        if proc.returncode == 0:
            for line in proc.stdout.decode("utf-8", errors="replace").splitlines():
                line = line.strip()
                if " : " in line:
                    cp_str, desc = line.split(" : ", 1)
                    if cp_str.startswith("U+"):
                        try:
                            cp_val = int(cp_str[2:], 16)
                            descriptions[cp_val] = desc.strip()
                        except ValueError:
                            pass
    except Exception as err:
        print(f"Warning: Failed batch running {ctb_binary}: {err}", file=sys.stderr)

    # Fallback for any missing codepoints using single --codepoint invocations
    for cp in sorted_cps:
        if cp not in descriptions and 0 <= cp <= 0x10FFFF:
            try:
                proc = subprocess.run(
                    [ctb_binary, "character_description", "--codepoint", f"U+{cp:04X}"],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                if proc.returncode == 0:
                    out = proc.stdout.strip()
                    desc = out.split(" : ", 1)[1] if " : " in out else out
                    descriptions[cp] = desc.strip()
            except Exception:
                pass

    return descriptions


def load_eite_mappings(
    workspace_root: Path,
) -> Tuple[Dict[int, int], Dict[int, int], Dict[int, int]]:
    """Load inbound/outbound Unicode and ASCII mappings from EITE data CSVs."""
    from_uni_path = workspace_root / "src/formats/eite/data/mappings/from/unicode.csv"
    to_uni_path = workspace_root / "src/formats/eite/data/mappings/to/unicode.csv"
    from_ascii_path = workspace_root / "src/formats/eite/data/mappings/from/ascii.csv"

    from_uni: Dict[int, int] = {}
    if from_uni_path.is_file():
        with open(from_uni_path, newline="", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            for row in reader:
                src = row.get("Source", "").strip()
                seq = row.get("Dc sequence", "").strip()
                if src and seq and seq.isdigit():
                    try:
                        from_uni[int(seq)] = int(src, 16)
                    except ValueError:
                        pass

    to_uni: Dict[int, int] = {}
    if to_uni_path.is_file():
        with open(to_uni_path, newline="", encoding="utf-8") as f:
            reader = csv.reader(f)
            try:
                next(reader)  # Skip header
            except StopIteration:
                pass
            for row in reader:
                if len(row) >= 2 and row[0].strip() and row[1].strip():
                    try:
                        to_uni[int(row[0].strip())] = int(row[1].strip(), 16)
                    except ValueError:
                        pass

    from_ascii: Dict[int, int] = {}
    if from_ascii_path.is_file():
        with open(from_ascii_path, newline="", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            for row in reader:
                src = row.get("Source", "").strip()
                seq = row.get("Dc sequence", "").strip()
                if src and seq and seq.isdigit():
                    try:
                        from_ascii[int(seq)] = int(src, 16)
                    except ValueError:
                        pass

    return from_uni, to_uni, from_ascii


def load_dc_definitions(workspace_root: Path) -> Dict[int, Dict[str, Any]]:
    """Load all short Dc definitions from src/formats/dc_data/data/categories/*.csv."""
    categories_dir = workspace_root / "src/formats/dc_data/data/categories"
    dcs: Dict[int, Dict[str, Any]] = {}

    for csv_file in sorted(glob.glob(str(categories_dir / "*.csv"))):
        base = os.path.basename(csv_file)
        if base == "schema.csv" or base.endswith(".generated.csv"):
            continue

        cat_name = base[:-4]
        with open(csv_file, newline="", encoding="utf-8") as f:
            reader = csv.reader(f)
            try:
                next(reader)  # Skip header
            except StopIteration:
                continue

            for row_idx, row in enumerate(reader, start=2):
                if not row or not row[0].strip():
                    continue

                try:
                    dc_id = int(row[0].strip())
                    short_id = int(row[1].strip())
                except (ValueError, IndexError):
                    continue

                raw_name = row[2].strip() if len(row) > 2 else ""
                is_dep = raw_name.startswith("!")
                clean_name = raw_name.lstrip("!").strip()
                aliases_col = row[8].strip() if len(row) > 8 else ""

                equiv_tokens: List[str] = []
                for match in re.finditer(r"<equiv>([^,>]*)", aliases_col):
                    equiv_tokens.append(match.group(1).strip())

                dcs[short_id] = {
                    "dc_id": dc_id,
                    "short_id": short_id,
                    "name": clean_name,
                    "category": cat_name,
                    "is_deprecated": is_dep,
                    "equiv_tokens": equiv_tokens,
                    "aliases_raw": aliases_col,
                    "source_file": base,
                    "line_number": row_idx,
                }

    return dcs


def collect_and_classify_entries(
    workspace_root: Path, ctb_binary: Optional[str]
) -> Tuple[
    List[DuplicativeDcEntry],
    List[DuplicativeDcEntry],
    List[DuplicativeDcEntry],
    List[DuplicativeDcEntry],
]:
    """Find all short Dcs duplicative of Unicode characters and partition into 4 groups."""
    from_uni, to_uni, from_ascii = load_eite_mappings(workspace_root)
    dcs = load_dc_definitions(workspace_root)

    # 1. Gather all candidate short Dc IDs that likely duplicate Unicode characters:
    #    - Included in EITE inbound mappings (from/unicode.csv or from/ascii.csv)
    #    - Included in EITE outbound overrides (to/unicode.csv)
    #    - Defined with an <equiv>u<hex> tag in the category tables
    candidate_ids: Set[int] = set(from_uni.keys()) | set(to_uni.keys()) | set(from_ascii.keys())
    for sid, d in dcs.items():
        for token in d["equiv_tokens"]:
            if token.lower().startswith("u"):
                try:
                    int(token[1:], 16)
                    candidate_ids.add(sid)
                except ValueError:
                    pass

    # 2. Gather codepoints to query descriptions for
    all_codepoints: Set[int] = set()
    for sid in candidate_ids:
        cp = to_uni.get(sid) or from_uni.get(sid) or from_ascii.get(sid)
        if cp is not None:
            all_codepoints.add(cp)
        d = dcs.get(sid)
        if d:
            for token in d["equiv_tokens"]:
                if token.lower().startswith("u"):
                    try:
                        all_codepoints.add(int(token[1:], 16))
                    except ValueError:
                        pass

    descriptions = fetch_character_descriptions(all_codepoints, ctb_binary)

    # 3. Classify into 4 groups
    g1: List[DuplicativeDcEntry] = []
    g2: List[DuplicativeDcEntry] = []
    g3: List[DuplicativeDcEntry] = []
    g4: List[DuplicativeDcEntry] = []

    for sid in sorted(candidate_ids):
        d = dcs.get(sid)
        if not d:
            continue

        # Expected codepoint: outbound mapping takes precedence over inbound
        expected_cp = to_uni.get(sid) or from_uni.get(sid) or from_ascii.get(sid)

        # Parse <equiv> codepoint
        equiv_cp: Optional[int] = None
        for token in d["equiv_tokens"]:
            if token.lower().startswith("u"):
                try:
                    equiv_cp = int(token[1:], 16)
                    break
                except ValueError:
                    pass

        # If expected is not in EITE CSVs but defined in <equiv>u..., fallback to equiv_cp
        if expected_cp is None and equiv_cp is not None:
            expected_cp = equiv_cp

        is_matching = (
            expected_cp is not None and equiv_cp is not None and expected_cp == equiv_cp
        )
        is_deprecated = d["is_deprecated"]

        expected_hex_str = f"U+{expected_cp:04X}" if expected_cp is not None else "None"
        equiv_hex_str = f"U+{equiv_cp:04X}" if equiv_cp is not None else None
        desc_text = (
            descriptions.get(expected_cp, "(description unavailable)")
            if expected_cp is not None
            else "(no Unicode mapping)"
        )

        entry = DuplicativeDcEntry(
            short_id=sid,
            dc_id=d["dc_id"],
            name=d["name"],
            category=d["category"],
            is_deprecated=is_deprecated,
            expected_codepoint=expected_cp,
            expected_hex=expected_hex_str,
            equiv_tokens=d["equiv_tokens"],
            equiv_codepoint=equiv_cp,
            equiv_hex=equiv_hex_str,
            is_matching=is_matching,
            description=desc_text,
            source_file=d["source_file"],
        )

        if is_deprecated and is_matching:
            g1.append(entry)
        elif is_deprecated and not is_matching:
            g2.append(entry)
        elif not is_deprecated and is_matching:
            g3.append(entry)
        else:
            g4.append(entry)

    return g1, g2, g3, g4


def print_text_format(
    g1: List[DuplicativeDcEntry],
    g2: List[DuplicativeDcEntry],
    g3: List[DuplicativeDcEntry],
    g4: List[DuplicativeDcEntry],
    filter_group: str,
    summary_only: bool,
) -> None:
    """Print results in a human-readable, formatted text view."""
    groups_data = [
        (
            1,
            "Already deprecated and have matching <equiv> equivalency",
            g1,
        ),
        (
            2,
            "Already deprecated but no or mismatched equivalency",
            g2,
        ),
        (
            3,
            "Correct equivalency but no deprecation",
            g3,
        ),
        (
            4,
            "No or mismatched equivalency and no deprecation",
            g4,
        ),
    ]

    total_candidates = len(g1) + len(g2) + len(g3) + len(g4)

    if summary_only:
        print("=" * 72)
        print("SUMMARY: Short Dcs Duplicative of Unicode Characters")
        print("=" * 72)
        for num, title, entries in groups_data:
            pct = (len(entries) / total_candidates * 100) if total_candidates else 0
            print(f"Group {num}: {len(entries):3} ({pct:5.1f}%) - {title}")
        print("-" * 72)
        print(f"Total Candidate Short Dcs: {total_candidates}")
        print("=" * 72)
        return

    for num, title, entries in groups_data:
        if filter_group not in ("all", str(num)):
            continue

        print("\n" + "=" * 78)
        print(f"GROUP {num}: {title} (Total: {len(entries)})")
        print("=" * 78)

        if not entries:
            print("  (None)")
            continue

        for e in entries:
            extra_info = []
            if e.equiv_tokens:
                equiv_str = ", ".join(f"<{t}>" if not t.startswith("<") else t for t in e.equiv_tokens)
                if not e.is_matching:
                    extra_info.append(f"Current equiv: {equiv_str}")
            else:
                if not e.is_matching:
                    extra_info.append("No <equiv>")

            extra_str = f" [{'; '.join(extra_info)}]" if extra_info else ""
            print(
                f"  Dc {e.short_id:3} [{e.category:12}] {e.expected_hex:7} : {e.description} (Dc: {e.name}){extra_str}"
            )

    if filter_group == "all":
        print("\n" + "-" * 78)
        print(
            f"Counts: Group 1: {len(g1)} | Group 2: {len(g2)} | Group 3: {len(g3)} | Group 4: {len(g4)} | Total: {total_candidates}"
        )
        print("-" * 78)


def print_markdown_format(
    g1: List[DuplicativeDcEntry],
    g2: List[DuplicativeDcEntry],
    g3: List[DuplicativeDcEntry],
    g4: List[DuplicativeDcEntry],
    filter_group: str,
) -> None:
    """Print results as a Markdown document."""
    print("# Short Dcs Duplicative of Unicode Characters\n")

    groups_data = [
        (1, "Already deprecated and have matching `<equiv>` equivalency", g1),
        (2, "Already deprecated but no or mismatched equivalency", g2),
        (3, "Correct equivalency but no deprecation", g3),
        (4, "No or mismatched equivalency and no deprecation", g4),
    ]

    for num, title, entries in groups_data:
        if filter_group not in ("all", str(num)):
            continue

        print(f"## Group {num}: {title} ({len(entries)})\n")
        if not entries:
            print("*None*\n")
            continue

        print("| Short ID | Category | Unicode Hex | Character Description | Dc Name | Existing Equiv |")
        print("|:---:|:---|:---:|:---|:---|:---|")
        for e in entries:
            equiv_str = ", ".join(e.equiv_tokens) if e.equiv_tokens else "*(none)*"
            print(
                f"| {e.short_id} | `{e.category}` | `{e.expected_hex}` | {e.description} | {e.name} | `{equiv_str}` |"
            )
        print()


def print_json_format(
    g1: List[DuplicativeDcEntry],
    g2: List[DuplicativeDcEntry],
    g3: List[DuplicativeDcEntry],
    g4: List[DuplicativeDcEntry],
) -> None:
    """Print results as structured JSON."""
    data = {
        "summary": {
            "group_1_deprecated_matching": len(g1),
            "group_2_deprecated_mismatched_or_none": len(g2),
            "group_3_not_deprecated_matching": len(g3),
            "group_4_not_deprecated_mismatched_or_none": len(g4),
            "total_candidates": len(g1) + len(g2) + len(g3) + len(g4),
        },
        "groups": {
            "group_1": [asdict(e) for e in g1],
            "group_2": [asdict(e) for e in g2],
            "group_3": [asdict(e) for e in g3],
            "group_4": [asdict(e) for e in g4],
        },
    }
    print(json.dumps(data, indent=2))


def print_csv_format(
    g1: List[DuplicativeDcEntry],
    g2: List[DuplicativeDcEntry],
    g3: List[DuplicativeDcEntry],
    g4: List[DuplicativeDcEntry],
) -> None:
    """Print results as flat CSV."""
    writer = csv.writer(sys.stdout)
    writer.writerow([
        "Group",
        "Group Title",
        "Short ID",
        "Dc ID",
        "Category",
        "Unicode Hex",
        "Character Description",
        "Dc Name",
        "Is Deprecated",
        "Is Equiv Matching",
        "Equiv Tokens",
        "Source File",
    ])

    groups_data = [
        (1, "Already deprecated and have matching <equiv> equivalency", g1),
        (2, "Already deprecated but no or mismatched equivalency", g2),
        (3, "Correct equivalency but no deprecation", g3),
        (4, "No or mismatched equivalency and no deprecation", g4),
    ]

    for num, title, entries in groups_data:
        for e in entries:
            writer.writerow([
                num,
                title,
                e.short_id,
                e.dc_id,
                e.category,
                e.expected_hex,
                e.description,
                e.name,
                e.is_deprecated,
                e.is_matching,
                ";".join(e.equiv_tokens),
                e.source_file,
            ])


def main() -> int:
    parser = argparse.ArgumentParser(
        description="List short Dcs duplicative of Unicode characters grouped by deprecation and equivalency status."
    )
    parser.add_argument(
        "--format",
        choices=["text", "markdown", "json", "csv"],
        default="text",
        help="Output format (default: text)",
    )
    parser.add_argument(
        "--group",
        choices=["1", "2", "3", "4", "all", "todo"],
        default="all",
        help="Filter output to a specific group, all, or 'todo' (groups 2, 3, 4)",
    )
    parser.add_argument(
        "--ctb-path",
        default=None,
        help="Path to ctb or ctoolbox executable (auto-discovered if not specified)",
    )
    parser.add_argument(
        "--summary-only",
        action="store_true",
        help="Print summary counts only",
    )

    args = parser.parse_args()
    workspace_root = find_workspace_root()

    ctb_binary = discover_ctb_binary(args.ctb_path)
    if not ctb_binary:
        print(
            "Warning: 'ctb' or 'ctoolbox' executable not found in PATH or build targets. Descriptions will be unavailable.",
            file=sys.stderr,
        )

    g1, g2, g3, g4 = collect_and_classify_entries(workspace_root, ctb_binary)

    if args.group == "todo":
        # todo means unaddressed groups 2, 3, 4
        filter_group = "all"  # Handled below
    else:
        filter_group = args.group

    if args.format == "text":
        if args.group == "todo":
            print_text_format([], g2, g3, g4, "all", args.summary_only)
        else:
            print_text_format(g1, g2, g3, g4, filter_group, args.summary_only)
    elif args.format == "markdown":
        if args.group == "todo":
            print_markdown_format([], g2, g3, g4, "all")
        else:
            print_markdown_format(g1, g2, g3, g4, filter_group)
    elif args.format == "json":
        print_json_format(g1, g2, g3, g4)
    elif args.format == "csv":
        print_csv_format(g1, g2, g3, g4)

    return 0


if __name__ == "__main__":
    sys.exit(main())
