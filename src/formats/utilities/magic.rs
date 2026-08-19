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

//! Magic header byte pattern matching utilities for format identification.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// A signature rule defining magic bytes to inspect in a file header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MagicPattern {
    /// Byte offset where signature starts in file header (typically 0).
    pub offset: usize,
    /// Byte pattern expected at offset.
    pub bytes: &'static [u8],
    /// Optional byte mask (same length as bytes).
    pub mask: Option<&'static [u8]>,
    /// Confidence or priority score when pattern matches (default: 100).
    pub priority: u32,
}

impl MagicPattern {
    /// Constructs an exact byte sequence magic pattern at offset 0.
    pub const fn exact(bytes: &'static [u8]) -> Self {
        Self {
            offset: 0,
            bytes,
            mask: None,
            priority: 100,
        }
    }

    /// Constructs an exact byte sequence magic pattern at a specific offset and priority.
    pub const fn exact_at(
        offset: usize,
        bytes: &'static [u8],
        priority: u32,
    ) -> Self {
        Self {
            offset,
            bytes,
            mask: None,
            priority,
        }
    }

    /// Constructs a masked byte magic pattern at offset 0.
    pub const fn masked(
        bytes: &'static [u8],
        mask: &'static [u8],
        priority: u32,
    ) -> Self {
        Self {
            offset: 0,
            bytes,
            mask: Some(mask),
            priority,
        }
    }

    /// Checks if a byte slice matches this magic pattern.
    pub fn matches(&self, data: &[u8]) -> bool {
        let end = self.offset.saturating_add(self.bytes.len());
        if data.len() < end {
            return false;
        }
        let slice = match data.get(self.offset..end) {
            Some(s) => s,
            None => return false,
        };

        match self.mask {
            Some(mask) => {
                if mask.len() != self.bytes.len() {
                    return false;
                }
                for ((&byte, &m), &expected) in
                    slice.iter().zip(mask).zip(self.bytes)
                {
                    if (byte & m) != expected {
                        return false;
                    }
                }
                true
            }
            None => slice == self.bytes,
        }
    }
}

#[cfg(test)]
#[expect(
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

    #[ctb_test]
    fn test_magic_pattern_matching() {
        let gzip_magic = MagicPattern::exact(&[0x1F, 0x8B]);
        assert!(gzip_magic.matches(&[0x1F, 0x8B, 0x08, 0x00]));
        assert!(!gzip_magic.matches(&[0x1F, 0xA0, 0x08, 0x00]));

        let lzw_block_magic =
            MagicPattern::masked(&[0x1F, 0x9D, 0x80], &[0xFF, 0xFF, 0x80], 110);
        assert!(lzw_block_magic.matches(&[0x1F, 0x9D, 0x90]));
        assert!(!lzw_block_magic.matches(&[0x1F, 0x9D, 0x10]));
    }
}
