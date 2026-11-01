#!/usr/bin/env bash
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

set -euo pipefail
cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/../.." || exit 1
REPO_ROOT="$(pwd)"
STEP3_MANIFEST="${REPO_ROOT}/src/formats/dcdata/data-src/recovery-audit/step3_manifest.json"

if [[ ! -f "${STEP3_MANIFEST}" ]]; then
  echo "1. Planning and verifying Step 3 CSV reconciliation manifest..."
  cargo run -p ctb-formats-dcdata --bin validate-data-tables -- \
    --plan-reconciliation-manifest "${STEP3_MANIFEST}"
else
  echo "1. Using existing Step 3 CSV reconciliation manifest ${STEP3_MANIFEST}..."
fi

echo "2. Executing Step 3 CSV reconciliation under manifest enforcement..."
cargo run -p ctb-formats-dcdata --bin validate-data-tables -- \
  --write --require-manifest "${STEP3_MANIFEST}"

echo "3. Verifying idempotent convergence (second --write must append 0 nodes)..."
SECOND_OUT="$(cargo run -p ctb-formats-dcdata --bin validate-data-tables -- --write 2>&1)"
echo "${SECOND_OUT}"
if ! grep -Fq "(0 appended," <<<"${SECOND_OUT}"; then
  echo "ERROR: Expected second --write pass to append 0 nodes!" >&2
  exit 1
fi

echo "Step 3 CSV reconciliation complete and idempotent."
