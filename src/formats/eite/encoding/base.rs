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

//! Base conversion



pub use ctb_formats_math::base::{Base, BaseAlphabet, NumeralSystem, BaseStringFormatSettings,  int_to_base36_char, int_from_base36_char, int_from_base_str_u32, int_from_base_str_u128, int_from_base_str_big_alphabet, int_to_base_str_big_alphabet, int_from_base_str_big, int_from_base_str_big, int_to_base_str, hex_to_dec_single, dec_to_hex_single, hex_to_dec_string, dec_to_hex_string, get_digits_needed, casefold_base_chars_in_string, format_base_string,  base_to_base_string, is_supported_base_with_default_alphabet as is_supported_base, is_base_digit_alphabet, is_base_str_alphabet, is_base_digit,  is_base_str, char_from_hex_byte};

/*
Maybe useful:

fn u32_slice_as_bytes_le(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn u32_slice_as_bytes_be(values: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for &v in values {
        out.extend_from_slice(&v.to_be_bytes());
    }
    out
}
 */

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
    use ctb_formats_utilities::{
        assert_string_ok_eq_no_errors, assert_string_ok_eq_no_warnings,
    };

    use super::*;

    #[crate::ctb_test]
    fn test_base36_digit_roundtrip() {
        for n in 0..=35 {
            let ch = int_to_base36_char(n).unwrap();
            let v = int_from_base36_char(&ch).unwrap();
            assert_eq!(n, v);
        }
        assert!(int_to_base36_char(36).is_err());
    }

    #[crate::ctb_test]
    fn test_hex_conversion_examples() {
        // Mirrors runTestsMath base conversion portion
        let hex = dec_to_hex_single(9917).unwrap();
        assert_eq!(hex, "26BD");
        let dec = hex_to_dec_single("26BD").unwrap();
        assert_eq!(dec, 9917);
    }

    #[crate::ctb_test]
    fn test_char_from_hex_byte() {
        assert_eq!(char_from_hex_byte("41").unwrap(), 'A');
        assert_eq!(char_from_hex_byte("7f").unwrap(), '\u{007F}');
        assert!(char_from_hex_byte("XYZ").is_err());
    }

    #[crate::ctb_test]
    fn test_base_to_base_string() {
        let format_settings = BaseStringFormatSettings::default();
        assert_string_ok_eq_no_warnings(
            "26",
            base_to_base_string("1A", 16, 10, &format_settings),
        );

        let (_result, _log) = assert_string_ok_eq_no_warnings(
            "26 16 4",
            base_to_base_string("0x1A, 0x10, 0x04", 16, 10, &format_settings),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            // This result doesn't make mathematical sense, as the outputs are
            // base 10.
            "0x26, 0x16, 0x4",
            base_to_base_string(
                "0x1A, 0x10, 0x04",
                16,
                10,
                &BaseStringFormatSettings {
                    separator: "".to_string(),
                    filter_chars: false,
                    parse_prefixes: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "12",
            base_to_base_string("10", 10, 8, &format_settings),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            "26,uuuu 4F,é 16, 4",
            base_to_base_string(
                "26,uuuu 4F,é 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    separator: "".to_string(),
                    filter_chars: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "0x026!0x04f!0x016!0x004",
            base_to_base_string(
                "26, 4F, 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: true,
                    pad: BaseConversionPaddingMode {
                        pad_l: 3,
                        pad_fit: false,
                    },
                    ..Default::default()
                },
            ),
        );

        assert_string_ok_eq_no_warnings(
            "0x26!0x4F!0x16!0x04",
            base_to_base_string(
                "26, 4F, 16, 0x04",
                16,
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: false,
                    limit: u64::from(u8::MAX),
                    pad: BaseConversionPaddingMode {
                        pad_l: 0,
                        pad_fit: true,
                    },
                    ..Default::default()
                },
            ),
        );

        let (_result, log) = assert_string_ok_eq_no_errors(
            "26 4 16",
            format_base_string(
                "26, 4F, 16",
                10,
                &BaseStringFormatSettings::default(),
            ),
        );
        assert!(log.has_warnings());

        assert_string_ok_eq_no_warnings(
            "26, 4F, 16, F, 0",
            format_base_string(
                "0x26, 4f, 16f, 0",
                16,
                &BaseStringFormatSettings {
                    separator: ", ".to_string(),
                    limit: 255,
                    ..Default::default()
                },
            ),
        );

        assert_string_ok_eq_no_warnings(
            "2 6 4 F 1 6",
            format_base_string(
                "0x26, 4f, 16",
                16,
                &BaseStringFormatSettings {
                    limit: 1,
                    ..Default::default()
                },
            ),
        );
    }

    #[crate::ctb_test]
    fn test_format_base_string() {
        let (_result, log) = assert_string_ok_eq_no_errors(
            "26 0 4F 0 16F",
            format_base_string(
                "26, 0n4F, 0x16fZz",
                16,
                &BaseStringFormatSettings {
                    parse_prefixes: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        let (_result, log) = assert_string_ok_eq_no_errors(
            "0x26!, 0x0!n0x4f!, 0x0!x0x16f!Zz",
            format_base_string(
                "26, 0n4F, 0x16fZz",
                16,
                &BaseStringFormatSettings {
                    prefix: "0x".to_string(),
                    separator: "!".to_string(),
                    lowercase: true,
                    parse_prefixes: false,
                    filter_chars: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_collapse_filtered() {
        let settings = BaseStringFormatSettings {
            collapse_filtered: true,
            ..Default::default()
        };
        // "10_000" should collapse '_' and parse as "10000"
        let res = base_to_base_string("10_0!00", 10, 10, &settings);
        let (res, log) = res.expect("Error");
        assert_eq!("10000", res);
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_collapse_only() {
        let settings = BaseStringFormatSettings {
            collapse_only: vec!["_".to_string()],
            filter_chars: true,
            ..Default::default()
        };
        // "10_000" should collapse '_' and parse as "10000", leaving other filtered chars as spaces
        assert_string_ok_eq_no_warnings(
            "10000",
            base_to_base_string("10_000", 10, 10, &settings),
        );
        let conv = base_to_base_string("10_000!", 10, 10, &settings);
        assert!(conv.is_ok());
        let (conv, log) = conv.expect("checked");
        assert_eq!("10000", conv);
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_base64_alphabet_digits() {
        let alpha = BaseAlphabet::Base64Standard;
        assert_eq!(alpha.char_for_digit(0).unwrap(), 'A');
        assert_eq!(alpha.char_for_digit(25).unwrap(), 'Z');
        assert_eq!(alpha.char_for_digit(26).unwrap(), 'a');
        assert_eq!(alpha.char_for_digit(51).unwrap(), 'z');
        assert_eq!(alpha.char_for_digit(52).unwrap(), '0');
        assert_eq!(alpha.char_for_digit(61).unwrap(), '9');
        assert_eq!(alpha.char_for_digit(62).unwrap(), '+');
        assert_eq!(alpha.char_for_digit(63).unwrap(), '/');
        assert!(alpha.char_for_digit(64).is_err());

        for d in 0..=63 {
            let ch = alpha.char_for_digit(d).unwrap();
            let parsed = alpha.digit_for_char(ch).unwrap();
            assert_eq!(d, parsed);
        }
    }

    #[crate::ctb_test]
    fn test_base64_standard_conversion() {
        // Base 10 (Standard) -> Base 64 (Base64Standard)
        let dec_to_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "A B / BA D/",
            base_to_base_string("0 1 63 64 255", 10, 64, &dec_to_b64),
        );

        // Base 64 (Base64Standard) -> Base 10 (Standard)
        let b64_to_dec = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Base64Standard,
            output_alphabet: BaseAlphabet::Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "0 1 63 64 255",
            base_to_base_string("A B / BA D/", 64, 10, &b64_to_dec),
        );

        // Base 16 (Standard) -> Base 64 (Base64Standard)
        let hex_to_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "D/ EA",
            base_to_base_string("ff 100", 16, 64, &hex_to_b64),
        );

        // Case sensitivity: 'a' is 26, 'A' is 0, '0' is 52
        assert_string_ok_eq_no_warnings(
            "26 0 52",
            base_to_base_string("a A 0", 64, 10, &b64_to_dec),
        );

        // Zero-padding with 'A'
        let pad_settings = BaseStringFormatSettings {
            output_alphabet: BaseAlphabet::Base64Standard,
            pad: BaseConversionPaddingMode {
                pad_l: 4,
                pad_fit: false,
            },
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "AAAF",
            base_to_base_string("5", 10, 64, &pad_settings),
        );

        // Base 10 (Standard) -> Base 30 (Base64Standard)
        let dec_to_b30_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "BBPBDU",
            base_to_base_string("25516010", 10, 30, &dec_to_b30_b64),
        );

        // Base 30 (Base64Standard) -> Base 10 (Standard)
        let b30_b64_to_dec = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Base64Standard,
            output_alphabet: BaseAlphabet::Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "25516010",
            base_to_base_string("BBPBDU", 30, 10, &b30_b64_to_dec),
        );

        // Base > 36 fails if Standard alphabet is configured
        let standard_settings = BaseStringFormatSettings::default();
        assert!(base_to_base_string("255", 10, 64, &standard_settings).is_err());

        // Base 1 (unary) tests with Standard alphabet
        assert_string_ok_eq_no_warnings(
            "00000",
            base_to_base_string("5", 10, 1, &BaseStringFormatSettings::default()),
        );
        assert_string_ok_eq_no_warnings(
            "5",
            base_to_base_string("00000", 1, 10, &BaseStringFormatSettings::default()),
        );

        // Base 1 (unary) tests with Base64Standard alphabet
        let dec_to_un_b64 = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Standard,
            output_alphabet: BaseAlphabet::Base64Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "AAAAA",
            base_to_base_string("5", 10, 1, &dec_to_un_b64),
        );
        let un_b64_to_dec = BaseStringFormatSettings {
            input_alphabet: BaseAlphabet::Base64Standard,
            output_alphabet: BaseAlphabet::Standard,
            ..Default::default()
        };
        assert_string_ok_eq_no_warnings(
            "5",
            base_to_base_string("AAAAA", 1, 10, &un_b64_to_dec),
        );
    }
}
