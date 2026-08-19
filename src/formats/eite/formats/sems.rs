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

//! SEMS format parser and exporter for EITE compatibility.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};
use const_default::ConstDefault;

use crate::formats::FormatLog;
use crate::formats::dcbasenb::dca_to_dcbnb_utf8;
use crate::formats::utf8::{UTF8FormatSettings, dca_from_utf8};
use crate::util::ascii::{ascii_is_digit, ascii_is_newline, crlf};

#[derive(ConstDefault, Default, Debug, Clone)]
pub struct SEMSFormatSettings {
    pub strict: bool,
}

fn get_index(res: &[u32]) -> u64 {
    // Reason for fallback: usize fits within u64 on 32-bit and 64-bit systems, so try_from conversion never overflows.
    u64::try_from(res.len()).unwrap_or(0)
}

/// Parse SEMS format (space-delimited integers with `#` comments).
/// Comments are bracketed with Dc 246 (begin single-line comment) and Dc 248 (end).
pub fn dca_from_sems(
    content: &[u8],
    settings: &SEMSFormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    // Strict mode?
    let bool_strict = settings.strict;

    let mut log: FormatLog = FormatLog::default();
    let mut res: Vec<u32> = Vec::new();
    let mut parser_state = "dc";
    let mut current_dc = String::new();

    let mut idx = 0usize;
    while let Some(&b) = content.get(idx) {
        match parser_state {
            "dc" => {
                if ascii_is_digit(b) {
                    current_dc.push(char::from(b));
                } else if b == b' ' || b == b'\n' || b == b'\r' {
                    if !current_dc.is_empty() {
                        // Error here would indicate a parser bug
                        let val = current_dc.parse::<u32>()?;
                        res.push(val);
                        current_dc.clear();
                    }
                } else if b == b'#' {
                    if !current_dc.is_empty() {
                        // Warning for missing trailing space before comment
                        if bool_strict {
                            log.import_error(get_index(&res), "No trailing space before comment present in sems format while importing. This is not allowed in strict mode.");
                            return Ok((res, log));
                        }
                        log.import_warning(get_index(&res), "No trailing space before comment present in sems format while importing.");
                        let val = current_dc.parse::<u32>()?;
                        res.push(val);
                        current_dc.clear();
                    }
                    // Push begin comment sentinel
                    res.push(246);
                    parser_state = "comment";
                } else {
                    log.import_error(
                        get_index(&res),
                        "Invalid character in sems format while importing.",
                    );
                    return Ok((res, log));
                }
            }
            "comment" => {
                // Append mapped Dc(s) from this unicode char byte
                // (The original code treated bytes as Unicode scalar values)
                let mut until_next_newline = Vec::new();
                let mut i = 0usize;
                while idx.saturating_add(i) < content.len() {
                    let offset = idx.saturating_add(i);
                    let b = content
                        .get(offset)
                        .copied()
                        .ok_or_else(|| anyhow!("Index out of bounds"))?;
                    if ascii_is_newline(b) {
                        parser_state = "dc";
                        // don't consume and advance past the newline here; give
                        // it back to the outer loop
                        break;
                    }
                    until_next_newline.push(b);
                    i = i.saturating_add(1);
                    if idx.saturating_add(i) == content.len() {
                        // End of content reached; will handle end-of-file below
                        break;
                    }
                }
                let (mapped, inner_log) = dca_from_utf8(
                    &until_next_newline,
                    &UTF8FormatSettings::default(),
                )?;
                res.extend(mapped);
                log.merge(&inner_log);
                // End comment sentinel
                res.push(248);
                idx = idx.saturating_add(i);
            }
            _ => bail!(
                "Internal error: unexpected parser state while parsing SEMS document"
            ),
        }

        idx = idx.saturating_add(1);
    }

    // End-of-file handling
    if parser_state == "comment" {
        if !current_dc.is_empty() {
            log.import_error(
                get_index(&res),
                format!(
                    "Internal error while parsing sems document: Unconsumed characters at end: {current_dc}."
                )
                .as_str(),
            );
            return Ok((res, log));
        }
    } else if !current_dc.is_empty() {
        if bool_strict {
            log.import_error(
                get_index(&res),
                "No trailing space present in sems format while importing. This is not allowed in strict mode.",
            );
            return Ok((res, log));
        }
        log.import_warning(
            get_index(&res),
            "No trailing space present in sems format while importing.",
        );
        // Error here would indicate a parser bug
        let val = current_dc.parse::<u32>()?;
        res.push(val);
    }

    Ok((res, log))
}

