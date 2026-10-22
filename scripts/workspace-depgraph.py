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
Generates a dependency diagram (SVG, Mermaid, or Text) of the crates in the
workspace, placing crates with no workspace dependencies at the top.

Runs using `ctb python` or standard `python3` and depends strictly on `cargo`
and the Python standard library. No third-party packages or external layout
tools are required.
"""

from __future__ import annotations

import html
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Dict, List, Set, Tuple


def find_repo_root() -> Path:
    """Locate the root directory of the repository."""
    start = Path(__file__).resolve().parent
    for parent in [start, *start.parents]:
        if (parent / "Cargo.toml").is_file() and (parent / "src").is_dir():
            return parent
    return start.parent


def get_workspace_metadata(manifest_path: str | None = None) -> dict:
    """Run `cargo metadata` to extract workspace members and dependencies."""
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    if manifest_path:
        cmd.extend(["--manifest-path", manifest_path])
    try:
        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        sys.stderr.write(f"Error running cargo metadata: {e.stderr}\n")
        sys.exit(1)
    except FileNotFoundError:
        sys.stderr.write("Error: 'cargo' executable not found in PATH.\n")
        sys.exit(1)

    return json.loads(proc.stdout)


def build_dependency_graph(
    metadata: dict,
    include_dev: bool = False,
    include_build: bool = True,
) -> Tuple[Set[str], Dict[str, Set[str]]]:
    """
    Build directed adjacency graph of workspace crates.
    Returns:
        packages: set of workspace package names
        adj: map of crate -> set of workspace crates it depends on
    """
    pkg_names = {p["name"] for p in metadata["packages"]}
    adj: Dict[str, Set[str]] = {name: set() for name in pkg_names}

    for p in metadata["packages"]:
        u = p["name"]
        for dep in p["dependencies"]:
            dep_name = dep["name"]
            if dep_name not in pkg_names:
                continue
            kind = dep.get("kind")
            if kind == "dev" and not include_dev:
                continue
            if kind == "build" and not include_build:
                continue
            adj[u].add(dep_name)

    return pkg_names, adj


def compute_transitive_reduction(
    pkg_names: Set[str], adj: Dict[str, Set[str]]
) -> Dict[str, Set[str]]:
    """
    Removes transitive edges: if A -> B and B -> C, then the direct edge
    A -> C is redundant in a layered hierarchy diagram.
    """
    reduced: Dict[str, Set[str]] = {}
    for u in pkg_names:
        indirect = set()
        for direct_dep in adj[u]:
            queue = list(adj[direct_dep])
            visited = set(queue)
            while queue:
                curr = queue.pop(0)
                indirect.add(curr)
                for nxt in adj[curr]:
                    if nxt not in visited:
                        visited.add(nxt)
                        queue.append(nxt)
        reduced[u] = adj[u] - indirect
    return reduced


def compute_layers(
    pkg_names: Set[str], adj: Dict[str, Set[str]]
) -> Dict[str, int]:
    """
    Computes topological layer for each crate such that:
    - Crates with 0 workspace dependencies are at layer 0 (the top).
    - Dependent crates are at layer = 1 + max(layer of dependencies).
    """
    layers: Dict[str, int] = {}

    def get_layer(u: str, path: Set[str]) -> int:
        if u in layers:
            return layers[u]
        if u in path:
            return 0  # Cycle guard
        new_path = path | {u}
        deps = adj.get(u, set())
        if not deps:
            layers[u] = 0
            return 0
        l = 1 + max(get_layer(d, new_path) for d in deps)
        layers[u] = l
        return l

    for u in pkg_names:
        get_layer(u, set())

    return layers


def get_color_scheme(name: str) -> Tuple[str, str, str]:
    """Returns (fill_color, border_color, text_color) based on crate subsystem."""
    if "macro" in name:
        return "#f5f3ff", "#8b5cf6", "#4c1d95"  # Purple / Macro
    if name.startswith("ctb-formats"):
        return "#ecfdf5", "#10b981", "#064e3b"  # Emerald / Formats
    if name.startswith("ctb-io"):
        return "#fffbeb", "#f59e0b", "#78350f"  # Amber / I/O
    if name.startswith("ctb-storage"):
        return "#eff6ff", "#3b82f6", "#1e3a8a"  # Blue / Storage
    if name.startswith("ctb-workspace"):
        return "#fdf2f8", "#ec4899", "#831843"  # Pink / Workspace Core
    if name in {"ctoolbox", "ctb-installer", "ctb-cli"}:
        return "#fef2f2", "#ef4444", "#7f1d1d"  # Red / Applications
    if name.startswith("ctb-utilities") or name.startswith("ctb-build"):
        return "#f8fafc", "#64748b", "#0f172a"  # Slate / Utilities & Build
    return "#f0fdfa", "#14b8a6", "#134e4a"      # Teal / Standalone crates


def generate_mermaid(
    pkg_names: Set[str],
    adj: Dict[str, Set[str]],
    layers: Dict[str, int],
    flow: str = "down",
) -> str:
    """Generates Mermaid markdown diagram."""
    lines = [
        "```mermaid",
        "flowchart TD" if flow == "down" else "flowchart BT",
    ]

    def m_id(name: str) -> str:
        return name.replace("-", "_").replace(".", "_")

    for u in sorted(pkg_names):
        lines.append(f'    {m_id(u)}["{u}"]')

    for u in sorted(pkg_names):
        for v in sorted(adj[u]):
            if flow == "down":
                lines.append(f"    {m_id(v)} --> {m_id(u)}")
            else:
                lines.append(f"    {m_id(u)} --> {m_id(v)}")

    lines.append("```")
    return "\n".join(lines)


def generate_builtin_svg(
    pkg_names: Set[str],
    adj: Dict[str, Set[str]],
    layers: Dict[str, int],
    flow: str = "down",
) -> str:
    """
    Renders a standalone SVG without requiring any external tools.
    Layers crates top-to-bottom (Layer 0 with 0 workspace dependencies at the top).
    """
    # Inverse map: who depends on crate u?
    dependents: Dict[str, Set[str]] = {u: set() for u in pkg_names}
    for u in pkg_names:
        for v in adj[u]:
            dependents[v].add(u)

    max_layer = max(layers.values()) if layers else 0
    layer_buckets: List[List[str]] = [[] for _ in range(max_layer + 1)]
    for u, l in layers.items():
        layer_buckets[l].append(u)

    # Order nodes within each layer using horizontal barycenter to reduce crossings
    node_x: Dict[str, float] = {}
    for l_idx, bucket in enumerate(layer_buckets):
        if l_idx == 0:
            bucket.sort()
        else:
            def get_barycenter(name: str) -> float:
                deps = adj.get(name, set())
                pos = [node_x[d] for d in deps if d in node_x]
                return sum(pos) / len(pos) if pos else 0.0

            bucket.sort(key=get_barycenter)

        for col, name in enumerate(bucket):
            node_x[name] = float(col)

    # Layout geometry
    node_w = 175
    node_h = 32
    col_spacing = 24
    layer_spacing = 80
    pad_top = 110  # Space for title header & legend
    pad_bottom = 60
    pad_x = 220    # Left margin for layer labels

    max_cols = max(len(b) for b in layer_buckets) if layer_buckets else 1
    content_w = max_cols * (node_w + col_spacing) - col_spacing
    canvas_w = int(pad_x + content_w + 60)
    canvas_h = int(pad_top + (max_layer + 1) * (node_h + layer_spacing) + pad_bottom)

    # Center nodes horizontally within each layer
    coords: Dict[str, Tuple[float, float]] = {}
    for l_idx, bucket in enumerate(layer_buckets):
        y = pad_top + l_idx * (node_h + layer_spacing)
        total_layer_w = len(bucket) * (node_w + col_spacing) - col_spacing
        start_x = pad_x + (content_w - total_layer_w) / 2.0
        for col, name in enumerate(bucket):
            x = start_x + col * (node_w + col_spacing)
            coords[name] = (x, y)

    svg: List[str] = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {canvas_w} {canvas_h}" '
        f'width="{canvas_w}" height="{canvas_h}">',
        "<defs>",
        '  <marker id="arrow-down" viewBox="0 0 10 10" refX="8" refY="5" '
        'markerWidth="6" markerHeight="6" orient="auto">',
        '    <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="#64748b" />',
        "  </marker>",
        '  <marker id="arrow-up" viewBox="0 0 10 10" refX="8" refY="5" '
        'markerWidth="6" markerHeight="6" orient="auto">',
        '    <path d="M 0 1.5 L 8 5 L 0 8.5 z" fill="#64748b" />',
        "  </marker>",
        '  <style type="text/css">',
        '    text { font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }',
        '    .header-title { font-size: 20px; font-weight: 700; fill: #0f172a; }',
        '    .header-sub { font-size: 12px; font-weight: 400; fill: #475569; }',
        '    .legend-text { font-size: 11px; font-weight: 500; fill: #334155; }',
        '    .layer-label { font-size: 11px; fill: #64748b; font-weight: 600; }',
        '    .layer-divider { stroke: #e2e8f0; stroke-dasharray: 4, 4; stroke-width: 1; }',
        '    .node-rect { transition: filter 0.15s ease, stroke-width 0.15s ease; cursor: pointer; }',
        '    .node:hover .node-rect { filter: drop-shadow(0 4px 8px rgba(0,0,0,0.18)); stroke-width: 2.2px; }',
        '    .node-text { font-size: 11px; font-weight: 500; pointer-events: none; }',
        '    .edge-path { stroke: #cbd5e1; stroke-width: 1.25; fill: none; transition: stroke 0.15s; }',
        '    .edge-path:hover { stroke: #2563eb; stroke-width: 2.5; }',
        '  </style>',
        "</defs>",
        f'<rect width="{canvas_w}" height="{canvas_h}" fill="#f8fafc" />',
    ]

    # Title Header
    svg.append('<!-- Header Title & Subtitle -->')
    svg.append(
        '<text x="40" y="42" class="header-title">Collective Toolbox – Workspace Crate Dependency Graph</text>'
    )
    flow_desc = (
        "Base Crates at Top (0 Workspace Dependencies) → Arrows point down toward consumers"
        if flow == "down"
        else "Base Crates at Top (0 Workspace Dependencies) → Arrows point up toward dependencies"
    )
    svg.append(
        f'<text x="40" y="65" class="header-sub">{html.escape(flow_desc)} ({len(pkg_names)} workspace crates)</text>'
    )

    # Subsystem Color Legend
    legend_items = [
        ("Macro", "#f5f3ff", "#8b5cf6"),
        ("Formats", "#ecfdf5", "#10b981"),
        ("I/O", "#fffbeb", "#f59e0b"),
        ("Storage", "#eff6ff", "#3b82f6"),
        ("Workspace", "#fdf2f8", "#ec4899"),
        ("Apps & CLI", "#fef2f2", "#ef4444"),
        ("Utilities/Build", "#f8fafc", "#64748b"),
        ("Standalone", "#f0fdfa", "#14b8a6"),
    ]
    legend_start_x = pad_x + 10
    legend_y = 86
    svg.append('<!-- Legend -->')
    svg.append('<g id="legend">')
    curr_lx = legend_start_x
    for label, bg, border in legend_items:
        svg.append(
            f'  <rect x="{curr_lx}" y="{legend_y - 10}" width="14" height="14" rx="3" ry="3" '
            f'fill="{bg}" stroke="{border}" stroke-width="1.2" />'
        )
        svg.append(
            f'  <text x="{curr_lx + 19}" y="{legend_y + 1}" class="legend-text">{label}</text>'
        )
        curr_lx += len(label) * 8 + 48
    svg.append('</g>')

    # Layer dividers and labels
    svg.append('<!-- Layer Guides -->')
    for l_idx in range(max_layer + 1):
        y = pad_top + l_idx * (node_h + layer_spacing) + node_h / 2.0
        label = "Layer 0 (0 Workspace Deps)" if l_idx == 0 else f"Layer {l_idx}"
        svg.append(
            f'<text x="35" y="{y:.1f}" class="layer-label" text-anchor="start" dominant-baseline="middle">{label}</text>'
        )
        div_y = pad_top + l_idx * (node_h + layer_spacing) + node_h + layer_spacing / 2.0
        if l_idx < max_layer:
            svg.append(
                f'<line x1="35" y1="{div_y:.1f}" x2="{canvas_w - 40}" y2="{div_y:.1f}" class="layer-divider" />'
            )

    # Edges
    svg.append('<!-- Edges -->')
    svg.append('<g id="edges">')
    for u in sorted(pkg_names):
        for v in sorted(adj[u]):
            if u not in coords or v not in coords:
                continue
            src_x, src_y = coords[u]
            dst_x, dst_y = coords[v]

            if flow == "down":
                # Dependency v (higher/top) -> Dependent u (lower/bottom)
                start_x, start_y = dst_x + node_w / 2.0, dst_y + node_h
                end_x, end_y = src_x + node_w / 2.0, src_y
                c1_x, c1_y = start_x, start_y + (end_y - start_y) * 0.5
                c2_x, c2_y = end_x, end_y - (end_y - start_y) * 0.5
                marker = 'marker-end="url(#arrow-down)"'
            else:
                # Dependent u (lower/bottom) -> Dependency v (higher/top)
                start_x, start_y = src_x + node_w / 2.0, src_y
                end_x, end_y = dst_x + node_w / 2.0, dst_y + node_h
                c1_x, c1_y = start_x, start_y - (start_y - end_y) * 0.5
                c2_x, c2_y = end_x, end_y + (start_y - end_y) * 0.5
                marker = 'marker-end="url(#arrow-up)"'

            path_d = f"M {start_x:.1f} {start_y:.1f} C {c1_x:.1f} {c1_y:.1f}, {c2_x:.1f} {c2_y:.1f}, {end_x:.1f} {end_y:.1f}"
            svg.append(f'  <path class="edge-path" d="{path_d}" {marker} />')
    svg.append('</g>')

    # Nodes
    svg.append('<!-- Nodes -->')
    svg.append('<g id="nodes">')
    for u in sorted(pkg_names):
        if u not in coords:
            continue
        x, y = coords[u]
        bg, border, text_color = get_color_scheme(u)
        cx = x + node_w / 2.0
        cy = y + node_h / 2.0

        my_deps = sorted(adj.get(u, set()))
        my_users = sorted(dependents.get(u, set()))
        deps_str = ", ".join(my_deps) if my_deps else "(none)"
        users_str = ", ".join(my_users) if my_users else "(none)"
        tooltip = html.escape(
            f"Crate: {u}\nLayer: {layers.get(u, 0)}\n\nDepends on ({len(my_deps)}):\n  {deps_str}\n\nDepended on by ({len(my_users)}):\n  {users_str}"
        )

        svg.append('  <g class="node">')
        svg.append(f'    <title>{tooltip}</title>')
        svg.append(
            f'    <rect class="node-rect" x="{x:.1f}" y="{y:.1f}" width="{node_w}" height="{node_h}" '
            f'rx="5" ry="5" fill="{bg}" stroke="{border}" stroke-width="1.3" />'
        )
        svg.append(
            f'    <text class="node-text" x="{cx:.1f}" y="{cy:.1f}" fill="{text_color}" '
            f'text-anchor="middle" dominant-baseline="middle">{html.escape(u)}</text>'
        )
        svg.append('  </g>')
    svg.append('</g>')

    svg.append('</svg>')
    return '\n'.join(svg)


def print_help() -> None:
    print(
        """Usage: workspace-depgraph.py [OPTIONS]

