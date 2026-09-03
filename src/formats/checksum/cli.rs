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

//! CLI execution helpers for checksum computations.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate as ctb_formats_checksum;
use std::path::{Path, PathBuf};

pub fn csum<FRead>(
    algo: &String,
    file: &PathBuf,
    prefix_0x: &bool,
    read_file_or_stdin: FRead,
) -> Result<ToolResult>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
            let data = read_file_or_stdin(file.as_path())?;
            let hash_algo =
                ctb_formats_checksum::HashAlgorithm::try_from(algo.as_str())?;
            let output = format!(
                "{}\n",
                ctb_formats_checksum::hash_hex(&data, hash_algo, *prefix_0x)
            );
            Ok(ToolResult::immediate_ok(output.into_bytes()))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_csum_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("csum_test_temp.txt");
        std::fs::write(&temp_file_path, b"hello world")
            .expect("Write temp file");

        let result = super::csum(
            &"xxhash32".to_string(),
            &temp_file_path,
            &false,
            |p| Ok(std::fs::read(p)?),
        ).expect("Run csum command");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "cebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let result_0x = super::csum(
            &"xxhash32".to_string(),
            &temp_file_path,
            &true,
            |p| Ok(std::fs::read(p)?),
        ).expect("Run csum command");
        match result_0x {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "0xcebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

}