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

use ctb_utilities::string::to_char;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod cli;

use anyhow::{Result, anyhow};

pub fn to_hex_dump(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}: ", i.saturating_mul(16)));
        for j in 0..16 {
            if let Some(&b) = chunk.get(j) {
                out.push_str(&format!("{b:02x} "));
            } else {
                out.push_str("   ");
            }
        }
        out.push_str(" |");
        for &b in chunk {
            if (0x20..=0x7e).contains(&b) {
                out.push(char::from(b));
            } else {
                out.push('.');
            }
        }
        out.push_str("|\n");
    }
    out
}

pub fn to_fancy_hex_dump(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}: ", i.saturating_mul(16)));
        for j in 0..16 {
            if let Some(&b) = chunk.get(j) {
                out.push_str(&format!("{b:02x} "));
            } else {
                out.push_str("   ");
            }
        }
        out.push_str(" |");
        for &b in chunk {
            out.push(byte_to_graphical(b));
        }
        out.push_str("|\n");
    }
    out
}

#[allow(
    clippy::expect_used,
    reason = "CP437 mapping is defined for all u8 byte values 0..=255"
)]
pub fn byte_to_graphical(b: u8) -> char {
    if b == 0 {
        '␀'
    } else if b == 1 {
        '␁'
    } else if b == 2 {
        '␂'
    } else if b == 3 {
        '␃'
    } else if b == 4 {
        '␄'
    } else if b == 5 {
        '␅'
    } else if b == 6 {
        '␆'
    } else if b == 7 {
        '␇'
    } else if b == 8 {
        '␈'
    } else if b == 9 {
        '␉'
    } else if b == 10 {
        '␊'
    } else if b == 11 {
        '␋'
    } else if b == 12 {
        '␌'
    } else if b == 13 {
        '␍'
    } else if b == 14 {
        '␎'
    } else if b == 15 {
        '␏'
    } else if b == 16 {
        '␐'
    } else if b == 17 {
        '␑'
    } else if b == 18 {
        '␒'
    } else if b == 19 {
        '␓'
    } else if b == 20 {
        '␔'
    } else if b == 21 {
        '␕'
    } else if b == 22 {
        '␖'
    } else if b == 23 {
        '␗'
    } else if b == 24 {
        '␘'
    } else if b == 25 {
        '␙'
    } else if b == 26 {
        '␚'
    } else if b == 27 {
        '␛'
    } else if b == 28 {
        '␜'
    } else if b == 29 {
        '␝'
    } else if b == 30 {
        '␞'
    } else if b == 31 {
        '␟'
    } else if b == 32 {
        '␠'
    } else if (33..=126).contains(&b) {
        char::from(b)
    } else if b == 127 {
        '␡'
    } else if b < 255 {
        ctb_formats_encoding::cp437::chr_char(b)
            .expect("CP437 mapping is defined for all u8 byte values 0..=255")
    } else {
        '⍽'
    }
}

/// Decodes a hexadecimal string into a byte vector.
/// Decodes a hexadecimal string into a byte vector.
///
/// Any prefix of "0x" or "0X" is stripped, and whitespace or colons are ignored.
pub fn hex2bin(s: &str) -> Result<Vec<u8>> {
    let mut s = s.trim();
    if let Some(stripped) = s.strip_prefix("0x") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("0X") {
        s = stripped;
    }
    let clean: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':')
        .collect();
    hex::decode(&clean).map_err(|e| anyhow!("Failed to decode hex string: {e}"))
}

/// Encodes binary data into a lowercase hexadecimal string.
pub fn bin2hex(data: &[u8]) -> String {
    utilities::bin2hex(data)
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

    #[crate::ctb_test]
    fn test_traditional_hexdump() {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_hex_dump(data);
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello, World!...|\n";
        assert_eq!(dump, expected);
    }

    #[crate::ctb_test]
    fn test_fancy_hexdump() {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_fancy_hex_dump(data);
        // Note: 'H' is 'H' (72), '\x00' is '␀', '\x01' is '␁', '\xff' is ' ' (NBSP in CP437, code 160)
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello,␠World!␀␁⍽|\n";
        assert_eq!(dump, expected);
    }

    #[crate::ctb_test]
    fn test_byte_to_graphical_all_bytes() {
        // Verify every byte 0..=255 returns a unique character
        let mut seen = std::collections::HashSet::new();
        for b in 0u8..=255 {
            let c = byte_to_graphical(b);
            assert!(seen.insert(c), "Duplicate glyph found for byte {b}: {c}");
        }
        assert_eq!(seen.len(), 256);
    }

    #[crate::ctb_test]
    fn test_hex2bin_and_bin2hex() {
        let original = b"Hello, World!";
        let encoded = bin2hex(original);
        assert_eq!(encoded, "48656c6c6f2c20576f726c6421");

        let decoded = hex2bin(&encoded).unwrap();
        assert_eq!(decoded, original);

        // Test with 0x prefix, whitespace, and colons
        let complex = "0x48:65:6c:6c:6f 2c 20 57 6f 72 6c 64 21";
        let decoded_complex = hex2bin(complex).unwrap();
        assert_eq!(decoded_complex, original);

        // Test invalid hex input
        hex2bin("not a hex string").unwrap_err();
    }
}
