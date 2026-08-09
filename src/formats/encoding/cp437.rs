// SPDX-License-Identifier for parts derived from codepage-437: MIT

// More-or-less independent implementation because Firefox reported that crate as malicious. I suspect it's a false positive, but used the test data and docs to create this alternative which is a bit easier to read for me but likely slower.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::utilities::csv_tools::{self, CsvParseOptions};
use ctb_utilities::anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct Cp437Mapping {
    decode_table: [char; 256],
    encode_table: HashMap<char, u8>,
}

impl Cp437Mapping {
    pub fn decode_byte(&self, code: u8) -> Result<char> {
        self.decode_table
            .get(usize::from(code))
            .copied()
            .ok_or_else(|| anyhow!("Invalid Code Page 437 byte {code}"))
    }

    #[expect(
        clippy::expect_used,
        reason = "u8 index is 0..=255 which always fits in 256-entry decode_table"
    )]
    pub fn chr(&self, code: u8) -> String {
        self.decode_table
            .get(usize::from(code))
            .copied()
            .expect("u8 fits in 256-entry decode table")
            .to_string()
    }

    pub fn asc(&self, s: &str) -> Option<u8> {
        let first_char = s.chars().next()?;
        self.encode_table.get(&first_char).copied()
    }

    pub fn encode(&self, input: &str) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(input.len());
        for c in input.chars() {
            if let Some(&byte) = self.encode_table.get(&c) {
                result.push(byte);
            } else {
                return Err(anyhow!(
                    "Encoding error: unmappable character '{c}'"
                ));
            }
        }
        Ok(result)
    }

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

    #[expect(
        clippy::expect_used,
        reason = "u8 index is 0..=255 which always fits in 256-entry decode_table"
    )]
    pub fn remap(&mut self, byte: u8, character: char) {
        let idx = usize::from(byte);
        let old_char = self.decode_table.get(idx).copied().expect("u8 fits in 256-entry decode table");
        if let Some(slot) = self.decode_table.get_mut(idx) {
            *slot = character;
        }

        if self.encode_table.get(&old_char) == Some(&byte) {
            self.encode_table.remove(&old_char);
        }
        self.encode_table.insert(character, byte);
    }
}

fn try_load_mapping(
    values_path: &str,
    variants_path: &str,
    is_control: bool,
) -> Result<Cp437Mapping> {
    let mut decode_table = ['\0'; 256];
    let mut encode_table = HashMap::new();

    // 1. Initialize standard ASCII ranges
    if is_control {
        for i in 0..128 {
            let byte = u8::try_from(i)?;
            let character = char::from(byte);
            if let Some(slot) = decode_table.get_mut(usize::from(byte)) {
                *slot = character;
            }
            encode_table.insert(character, byte);
        }
    } else {
        if let Some(slot) = decode_table.get_mut(0) {
            *slot = '\0';
        }
        encode_table.insert('\0', 0);
        for i in 0x20..0x7F {
            let byte = u8::try_from(i)?;
            let character = char::from(byte);
            if let Some(slot) = decode_table.get_mut(usize::from(byte)) {
                *slot = character;
            }
            encode_table.insert(character, byte);
        }
    }

    // 2. Load values.tsv
    if let Some(values_bytes) = crate::get_encoding_data(values_path) {
        let table = csv_tools::parse_csv_reader(
            &values_bytes,
            CsvParseOptions {
                has_header: true,
                delimiter: b'\t',
            },
        )?;

        for row in table.rows_iter() {
            if let (Some(r0), Some(r1)) = (row.first(), row.get(1)) {
                let byte_str = r0.trim().strip_prefix("0x").unwrap_or(r0);
                let uni_str = r1.trim().strip_prefix("0x").unwrap_or(r1);
                let byte = u8::from_str_radix(byte_str, 16).map_err(|e| {
                    anyhow!("Failed to parse byte hex '{byte_str}': {e}")
                })?;
                let uni_code =
                    u32::from_str_radix(uni_str, 16).map_err(|e| {
                        anyhow!("Failed to parse Unicode hex '{uni_str}': {e}")
                    })?;
                let character = char::from_u32(uni_code).ok_or_else(|| {
                    anyhow!("Invalid Unicode code point '{uni_str}'")
                })?;

                if let Some(slot) = decode_table.get_mut(usize::from(byte)) {
                    *slot = character;
                }
                encode_table.insert(character, byte);
            }
        }
    }

    // 3. Load variants.tsv
    if let Some(variants_bytes) = crate::get_encoding_data(variants_path) {
        let table = csv_tools::parse_csv_reader(
            &variants_bytes,
            CsvParseOptions {
                has_header: true,
                delimiter: b'\t',
            },
        )?;

        for row in table.rows_iter() {
            if let (Some(r0), Some(r1)) = (row.first(), row.get(1)) {
                let byte_str = r0.trim().strip_prefix("0x").unwrap_or(r0);
                let uni_str = r1.trim().strip_prefix("0x").unwrap_or(r1);
                let byte = u8::from_str_radix(byte_str, 16).map_err(|e| {
                    anyhow!("Failed to parse byte hex '{byte_str}': {e}")
                })?;
                let uni_code =
                    u32::from_str_radix(uni_str, 16).map_err(|e| {
                        anyhow!("Failed to parse Unicode hex '{uni_str}': {e}")
                    })?;
                let character = char::from_u32(uni_code).ok_or_else(|| {
                    anyhow!("Invalid Unicode code point '{uni_str}'")
                })?;

                // Variants only extend the encode_table, do not overwrite the decode_table
                encode_table.insert(character, byte);
            }
        }
    }

    Ok(Cp437Mapping {
        decode_table,
        encode_table,
    })
}

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static CP437_DINGBATS: LazyLock<Cp437Mapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_dingbats/values.tsv",
        "cp437/cp437_dingbats/variants.tsv",
        false,
    )
    .expect("Failed to load CP437 dingbats mapping")
});

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static CP437_CONTROL: LazyLock<Cp437Mapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_control/values.tsv",
        "cp437/cp437_control/variants.tsv",
        true,
    )
    .expect("Failed to load CP437 control mapping")
});

