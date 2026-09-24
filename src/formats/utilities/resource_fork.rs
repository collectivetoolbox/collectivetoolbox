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

//! Safe extraction of resource type codes from classic Macintosh resource forks.
//!
//! Parses the 16-byte Resource Manager header and resource type map without
//! allocating full resource payloads, enabling rapid format detection for
//! files whose entire type identity resides in the resource fork.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Extracts 4-byte resource type codes from raw resource fork data.
///
/// Classic Macintosh resource forks begin with a 16-byte header:
/// - `[0..4]`: Resource data offset
/// - `[4..8]`: Resource map offset
/// - `[8..12]`: Resource data length
/// - `[12..16]`: Resource map length
///
/// Within the resource map, the type list offset is located at map offset 24.
/// The type list begins with a 16-bit count (minus 1), followed by 8-byte
/// type records whose first 4 bytes specify the 4-character resource type
/// code (e.g. `snd `, `FONT`, `APPL`, `PICT`, `alis`).
#[must_use]
pub fn extract_resource_fork_type_codes(bytes: &[u8]) -> Vec<[u8; 4]> {
    let Some(header) = bytes.get(..16) else {
        return Vec::new();
    };

    let Some(map_offset_slice) = header.get(4..8) else {
        return Vec::new();
    };
    let mut map_offset_arr = [0u8; 4];
    map_offset_arr.copy_from_slice(map_offset_slice);
    let map_offset_u32 = u32::from_be_bytes(map_offset_arr);

    let Some(map_len_slice) = header.get(12..16) else {
        return Vec::new();
    };
    let mut map_len_arr = [0u8; 4];
    map_len_arr.copy_from_slice(map_len_slice);
    let map_len_u32 = u32::from_be_bytes(map_len_arr);

    let Ok(map_offset) = usize::try_from(map_offset_u32) else {
        return Vec::new();
    };
    let Ok(map_len) = usize::try_from(map_len_u32) else {
        return Vec::new();
    };

    if map_offset == 0 || map_offset >= bytes.len() || map_len < 28 {
        return Vec::new();
    }

    let map_end = map_offset.saturating_add(map_len).min(bytes.len());
    let Some(map_slice) = bytes.get(map_offset..map_end) else {
        return Vec::new();
    };

    if map_slice.len() < 28 {
        return Vec::new();
    }

    // Type list offset relative to beginning of resource map (at bytes 24..26)
    let Some(type_list_off_slice) = map_slice.get(24..26) else {
        return Vec::new();
    };
    let mut type_list_off_arr = [0u8; 2];
    type_list_off_arr.copy_from_slice(type_list_off_slice);
    let type_list_offset_u16 = u16::from_be_bytes(type_list_off_arr);
    let type_list_offset = usize::from(type_list_offset_u16);

    let Some(type_list) = map_slice.get(type_list_offset..) else {
        return Vec::new();
    };

    if type_list.len() < 2 {
        return Vec::new();
    }

    // Number of types minus 1 (0-indexed count)
    let Some(type_count_slice) = type_list.get(0..2) else {
        return Vec::new();
    };
    let mut type_count_arr = [0u8; 2];
    type_count_arr.copy_from_slice(type_count_slice);
    let type_count_m1 = u16::from_be_bytes(type_count_arr);
    let type_count = usize::from(type_count_m1).saturating_add(1).min(1024);

    let mut result = Vec::new();
    let mut current_offset = 2usize;

    for _ in 0..type_count {
        let entry_end = current_offset.saturating_add(8);
        if entry_end > type_list.len() {
            break;
        }
        if let Some(entry) = type_list.get(current_offset..entry_end) {
            if let Some(code_slice) = entry.get(0..4) {
                let mut code = [0u8; 4];
                code.copy_from_slice(code_slice);
                if !result.contains(&code) {
                    result.push(code);
                }
            }
        }
        current_offset = entry_end;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_extract_resource_fork_type_codes_empty() {
        let empty = [];
        assert!(extract_resource_fork_type_codes(&empty).is_empty());
    }

    #[crate::ctb_test]
    fn test_extract_resource_fork_type_codes_valid() {
        // Construct a minimal valid resource fork buffer:
        // Header: 16 bytes (data_offset=256, map_offset=16, data_len=0, map_len=60)
        let mut fork = vec![0u8; 100];
        fork[0..4].copy_from_slice(&256u32.to_be_bytes());
        fork[4..8].copy_from_slice(&16u32.to_be_bytes()); // map_offset = 16
        fork[8..12].copy_from_slice(&0u32.to_be_bytes());
        fork[12..16].copy_from_slice(&60u32.to_be_bytes()); // map_len = 60

        // In map (offset 16):
        // Bytes 24..26 of map (index 16 + 24 = 40): type_list_offset = 28
        fork[40..42].copy_from_slice(&28u16.to_be_bytes());

        // Type list (offset 16 + 28 = 44):
        // Type count minus 1 = 1 (2 types)
        fork[44..46].copy_from_slice(&1u16.to_be_bytes());

        // Type 1 at index 46..54: 'snd '
        fork[46..50].copy_from_slice(b"snd ");
        // Type 2 at index 54..62: 'PICT'
        fork[54..58].copy_from_slice(b"PICT");

        let types = extract_resource_fork_type_codes(&fork);
        assert_eq!(types.len(), 2);
        assert_eq!(&types[0], b"snd ");
        assert_eq!(&types[1], b"PICT");
    }
}
