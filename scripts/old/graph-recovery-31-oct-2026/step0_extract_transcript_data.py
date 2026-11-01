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

"""Step 0: Extract transcript evidence, audit wall-clock execution windows,
replay the unmodified historical generator on commit 3345d2dff + Run 1/Run 2 inputs,
and precompute collision_manifest.json, step2a_manifest.json, and step2b_manifest.json.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
TRANSCRIPT_FULL_PATH = Path(
    "/root/.gemini/antigravity/brain/23f9eaf6-7cd6-416e-8470-852b9878ea88"
    "/.system_generated/logs/transcript_full.jsonl"
)
AUDIT_DIR = REPO_ROOT / "src/formats/dcdata/data-src/recovery-audit"
GRAPH_DIR = REPO_ROOT / "assets/data/graph"
BASELINE_COMMIT = "3345d2dff"
RUN2_COMMIT = "a90464028"
MAX_NODES_PER_CHUNK = 100
SYSTEM_REGION_START = 8589934592


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def parse_chunk_start_id(filename: str) -> int | None:
    if not filename.endswith(".ctbnodes"):
        return None
    stem = filename[: -len(".ctbnodes")]
    digits = stem.replace("_", "")
    if not digits.isdigit():
        return None
    return int(digits)


def format_chunk_filename(start_id: int) -> str:
    s = f"{start_id:010d}"
    return f"{s[:5]}_{s[5:]}.ctbnodes"


def chunk_filename_for_node_id(node_id: int) -> str:
    offset = node_id - SYSTEM_REGION_START
    chunk_start = SYSTEM_REGION_START + (offset // MAX_NODES_PER_CHUNK) * MAX_NODES_PER_CHUNK
    return format_chunk_filename(chunk_start)


def compute_ledger_dir_sha256(graph_dir: Path) -> str:
    entries: list[tuple[int, str, bytes]] = []
    for entry in os.listdir(graph_dir):
        start_id = parse_chunk_start_id(entry)
        if start_id is not None:
            entries.append((start_id, entry, (graph_dir / entry).read_bytes()))
    entries.sort(key=lambda item: item[0])
    h = hashlib.sha256()
    for _, name, content in entries:
        h.update(name.encode("utf-8"))
        h.update(b"\0")
        h.update(content)
        h.update(b"\0")
    return h.hexdigest()


def compute_chunks_map_sha256(chunks: dict[str, list[str]]) -> str:
    entries: list[tuple[int, str, bytes]] = []
    for name, lines in chunks.items():
        start_id = parse_chunk_start_id(name)
        assert start_id is not None, f"Invalid chunk filename: {name}"
        content = "".join(line + "\n" for line in lines).encode("utf-8")
        entries.append((start_id, name, content))
    entries.sort(key=lambda item: item[0])
    h = hashlib.sha256()
    for _, name, content in entries:
        h.update(name.encode("utf-8"))
        h.update(b"\0")
        h.update(content)
        h.update(b"\0")
    return h.hexdigest()


def parse_ctbnodes_line(raw_line: str) -> tuple[int, list[str], str]:
    line = raw_line.rstrip("\r\n")
    if "#" in line:
        body_part, comment_part = line.split("#", 1)
        comment = comment_part.strip()
    else:
        body_part = line
        comment = ""
    lhs, rhs = body_part.split(":", 1)
    lhs = lhs.strip()
    assert lhs.startswith("l"), f"Expected l<id> prefix in {raw_line!r}"
    node_id = int(lhs[1:])
    tokens = rhs.strip().split() if rhs.strip() else []
    return node_id, tokens, comment


def load_ctbnodes_dir(graph_dir: Path) -> tuple[dict[int, tuple[list[str], str, str]], dict[str, list[str]]]:
    chunk_files: list[tuple[int, str]] = []
    for entry in os.listdir(graph_dir):
        start_id = parse_chunk_start_id(entry)
        if start_id is not None:
            chunk_files.append((start_id, entry))
    chunk_files.sort()

    nodes: dict[int, tuple[list[str], str, str]] = {}
    chunks: dict[str, list[str]] = {}
    for _, fname in chunk_files:
        raw_text = (graph_dir / fname).read_text(encoding="utf-8")
        lines = [ln for ln in raw_text.splitlines() if ln.strip()]
        chunks[fname] = lines
        for ln in lines:
            nid, toks, comment = parse_ctbnodes_line(ln)
            nodes[nid] = (toks, comment, ln)
    return nodes, chunks


def parse_statement_tokens(tokens: list[str]) -> tuple[str, str, str] | None:
    relationship_verbs = {
        "582", "583", "584", "585", "586", "587", "588", "589",
        "590", "591", "592", "598", "610", "614",
    }
    if not tokens or tokens[0] not in relationship_verbs:
        return None
    verb = tokens[0]
    idx = 1

    def read_atom(pos: int) -> tuple[str, int] | None:
        if pos >= len(tokens):
            return None
        if tokens[pos] == "255":
            if pos + 1 >= len(tokens):
                return None
            return tokens[pos + 1], pos + 2
        if tokens[pos] == "616":
            if pos + 1 >= len(tokens):
                return None
            return tokens[pos + 1], pos + 2
        return tokens[pos], pos + 1

    subj_res = read_atom(idx)
    if subj_res is None:
        return None
    subj, idx = subj_res
    obj_res = read_atom(idx)
    if obj_res is None:
        return None
    obj, idx = obj_res
    if idx != len(tokens):
        return None
    return verb, subj, obj


def compute_effective_inactive_ids(nodes_in_order: list[tuple[int, list[str]]]) -> set[int]:
    inactive_ids: set[int] = set()
    for node_id, tokens in reversed(nodes_in_order):
        if node_id in inactive_ids:
            continue
        stmt = parse_statement_tokens(tokens)
        if stmt is None:
            continue
        verb, subj, obj = stmt
        if verb == "591":
            if obj.startswith("l") and obj[1:].isdigit():
                inactive_ids.add(int(obj[1:]))
        elif verb == "592" and subj == "615":
            if obj.startswith("l") and obj[1:].isdigit():
                inactive_ids.add(int(obj[1:]))
    return inactive_ids


def extract_git_tree_subset(commit: str, dest_dir: Path, paths: list[str]) -> None:
    cmd = ["git", "archive", commit, "--", *paths]
    proc = subprocess.run(cmd, cwd=REPO_ROOT, check=True, capture_output=True)
    with tempfile.NamedTemporaryFile(suffix=".tar", delete=False) as tmp_tar:
        tmp_tar.write(proc.stdout)
        tmp_tar_path = Path(tmp_tar.name)
    try:
        subprocess.run(["tar", "-xf", str(tmp_tar_path), "-C", str(dest_dir)], check=True)
    finally:
        tmp_tar_path.unlink()


def build_historical_a90464028_binary(dest_bin: Path) -> None:
    files = ["src/build_support/ctbnbdl.rs", "src/formats/dcdata/graph_ledger.rs"]
    subprocess.run(
        ["git", "checkout", RUN2_COMMIT, "--", *files],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    try:
        proc = subprocess.run(
            ["cargo", "build", "-p", "ctb-formats-dcdata", "--bin", "validate-data-tables"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise RuntimeError(f"cargo build a90464028 failed:\n{proc.stderr}")
        built = REPO_ROOT / "target/debug/validate-data-tables"
        shutil.copy2(built, dest_bin)
    finally:
        subprocess.run(
            ["git", "checkout", "HEAD", "--", *files],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
        )


def run_historical_replay(
    validate_bin: Path,
    document_csv_override: bytes | None,
) -> tuple[str, dict[int, tuple[list[str], str, str]]]:
    with tempfile.TemporaryDirectory(prefix="ctb-step0-replay-") as tmp_str:
        sandbox = Path(tmp_str)
        # Extract workspace root files, src/formats, src/storage/minimal, and src/utilities from RUN2_COMMIT (a90464028)
        extract_git_tree_subset(
            RUN2_COMMIT,
            sandbox,
            [
                ".ctoolbox_workspace_root",
                "Cargo.toml",
                "src/formats",
                "src/storage/minimal",
                "src/utilities",
            ],
        )
        # Extract baseline 3345d2dff assets (including assets/data/graph: 17,207 nodes l8589934592..l8589951798)
        extract_git_tree_subset(
            BASELINE_COMMIT,
            sandbox,
            ["assets"],
        )
        (sandbox / "vendor").symlink_to(REPO_ROOT / "vendor")

        if document_csv_override is not None:
            doc_csv_path = (
                sandbox / "src/formats/dcdata/fixtures/categories/formats/document.csv"
            )
            doc_csv_path.write_bytes(document_csv_override)

        env = os.environ.copy()
        env.pop("CARGO_MANIFEST_DIR", None)
        proc = subprocess.run(
            [str(validate_bin), "--write"],
            cwd=sandbox,
            env=env,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise RuntimeError(
                f"validate-data-tables failed (code {proc.returncode}):\nSTDOUT:\n{proc.stdout}\nSTDERR:\n{proc.stderr}"
            )
        nodes, _ = load_ctbnodes_dir(sandbox / "assets/data/graph")
        return proc.stdout, nodes


def main() -> None:
    AUDIT_DIR.mkdir(parents=True, exist_ok=True)

    # 1. Inspect transcript_full.jsonl and extract observed records + wall-clock audit
    records_by_step: dict[int, tuple[int, dict[str, Any]]] = {}
    with TRANSCRIPT_FULL_PATH.open("r", encoding="utf-8") as f:
        for phys_line, raw in enumerate(f, start=1):
            obj = json.loads(raw)
            step_idx = obj.get("step_index")
            if isinstance(step_idx, int):
                records_by_step[step_idx] = (phys_line, obj)

    assert 684 in records_by_step, "Missing step_index == 684 in transcript_full.jsonl"
    phys_684, rec_684 = records_by_step[684]
    assert phys_684 == 685, f"Expected step_index == 684 on physical line 685, got {phys_684}"

    tool_calls_684 = rec_684.get("tool_calls", [])
    doc_tc = [
        tc
        for tc in tool_calls_684
        if tc.get("args", {}).get("Path", "").endswith("categories/formats/document.csv")
    ]
    assert len(doc_tc) == 1, f"Expected 1 document.csv edit_file call in step 684, got {len(doc_tc)}"
    args_684 = doc_tc[0]["args"]
    target_slice: str = args_684["Target"]
    replacement_slice: str = args_684["Replacement"]

    # Extract key step records for observed_step687_and_698_logs.json
    key_steps = [679, 680, 684, 685, 686, 687, 688, 689, 692, 693, 698]
    observed_logs: dict[str, Any] = {}
    for s in key_steps:
        if s in records_by_step:
            p_line, rec = records_by_step[s]
            observed_logs[str(s)] = {
                "physical_line": p_line,
                "step_index": s,
                "type": rec.get("type"),
                "status": rec.get("status"),
                "created_at": rec.get("created_at"),
                "content": rec.get("content"),
            }

    # Verify wall-clock timestamps for task-680 vs step 685 edit_file
    step685_content = observed_logs["685"]["content"]
    step687_content = observed_logs["687"]["content"]
    m_685_created = re.search(r"Created At:\s*(\S+)", step685_content)
    m_685_completed = re.search(r"Completed At:\s*(\S+)", step685_content)
    m_680_finished = re.search(r"\[Message\]\s*timestamp=(\S+)", step687_content)
    assert m_685_created and m_685_completed and m_680_finished
    task680_exit_ts = m_680_finished.group(1)
    step685_start_ts = m_685_created.group(1)
    step685_end_ts = m_685_completed.group(1)
    assert task680_exit_ts == "2026-10-31T19:28:04Z"
    assert step685_start_ts == "2026-10-31T19:28:43Z"
    assert step685_end_ts == "2026-10-31T19:28:46Z"

    # Audit git diff between 3345d2dff and a90464028
    git_diff_stat = subprocess.run(
        ["git", "diff", "--stat", f"{BASELINE_COMMIT}..{RUN2_COMMIT}"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout

    # Write observed evidence files
    (AUDIT_DIR / "observed_step684_record.json").write_text(
        json.dumps({"physical_line": phys_684, "record": rec_684}, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (AUDIT_DIR / "observed_step687_and_698_logs.json").write_text(
        json.dumps(observed_logs, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (AUDIT_DIR / "observed_run1_document_csv_slice.csv").write_text(target_slice, encoding="utf-8")
    (AUDIT_DIR / "observed_run2_document_csv_slice.csv").write_text(replacement_slice, encoding="utf-8")

    current_nodes, current_chunks = load_ctbnodes_dir(GRAPH_DIR)
    assert len(current_nodes) == 17833, f"Expected 17833 nodes, got {len(current_nodes)}"
    assert max(current_nodes) == 8589952424

    observed_run2_lines = [
        current_nodes[nid][2] for nid in range(8589951799, 8589952425)
    ]
    (AUDIT_DIR / "observed_run2_nodes_8589951799_8589952424.ctbnodes").write_text(
        "\n".join(observed_run2_lines) + "\n",
        encoding="utf-8",
    )

    # 2. Replay unmodified historical a90464028 generator on Run 2 and Run 1 inputs
    validate_bin = REPO_ROOT / "target/debug/validate-data-tables-a90464028"
    build_historical_a90464028_binary(validate_bin)
    try:
        run2_stdout, replay_run2_nodes = run_historical_replay(validate_bin, None)
        assert "System Graph Ledger: 17833 total nodes (626 appended, 1 superseded, 0 retracted, 7 chunk files updated)." in run2_stdout
        for nid in range(8589951799, 8589952425):
            assert replay_run2_nodes[nid][2] == current_nodes[nid][2], (
                f"Run 2 replay mismatch at l{nid}: {replay_run2_nodes[nid][2]!r} != {current_nodes[nid][2]!r}"
            )

        # Reconstruct Run 1 document.csv by reversing Replacement -> Target on commit a90464028's document.csv
        run2_doc_csv = subprocess.run(
            ["git", "show", f"{RUN2_COMMIT}:src/formats/dcdata/fixtures/categories/formats/document.csv"],
            cwd=REPO_ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        assert replacement_slice in run2_doc_csv, "Replacement slice not found in a90464028 document.csv"
        run1_doc_csv = run2_doc_csv.replace(replacement_slice, target_slice, 1)

        run1_stdout, replay_run1_nodes = run_historical_replay(
            validate_bin, run1_doc_csv.encode("utf-8")
        )
        assert "System Graph Ledger: 17890 total nodes (683 appended, 1 superseded, 0 retracted, 7 chunk files updated)." in run1_stdout
        assert len(replay_run1_nodes) == 17890
        assert max(replay_run1_nodes) == 8589952481
    finally:
        if validate_bin.is_file():
            validate_bin.unlink()

    reconstructed_run1_lines = [
        replay_run1_nodes[nid][2] for nid in range(8589951799, 8589952482)
    ]
    (AUDIT_DIR / "reconstructed_run1_nodes_8589951799_8589952481.ctbnodes").write_text(
        "\n".join(reconstructed_run1_lines) + "\n",
        encoding="utf-8",
    )

    # Write invocation_and_mutation_window_audit.json
    window_audit = {
        "baseline_commit": BASELINE_COMMIT,
        "run2_commit": RUN2_COMMIT,
        "run1_task": {
            "task_id": "23f9eaf6-7cd6-416e-8470-852b9878ea88/task-680",
            "launch_step_index": 679,
            "launch_timestamp": "2026-10-31T19:27:09Z",
            "backgrounded_step_index": 680,
            "backgrounded_timestamp": "2026-10-31T19:27:33Z",
            "process_exit_timestamp": task680_exit_ts,
            "notification_delivered_step_index": 687,
            "notification_delivered_timestamp": observed_logs["687"]["created_at"],
            "reported_summary": "System Graph Ledger: 17890 total nodes (683 appended, 1 superseded, 0 retracted, 7 chunk files updated).",
        },
        "document_csv_edit": {
            "proposal_step_index": 684,
            "proposal_physical_line": phys_684,
            "execution_step_index": 685,
            "execution_started_at": step685_start_ts,
            "execution_completed_at": step685_end_ts,
            "seconds_after_task680_exit": 39,
        },
        "run2_task": {
            "task_id": "23f9eaf6-7cd6-416e-8470-852b9878ea88/task-693",
            "launch_step_index": 692,
            "launch_timestamp": "2026-10-31T19:29:15Z",
            "process_exit_timestamp": "2026-10-31T19:30:01Z",
            "notification_delivered_step_index": 698,
            "reported_summary": "System Graph Ledger: 17833 total nodes (626 appended, 1 superseded, 0 retracted, 7 chunk files updated).",
        },
        "epistemic_boundary": (
            "Run 1's .ctbnodes files were overwritten at steps 688..693 before being read into the transcript, "
            "and task-680 ran without per-syscall file-access tracing. Therefore, "
            "reconstructed_run1_nodes_8589951799_8589952481.ctbnodes is labeled as deterministic reconstruction "
            "from the unmodified generator corroborated by the 19:28:04Z summary metrics (17,890 total / 683 appended / "
            "1 superseded / 0 retracted / 7 chunk files updated) and 100% byte-for-byte Run 2 replay."
        ),
        "git_diff_stat_3345d2dff_to_a90464028": git_diff_stat,
    }
    (AUDIT_DIR / "invocation_and_mutation_window_audit.json").write_text(
        json.dumps(window_audit, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    # 3. Compute transitive dependency-ambiguity closure over 8589951799..=8589952424
    collision_ids = list(range(8589951799, 8589952425))
    tail_ids = list(range(8589952425, 8589952482))
    ambiguous_set: set[int] = set()
    for nid in collision_ids:
        if replay_run1_nodes[nid][0] != current_nodes[nid][0]:
            ambiguous_set.add(nid)

    changed = True
    while changed:
        changed = False
        for nid in collision_ids:
            if nid in ambiguous_set:
                continue
            r1_toks = replay_run1_nodes[nid][0]
            r2_toks = current_nodes[nid][0]
            for tok in r1_toks + r2_toks:
                if tok.startswith("l") and tok[1:].isdigit():
                    ref_id = int(tok[1:])
                    if ref_id in ambiguous_set:
                        ambiguous_set.add(nid)
                        changed = True
                        break

    ambiguous_ids = sorted(ambiguous_set)
    unambiguous_ids = sorted(set(collision_ids) - ambiguous_set)
    assert len(unambiguous_ids) == 24, f"Expected 24 unambiguous IDs, got {len(unambiguous_ids)}"
    assert len(ambiguous_ids) == 602, f"Expected 602 ambiguous IDs, got {len(ambiguous_ids)}"
    assert len(tail_ids) == 57, f"Expected 57 tail IDs, got {len(tail_ids)}"

    collision_manifest = {
        "baseline_commit": BASELINE_COMMIT,
        "run2_commit": RUN2_COMMIT,
        "collision_range_start": 8589951799,
        "collision_range_end": 8589952424,
        "tail_range_start": 8589952425,
        "tail_range_end": 8589952481,
        "unambiguous_count": len(unambiguous_ids),
        "unambiguous_ids": unambiguous_ids,
        "ambiguous_count": len(ambiguous_ids),
        "ambiguous_ids": ambiguous_ids,
        "erased_tail_count": len(tail_ids),
        "erased_tail_ids": tail_ids,
        "ambiguous_entries": [
            {
                "node_id": nid,
                "reason": (
                    "token_body_differs"
                    if replay_run1_nodes[nid][0] != current_nodes[nid][0]
                    else "references_ambiguous_node"
                ),
                "run1_tokens": replay_run1_nodes[nid][0],
                "run1_comment": replay_run1_nodes[nid][1],
                "run2_tokens": current_nodes[nid][0],
                "run2_comment": current_nodes[nid][1],
            }
            for nid in ambiguous_ids
        ],
        "erased_tail_entries": [
            {
                "node_id": nid,
                "run1_tokens": replay_run1_nodes[nid][0],
                "run1_comment": replay_run1_nodes[nid][1],
            }
            for nid in tail_ids
        ],
    }
    (AUDIT_DIR / "collision_manifest.json").write_text(
        json.dumps(collision_manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    # 4. Precompute Step 2a and Step 2b manifests & exact ledger simulation
    sim_nodes_order: list[tuple[int, list[str]]] = [
        (nid, current_nodes[nid][0]) for nid in sorted(current_nodes)
    ]
    sim_chunks: dict[str, list[str]] = {
        fname: list(lines) for fname, lines in current_chunks.items()
    }
    baseline_ledger_sha256 = compute_ledger_dir_sha256(GRAPH_DIR)
    assert baseline_ledger_sha256 == compute_chunks_map_sha256(sim_chunks)

    inactive_ids = compute_effective_inactive_ids(sim_nodes_order)
    active_by_tokens: dict[tuple[str, ...], set[int]] = {}
    for nid, toks in sim_nodes_order:
        if nid not in inactive_ids:
            active_by_tokens.setdefault(tuple(toks), set()).add(nid)

    def append_simulated_item(
        next_id: int,
        inbox_name: str,
        tokens: list[str],
        comment: str,
        provenance: str = "csv",
    ) -> dict[str, Any]:
        nonlocal inactive_ids
        tok_key = tuple(tokens)
        active_matches = sorted(active_by_tokens.get(tok_key, set()))
        header_lines = [f"# @provenance: {provenance}"]
        if active_matches:
            allow_str = ", ".join(f"l{m}" for m in active_matches)
            header_lines.append(f"# @allow-duplicate-of: {allow_str}")
        header_lines.append(f"# @comment: {comment}")
        body_str = " ".join(tokens)
        dcsh_content = "\n".join(header_lines) + "\n" + body_str + "\n"
        inbox_sha256 = sha256_bytes(dcsh_content.encode("utf-8"))
        allocated_name = f"{next_id}_{inbox_name}"
        raw_line = f"l{next_id}: {body_str} # {comment}" if comment else f"l{next_id}: {body_str}"
        target_chunk = chunk_filename_for_node_id(next_id)
        sim_chunks.setdefault(target_chunk, []).append(raw_line)
        sim_nodes_order.append((next_id, tokens))

        stmt = parse_statement_tokens(tokens)
        if stmt is not None:
            verb, subj, obj = stmt
            if verb == "591" or (verb == "592" and subj == "615"):
                inactive_ids = compute_effective_inactive_ids(sim_nodes_order)
                active_by_tokens.clear()
                for n_id, t_list in sim_nodes_order:
                    if n_id not in inactive_ids:
                        active_by_tokens.setdefault(tuple(t_list), set()).add(n_id)
            else:
                active_by_tokens.setdefault(tok_key, set()).add(next_id)
        else:
            active_by_tokens.setdefault(tok_key, set()).add(next_id)

        return {
            "node_id": next_id,
            "raw_line": raw_line,
            "target_chunk": target_chunk,
            "provenance": provenance,
            "allow_duplicate_of": active_matches,
            "comment": comment,
            "tokens": tokens,
            "inbox_name": inbox_name,
            "allocated_name": allocated_name,
            "inbox_sha256": inbox_sha256,
            "dcsh_content": dcsh_content,
        }

    next_id = 8589952425
    step2a_items: list[dict[str, Any]] = []
    step2a_pinned_inputs: dict[str, str] = {}

    # Step 2a Part 1: Restore 57 erased Run 1 tail nodes at l8589952425..l8589952481
    for nid in tail_ids:
        assert next_id == nid
        toks, comment, _ = replay_run1_nodes[nid]
        inbox_name = f"step2a_0000_tail_{nid}.dcsh"
        item = append_simulated_item(next_id, inbox_name, toks, comment, "csv")
        step2a_items.append(item)
        step2a_pinned_inputs[inbox_name] = item["inbox_sha256"]
        next_id += 1

    # Step 2a Part 2: Allocate 603 quadruplets (2,412 nodes at l8589952482..l8589954893)
    r1_raw_map: dict[int, int] = {}
    r2_raw_map: dict[int, int] = {}
    ambig_raw_map: dict[int, int] = {}
    sup_orig_map: dict[int, int] = {}

    for idx, orig_id in enumerate(ambiguous_ids, start=1):
        r1_toks, r1_comment, _ = replay_run1_nodes[orig_id]
        r2_toks, r2_comment, _ = current_nodes[orig_id]

        # 1. r1_raw(N)
        r1_id = next_id
        r1_raw_map[orig_id] = r1_id
        inbox_1 = f"step2a_{idx:04d}_{orig_id}_1_r1_raw.dcsh"
        item_1 = append_simulated_item(
            r1_id,
            inbox_1,
            r1_toks,
            f"Run 1 raw candidate for l{orig_id}: {r1_comment}",
            "csv",
        )
        step2a_items.append(item_1)
        step2a_pinned_inputs[inbox_1] = item_1["inbox_sha256"]
        next_id += 1

        # 2. r2_raw(N)
        r2_id = next_id
        r2_raw_map[orig_id] = r2_id
        inbox_2 = f"step2a_{idx:04d}_{orig_id}_2_r2_raw.dcsh"
        item_2 = append_simulated_item(
            r2_id,
            inbox_2,
            r2_toks,
            f"Run 2 raw candidate for l{orig_id}: {r2_comment}",
            "csv",
        )
        step2a_items.append(item_2)
        step2a_pinned_inputs[inbox_2] = item_2["inbox_sha256"]
        next_id += 1

        # 3. ambig_raw(N) = 348 310 275 l<r1_raw(N)> 312 l<r2_raw(N)> 311
        ambig_id = next_id
        ambig_raw_map[orig_id] = ambig_id
        inbox_3 = f"step2a_{idx:04d}_{orig_id}_3_ambig_raw.dcsh"
        ambig_toks = ["348", "310", "275", f"l{r1_id}", "312", f"l{r2_id}", "311"]
        item_3 = append_simulated_item(
            ambig_id,
            inbox_3,
            ambig_toks,
            f"Ambiguous values (l{r1_id}, l{r2_id}) for overwritten node l{orig_id}",
            "csv",
        )
        step2a_items.append(item_3)
        step2a_pinned_inputs[inbox_3] = item_3["inbox_sha256"]
        next_id += 1

        # 4. sup_orig(N) = 591 l<ambig_raw(N)> l<orig_id>
        sup_id = next_id
        sup_orig_map[orig_id] = sup_id
        inbox_4 = f"step2a_{idx:04d}_{orig_id}_4_supersede_orig.dcsh"
        sup_toks = ["591", f"l{ambig_id}", f"l{orig_id}"]
        item_4 = append_simulated_item(
            sup_id,
            inbox_4,
            sup_toks,
            f"Supersedes (l{ambig_id}, l{orig_id})",
            "csv",
        )
        step2a_items.append(item_4)
        step2a_pinned_inputs[inbox_4] = item_4["inbox_sha256"]
        next_id += 1

    assert next_id - 1 == 8589954889, f"Expected Step 2a final_last_id 8589954889, got {next_id - 1}"
    assert len(step2a_items) == 2465
    # Verify Step 2a inbox filenames are strictly sorted lexicographically
    assert [it["inbox_name"] for it in step2a_items] == sorted(
        it["inbox_name"] for it in step2a_items
    ), "Step 2a inbox filenames must be in strict lexicographical order"

    step2a_final_sha256 = compute_chunks_map_sha256(sim_chunks)
    step2a_manifest = {
        "manifest_id": "step2a",
        "kind": "inbox_allocation",
        "baseline_last_id": 8589952424,
        "baseline_node_count": 17833,
        "baseline_ledger_sha256": baseline_ledger_sha256,
        "pinned_input_sha256s": step2a_pinned_inputs,
        "planned_items": step2a_items,
        "final_last_id": 8589954889,
        "final_node_count": len(sim_nodes_order),
        "final_ledger_sha256": step2a_final_sha256,
    }
    (AUDIT_DIR / "step2a_manifest.json").write_text(
        json.dumps(step2a_manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    # 5. Precompute Step 2b: Branch-resolved statements, Supersedes links, and updated Dc 348 records
    step2b_items: list[dict[str, Any]] = []
    step2b_pinned_inputs: dict[str, str] = {}

    r1_eff_map: dict[int, int] = {}
    r2_eff_map: dict[int, int] = {}
    ambig_eff_map: dict[int, int] = {}

    for idx, orig_id in enumerate(ambiguous_ids, start=1):
        r1_toks, r1_comment, _ = replay_run1_nodes[orig_id]
        r2_toks, r2_comment, _ = current_nodes[orig_id]
        r1_raw_id = r1_raw_map[orig_id]
        r2_raw_id = r2_raw_map[orig_id]
        ambig_raw_id = ambig_raw_map[orig_id]

        # Check if r1_toks references any ambiguous ID
        r1_rewritten = [
            f"l{r1_eff_map[int(t[1:])]}"
            if (t.startswith("l") and t[1:].isdigit() and int(t[1:]) in ambiguous_set)
            else t
            for t in r1_toks
        ]
        if r1_rewritten != r1_toks:
            r1_res_id = next_id
            inbox_r1 = f"step2b_{idx:04d}_{orig_id}_1_r1_resolved.dcsh"
            item_r1 = append_simulated_item(
                r1_res_id,
                inbox_r1,
                r1_rewritten,
                f"Run 1 branch-resolved candidate for l{orig_id}: {r1_comment}",
                "csv",
            )
            step2b_items.append(item_r1)
            step2b_pinned_inputs[inbox_r1] = item_r1["inbox_sha256"]
            next_id += 1

            sup_r1_id = next_id
            inbox_sup_r1 = f"step2b_{idx:04d}_{orig_id}_2_supersede_r1.dcsh"
            item_sup_r1 = append_simulated_item(
                sup_r1_id,
                inbox_sup_r1,
                ["591", f"l{r1_res_id}", f"l{r1_raw_id}"],
                f"Supersedes (l{r1_res_id}, l{r1_raw_id})",
                "csv",
            )
            step2b_items.append(item_sup_r1)
            step2b_pinned_inputs[inbox_sup_r1] = item_sup_r1["inbox_sha256"]
            next_id += 1
            r1_eff_map[orig_id] = r1_res_id
        else:
            r1_eff_map[orig_id] = r1_raw_id

        # Check if r2_toks references any ambiguous ID
        r2_rewritten = [
            f"l{r2_eff_map[int(t[1:])]}"
            if (t.startswith("l") and t[1:].isdigit() and int(t[1:]) in ambiguous_set)
            else t
            for t in r2_toks
        ]
        if r2_rewritten != r2_toks:
            r2_res_id = next_id
            inbox_r2 = f"step2b_{idx:04d}_{orig_id}_3_r2_resolved.dcsh"
            item_r2 = append_simulated_item(
                r2_res_id,
                inbox_r2,
                r2_rewritten,
                f"Run 2 branch-resolved candidate for l{orig_id}: {r2_comment}",
                "csv",
            )
            step2b_items.append(item_r2)
            step2b_pinned_inputs[inbox_r2] = item_r2["inbox_sha256"]
            next_id += 1

            sup_r2_id = next_id
            inbox_sup_r2 = f"step2b_{idx:04d}_{orig_id}_4_supersede_r2.dcsh"
            item_sup_r2 = append_simulated_item(
                sup_r2_id,
                inbox_sup_r2,
                ["591", f"l{r2_res_id}", f"l{r2_raw_id}"],
                f"Supersedes (l{r2_res_id}, l{r2_raw_id})",
                "csv",
            )
            step2b_items.append(item_sup_r2)
            step2b_pinned_inputs[inbox_sup_r2] = item_sup_r2["inbox_sha256"]
            next_id += 1
            r2_eff_map[orig_id] = r2_res_id
        else:
            r2_eff_map[orig_id] = r2_raw_id

        # If either branch was resolved, allocate updated Dc 348 record and supersede ambig_raw(N)
        if r1_eff_map[orig_id] != r1_raw_id or r2_eff_map[orig_id] != r2_raw_id:
            ambig_res_id = next_id
            r1_e = r1_eff_map[orig_id]
            r2_e = r2_eff_map[orig_id]
            inbox_ambig = f"step2b_{idx:04d}_{orig_id}_5_ambig_resolved.dcsh"
            item_ambig = append_simulated_item(
                ambig_res_id,
                inbox_ambig,
                ["348", "310", "275", f"l{r1_e}", "312", f"l{r2_e}", "311"],
                f"Ambiguous values (l{r1_e}, l{r2_e}) for overwritten node l{orig_id}",
                "csv",
            )
            step2b_items.append(item_ambig)
            step2b_pinned_inputs[inbox_ambig] = item_ambig["inbox_sha256"]
            next_id += 1

            sup_ambig_id = next_id
            inbox_sup_ambig = f"step2b_{idx:04d}_{orig_id}_6_supersede_ambig.dcsh"
            item_sup_ambig = append_simulated_item(
                sup_ambig_id,
                inbox_sup_ambig,
                ["591", f"l{ambig_res_id}", f"l{ambig_raw_id}"],
                f"Supersedes (l{ambig_res_id}, l{ambig_raw_id})",
                "csv",
            )
            step2b_items.append(item_sup_ambig)
            step2b_pinned_inputs[inbox_sup_ambig] = item_sup_ambig["inbox_sha256"]
            next_id += 1
            ambig_eff_map[orig_id] = ambig_res_id
        else:
            ambig_eff_map[orig_id] = ambig_raw_id

    # Also resolve any restored tail nodes (l8589952425..l8589952481) that reference an ambiguous ID
    tail_eff_map: dict[int, int] = {}
    for t_idx, tail_id in enumerate(tail_ids, start=1):
        t_toks, t_comment, _ = replay_run1_nodes[tail_id]
        t_rewritten = [
            f"l{r1_eff_map[int(t[1:])]}"
            if (t.startswith("l") and t[1:].isdigit() and int(t[1:]) in ambiguous_set)
            else (
                f"l{tail_eff_map[int(t[1:])]}"
                if (t.startswith("l") and t[1:].isdigit() and int(t[1:]) in tail_eff_map)
                else t
            )
            for t in t_toks
        ]
        if t_rewritten != t_toks:
            t_res_id = next_id
            inbox_t = f"step2b_9000_tail_{t_idx:04d}_{tail_id}_1_resolved.dcsh"
            item_t = append_simulated_item(
                t_res_id,
                inbox_t,
                t_rewritten,
                f"Run 1 branch-resolved tail node for l{tail_id}: {t_comment}",
                "csv",
            )
            step2b_items.append(item_t)
            step2b_pinned_inputs[inbox_t] = item_t["inbox_sha256"]
            next_id += 1

            sup_t_id = next_id
            inbox_sup_t = f"step2b_9000_tail_{t_idx:04d}_{tail_id}_2_supersede.dcsh"
            item_sup_t = append_simulated_item(
                sup_t_id,
                inbox_sup_t,
                ["591", f"l{t_res_id}", f"l{tail_id}"],
                f"Supersedes (l{t_res_id}, l{tail_id})",
                "csv",
            )
            step2b_items.append(item_sup_t)
            step2b_pinned_inputs[inbox_sup_t] = item_sup_t["inbox_sha256"]
            next_id += 1
            tail_eff_map[tail_id] = t_res_id
        else:
            tail_eff_map[tail_id] = tail_id

    assert [it["inbox_name"] for it in step2b_items] == sorted(
        it["inbox_name"] for it in step2b_items
    ), "Step 2b inbox filenames must be in strict lexicographical order"

    step2b_final_last_id = next_id - 1
    step2b_final_sha256 = compute_chunks_map_sha256(sim_chunks)
    step2b_manifest = {
        "manifest_id": "step2b",
        "kind": "inbox_allocation",
        "baseline_last_id": 8589954889,
        "baseline_node_count": 20298,
        "baseline_ledger_sha256": step2a_final_sha256,
        "pinned_input_sha256s": step2b_pinned_inputs,
        "planned_items": step2b_items,
        "final_last_id": step2b_final_last_id,
        "final_node_count": len(sim_nodes_order),
        "final_ledger_sha256": step2b_final_sha256,
    }
    (AUDIT_DIR / "step2b_manifest.json").write_text(
        json.dumps(step2b_manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    summary = {
        "baseline_commit": BASELINE_COMMIT,
        "run2_commit": RUN2_COMMIT,
        "current_baseline_last_id": 8589952424,
        "current_baseline_node_count": 17833,
        "current_baseline_ledger_sha256": baseline_ledger_sha256,
        "unambiguous_count": len(unambiguous_ids),
        "ambiguous_count": len(ambiguous_ids),
        "erased_tail_count": len(tail_ids),
        "step2a_appended_count": len(step2a_items),
        "step2a_final_last_id": 8589954889,
        "step2a_final_node_count": 20298,
        "step2a_final_ledger_sha256": step2a_final_sha256,
        "step2b_appended_count": len(step2b_items),
        "step2b_final_last_id": step2b_final_last_id,
        "step2b_final_node_count": len(sim_nodes_order),
        "step2b_final_ledger_sha256": step2b_final_sha256,
        "artifact_sha256s": {
            fname: sha256_file(AUDIT_DIR / fname)
            for fname in sorted(os.listdir(AUDIT_DIR))
            if (AUDIT_DIR / fname).is_file() and fname != "step0_recovery_summary.json"
        },
    }
    (AUDIT_DIR / "step0_recovery_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(summary, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
