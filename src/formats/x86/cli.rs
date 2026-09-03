// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! CLI execution helpers for x86 instruction sets extraction.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Context;
use std::path::Path;

pub fn instruction_sets(path: &Path) -> Result<ToolResult> {
    let data = std::fs::read(path).with_context(|| {
        format!("Failed to read file: {}", path.display())
    })?;
    let sets = crate::extract_instruction_sets(&data)?;
    let mut output = sets.join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    Ok(ToolResult::immediate_ok(output.into_bytes()))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_x86_instruction_sets_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sample_file = temp_dir.path().join("dummy.bin");
        std::fs::write(&sample_file, b"not a binary").unwrap();

        assert!(super::instruction_sets(&sample_file).is_err());
    }
}