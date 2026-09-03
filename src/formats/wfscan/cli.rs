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

//! CLI execution helpers for wfscan and wfparser.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::Path;

pub fn wfparser<FRead>(
    file: &Path,
    read_file_or_stdin: FRead,
) -> Result<ToolResult>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let data = read_file_or_stdin(file)?;
    let output = crate::wfparse(&data)?;
    Ok(ToolResult::immediate_ok(output))
}

pub fn wfscan<FRead>(
    file: &Path,
    read_file_or_stdin: FRead,
) -> Result<ToolResult>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let data = read_file_or_stdin(file)?;
    let output = crate::wfscan(&data)?;
    Ok(ToolResult::immediate_ok(output))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_wfparser_wfscan_commands() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("wf_test.pan");
        std::fs::write(&temp_file_path, b"(Hello <tag> World)")
            .expect("Write temp file");

        let parser_result = super::wfparser(&temp_file_path, |p| Ok(std::fs::read(p)?))
            .expect("Run parser command");
        match parser_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8_lossy(&stdout),
                    "(Hello   World)\n"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let scan_result = super::wfscan(&temp_file_path, |p| Ok(std::fs::read(p)?))
            .expect("Run scan command");
        match scan_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), " hello world \n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }
}