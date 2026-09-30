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
Automates git mv of all data/fixtures to fixtures,
relocates DROID tests, scans and reports license/provenance files,
and cleans up empty data dirs.
"""
import glob
import os
import shutil
import subprocess
import sys

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
os.chdir(REPO_ROOT)

MOVES = [
    ("src/build_support/data/fixtures", "src/build_support/fixtures"),
    ("src/formats/alias/data/fixtures", "src/formats/alias/fixtures"),
    ("src/formats/alias/data/docs", "src/formats/alias/docs"),
    ("src/formats/apple_single_double/data/fixtures", "src/formats/apple_single_double/fixtures"),
    ("src/formats/archive/data/fixtures", "src/formats/archive/fixtures"),
    ("src/formats/compression/data/fixtures", "src/formats/compression/fixtures"),
    ("src/formats/compression/data/docs", "src/formats/compression/docs"),
    ("src/formats/ctb_asset_bundle/data/fixtures", "src/formats/ctb_asset_bundle/fixtures"),
    ("src/formats/dct_el/data/fixtures", "src/formats/dct_el/fixtures"),
    ("src/formats/docker/data/fixtures", "src/formats/docker/fixtures"),
    ("src/formats/encoding/data/fixtures", "src/formats/encoding/fixtures"),
    ("src/formats/javascript/data/fixtures", "src/formats/javascript/fixtures"),
    ("src/formats/kaitai/data/fixtures", "src/formats/kaitai/fixtures"),
    ("src/formats/lnk/data/fixtures", "src/formats/lnk/fixtures"),
    ("src/formats/pan/data/fixtures", "src/formats/pan/fixtures"),
    ("src/formats/pdf/data/fixtures", "src/formats/pdf/fixtures"),
    ("src/formats/stagel/data/fixtures", "src/formats/stagel/fixtures"),
    ("src/formats/syndication/data/fixtures", "src/formats/syndication/fixtures"),
    ("src/formats/troff/data/fixtures", "src/formats/troff/fixtures"),
    ("src/formats/wfscan/data/fixtures", "src/formats/wfscan/fixtures"),
    ("src/storage/data/fixtures", "src/storage/fixtures"),
    ("src/utilities/https/data/fixtures", "src/utilities/https/fixtures"),
    ("src/formats/dcdata/data/droid/tests", "src/formats/dcdata/fixtures/droid"),
]

print("=== Scanning Enclosing data/ Directories for Licenses / Readmes ===")
for src, dst in MOVES:
    if os.path.exists(src):
        enclosing_data = os.path.dirname(src)
        if os.path.basename(enclosing_data) != "data" and "data" in enclosing_data.split(os.sep):
            parts = enclosing_data.split(os.sep)
            data_idx = parts.index("data")
            enclosing_data = os.sep.join(parts[:data_idx + 1])

        candidates = [
            "COPYING*", "LICENSE*", "README*", "readme*", "source*",
            "*.LICENSE", "*.txt", "*.md", "*.url"
        ]
        found_docs = []
        for pat in ["COPYING*", "LICENSE*", "README*", "readme*", "source*"]:
            for match in glob.glob(os.path.join(enclosing_data, pat)):
                if os.path.isfile(match):
                    found_docs.append(match)

        # Also check immediate parent if different (e.g. data/droid for tests)
        immediate_parent = os.path.dirname(src)
        if immediate_parent != enclosing_data and os.path.exists(immediate_parent):
            for pat in ["COPYING*", "LICENSE*", "README*", "readme*", "source*"]:
                for match in glob.glob(os.path.join(immediate_parent, pat)):
                    if os.path.isfile(match) and match not in found_docs:
                        found_docs.append(match)

        if found_docs:
            print(f"\n[NOTICE] Enclosing data dir '{enclosing_data}' (for '{src}' -> '{dst}'):")
            for doc in sorted(found_docs):
                print(f"  - {doc}")

print("\n=== Executing Moves via git mv ===")
for src, dst in MOVES:
    if os.path.exists(src):
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        print(f"git mv {src} -> {dst}")
        subprocess.run(["git", "mv", src, dst], check=True)
    else:
        print(f"[SKIP] Source does not exist: {src}")

# Special copy for DROID license & provenance into fixtures/droid
droid_license = "src/formats/dcdata/data/droid/LICENSE"
droid_source = "src/formats/dcdata/data/droid/source.txt"
if os.path.exists(droid_license) and os.path.exists("src/formats/dcdata/fixtures/droid"):
    shutil.copy2(droid_license, "src/formats/dcdata/fixtures/droid/LICENSE")
    subprocess.run(["git", "add", "src/formats/dcdata/fixtures/droid/LICENSE"], check=True)
    print("Copied DROID LICENSE into fixtures/droid/")
if os.path.exists(droid_source) and os.path.exists("src/formats/dcdata/fixtures/droid"):
    shutil.copy2(droid_source, "src/formats/dcdata/fixtures/droid/source.txt")
    subprocess.run(["git", "add", "src/formats/dcdata/fixtures/droid/source.txt"], check=True)
    print("Copied DROID source.txt into fixtures/droid/")

print("\n=== Cleaning Up Empty data Directories ===")
# First remove any .keep files in data directories that are otherwise empty
for root, dirs, files in os.walk("src", topdown=False):
    if os.path.basename(root) == "data":
        if files == [".keep"] and not dirs:
            keep_file = os.path.join(root, ".keep")
            print(f"Removing unused {keep_file}")
            subprocess.run(["git", "rm", "-f", keep_file], check=True)

# Remove empty data directories
for root, dirs, files in os.walk("src", topdown=False):
    if os.path.basename(root) == "data" and not os.listdir(root):
        print(f"Removing empty directory: {root}")
        os.rmdir(root)

print("\n=== Filesystem Migration Complete ===")
