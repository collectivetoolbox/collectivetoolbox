#!/usr/bin/env python3
"""Read-only verification of committed recovery evidence and ledger allocations."""

from __future__ import annotations

import io
import json
import subprocess
import sys
import tarfile
from pathlib import Path

sys.dont_write_bytecode = True

from step0_extract_transcript_data import (
    AUDIT_DIR,
    GRAPH_DIR,
    REPO_ROOT,
    chunk_filename_for_node_id,
    compute_chunks_map_sha256,
    load_ctbnodes_dir,
    parse_ctbnodes_line,
    sha256_bytes,
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def read_json(path: Path) -> dict:
    return json.loads(path.read_bytes())


def main() -> None:
    nodes, _ = load_ctbnodes_dir(GRAPH_DIR)
    ordered_ids = sorted(nodes)
    require(
        ordered_ids == list(range(ordered_ids[0], ordered_ids[-1] + 1)),
        "Ledger IDs are not contiguous",
    )
    archive = subprocess.run(
        ["git", "archive", "a9046402890cfcccc1b50d2c2e76a4889694814b", "assets/data/graph"],
        cwd=REPO_ROOT, check=True, capture_output=True,
    ).stdout
    baseline_count = 0
    with tarfile.open(fileobj=io.BytesIO(archive)) as tree:
        for member in tree.getmembers():
            if member.isfile() and member.name.endswith(".ctbnodes"):
                source = tree.extractfile(member)
                require(source is not None, f"Missing archive member {member.name}")
                historical = source.read()
                current = (REPO_ROOT / member.name).read_bytes()
                require(current.startswith(historical), f"Incident ledger prefix changed: {member.name}")
                baseline_count += len(historical.splitlines())
    summary = read_json(AUDIT_DIR / "step0_recovery_summary.json")
    for name, expected in summary["artifact_sha256s"].items():
        require(sha256_bytes((AUDIT_DIR / name).read_bytes()) == expected, f"Evidence digest changed: {name}")
    manifests = [read_json(AUDIT_DIR / f"{phase}_manifest.json") for phase in ("step2a", "step2b", "step3")]
    allocated = REPO_ROOT / "src/formats/dcdata/data-src/nodes/allocated"
    planned = {}
    for manifest in manifests:
        for boundary in ("baseline", "final"):
            prefix_chunks = {}
            for node_id in ordered_ids:
                if node_id > manifest[f"{boundary}_last_id"]:
                    break
                prefix_chunks.setdefault(chunk_filename_for_node_id(node_id), []).append(nodes[node_id][2])
            require(sum(map(len, prefix_chunks.values())) == manifest[f"{boundary}_node_count"], f"{manifest['manifest_id']} {boundary} count mismatch")
            require(compute_chunks_map_sha256(prefix_chunks) == manifest[f"{boundary}_ledger_sha256"], f"{manifest['manifest_id']} {boundary} digest mismatch")
        items = manifest["planned_items"]
        require([item["node_id"] for item in items] == list(range(manifest["baseline_last_id"] + 1, manifest["final_last_id"] + 1)), f"{manifest['manifest_id']} allocation gap")
        for item in items:
            node_id = item["node_id"]
            require(nodes[node_id][2] == item["raw_line"], f"Allocation differs from manifest: l{node_id}")
            require(node_id not in planned, f"Repeated planned ID: l{node_id}")
            planned[node_id] = item
            if "allocated_name" in item:
                content = (allocated / item["allocated_name"]).read_bytes()
                require(sha256_bytes(content) == item["inbox_sha256"], f"Allocated source digest mismatch: l{node_id}")
                require(content.decode() == item["dcsh_content"], f"Allocated source content mismatch: l{node_id}")
        print(f"PASS {manifest['manifest_id']}: {len(items)} exact allocations and both ledger boundary hashes")
    records = [json.loads(line) for line in (allocated / "ledger-journal.jsonl").read_bytes().splitlines()]
    require(len(records) % 2 == 0, "Journal has an unfinished transaction")
    journal_items = {}
    for pending, committed in zip(records[::2], records[1::2]):
        require(pending["state"] == "BATCH_PENDING" and committed["state"] == "BATCH_COMMITTED", "Invalid journal state sequence")
        for key in ("batch_id", "kind", "baseline_last_id", "final_last_id", "baseline_ledger_sha256", "final_ledger_sha256"):
            require(pending[key] == committed[key], f"Journal pair disagrees: {key}")
        for item in pending.get("items", []):
            node_id, _, _ = parse_ctbnodes_line(item["raw_line"])
            require(node_id not in journal_items, f"Repeated journal allocation: l{node_id}")
            journal_items[node_id] = item["raw_line"]
    require(journal_items == {node_id: item["raw_line"] for node_id, item in planned.items()}, "Tracked journal does not cover exactly the manifested allocations")
    require(len(nodes) == baseline_count + len(planned), "Unmanifested post-incident nodes exist")
    print(f"PASS incident prefix: {baseline_count} unchanged nodes; tracked journal covers all {len(planned)} additions")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)