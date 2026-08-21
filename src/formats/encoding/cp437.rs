// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from codepage-437: MIT
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

// Copyright (c) 2018 nabijaczleweli

// See additional licensing details at end of file.

// More-or-less independent implementation; used the test data and docs to create this alternative which is a bit easier to read for me but likely slower.

//! Code Page 437 (DOS Latin US) character encoding and decoding utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::mapping::SingleByteMapping;
use crate::utilities::csv_tools::{self, CsvParseOptions};
use ctb_formats_utilities::encoding::LowArea;
use ctb_utilities::anyhow::anyhow;

fn try_load_mapping(
    values_path: &str,
    variants_path: Option<&str>,
    is_control: bool,
) -> Result<SingleByteMapping> {
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
                // Reason for fallback: strip_prefix("0x") returns None if "0x" prefix is absent; falling back to original string allows parsing raw hex numbers.
                let byte_str = r0.trim().strip_prefix("0x").unwrap_or(r0);
                // Reason for fallback: strip_prefix("0x") returns None if "0x" prefix is absent; falling back to original string allows parsing raw hex numbers.
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

    // 3. Load variants.tsv if requested
    if let Some(vpath) = variants_path {
        if let Some(variants_bytes) = crate::get_encoding_data(vpath) {
            let table = csv_tools::parse_csv_reader(
                &variants_bytes,
                CsvParseOptions {
                    has_header: true,
                    delimiter: b'\t',
                },
            )?;

            for row in table.rows_iter() {
                if let (Some(r0), Some(r1)) = (row.first(), row.get(1)) {
                    // Reason for fallback: strip_prefix("0x") returns None if "0x" prefix is absent; falling back to original string allows parsing raw hex numbers.
                    let byte_str = r0.trim().strip_prefix("0x").unwrap_or(r0);
                    // Reason for fallback: strip_prefix("0x") returns None if "0x" prefix is absent; falling back to original string allows parsing raw hex numbers.
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
    }

    Ok(SingleByteMapping::from_raw(decode_table, encode_table))
}

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static CP437_DINGBATS: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_dingbats/values.tsv",
        Some("cp437/cp437_dingbats/variants.tsv"),
        false,
    )
    .expect("Failed to load CP437 dingbats mapping")
});

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static CP437_DINGBATS_BASE: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_dingbats/values.tsv",
        None,
        false,
    )
    .expect("Failed to load CP437 dingbats base mapping")
});

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static CP437_CONTROL: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_control/values.tsv",
        Some("cp437/cp437_control/variants.tsv"),
        true,
    )
    .expect("Failed to load CP437 control mapping")
});

#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static CP437_CONTROL_BASE: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    try_load_mapping(
        "cp437/cp437_control/values.tsv",
        None,
        true,
    )
    .expect("Failed to load CP437 control base mapping")
});

pub(crate) fn get_mapping(low_area: LowArea, include_variants: bool) -> &'static SingleByteMapping {
    match (low_area, include_variants) {
        (LowArea::Graphical, true) => &CP437_DINGBATS,
        (LowArea::Graphical, false) => &CP437_DINGBATS_BASE,
        (LowArea::Control, true) => &CP437_CONTROL,
        (LowArea::Control, false) => &CP437_CONTROL_BASE,
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
    use ctb_formats_utilities::encoding::CharEncoding;

    #[crate::ctb_test]
    fn test_cp437_control_decoding() -> Result<()> {
        let all_bytes = get_all_bytes()?;
        let enc = CharEncoding::cp437_control();
        let decoded = crate::decode(enc, &all_bytes)?;

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
        let enc = CharEncoding::cp437();
        let decoded = crate::decode(enc, &all_bytes)?;

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
        let enc = CharEncoding::cp437_control();
        let variants_utf8_bytes = crate::get_encoding_data(
            "fixtures/cp437/cp437_control/variants.utf8",
        )
        .ok_or_else(|| anyhow!("Missing variants.utf8 fixture for control"))?;
        let variants_utf8 = String::from_utf8(variants_utf8_bytes)?;

        let expected_cp437 = crate::get_encoding_data(
            "fixtures/cp437/cp437_control/variants.cp437",
        )
        .ok_or_else(|| anyhow!("Missing variants.cp437 fixture for control"))?;

        let encoded = crate::encode(enc, &variants_utf8)?;
        assert_eq!(encoded, expected_cp437);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_cp437_dingbats_variants() -> Result<()> {
        let enc = CharEncoding::cp437();
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

        let encoded = crate::encode(enc, &variants_utf8)?;
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
        let enc_dingbats = CharEncoding::cp437();
        let enc_control = CharEncoding::cp437_control();

        for code in 0u8..=255 {
            let character = crate::chr(enc_dingbats, code);
            let retrieved_code = crate::asc(enc_dingbats, &character);
            assert_eq!(Some(code), retrieved_code);
        }

        for code in 0u8..=255 {
            let character = crate::chr(enc_control, code);
            let retrieved_code = crate::asc(enc_control, &character);
            assert_eq!(Some(code), retrieved_code);
        }

        assert_eq!(Some(65), crate::asc(enc_dingbats, "A"));
        assert_eq!("A", crate::chr(enc_dingbats, 65));
        assert_eq!(Some(65), crate::asc(enc_control, "A"));
        assert_eq!("A", crate::chr(enc_control, 65));

        Ok(())
    }
}

/*
Licensing notice for parts derived from codepage-437 (https://crates.io/crates/codepage-437):
======

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
