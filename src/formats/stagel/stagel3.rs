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

//! StageL version 3 normalizer and source tools.
//!
//! StageL 3 is the core language, to be implemented directly in WAT. StageR 3
//! builds on it with `i128`, `u128`, `int`, `rat`, `dc`, `view`, `cursor`, `s`,
//! `dcs`, `option`, and `result`. Only EM3, the full VSO language built on
//! StageR 3, adds first-class functions and escaping closures. Named nested
//! functions and direct calls do not require first-class function values.
//!
//! All three languages are expression-oriented: every normally completing
//! expression returns a value, including unit for `void`. A body returns its
//! final expression's value; `never` means no normal return. Tail calls,
//! including mutual tail calls, must not grow the call stack. Nested functions
//! capture shared lexical bindings at their definition site, not their call
//! site. Only EM3 permits captured bindings to escape in function values.
//!
//! These are design requirements. This module only normalizes source bytes;
//! it does not yet implement parsing, type checking, execution, or WAT lowering.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Normalizes StageL v3 source bytes into the 32-character StageL v3
/// intermediate character set (capital `A-Z`, `/`, `:`, space, newline, `}`,
/// and `.`).
///
/// Converts lowercase ASCII characters to uppercase, normalizes line endings,
/// normalizes `;` to `.`, identifies comments starting with `: ` after optional
/// indentation, escapes disallowed characters within comments as
/// `:CHAR:<shifted_dec>:`, encodes digits as `N:<char>:` (shifted by +17), and
/// returns an error for any disallowed byte outside of comments.
pub fn normalize(input: &[u8]) -> Result<Vec<u8>> {
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

        // Semicolon to dot
        if c == b';' {
            c = b'.';
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

        // 10: lf; 32: space; 46: .; 47: /; 58: :; 125: }; 48..=57: 0-9; 65..=90: A-Z
        let is_allowed = c == b'\n'
            || c == b' '
            || c == b'.'
            || c == b'/'
            || c == b':'
            || c == b'}'
            || c.is_ascii_digit()
            || c.is_ascii_uppercase();

        if !is_allowed {
            if comment {
                output.extend_from_slice(b":CHAR:");
                for digit_byte in c.to_string().bytes() {
                    /* Escape numbers - this gets the character set for
                     * normalized StageL v3 to 32 characters (A-Z, /, :, space,
                     * newline, }, .) allowing it to be encoded with 5 bits per
                     * character, if that's any use. */
                    let shifted = digit_byte
                        .checked_add(17)
                        .context("Overflow shifting decimal digit")?;
                    output.push(shifted);
                }
                output.push(b':');
            } else {
                bail!("Disallowed byte in StageL v3 source: {c}");
            }
        } else if c.is_ascii_digit() {
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
    use super::*;
    use crate::get_stagel_data;

    #[crate::ctb_test]
    #[ignore = "WIP StageL v3 normalization"]
    fn test_normalize_sl3_fixtures() {
        let input = get_stagel_data("fixtures/sl3/input.sl3")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sl3/input.sl3"));
        let expected = get_stagel_data("fixtures/sl3/output.sli3")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sl3/output.sli3")
            });

        let normalized = normalize(&input).unwrap();
        assert_eq!(normalized, expected);
    }
}
