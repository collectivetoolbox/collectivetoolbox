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

//! Unified table-driven single-byte character mapping and encoding abstractions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use ctb_utilities::anyhow::anyhow;

pub use ctb_formats_utilities::encoding::{
    CharEncoding, LineEndingFormat, LineEndingKind, LineEndingOption, LowArea,
    NeoRegion, TerminationMode,
};

/// A 256-entry bidirectional mapping between single bytes and Unicode characters.
#[derive(Debug, Clone)]
pub struct SingleByteMapping {
    decode_table: [char; 256],
    encode_table: HashMap<char, u8>,
}

impl SingleByteMapping {
    /// Creates a mapping from raw decode and encode tables.
    #[must_use]
    pub const fn from_raw(
        decode_table: [char; 256],
        encode_table: HashMap<char, u8>,
    ) -> Self {
        Self {
            decode_table,
            encode_table,
        }
    }

    /// Decodes a single byte to its corresponding Unicode character.
    pub fn decode_byte(&self, code: u8) -> Result<char> {
        self.decode_table
            .get(usize::from(code))
            .copied()
            .ok_or_else(|| anyhow!("Invalid byte {code}"))
    }

    /// Returns the character string for a single byte.
    #[expect(
        clippy::expect_used,
        reason = "u8 index is 0..=255 which always fits in 256-entry decode_table"
    )]
    #[must_use]
    pub fn chr(&self, code: u8) -> String {
        self.decode_table
            .get(usize::from(code))
            .copied()
            .expect("u8 fits in 256-entry decode table")
            .to_string()
    }

    /// Returns the byte value for the first character of a string, if mappable.
    pub fn asc(&self, s: &str) -> Option<u8> {
        let first_char = s.chars().next()?;
        self.encode_table.get(&first_char).copied()
    }

    /// Encodes a Unicode string into a vector of bytes.
    pub fn encode(&self, input: &str) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(input.len());
        for c in input.chars() {
            if let Some(&byte) = self.encode_table.get(&c) {
                result.push(byte);
            } else {
                return Err(anyhow!("Encoding error: unmappable character '{c}'"));
            }
        }
        Ok(result)
    }

    /// Decodes a slice of bytes into a Unicode string.
    #[expect(
        clippy::expect_used,
        reason = "u8 index is 0..=255 which always fits in 256-entry decode_table"
    )]
    pub fn decode(&self, input: &[u8]) -> Result<String> {
        let mut result = String::with_capacity(input.len());
        for &byte in input {
            result.push(
                self.decode_table
                    .get(usize::from(byte))
                    .copied()
                    .expect("u8 fits in 256-entry decode table"),
            );
        }
        Ok(result)
    }

    /// Remaps a single byte to a new Unicode character in both tables.
    #[expect(
        clippy::expect_used,
        reason = "u8 index is 0..=255 which always fits in 256-entry decode_table"
    )]
    pub fn remap(&mut self, byte: u8, character: char) {
        let idx = usize::from(byte);
        let old_char = self
            .decode_table
            .get(idx)
            .copied()
            .expect("u8 fits in 256-entry decode table");
        if let Some(slot) = self.decode_table.get_mut(idx) {
            *slot = character;
        }

        if self.encode_table.get(&old_char) == Some(&byte) {
            self.encode_table.remove(&old_char);
        }
        self.encode_table.insert(character, byte);
    }
}

/// Returns the static `SingleByteMapping` for the specified `CharEncoding`.
#[must_use]
pub fn mapping(enc: CharEncoding) -> &'static SingleByteMapping {
    match enc {
        CharEncoding::Cp437 {
            low_area,
            include_variants,
        } => crate::cp437::get_mapping(low_area, include_variants),
        CharEncoding::Neo { region, low_area } => {
            crate::neo::get_mapping(region, low_area)
        }
        CharEncoding::MacRoman => &crate::standard::MACROMAN_MAPPING,
        CharEncoding::Windows1252 => &crate::standard::WINDOWS_1252_MAPPING,
    }
}

/// Encodes a Unicode string using the specified character encoding.
pub fn encode(enc: CharEncoding, input: &str) -> Result<Vec<u8>> {
    mapping(enc).encode(input)
}

/// Encodes a Unicode string with optional line ending conversion.
pub fn encode_with_options(
    enc: CharEncoding,
    input: &str,
    line_ending_opt: LineEndingOption,
) -> Result<Vec<u8>> {
    let converted = crate::line_endings::apply_line_ending_option(
        input,
        enc,
        line_ending_opt,
    )?;
    mapping(enc).encode(&converted)
}

/// Decodes a byte slice into a Unicode string using the specified character encoding.
pub fn decode(enc: CharEncoding, input: &[u8]) -> Result<String> {
    mapping(enc).decode(input)
}

/// Decodes a byte slice into a Unicode string with optional line ending conversion.
pub fn decode_with_options(
    enc: CharEncoding,
    input: &[u8],
    line_ending_opt: LineEndingOption,
) -> Result<String> {
    let decoded = mapping(enc).decode(input)?;
    crate::line_endings::apply_line_ending_option(&decoded, enc, line_ending_opt)
}

/// Transcodes bytes from one character encoding to another with optional line ending conversion.
pub fn transcode(
    input: &[u8],
    src_enc: CharEncoding,
    dst_enc: CharEncoding,
    line_ending_opt: LineEndingOption,
) -> Result<Vec<u8>> {
    let decoded = decode(src_enc, input)?;
    encode_with_options(dst_enc, &decoded, line_ending_opt)
}

/// Returns the string representation of a character code in the specified encoding.
#[must_use]
pub fn chr(enc: CharEncoding, code: u8) -> String {
    mapping(enc).chr(code)
}

/// Returns the character corresponding to a byte in the specified encoding.
pub fn chr_char(enc: CharEncoding, code: u8) -> Result<char> {
    mapping(enc).decode_byte(code)
}

/// Returns the byte value for the first character of a string in the specified encoding.
#[must_use]
pub fn asc(enc: CharEncoding, s: &str) -> Option<u8> {
    mapping(enc).asc(s)
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
    fn test_unified_cp437_encoding() -> Result<()> {
        let enc = CharEncoding::cp437();
        let encoded = encode(enc, "A")?;
        assert_eq!(encoded, vec![65]);
        let decoded = decode(enc, &[65])?;
        assert_eq!(decoded, "A");
        assert_eq!(chr(enc, 65), "A");
        assert_eq!(asc(enc, "A"), Some(65));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_encode_decode_with_options() -> Result<()> {
        let mac = CharEncoding::mac_roman();
        let unix_text = "Hello\nWorld\n";

        // encode with EncodingDefault on MacRoman should produce CR line endings (0x0D)
        let mac_bytes =
            encode_with_options(mac, unix_text, LineEndingOption::EncodingDefault)?;
        assert_eq!(mac_bytes, b"Hello\rWorld\r");

        // decode with EncodingDefault on Windows1252 should convert to CRLF
        let win = CharEncoding::windows_1252();
        let win_text =
            decode_with_options(win, b"Hello\nWorld\n", LineEndingOption::EncodingDefault)?;
        assert_eq!(win_text, "Hello\r\nWorld\r\n");

        // Transcode from Windows-1252 (with CRLF) to MacRoman with EncodingDefault -> CR (0x0D)
        let win_bytes = b"Hello\r\nWorld\r\n";
        let transcoded = transcode(
            win_bytes,
            win,
            mac,
            LineEndingOption::EncodingDefault,
        )?;
        assert_eq!(transcoded, b"Hello\rWorld\r");

        Ok(())
    }
}
