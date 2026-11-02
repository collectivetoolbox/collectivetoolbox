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

//! Corrected-converter comparisons and independent conformance regressions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[crate::ctb_test]
fn test_libdce_compatibility() {
    let mut failures = Vec::new();
    crate::dceutils_tests::check_libdce_compatibility(
        crate::dce_convert_new,
        |name, result, expected| {
            let expected = match name {
                "UTF-8 to Dc, messy"
                | "Base64 UTF-8 to Dc, messy"
                | "UTF-32 to Dc, messy" => &b"114,57,86,93,93,96,191,183,167,129,147,192,18,191,183,167,129,146,192,72,96,99,93,85,19,115"[..],
                "UTF-8 to Dc, non-BMP"
                | "Base64 UTF-8 to Dc, non-BMP"
                | "UTF-32 to Dc, non-BMP" => &b"114,191,187,136,189,139,160,127,195,195,192,18,57,86,93,93,96,18,72,96,99,93,85,19,18,191,187,136,189,139,160,127,195,195,192,115"[..],
                "Improper CDCE string to Dc" if expected.ends_with(b"CDCE decoding error!") => b"CDCE decoding error: invalid escape sequence at codepoint index 6",
                _ => expected,
            };
            let actual = match result {
                Ok(bytes) => bytes,
                Err(error) => error.to_string().into_bytes(),
            };
            if actual != expected {
                failures.push(format!(
                    "{name}: expected {:?}, got {:?}",
                    String::from_utf8_lossy(expected),
                    String::from_utf8_lossy(&actual),
                ));
            }
        },
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[crate::ctb_test]
fn test_unicode_preservation() {
    for text in [
        "Hello\u{2014} \u{2013}World!",
        "\u{1F304} Hello World! \u{1F304}",
    ] {
        for format in ["dc", "dce", "3_01a", "cdce"] {
            let (encoded, log) = crate::dce_convert_new_with_log(
                text.as_bytes(),
                "utf8",
                format,
            )
            .unwrap();
            assert!(log.has_no_errors());
            let (decoded, log) =
                crate::dce_convert_new_with_log(&encoded, format, "utf8")
                    .unwrap();
            assert!(log.has_no_errors());
            assert_eq!(decoded, text.as_bytes(), "{format}");
        }
    }
}

#[crate::ctb_test]
fn test_spreadsheet_byte_tables() {
    let specifications = [
        (
            "3_0a_raw",
            include_str!(
                "docs/converted/DCE, version 3.0a r2 (26 December 2012).md"
            ),
        ),
        (
            "3_01a_raw",
            include_str!(
                "docs/converted/DCE, version 3.01a r2 (28 December 2012).md"
            ),
        ),
    ];
    let mut failures = Vec::new();
    for (format, specification) in specifications {
        for (sheet, selector) in [
            ("Core", None),
            ("DCE Version Selectors", Some(0x01)),
            ("Variant Selectors", Some(0xC5)),
            ("Semantic Records", Some(0xC6)),
            ("Mathematics", Some(0xC7)),
            ("Punctuation and Whitespace", Some(0xC8)),
        ] {
            let heading = format!("## {sheet}\n\n```csv\n");
            let Some((_, rest)) = specification.split_once(&heading) else {
                assert_eq!(format, "3_0a_raw");
                continue;
            };
            let (table, _) = rest.split_once("```").unwrap();
            let mut reader = csv::ReaderBuilder::new()
                .flexible(true)
                .from_reader(table.as_bytes());
            for record in reader.records() {
                let record = record.unwrap();
                let byte =
                    u8::from_str_radix(record.get(0).unwrap(), 16).unwrap();
                let expected = match (format, sheet, byte) {
                    (_, "Core", 0xC5..=0xC8) if format != "3_0a_raw" => {
                        Some(u32::from(byte).saturating_sub(1))
                    }
                    _ => record
                        .get(1)
                        .and_then(|value| value.parse::<u32>().ok()),
                };
                let mut bytes = vec![0x80];
                if let Some(selector) = selector {
                    bytes.push(selector);
                }
                bytes.extend_from_slice(&[byte, 0xFD, 0x81]);
                let mut log = crate::FormatLog::default();
                let decoded = crate::decode_dce_preserving_controls(
                    &bytes, format, &mut log,
                )
                .unwrap();
                let offset = if selector.is_some() { 2 } else { 1 };
                if let Some(expected) = expected {
                    if decoded.get(offset) != Some(&expected) {
                        failures.push(format!("{format} {sheet}:{byte:02X}: expected {expected}, got {decoded:?}"));
                    }
                } else if !log.has_warnings() {
                    failures.push(format!(
                        "{format} {sheet}:{byte:02X}: reserved byte accepted"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[crate::ctb_test]
fn test_empty_text_and_literal_cdce_roundtrips() {
    for text in ["", "@1@", "@123@", "@@", "a-b", "\u{85}"] {
        for format in [
            "cdce",
            "utf32",
            "utf8_dc64",
            "utf8_dc64_enc",
            "utf8_dc64_bin_enc",
        ] {
            let encoded =
                crate::dce_convert_new(text.as_bytes(), "utf8", format)
                    .unwrap();
            let decoded =
                crate::dce_convert_new(&encoded, format, "utf8").unwrap();
            assert_eq!(decoded, text.as_bytes(), "{text:?} via {format}");
        }
    }
}

#[crate::ctb_test]
fn test_malformed_encapsulated_utf8_is_rejected() {
    for data in [
        "191,192",
        "191,127,192",
        "191,190,190,195,195,192",
        "191,127,127",
        "191,50,192",
    ] {
        if data == "191,192" {
            assert_eq!(
                crate::dce_convert_new(data.as_bytes(), "dc", "utf8").unwrap(),
                b""
            );
        } else {
            assert!(
                crate::dce_convert_new(data.as_bytes(), "dc", "utf8").is_err(),
                "{data}"
            );
        }
    }
}

#[crate::ctb_test]
fn test_dce_envelope_validation() {
    for (format, version) in [("3_0a", 1), ("3_01a", 2)] {
        let valid = [
            b'D', b'C', b'E', b'e', 2, 1, version, 0xFD, 0x80, b'A', 0x81,
            0xFD, 3,
        ];
        for length in 7..valid.len() {
            let result = crate::dce_convert_new(&valid[..length], format, "dc");
            assert!(
                result.is_err(),
                "{format}: accepted prefix length {length}"
            );
        }
        let mut bad_reset = valid;
        bad_reset[7] = 0;
        assert!(crate::dce_convert_new(&bad_reset, format, "dc").is_err());
        let mut trailing = valid.to_vec();
        trailing.push(b'A');
        assert!(crate::dce_convert_new(&trailing, format, "dc").is_err());
    }
}

#[crate::ctb_test]
fn test_mixed_sheet_sequences() {
    for format in ["3_01a", "hex_3_01a_raw"] {
        for values in [
            "114,2,3,12,119,122,123,50,0,17,115",
            "114,122,0,123,2,0,3,12,0,119,50,115",
            "114,127,128,135,161,190,195,193,116,117,118,120,121,115",
        ] {
            let encoded =
                crate::dce_convert_new(values.as_bytes(), "dc", format)
                    .unwrap();
            let decoded =
                crate::dce_convert_new(&encoded, format, "dc").unwrap();
            assert_eq!(decoded, values.as_bytes(), "{format}");
        }
    }
}

#[crate::ctb_test]
fn test_invalid_input_returns_errors() {
    for (data, input, output) in [
        (&b"invalid"[..], "not_a_format", "not_a_format"),
        (&b"invalid"[..], "dce", "dce"),
        (&b"invalid"[..], "utf8_dc64", "utf8_dc64"),
        (&b"50,nope,51"[..], "dc", "utf8"),
        (&b"50,nope,51"[..], "dc", "dc"),
        (&b"50,4294967296,51"[..], "dc", "3_01a"),
        (&b"@1broken"[..], "cdce_lstrict", "dc"),
        (&b"\x80A"[..], "3_01a_raw", "dc"),
    ] {
        assert!(
            crate::dce_convert_new(data, input, output).is_err(),
            "{input} -> {output}: {data:?}"
        );
    }
    assert_eq!(crate::dce_convert_new(b"12", "dc", "utf8").unwrap(), b".");
}