pub fn chr(code: u8) -> String {
    CP437_DINGBATS.chr(code)
}

pub fn chr_char(code: u8) -> Result<char> {
    CP437_DINGBATS.decode_byte(code)
}

pub fn asc(s: &str) -> Option<u8> {
    CP437_DINGBATS.asc(s)
}

pub fn encode(input: &str) -> Result<Vec<u8>> {
    CP437_DINGBATS.encode(input)
}

pub fn decode(input: &[u8]) -> Result<String> {
    CP437_DINGBATS.decode(input)
}

pub fn chr_control(code: u8) -> String {
    CP437_CONTROL.chr(code)
}

pub fn asc_control(s: &str) -> Option<u8> {
    CP437_CONTROL.asc(s)
}

pub fn encode_control(input: &str) -> Result<Vec<u8>> {
    CP437_CONTROL.encode(input)
}

pub fn decode_control(input: &[u8]) -> Result<String> {
    CP437_CONTROL.decode(input)
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
    fn test_cp437_control_decoding() -> Result<()> {
        let all_bytes = get_all_bytes()?;
        let decoded = decode_control(&all_bytes)?;

        let expected_bytes = crate::get_encoding_data(
            "fixtures/cp437/cp437_control/all.utf8",
        )
        .ok_or_else(|| anyhow!("Missing all.utf8 fixture for control"))?;
        let expected = String::from_utf8(expected_bytes)?;

        assert_eq!(decoded, expected);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_cp437_dingbats_decoding() -> Result<()> {
        let all_bytes = get_all_bytes()?;
        let decoded = decode(&all_bytes)?;

        let expected_bytes =
            crate::get_encoding_data("fixtures/cp437/cp437_dingbats/all.utf8")
                .ok_or_else(|| {
                    anyhow!("Missing all.utf8 fixture for dingbats")
                })?;
        let expected = String::from_utf8(expected_bytes)?;

        assert_eq!(decoded, expected);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_cp437_control_variants() -> Result<()> {
        let variants_utf8_bytes = crate::get_encoding_data(
            "fixtures/cp437/cp437_control/variants.utf8",
        )
        .ok_or_else(|| anyhow!("Missing variants.utf8 fixture for control"))?;
        let variants_utf8 = String::from_utf8(variants_utf8_bytes)?;

        let expected_cp437 = crate::get_encoding_data(
            "fixtures/cp437/cp437_control/variants.cp437",
        )
        .ok_or_else(|| anyhow!("Missing variants.cp437 fixture for control"))?;

        let encoded = encode_control(&variants_utf8)?;
        assert_eq!(encoded, expected_cp437);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_cp437_dingbats_variants() -> Result<()> {
        let variants_utf8_bytes = crate::get_encoding_data(
            "fixtures/cp437/cp437_dingbats/variants.utf8",
        )
        .ok_or_else(|| anyhow!("Missing variants.utf8 fixture for dingbats"))?;
        let variants_utf8 = String::from_utf8(variants_utf8_bytes)?;

        let expected_cp437 = crate::get_encoding_data(
            "fixtures/cp437/cp437_dingbats/variants.cp437",
        )
        .ok_or_else(|| {
            anyhow!("Missing variants.cp437 fixture for dingbats")
        })?;

        let encoded = encode(&variants_utf8)?;
        assert_eq!(encoded, expected_cp437);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_remap() -> Result<()> {
        let mut mapping = CP437_DINGBATS.clone();
        let square_root_or_checkmark = 0xFB;

        // Before remapping: 0xFB -> '√'
        assert_eq!(mapping.chr(square_root_or_checkmark), "√");
        assert_eq!(mapping.asc("√"), Some(square_root_or_checkmark));

        // Remap to check mark '✓'
        mapping.remap(square_root_or_checkmark, '✓');

        // After remapping: 0xFB -> '✓'
        assert_eq!(mapping.chr(square_root_or_checkmark), "✓");
        assert_eq!(mapping.asc("✓"), Some(square_root_or_checkmark));
        assert_eq!(mapping.decode(&[square_root_or_checkmark])?, "✓");
        assert_eq!(mapping.encode("✓")?, vec![square_root_or_checkmark]);

        // Remapping should not have affected original
        assert_eq!(CP437_DINGBATS.chr(square_root_or_checkmark), "√");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_chr_asc() -> Result<()> {
        for code in 0u8..=255 {
            let character = chr(code);
            let retrieved_code = asc(&character);
            assert_eq!(Some(code), retrieved_code);
        }

        for code in 0u8..=255 {
            let character = chr_control(code);
            let retrieved_code = asc_control(&character);
            assert_eq!(Some(code), retrieved_code);
        }

        assert_eq!(Some(65), asc("A"));
        assert_eq!("A", chr(65));

        assert_eq!(Some(65), asc_control("A"));
        assert_eq!("A", chr_control(65));
        Ok(())
    }
}

/*

// From codepage_437 (https://crates.io/crates/codepage-437):

The MIT License (MIT)

Copyright (c) 2018 nabijaczleweli

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.


*/
