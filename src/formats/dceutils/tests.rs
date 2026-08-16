// Translated from PHP by test_converter.py.
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[expect(clippy::uninlined_format_args, reason = "Assertion helper formatting")]
fn run_test(name: &str, result: Result<Vec<u8>>, expected: &[u8]) {
    match result {
        Ok(bytes) => {
            assert_eq!(
                bytes,
                expected,
                "Test '{}' failed: expected {:?}, got {:?}",
                name,
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&bytes)
            );
        }
        Err(e) => {
            let err_str = e.to_string();
            assert_eq!(
                err_str.as_bytes(),
                expected,
                "Test '{}' failed: expected error {:?}, got {:?}",
                name,
                String::from_utf8_lossy(expected),
                err_str
            );
        }
    }
}

#[expect(
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    reason = "Assertion helper formatting"
)]
fn run_test_ver(name: &str, result: String, expected: &str) {
    assert_eq!(
        result, expected,
        "Test '{}' failed: expected '{}', got '{}'",
        name, expected, result
    );
}

#[crate::ctb_test]
fn test_libdce_compatibility() {
    // --- Begin test results ---
    // --- Begin input translator tests ---
    // --- Begin CDCE tests ---
    // --- cdce: This translator has not been implemented. ---
    // --- legacy_cdce ---
    // --- Input: ---
    run_test(
        "Plain UTF-8 string to Dc",
        crate::dce_convert(&b"Hello World!"[..], "legacy_cdce", "dc"),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Plain CDCE string to Dc",
        crate::dce_convert(&b"Hello @1@World@8@!"[..], "legacy_cdce", "dc"),
        &b"114,57,86,93,93,96,18,1,72,96,99,93,85,8,19,115"[..],
    );
    run_test(
        "Improper CDCE string to Dc",
        crate::dce_convert(&b"Hello @1World@13@@8@!"[..], "legacy_cdce", "dc"),
        &b"114,57,86,93,93,96,18,1,35,72,96,99,93,85,1,35,37,1,8,19,115"[..],
    );
    // --- cdce_lstrict ---
    // --- Input: ---
    run_test(
        "Plain UTF-8 string to Dc",
        crate::dce_convert(&b"Hello World!"[..], "cdce_lstrict", "dc"),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Plain CDCE string to Dc",
        crate::dce_convert(&b"Hello @1@World@8@!"[..], "cdce_lstrict", "dc"),
        &b"114,57,86,93,93,96,18,1,72,96,99,93,85,8,19,115"[..],
    );
    run_test(
        "Improper CDCE string to Dc",
        crate::dce_convert(&b"Hello @1World@13@@8@!"[..], "cdce_lstrict", "dc"),
        "114,57,86,93,93,96,18… CDCE decoding error!".as_bytes(),
    );
    // --- Output: ---
    run_test(
        "Dc list to Legacy CDCE",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "legacy_cdce",
        ),
        &b"Hello World!"[..],
    );
    run_test(
        "Messy Dc list to Legacy CDCE",
        crate::dce_convert(
            &b"114,1,57,86,93,93,96,18,72,96,99,93,85,19,9,115"[..],
            "dc",
            "legacy_cdce",
        ),
        &b"@1@Hello World!@9@"[..],
    );
    // --- Begin DCE tests ---
    // --- dce ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "dce",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Simple DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "dce",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Hex DCE to Dc",
        crate::dce_convert(
            &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..],
            "dce",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to DCE",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "dce"),
        &ctb_formats_hexdump::hex2bin(
            "44434565020101FD8048656C6C6F20576F726C642181FD03",
        )
        .unwrap(),
    );
    // --- hex_dce ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "hex_dce",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Hex DCE 3.0a to Dc",
        crate::dce_convert(
            &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_dce",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Mixed-case hex DCE 3.0a to Dc",
        crate::dce_convert(
            &b"44434565020101fd8048656c6C6F20576F726C642181FD03"[..],
            "hex_dce",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Simple Hex DCE 3.01a to Dc",
        crate::dce_convert(
            &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_dce",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to Hex DCE",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "hex_dce"),
        &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
    );
    // --- Begin DCE 3.0a tests ---
    // --- 3_0a ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_0a",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Simple DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_0a",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to DCE 3.0a",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "3_0a"),
        &ctb_formats_hexdump::hex2bin(
            "44434565020101FD8048656C6C6F20576F726C642181FD03",
        )
        .unwrap(),
    );
    // --- 3_0a_raw ---
    // --- Input: ---
    run_test(
        "DCE 3.0a Raw to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin("8048656C6C6F20576F726C642181")
                .unwrap(),
            "3_0a_raw",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_0a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    // --- Output: ---
    run_test(
        "Dc to DCE 3.0a Raw",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "3_0a_raw",
        ),
        &ctb_formats_hexdump::hex2bin("8048656C6C6F20576F726C642181").unwrap(),
    );
    // --- hex_3_0a ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "hex_3_0a",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Hex DCE 3.0a to Dc",
        crate::dce_convert(
            &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_0a",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Simple Hex DCE 3.01a to Dc",
        crate::dce_convert(
            &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_0a",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to Hex DCE 3.0a",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "hex_3_0a"),
        &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
    );
    // --- hex_3_0a_raw ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "hex_3_0a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Hex DCE 3.0a to Dc",
        crate::dce_convert(
            &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_0a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Hex DCE 3.0a Raw to Dc",
        crate::dce_convert(
            &b"8048656C6C6F20576F726C642181"[..],
            "hex_3_0a_raw",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "Dc to Hex DCE 3.0a Raw",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "hex_3_0a_raw",
        ),
        &b"8048656C6C6F20576F726C642181"[..],
    );
    // --- Begin DCE 3.01a tests ---
    // --- 3_01a ---
    // --- Input: ---
    run_test(
        "DCE 3.0a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_01a",
            "dc",
        ),
        &b"This document is not stored using the specified version of DCE."[..],
    );
    run_test(
        "Simple DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_01a",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Complex DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD80C501FE48656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_01a",
            "dc",
        ),
        &b"114,122,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    // --- 3_01a: This translator currently does not work well. ---
    //     run_test("UTF-8 to DCE 3.01a", crate::dce_convert(&b"Hello World!"[..], "utf8", "3_01a"), &ctb_formats_hexdump::hex2bin("44434565020102FD8048656C6C6F20576F726C642181FD03").unwrap());
    // --- 3_01a_raw ---
    // --- Input: ---
    run_test(
        "Simple DCE 3.01a Raw to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin("8048656C6C6F20576F726C642181")
                .unwrap(),
            "3_01a_raw",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Complex DCE 3.01a Raw to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin("80C501FE48656C6C6F20576F726C642181")
                .unwrap(),
            "3_01a_raw",
            "dc",
        ),
        &b"114,122,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "3_01a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    // --- hex_3_01a ---
    // --- Input: ---
    run_test(
        "Simple DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "hex_3_01a",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Hex DCE 3.0a to Dc",
        crate::dce_convert(
            &b"44434565020101FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_01a",
            "dc",
        ),
        &b"This document is not stored using the specified version of DCE."[..],
    );
    run_test(
        "Simple Hex DCE 3.01a to Dc",
        crate::dce_convert(
            &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_01a",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Complex Hex DCE 3.01a to Dc",
        crate::dce_convert(
            &b"44434565020102FD80C501FE48656C6C6F20576F726C642181FD03"[..],
            "hex_3_01a",
            "dc",
        ),
        &b"114,122,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    // --- 3_01a: This translator currently does not work well. ---
    //     run_test("UTF-8 to Hex DCE 3.01a", crate::dce_convert(&b"Hello World!"[..], "utf8", "hex_3_01a"), &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..]);
    // --- hex_3_01a_raw ---
    // --- Input: ---
    run_test(
        "Simple DCE 3.01a to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
            "hex_3_01a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Simple DCE 3.01a Raw to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin("8048656C6C6F20576F726C642181")
                .unwrap(),
            "hex_3_01a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Simple Hex DCE 3.01a to Dc",
        crate::dce_convert(
            &b"44434565020102FD8048656C6C6F20576F726C642181FD03"[..],
            "hex_3_01a_raw",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "Simple Hex DCE 3.01a Raw to Dc",
        crate::dce_convert(
            &b"8048656C6C6F20576F726C642181"[..],
            "hex_3_01a_raw",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Complex Hex DCE 3.01a Raw to Dc",
        crate::dce_convert(
            &b"80C501FE48656C6C6F20576F726C642181"[..],
            "hex_3_01a_raw",
            "dc",
        ),
        &b"114,122,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Begin Dc tests ---
    // --- dc ---
    // --- Input: ---
    run_test(
        "Dc to Dc",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "Dc to Dc, simple",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,1,72,96,99,93,85,8,19,115"[..],
            "dc",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,1,72,96,99,93,85,8,19,115"[..],
    );
    run_test(
        "Dc to Dc, source missing boundedness markings",
        crate::dce_convert(
            &b"57,86,93,93,96,18,1,72,96,99,93,85,8,19"[..],
            "dc",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,1,72,96,99,93,85,8,19,115"[..],
    );
    // --- Begin Unicode tests ---
    // --- utf8 ---
    // --- Input: ---
    run_test(
        "UTF-8 to Dc, simple",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "dc"),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "UTF-8 to Dc, messy",
        crate::dce_convert("Hello— –World!".as_bytes(), "utf8", "dc"),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "UTF-8 to Dc, non-BMP",
        crate::dce_convert("🌄 Hello World! 🌄".as_bytes(), "utf8", "dc"),
        &b"114,18,57,86,93,93,96,18,72,96,99,93,85,19,18,115"[..],
    );
    // --- Output: ---
    run_test(
        "Dc to UTF-8, simple",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "utf8",
        ),
        &b"Hello World!"[..],
    );
    run_test(
        "Dc to UTF-8, messy",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,8,19,115"[..],
            "dc",
            "utf8",
        ),
        "Hello World�!".as_bytes(),
    );
    // --- utf8_base64 ---
    // --- Input: ---
    run_test(
        "Base64 UTF-8 to Dc, simple",
        crate::dce_convert(
            &ctb_formats_base64::base64_encode(b"Hello World!").into_bytes(),
            "utf8_base64",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Base64 UTF-8 to Dc, messy",
        crate::dce_convert(
            &ctb_formats_base64::base64_encode("Hello— –World!".as_bytes())
                .into_bytes(),
            "utf8_base64",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Base64 UTF-8 to Dc, non-BMP",
        crate::dce_convert(
            &ctb_formats_base64::base64_encode("🌄 Hello World! 🌄".as_bytes())
                .into_bytes(),
            "utf8_base64",
            "dc",
        ),
        &b"114,18,57,86,93,93,96,18,72,96,99,93,85,19,18,115"[..],
    );
    // --- Output: ---
    run_test(
        "Dc to Base64 UTF-8, simple",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "utf8_base64",
        ),
        &b"SGVsbG8gV29ybGQh"[..],
    );
    run_test(
        "Dc to Base64 UTF-8, messy",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,8,19,115"[..],
            "dc",
            "utf8_base64",
        ),
        &b"SGVsbG8gV29ybGTvv70h"[..],
    );
    // --- utf8_dc64 ---
    // --- Input: ---
    run_test(
        "Raw Base64 Dc list encapsulated Unicode to Dc",
        crate::dce_convert(
            &b"145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160"
                [..],
            "utf8_dc64",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "Dc to Dc",
        crate::dce_convert(
            &b"114,18,57,86,93,93,96,18,72,96,99,93,85,19,18,115"[..],
            "utf8_dc64",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test("Base64 Dc list encapsulated Unicode to Dc", crate::dce_convert(&b"191,145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160,192"[..], "utf8_dc64", "dc"), &b"This document is not stored using the specified format."[..]);
    // --- Output: ---
    run_test(
        "UTF-8 to raw UTF-8 Base64 Dc list",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "utf8_dc64"),
        &b"145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160"[..],
    );
    // --- utf8_dc64_enc ---
    // --- Input: ---
    run_test(
        "Raw Base64 Dc list encapsulated Unicode to Dc",
        crate::dce_convert(
            &b"145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160"
                [..],
            "utf8_dc64_enc",
            "dc",
        ),
        &b"This document is not stored using the specified format."[..],
    );
    run_test("Base64 Dc list encapsulated Unicode to Dc", crate::dce_convert(&b"191,145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160,192"[..], "utf8_dc64_enc", "dc"), &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..]);
    // --- Output: ---
    run_test("UTF-8 to UTF-8 Base64 Dc list", crate::dce_convert(&b"Hello World!"[..], "utf8", "utf8_dc64_enc"), &b"191,145,133,148,171,154,133,187,159,148,181,188,177,154,133,143,160,192"[..]);
    // --- utf8_dc64_bin ---
    // --- Input: ---
    run_test(
        "Raw UTF-8 Base64 DCE binary fragment to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin("948897AE9D88BEA297B8BFB49D8892A3")
                .unwrap(),
            "utf8_dc64_bin",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to raw UTF-8 Base64 DCE binary fragment",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "utf8_dc64_bin"),
        &ctb_formats_hexdump::hex2bin("948897AE9D88BEA297B8BFB49D8892A3")
            .unwrap(),
    );
    // --- utf8_dc64_bin_hex ---
    // --- Input: ---
    run_test(
        "Raw UTF-8 Base64 DCE hex fragment to Dc",
        crate::dce_convert(
            &b"948897AE9D88BEA297B8BFB49D8892A3"[..],
            "utf8_dc64_bin_hex",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to raw UTF-8 Base64 DCE hex fragment",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "utf8_dc64_bin_hex"),
        &b"948897AE9D88BEA297B8BFB49D8892A3"[..],
    );
    // --- utf8_dc64_bin_enc ---
    // --- Input: ---
    run_test(
        "Encapsulated UTF-8 Base64 DCE binary fragment to Dc",
        crate::dce_convert(
            &ctb_formats_hexdump::hex2bin(
                "C3948897AE9D88BEA297B8BFB49D8892A3C4",
            )
            .unwrap(),
            "utf8_dc64_bin_enc",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to UTF-8 Base64 DCE encapsulated binary",
        crate::dce_convert(&b"Hello World!"[..], "utf8", "utf8_dc64_bin_enc"),
        &ctb_formats_hexdump::hex2bin("C3948897AE9D88BEA297B8BFB49D8892A3C4")
            .unwrap(),
    );
    // --- utf8_dc64_bin_enc_hex ---
    // --- Input: ---
    run_test(
        "Encapsulated UTF-8 Base64 DCE hex fragment to Dc",
        crate::dce_convert(
            &b"C3948897AE9D88BEA297B8BFB49D8892A3C4"[..],
            "utf8_dc64_bin_enc_hex",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    // --- Output: ---
    run_test(
        "UTF-8 to UTF-8 Base64 DCE encapsulated hex",
        crate::dce_convert(
            &b"Hello World!"[..],
            "utf8",
            "utf8_dc64_bin_enc_hex",
        ),
        &b"C3948897AE9D88BEA297B8BFB49D8892A3C4"[..],
    );
    // --- utf32 ---
    // --- Input: ---
    run_test(
        "UTF-8 to Dc",
        crate::dce_convert(&b"Hello World!"[..], "utf32", "dc"),
        &b"This document is not stored using the specified format."[..],
    );
    run_test(
        "UTF-32 to Dc, simple",
        crate::dce_convert(
            &ctb_formats_encoding::unicode::utf8_to_utf32be(b"Hello World!")
                .unwrap(),
            "utf32",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "UTF-32 to Dc, messy",
        crate::dce_convert(
            &ctb_formats_encoding::unicode::utf8_to_utf32be(
                "Hello— –World!".as_bytes(),
            )
            .unwrap(),
            "utf32",
            "dc",
        ),
        &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
    );
    run_test(
        "UTF-32 to Dc, non-BMP",
        crate::dce_convert(
            &ctb_formats_encoding::unicode::utf8_to_utf32be(
                "🌄 Hello World! 🌄".as_bytes(),
            )
            .unwrap(),
            "utf32",
            "dc",
        ),
        &b"114,18,57,86,93,93,96,18,72,96,99,93,85,19,18,115"[..],
    );
    // --- Output: ---
    run_test(
        "Dc to UTF-32, simple",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "utf32",
        ),
        &ctb_formats_encoding::unicode::utf8_to_utf32be(b"Hello World!")
            .unwrap(),
    );
    run_test(
        "Dc to UTF-32, messy",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,8,19,115"[..],
            "dc",
            "utf32",
        ),
        &ctb_formats_encoding::unicode::utf8_to_utf32be(
            "Hello World�!".as_bytes(),
        )
        .unwrap(),
    );
    // --- Begin miscellaneous tests ---
    run_test(
        "Nonexistent input format",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "foo",
            "dc",
        ),
        &b"Unknown input format."[..],
    );
    run_test(
        "Nonexistent output format",
        crate::dce_convert(
            &b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"[..],
            "dc",
            "foo",
        ),
        &b"Unknown output format."[..],
    );
    run_test_ver(
        "Get DCE version: DCE 3.0a",
        crate::get_dce_version(
            &ctb_formats_hexdump::hex2bin(
                "44434565020101FD8048656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
        ),
        "3_0a",
    );
    run_test_ver(
        "Get DCE version: DCE 3.01a, complex",
        crate::get_dce_version(
            &ctb_formats_hexdump::hex2bin(
                "44434565020102FD80C501FE48656C6C6F20576F726C642181FD03",
            )
            .unwrap(),
        ),
        "3_01a",
    );
    run_test_ver(
        "Get DCE version: Not DCE",
        crate::get_dce_version(&b"DOOM"[..]),
        "This document does not appear to be stored using DCE.",
    );
    //     run_test("Raw Base64 Dc list encapsulated Unicode to Base64 Dc list encapsulated Unicode", crate::dce_convert(&b"156,133,148,178,156,127,195,195"[..], "utf8_dc64", "utf8_dc64_enc"), &b"114,191,156,133,148,178,156,127,195,195,192,115"[..]);
    // --- SOME TESTS FAILED! LIBDCE DOES NOT APPEAR TO BE WORKING PROPERLY! ---
    // --- End test results ---
}

#[crate::ctb_test]
fn test_legacy_conversions() {
    // Test onestep_3_0a_old_to_none (DCE 3.0a decoding)
    let dce_3_0a = ctb_formats_hexdump::hex2bin(
        "44434565020101FD8048656C6C6F20576F726C642181FD03",
    )
    .unwrap();
    let res = crate::legacy::onestep_3_0a_old_to_none(&dce_3_0a).unwrap();
    assert_eq!(res, b"Hello World!");

    // Test onestep_dce2txt_to_none
    let res2 = crate::legacy::onestep_dce2txt_to_none(&dce_3_0a).unwrap();
    assert_eq!(res2, b"Hello World!");

    // Test hex2dce (standard hex decode)
    let hex_val = b"48656c6c6f";
    let res3 = crate::legacy::hex2dce(hex_val);
    assert_eq!(res3, b"Hello");

    // Test dce2hex (low nibble first hex decode)
    let dce_hex_val = b"8456c6c6f6"; // 'H'=0x48 -> '84', 'e'=0x65 -> '56', etc.
    let res4 = crate::legacy::dce2hex(dce_hex_val);
    assert_eq!(res4, b"Hello");

    // Test legacy_cdce_to_html_snippet
    let cdce_data = b"Hello @1@World@8@!";
    let res5 = crate::legacy::onestep_legacy_cdce_to_html_snippet_l(cdce_data)
        .unwrap();
    assert_eq!(res5, b"Hello @World<font color=\"red\">#</font>!");

    // Test legacy_cdce_to_html_l
    let res6 = crate::legacy::onestep_legacy_cdce_to_html_l(cdce_data).unwrap();
    assert!(res6.starts_with(b"<html>"));
    assert!(res6.ends_with(b"</html>"));
    let res6_str = String::from_utf8_lossy(&res6);
    assert!(res6_str.contains("<font color=\"red\">#</font>"));
}

#[crate::ctb_test]
fn test_dceutils_format_log_error_detection() {
    // 1. Test convert_dc_to_dc_output logging on non-digit payload
    let mut log = crate::FormatLog::default();
    let dc_out = crate::convert_dc_to_dc_output("114,57,86,93,93,96,18… CDCE decoding error!", &mut log);
    assert_eq!(dc_out, "114,57,86,93,93,96,18… CDCE decoding error!");
    assert!(log.has_errors());
    assert!(log.get_errors().iter().any(|e| e.contains("Non-digit character")));

    // 2. Test dce_convert_with_log on strict CDCE error
    let (res, log2) = crate::dce_convert_with_log(
        b"Hello @1World@13@@8@!",
        "cdce_lstrict",
        "dc",
    )
    .unwrap();
    assert_eq!(
        res,
        b"114,57,86,93,93,96,18\xe2\x80\xa6 CDCE decoding error!"
    );
    assert!(log2.has_errors());

    // 3. Test dce_convert_with_log on invalid format header
    let err_result = crate::dce_convert_with_log(
        b"not a dce document",
        "dce",
        "dc",
    );
    assert!(err_result.is_err());

    // 4. Test dce_convert_with_log on valid conversion
    let (res3, log3) = crate::dce_convert_with_log(
        b"Hello World!",
        "utf8",
        "dc",
    )
    .unwrap();
    assert_eq!(
        res3,
        b"114,57,86,93,93,96,18,72,96,99,93,85,19,115"
    );
    assert!(log3.has_no_errors());
}
