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

//! CLI execution helpers for DCE utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::Path;

pub fn php_to_csv(php_file: &Path) -> Result<ToolResult> {
    crate::to_csv::php_file_to_csv_files(php_file)?;
    Ok(ToolResult::immediate_ok(Vec::new()))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    #[crate::ctb_test]
    fn test_dceutils_php_to_csv_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let random_num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let php_filename = format!("test_cmd_{random_num}.php");
        let php_path = temp_dir.path().join(&php_filename);

        let php_content = r"<?php
$my_test_array = array('a' => '1', 'b' => '2');
?>";
        std::fs::write(&php_path, php_content).expect("Write temp PHP file");

        let expected_csv_name = format!("{php_filename}-my_test_array.csv");
        let expected_csv_path = std::path::Path::new(&expected_csv_name);

        if expected_csv_path.exists() {
            let _ = std::fs::remove_file(expected_csv_path);
        }

        let result = super::php_to_csv(&php_path).expect("Run php_to_csv");
        match result {
            ToolResult::Immediate { .. } => {
                assert!(expected_csv_path.exists());
                let csv_content = std::fs::read_to_string(expected_csv_path)
                    .expect("Read CSV content");
                assert_eq!(csv_content, "a,1\nb,2\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let _ = std::fs::remove_file(expected_csv_path);
    }
}