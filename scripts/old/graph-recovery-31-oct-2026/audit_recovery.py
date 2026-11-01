#!/usr/bin/env python3
"""Read-only verification of committed recovery evidence and ledger allocations."""

from __future__ import annotations

import io
import json
import subprocess
import sys
import tarfile
from collections import Counter
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
    parse_statement_tokens,
    sha256_bytes,
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def read_json(path: Path) -> dict:
    return json.loads(path.read_bytes())


def historical_nodes(path: Path) -> dict:
    result = {}
    for line in path.read_text().splitlines():
        node_id, tokens, comment = parse_ctbnodes_line(line)
        require(node_id not in result, f"Repeated historical ID: l{node_id}")
        result[node_id] = (tokens, comment, line)
    return result


def effective_state(nodes: dict) -> tuple[set, dict]:
    inactive = set()
    replacements = {}
    for node_id in sorted(nodes, reverse=True):
        if node_id in inactive:
            continue
        statement = parse_statement_tokens(nodes[node_id][0])
        if statement is not None:
            verb, subject, target = statement
            if verb == "591" or (verb == "592" and subject == "615"):
                require(target.startswith("l"), f"Invalid revision target: l{node_id}")
                target_id = int(target[1:])
                inactive.add(target_id)
                if verb == "591":
                    require(target_id not in replacements, f"Competing effective replacements: l{target_id}")
                    replacements[target_id] = int(subject[1:])
    return inactive, replacements


def active_facts(nodes: dict, aliases: dict) -> Counter:
    inactive, _ = effective_state(nodes)
    statements = {}
    for node_id, (tokens, _, _) in nodes.items():
        if node_id not in inactive:
            statement = parse_statement_tokens(tokens)
            if statement is not None and statement[0] not in ("591", "592"):
                statements[node_id] = statement
    valued_claims = {
        int(subject[1:])
        for verb, subject, _ in statements.values()
        if verb == "610" and subject.startswith("l") and int(subject[1:]) not in inactive
    }
    facts = Counter()
    for node_id, statement in statements.items():
        verb, subject, target = statement
        if verb == "610" and (not subject.startswith("l") or int(subject[1:]) in inactive):
            continue
        if verb == "584" and target != "570" and node_id not in valued_claims:
            continue
        if verb == "584" and target == "570" and subject.startswith("l") and int(subject[1:]) not in valued_claims:
            continue
        facts[tuple(aliases.get(token, token) for token in statement)] += 1
    return facts


