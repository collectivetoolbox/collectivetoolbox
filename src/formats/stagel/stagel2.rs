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

//! StageL version 2 normalizer and source tools.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NormalizerDialect {
    New,
    Old,
}

/// Normalizes StageL v2 source bytes according to the StageL v2 JavaScript
/// implementation.
///
/// Ported from `old/eite-older/sl2/stagel.js`.
///
/// Converts lowercase ASCII characters to uppercase, normalizes line endings,
/// identifies comments starting with `: ` after optional indentation, escapes
/// non-alphanumeric/non-symbol characters within comments as
/// `:CHAR:<shifted_dec>:`, encodes digits outside of comments as `N:<char>:`
/// (shifted by +17 to reduce character set to 32 characters), and returns an
/// error for any disallowed byte outside of comments.
pub fn normalize(input: &[u8]) -> Result<Vec<u8>> {
    normalize_with_dialect(input, NormalizerDialect::New)
}

/// Normalizes StageL v2 source bytes according to the C implementation of the
/// StageL v2 normalization rules.
///
/// Ported from `old/eite-older/sl2/stagel-normalize.c`.
///
/// Converts lowercase ASCII characters to uppercase, normalizes line endings,
/// identifies comments starting with `: ` after optional indentation, escapes
/// non-alphanumeric/non-symbol characters within comments as `:CHAR:<dec>:`,
/// and returns an error for any disallowed byte outside of comments.
pub fn normalize_old(input: &[u8]) -> Result<Vec<u8>> {
    normalize_with_dialect(input, NormalizerDialect::Old)
}

fn normalize_with_dialect(
    input: &[u8],
    dialect: NormalizerDialect,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut c: u8 = 0;
    let mut prev: u8;
    let mut potential_comment: u8 = 0;
    let mut comment = false;

    for &raw_byte in input {
        prev = c;
        c = raw_byte;

        // Uppercase each letter
        if c.is_ascii_lowercase() {
            c = c.to_ascii_uppercase();
        }

        // CRLF: skip the LF
        if c == b'\n' && prev == b'\r' {
            continue;
        }

        // CR to LF
        if c == b'\r' {
            c = b'\n';
        }

        if c == b'\n' {
            potential_comment = 1;
        }

        if potential_comment > 0 {
            if c != b'\n' && c != b' ' && c != b':' {
                potential_comment = 0;
            }
            if potential_comment == 1 && c == b':' {
                potential_comment = 2;
            }
            if potential_comment == 2 && c == b' ' {
                comment = true;
                potential_comment = 0;
            }
        }

        if comment && c == b'\n' {
            comment = false;
        }

        // 10: lf; 32: space; 47: /; 58: :; 48..=57: 0-9; 65..=90: A-Z
        let is_allowed = c == b'\n'
            || c == b' '
            || c == b'/'
            || c == b':'
            || c.is_ascii_digit()
            || c.is_ascii_uppercase();

        if !is_allowed {
            if comment {
                match dialect {
                    NormalizerDialect::New => {
                        output.extend_from_slice(b":CHAR:");
                        for digit_byte in c.to_string().bytes() {
                            /* Escape numbers - this should get the character
                             * set for normalized StageL to 32 characters
                             * (A-Z, /, :, space, newline) allowing it to be
                             * encoded with 5 bits per character, if that's any
                             * use. */
                            let shifted = digit_byte
                                .checked_add(17)
                                .context("Overflow shifting decimal digit")?;
                            output.push(shifted);
                        }
                        output.push(b':');
                    }
                    NormalizerDialect::Old => {
                        write!(&mut output, ":CHAR:{c}:")?;
                    }
                }
            } else {
                bail!("Disallowed byte in StageL v2 source: {c}");
            }
        } else if c.is_ascii_digit() && dialect == NormalizerDialect::New
        {
            let shifted = c
                .checked_add(17)
                .context("Overflow shifting digit character")?;
            output.extend_from_slice(&[b'N', b':', shifted, b':']);
        } else {
            output.push(c);
        }
    }

    Ok(output)
}

#[cfg(test)]
#[expect(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use crate::get_stagel_data;

    #[crate::ctb_test]
    fn test_normalize_sl2_js_fixtures() {
        let input = get_stagel_data("fixtures/sl2/input.sl")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sl2/input.sl"));
        let expected = get_stagel_data("fixtures/sl2/intermediate-js.sli")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sl2/intermediate-js.sli")
            });

        let normalized = normalize(&input).unwrap();
        assert_eq!(normalized, expected);
    }

    #[crate::ctb_test]
    fn test_normalize_sl2_c_fixtures() {
        let input = get_stagel_data("fixtures/sl2/input.sl")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sl2/input.sl"));
        let expected = get_stagel_data("fixtures/sl2/intermediate-c.sli")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sl2/intermediate-c.sli")
            });

        let normalized = normalize_old(&input).unwrap();
        assert_eq!(normalized, expected);
    }
}