/// Export Dc array to SEMS format.
///
/// - Dc 246 begins a single-line comment region
/// - Dc 248 ends the comment region
///
/// Comment contents are encoded with `dca_to_dcbnb_utf8`.
pub fn dca_to_sems(dc_array: &[u32]) -> Result<(Vec<u8>, FormatLog)> {
    let mut log: FormatLog = FormatLog::default();
    let mut out: Vec<u8> = Vec::new();

    let mut in_comment = false;
    let mut at_comment_end = false;
    let mut current_comment: Vec<u32> = Vec::new();

    for &dc in dc_array {
        if at_comment_end {
            at_comment_end = false;
        }
        if dc == 246 {
            in_comment = true;
            out.push(b'#');
        } else if dc == 248 {
            in_comment = false;
            at_comment_end = true;
            let (enc, dcbnb_log) = dca_to_dcbnb_utf8(
                &current_comment,
                &UTF8FormatSettings::default(),
            )?;
            log.merge(&dcbnb_log);
            out.extend(enc);
            current_comment.clear();
            out.extend(crlf());
        } else if in_comment {
            current_comment.push(dc);
        } else {
            out.extend(dc.to_string().as_bytes());
            out.push(b' ');
        }
    }

    if !at_comment_end {
        out.extend(crlf());
    }

    Ok((out, log))
}

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
    use crate::utilities::assert_vec_u32_eq;
    use crate::{
        assert_vec_dc_ok_eq_no_errors, assert_vec_dc_ok_eq_no_warnings,
    };
    use ctb_formats_utilities::assert_vec_u8_ok_eq_no_warnings;

    use super::*;

    const SETTINGS: SEMSFormatSettings =
        <SEMSFormatSettings as ConstDefault>::DEFAULT;

    // Tests translated from original JS runTestsFormatSems / runTestsFormatAscii.
    //
    // Original JS snippet (for reference):
    // runTestsFormatSems(boolV) { ... }
    // runTestsFormatAscii(boolV) { ... }
    //
    // Notes:
    // - The original harness used runTest / arrEq / debug helpers and a boolV verbosity flag.
    //   Here we use standard Rust #[crate::ctb_test] functions with direct assertions.
    // - A commented (not yet implemented) failure case in the original JS for
    //   double-space separation (`49 32 32 50`) is preserved as a TODO.
    // - The "Currently does not output the 65 ..." FIXME from the source is kept
    //   as a comment; the expected value (including 65) is asserted to match the
    //   source test vector.
    //
    // Required public functions (expected to exist elsewhere in crate::):
    //   - dca_from_sems(content: &[u8]) -> Result<Vec<i32>>
    //   - dca_to_sems(dc_array: &[u32]) -> Result<Vec<u8>>   (assumed, based on JS test dcaToSems)
    //   - dca_from_ascii(content: &[u8]) -> Result<Vec<i32>>
    //   - dca_to_ascii(state: &EiteState, dc_array: &[u32]) -> Result<Vec<u8>>
    //
    // If any of these are not yet implemented, these tests will fail to compile;
    // implement them (or adjust module paths) accordingly.

    // ---------------------------
    // Sems format tests
    // ---------------------------

    #[crate::ctb_test]
    fn test_dca_from_sems_basic_no_trailing_space() {
        // Input bytes: '1', ' ', '2'
        let input = [49u8, 32, 50];
        let expected = [1, 2];
        let (actual, _log) =
            dca_from_sems(&input, &SEMSFormatSettings::default())
                .expect("dca_from_sems failed");
        assert_vec_u32_eq(&expected, &actual);
    }

    // Original JS had (commented out):
    // /* Should fail but I do not yet have a failure assertion:
    //    runTest b/v arrEq ( 1 2 ) dcaFromSems ( 49 32 32 50 ) */
    // Keeping as a TODO.
    //
    // #[crate::ctb_test]
    // fn test_dca_from_sems_double_space_should_fail() {
    //     let input = [49u8, 32, 32, 50];
    //     // Decide expected behavior once implementation defines error vs. leniency.
    // }

    #[crate::ctb_test]
    fn test_dca_to_sems_basic_trailing_space_crlf() {
        // Expected bytes: '1', ' ', '2', ' ', '\r', '\n'
        let input = [1, 2];
        let expected = [49u8, 32, 50, 32, 13, 10];
        assert_vec_u8_ok_eq_no_warnings(&expected, dca_to_sems(&input));
    }

    #[crate::ctb_test]
    fn test_dca_from_sems_comment_preservation() {
        // Input bytes: '1',' ','2','#','A'
        let input = [49u8, 32, 50, 35, 65];
        let expected = [1u32, 2, 246, 50, 248];
        let (_out, log) = assert_vec_dc_ok_eq_no_errors(
            &expected,
            dca_from_sems(&input, &SETTINGS),
        );
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_dca_to_sems_comment_preservation() {
        // Expected bytes: '1',' ','2',' ','#','A','\r','\n'
        // Original snippet note:
        // /* Currently does not output the 65 from the desired result (FIXME not implemented) */
        // The test still expects 65 ( 'A' ) per original arrEq.
        let input = [1, 2, 246, 50, 248];
        let expected = [49u8, 32, 50, 32, 35, 65, 13, 10];
        assert_vec_u8_ok_eq_no_warnings(&expected, dca_to_sems(&input));
    }

    #[crate::ctb_test]
    fn test_dca_from_sems_utf8_comments() {
        // Long UTF-8-with-comments input:
        // This is the following two 0x0A-delimited lines:
        // 256 258 260 262 264 263 57 86 93 93 96 30 18 286 72 96 99 93 85 287 19 18 284 261 259 # say "Hello, /World/! ⚽"
        // 1 2 # ⚽
        let input: Vec<u8> = vec![
            50, 53, 54, 32, 50, 53, 56, 32, 50, 54, 48, 32, 50, 54, 50, 32, 50,
            54, 52, 32, 50, 54, 51, 32, 53, 55, 32, 56, 54, 32, 57, 51, 32, 57,
            51, 32, 57, 54, 32, 51, 48, 32, 49, 56, 32, 50, 56, 54, 32, 55, 50,
            32, 57, 54, 32, 57, 57, 32, 57, 51, 32, 56, 53, 32, 50, 56, 55, 32,
            49, 57, 32, 49, 56, 32, 50, 56, 52, 32, 50, 54, 49, 32, 50, 53, 57,
            32, 35, 32, 115, 97, 121, 32, 34, 72, 101, 108, 108, 111, 44, 32,
            47, 87, 111, 114, 108, 100, 47, 33, 32, 226, 154, 189, 34, 10, 49,
            32, 50, 32, 35, 32, 226, 154, 189, 10,
        ];

        let expected: Vec<u32> = vec![
            256, 258, 260, 262, 264, 263, 57, 86, 93, 93, 96, 30, 18, 286, 72,
            96, 99, 93, 85, 287, 19, 18, 284, 261, 259, 246, 18, 100, 82, 106,
            18, 20, 57, 86, 93, 93, 96, 30, 18, 33, 72, 96, 99, 93, 85, 33, 19,
            18, 281, 20, 248, 1, 2, 246, 18, 281, 248,
        ];

        assert_vec_dc_ok_eq_no_warnings(
            &expected,
            dca_from_sems(&input, &SEMSFormatSettings::default()),
        );
    }

    #[crate::ctb_test]
    fn test_sems_comment_basic() {
        // "12 34 #comment\n56 "
        let input = b"12 34 #hello\n56 ";
        let (dca, _log) =
            dca_from_sems(input, &SEMSFormatSettings::default()).unwrap();
        // Should contain 12,34,246,<comment content...>,248,56
        assert!(dca.starts_with(&[12, 34, 246]));
        assert!(dca.contains(&248));
        assert!(dca.ends_with(&[56]));
    }
}
