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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub fn utf8_to_utf32be(input: &[u8]) -> Result<Vec<u8>> {
    let s = std::str::from_utf8(input).context("invalid UTF-8 sequence")?;
    let mut result = Vec::with_capacity(s.len().saturating_mul(4));
    for c in s.chars() {
        result.extend_from_slice(&u32::from(c).to_be_bytes());
    }
    Ok(result)
}

pub fn utf32be_to_utf8(input: &[u8]) -> Result<Vec<u8>> {
    if !input.len().is_multiple_of(4) {
        bail!("input length must be a multiple of 4");
    }
    let mut result = Vec::with_capacity(input.len());
    for chunk in input.chunks_exact(4) {
        let bytes: [u8; 4] = chunk.try_into().context("invalid chunk size")?;
        let code_point = u32::from_be_bytes(bytes);
        let c =
            char::from_u32(code_point).context("invalid unicode code point")?;
        let mut buf = [0u8; 4];
        let utf8_str = c.encode_utf8(&mut buf);
        result.extend_from_slice(utf8_str.as_bytes());
    }
    Ok(result)
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
    fn test_utf_conversions() {
        let original_str = "Hello, World! 🌄 ñ ü á";
        let original_bytes = original_str.as_bytes();

        let utf32 = utf8_to_utf32be(original_bytes).unwrap();
        let decoded = utf32be_to_utf8(&utf32).unwrap();

        assert_eq!(std::str::from_utf8(&decoded).unwrap(), original_str);

        // Test specific literal bytes example: U+004D, U+0061, and U+10000
        let spec_utf8 = "Ma\u{10000}".as_bytes();
        let spec_expected_utf32 = [
            0x00, 0x00, 0x00, 0x4D, // U+004D
            0x00, 0x00, 0x00, 0x61, // U+0061
            0x00, 0x01, 0x00, 0x00, // U+10000
        ];
        let spec_utf32 = utf8_to_utf32be(spec_utf8).unwrap();
        assert_eq!(spec_utf32, spec_expected_utf32);

        let spec_decoded = utf32be_to_utf8(&spec_expected_utf32).unwrap();
        assert_eq!(spec_decoded, spec_utf8);

        // Test invalid inputs
        utf8_to_utf32be(&[0xff, 0xff]).unwrap_err();
        utf32be_to_utf8(&[0, 0, 0]).unwrap_err();
        utf32be_to_utf8(&[0xff, 0xff, 0xff, 0xff]).unwrap_err(); // invalid code point
    }
}