def verify_branches(nodes: dict, manifests: list, baseline_nodes: dict) -> None:
    collision = read_json(AUDIT_DIR / "collision_manifest.json")
    run1 = historical_nodes(AUDIT_DIR / "reconstructed_run1_nodes_8589951799_8589952481.ctbnodes")
    run2 = historical_nodes(AUDIT_DIR / "observed_run2_nodes_8589951799_8589952424.ctbnodes")
    ambiguous = {node_id for node_id in run2 if run1[node_id][0] != run2[node_id][0]}
    while True:
        expanded = ambiguous | {
            node_id for node_id in run2
            if any(token.startswith("l") and int(token[1:]) in ambiguous for token in run1[node_id][0] + run2[node_id][0])
        }
        if expanded == ambiguous:
            break
        ambiguous = expanded
    require(sorted(ambiguous) == collision["ambiguous_ids"], "Collision dependency closure differs from evidence")
    require(sorted(set(run2) - ambiguous) == collision["unambiguous_ids"], "Unambiguous classification differs from evidence")
    for node_id in run2:
        require(run2[node_id] == baseline_nodes[node_id], f"Observed Run 2 differs from incident commit: l{node_id}")
    tail_ids = collision["erased_tail_ids"]
    for node_id in tail_ids:
        require(nodes[node_id] == run1[node_id], f"Restored tail differs from Run 1 evidence: l{node_id}")
    items2a = manifests[0]["planned_items"][len(tail_ids):]
    require(len(items2a) == 4 * len(ambiguous), "Incorrect ambiguity quadruplet count")
    items2b = {item["inbox_name"]: item for item in manifests[1]["planned_items"]}
    branches = [{}, {}]
    _, replacements = effective_state(nodes)
    for index, original in enumerate(sorted(ambiguous), start=1):
        group = items2a[(index - 1) * 4:index * 4]
        for branch, historical in enumerate((run1, run2)):
            raw_id = group[branch]["node_id"]
            require(nodes[raw_id][0] == historical[original][0], f"Raw branch body mismatch: l{original}, branch {branch + 1}")
            rewritten = [f"l{branches[branch][int(token[1:])]}" if token.startswith("l") and int(token[1:]) in ambiguous else token for token in historical[original][0]]
            name = f"step2b_{index:04d}_{original}_{1 if branch == 0 else 3}_r{branch + 1}_resolved.dcsh"
            resolved_id = items2b[name]["node_id"] if name in items2b else raw_id
            require(nodes[resolved_id][0] == rewritten, f"Resolved branch body mismatch: l{original}, branch {branch + 1}")
            branches[branch][original] = resolved_id
        latest = original
        visited = set()
        while latest in replacements:
            require(latest not in visited, f"Replacement cycle: l{original}")
            visited.add(latest)
            latest = replacements[latest]
        expected = ["348", "310", "275", f"l{branches[0][original]}", "312", f"l{branches[1][original]}", "311"]
        require(nodes[latest][0] == expected, f"Terminal ambiguity has incorrect candidates: l{original}")
    aliases = {f"l{resolved}": f"l{original}" for original, resolved in branches[1].items()}
    before = active_facts(baseline_nodes, {})
    after = active_facts(nodes, aliases)
    require(before == after, f"Active Run 2 facts differ after recovery: removed={list((before - after).items())[:5]}, added={list((after - before).items())[:5]}")
    print(f"PASS branch bodies, dependency closure, terminal ambiguity records, and all {sum(before.values())} active Run 2 statement occurrences")


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
    baseline_nodes = {}
    with tarfile.open(fileobj=io.BytesIO(archive)) as tree:
        for member in tree.getmembers():
            if member.isfile() and member.name.endswith(".ctbnodes"):
                source = tree.extractfile(member)
                require(source is not None, f"Missing archive member {member.name}")
                historical = source.read()
                current = (REPO_ROOT / member.name).read_bytes()
                require(current.startswith(historical), f"Incident ledger prefix changed: {member.name}")
                baseline_count += len(historical.splitlines())
                for line in historical.decode().splitlines():
                    node_id, tokens, comment = parse_ctbnodes_line(line)
                    baseline_nodes[node_id] = (tokens, comment, line)
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
    for record in records:
        canonical = dict(record)
        canonical["record_sha256"] = ""
        serialized = json.dumps(canonical, ensure_ascii=False, separators=(",", ":")).encode()
        require(sha256_bytes(serialized) == record["record_sha256"], f"Journal checksum mismatch: {record['batch_id']}, {record['state']}")
    journal_items = {}
    previous = None
    for pending, committed in zip(records[::2], records[1::2]):
        require(pending["state"] == "BATCH_PENDING" and committed["state"] == "BATCH_COMMITTED", "Invalid journal state sequence")
        for key in ("batch_id", "kind", "baseline_last_id", "final_last_id", "baseline_ledger_sha256", "final_ledger_sha256"):
            require(pending[key] == committed[key], f"Journal pair disagrees: {key}")
        if previous is not None:
            require(pending["baseline_last_id"] == previous["final_last_id"] and pending["baseline_ledger_sha256"] == previous["final_ledger_sha256"], "Journal transaction chain has a gap")
        if pending["kind"] != "bootstrap_checkpoint":
            matching = [manifest for manifest in manifests if manifest["final_last_id"] == pending["final_last_id"]]
            require(len(matching) == 1, "Journal batch has no unique phase manifest")
            phase = ("step2a" if pending["final_last_id"] == manifests[0]["final_last_id"] else "step2b" if pending["final_last_id"] == manifests[1]["final_last_id"] else "step3")
            expected_sha = sha256_bytes((AUDIT_DIR / f"{phase}_manifest.json").read_bytes())
            require(pending.get("manifest_sha256") == committed.get("manifest_sha256") == expected_sha, "Journal manifest digest mismatch")
        for item in pending.get("items", []):
            node_id, _, _ = parse_ctbnodes_line(item["raw_line"])
            require(node_id not in journal_items, f"Repeated journal allocation: l{node_id}")
            journal_items[node_id] = item["raw_line"]
        previous = committed
    require(journal_items == {node_id: item["raw_line"] for node_id, item in planned.items()}, "Tracked journal does not cover exactly the manifested allocations")
    require(len(nodes) == baseline_count + len(planned), "Unmanifested post-incident nodes exist")
    print(f"PASS incident prefix: {baseline_count} unchanged nodes; tracked journal covers all {len(planned)} additions")
    for field, directory in (("active_bundles_sha256", GRAPH_DIR), ("history_bundles_sha256", GRAPH_DIR / "system")):
        actual = {path.name: sha256_bytes(path.read_bytes()) for path in directory.glob("*.ctbnbdl")}
        require(actual == records[-1][field], f"Current bundle inventory differs from tracked journal: {field}")
    print("PASS all journal record checksums, transaction chaining, manifest digests, and current bundle inventories")
    verify_branches(nodes, manifests, baseline_nodes)
    snapshot_dirs = sorted({path.parent for path in (REPO_ROOT / "git").rglob("*.ctbnodes")})
    for directory in snapshot_dirs:
        snapshot, _ = load_ctbnodes_dir(directory)
        for node_id, body in snapshot.items():
            require(nodes.get(node_id) == body, f"Saved high-water snapshot disagrees with ledger: {directory}, l{node_id}")
        print(f"PASS saved high-water snapshot {directory.relative_to(REPO_ROOT)}: {len(snapshot)} matching nodes")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"FAIL: {error}", file=sys.stderr)
        sys.exit(1)