Generates a workspace crate dependency diagram with zero-dependency crates at the top.

Options:
  -h, --help            Show this help message and exit
  -o, --output FILE     Output file path (default: 'workspace-dependencies.svg' or stdout for text)
  -f, --format FORMAT   Output format: svg (default), mermaid, or text
  --flow DIRECTION      Flow direction: 'down' (default, foundation at top pointing to consumers)
                        or 'up' (arrows point up toward dependencies)
  --no-reduce           Disable transitive reduction (keeps all direct edges)
  --include-dev         Include dev-dependencies (may introduce cycles)
  --manifest-path PATH  Path to Cargo.toml (default: locates repo root)
"""
    )


class Options:
    def __init__(self) -> None:
        self.output: str | None = None
        self.format: str = "svg"
        self.flow: str = "down"
        self.no_reduce: bool = False
        self.include_dev: bool = False
        self.manifest_path: str | None = None


def parse_arguments(args: List[str]) -> Options:
    opts = Options()
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in ("-h", "--help"):
            print_help()
            sys.exit(0)
        elif arg in ("-o", "--output"):
            i += 1
            if i >= len(args):
                sys.stderr.write("Error: -o/--output requires a path argument\n")
                sys.exit(1)
            opts.output = args[i]
        elif arg.startswith("--output="):
            opts.output = arg.split("=", 1)[1]
        elif arg in ("-f", "--format"):
            i += 1
            if i >= len(args):
                sys.stderr.write("Error: -f/--format requires an argument (svg, mermaid, text)\n")
                sys.exit(1)
            opts.format = args[i].lower()
        elif arg.startswith("--format="):
            opts.format = arg.split("=", 1)[1].lower()
        elif arg == "--flow":
            i += 1
            if i >= len(args):
                sys.stderr.write("Error: --flow requires an argument (down or up)\n")
                sys.exit(1)
            opts.flow = args[i].lower()
        elif arg.startswith("--flow="):
            opts.flow = arg.split("=", 1)[1].lower()
        elif arg == "--no-reduce":
            opts.no_reduce = True
        elif arg == "--include-dev":
            opts.include_dev = True
        elif arg == "--manifest-path":
            i += 1
            if i >= len(args):
                sys.stderr.write("Error: --manifest-path requires a path argument\n")
                sys.exit(1)
            opts.manifest_path = args[i]
        elif arg.startswith("--manifest-path="):
            opts.manifest_path = arg.split("=", 1)[1]
        elif arg.startswith("-"):
            sys.stderr.write(f"Unknown option: {arg}\n")
            print_help()
            sys.exit(1)
        else:
            if opts.output is None:
                opts.output = arg
        i += 1
    return opts


def main() -> None:
    repo_root = find_repo_root()
    os.chdir(repo_root)

    opts = parse_arguments(sys.argv[1:])

    metadata = get_workspace_metadata(opts.manifest_path)
    pkg_names, raw_adj = build_dependency_graph(
        metadata, include_dev=opts.include_dev
    )

    if opts.no_reduce:
        adj = raw_adj
    else:
        adj = compute_transitive_reduction(pkg_names, raw_adj)

    layers = compute_layers(pkg_names, adj)

    target_format = opts.format
    if opts.output:
        ext = os.path.splitext(opts.output)[1].lower()
        if ext in {".mmd", ".mermaid"}:
            target_format = "mermaid"
        elif ext == ".svg":
            target_format = "svg"
        elif ext in {".txt", ".text"}:
            target_format = "text"

    output_content = ""
    if target_format == "text":
        max_layer = max(layers.values()) if layers else 0
        layer_groups: Dict[int, List[str]] = {i: [] for i in range(max_layer + 1)}
        for u, l in layers.items():
            layer_groups[l].append(u)

        lines = [
            f"Collective Toolbox Workspace Dependency Hierarchy ({len(pkg_names)} crates):",
            "=" * 70,
        ]
        for l in range(max_layer + 1):
            crates = sorted(layer_groups[l])
            hdr = f"Layer {l} (0 Workspace Dependencies)" if l == 0 else f"Layer {l}"
            lines.append(f"\n{hdr} [{len(crates)} crates]:")
            for c in crates:
                deps = sorted(adj.get(c, set()))
                dep_str = f" -> depends on: {', '.join(deps)}" if deps else " (leaf/foundation)"
                lines.append(f"  • {c}{dep_str}")
        output_content = "\n".join(lines) + "\n"

    elif target_format == "mermaid":
        output_content = generate_mermaid(pkg_names, adj, layers, flow=opts.flow)

    else:  # SVG
        output_content = generate_builtin_svg(
            pkg_names, adj, layers, flow=opts.flow
        )

    if opts.output:
        with open(opts.output, "w", encoding="utf-8") as f:
            f.write(output_content)
        print(f"Generated {target_format.upper()} diagram at: {opts.output}")
    else:
        if target_format == "svg":
            default_out = "workspace-dependencies.svg"
            with open(default_out, "w", encoding="utf-8") as f:
                f.write(output_content)
            print(f"Generated SVG diagram at: {default_out}")
        else:
            sys.stdout.write(output_content)


if __name__ == "__main__":
    main()
