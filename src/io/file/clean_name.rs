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

//! Filename sanitization, character replacement, and length limiting utilities.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

/// Maximum filename byte length allowed by standard filesystems (NAME_MAX).
pub const MAX_FILENAME_BYTES: usize = 255;

/// Safe path length limit to prevent issues with path length restrictions.
pub const SAFE_PATH_MAX: usize = 260;

/// Assumed typical enclosing directory path length when none is known.
pub const TYPICAL_ENCLOSING_DIR_LEN: usize = 120;

/// Linux and POSIX maximum path length.
pub const POSIX_PATH_MAX: usize = 4096;

/// Cleans a file name by dropping null bytes, replacing slashes with `⌿`, and
/// limiting the resulting filename length so that it avoids overly long path
/// names.
///
/// When an enclosing directory is provided, the allowable filename length is
/// calculated by subtracting the enclosing directory length and a path
/// separator from the safe path length budget (or POSIX path budget for deep
/// directories), clamped to `MAX_FILENAME_BYTES` (255).
///
/// When no enclosing directory is known, a typical enclosing directory length
/// (`TYPICAL_ENCLOSING_DIR_LEN`) is subtracted from `SAFE_PATH_MAX`, clamped to
/// `MAX_FILENAME_BYTES`.
///
/// The truncation is strictly UTF-8 character boundary safe.
pub fn clean_file_names<P: AsRef<Path>>(
    path: P,
    enclosing_dir: Option<&Path>,
) -> PathBuf {
    let raw_str = path.as_ref().to_string_lossy();

    let max_len = match enclosing_dir {
        Some(dir) => {
            let dir_len = dir.as_os_str().len();
            let total_dir_overhead = dir_len.saturating_add(1);
            if SAFE_PATH_MAX > total_dir_overhead {
                let available =
                    SAFE_PATH_MAX.saturating_sub(total_dir_overhead);
                available.min(MAX_FILENAME_BYTES)
            } else {
                let available =
                    POSIX_PATH_MAX.saturating_sub(total_dir_overhead);
                available.min(MAX_FILENAME_BYTES)
            }
        }
        None => {
            let total_overhead =
                TYPICAL_ENCLOSING_DIR_LEN.saturating_add(1);
            let available = SAFE_PATH_MAX.saturating_sub(total_overhead);
            available.min(MAX_FILENAME_BYTES)
        }
    };
    let max_len = max_len.max(1);

    let mut cleaned = String::new();
    let mut current_bytes: usize = 0;

    for c in raw_str.chars() {
        if c == '\0' {
            continue;
        }
        let mapped_char = if c == '/' { '⌿' } else { c };
        let ch_len = mapped_char.len_utf8();
        if current_bytes.saturating_add(ch_len) > max_len {
            break;
        }
        cleaned.push(mapped_char);
        current_bytes = current_bytes.saturating_add(ch_len);
    }

    if cleaned.is_empty() {
        cleaned.push_str("unnamed");
    }

    PathBuf::from(cleaned)
}

/// Alias for [`clean_file_names`].
pub fn clean_file_name<P: AsRef<Path>>(
    path: P,
    enclosing_dir: Option<&Path>,
) -> PathBuf {
    clean_file_names(path, enclosing_dir)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_clean_file_names_replaces_slash_and_drops_null() {
        let input = "Procedure/With/Slashes\0And\0Nulls";
        let cleaned = clean_file_names(input, None);
        assert_eq!(
            cleaned.to_string_lossy(),
            "Procedure⌿With⌿SlashesAndNulls"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_names_without_enclosing_dir_limits_length() {
        let long_name = "A".repeat(300);
        let cleaned = clean_file_names(&long_name, None);
        let expected_max = SAFE_PATH_MAX
            .saturating_sub(TYPICAL_ENCLOSING_DIR_LEN)
            .saturating_sub(1)
            .min(MAX_FILENAME_BYTES);
        assert_eq!(cleaned.to_string_lossy().len(), expected_max);
    }

    #[crate::ctb_test]
    fn test_clean_file_names_with_enclosing_dir_limits_length() {
        let dir = Path::new("/var/log/my_app/export");
        let dir_len = dir.as_os_str().len();
        let long_name = "B".repeat(300);
        let cleaned = clean_file_names(&long_name, Some(dir));
        let expected_max = SAFE_PATH_MAX
            .saturating_sub(dir_len)
            .saturating_sub(1)
            .min(MAX_FILENAME_BYTES);
        assert_eq!(cleaned.to_string_lossy().len(), expected_max);
    }

    #[crate::ctb_test]
    fn test_clean_file_names_utf8_boundary_safety_with_slash_bar() {
        // '⌿' is 3 bytes in UTF-8
        let input = "/".repeat(100);
        let cleaned = clean_file_names(&input, None);
        // Ensure the string is valid UTF-8 and contains valid '⌿'
        let s = cleaned.to_str().expect("valid utf8");
        assert!(s.chars().all(|c| c == '⌿'));
    }

    #[crate::ctb_test]
    fn test_clean_file_names_empty_fallback() {
        let cleaned = clean_file_names("\0\0", None);
        assert_eq!(cleaned.to_string_lossy(), "unnamed");
    }

    #[crate::ctb_test]
    fn test_clean_file_name_alias() {
        let cleaned = clean_file_name("test/name", None);
        assert_eq!(cleaned.to_string_lossy(), "test⌿name");
    }
}
