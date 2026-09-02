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

/// Normalizes StageL v2 source bytes according to the StageL v2 normalization
/// rules.
///
/// Ported from `old/eite-older/sl2/stagel-normalize.c`.
///
/// Converts lowercase ASCII characters to uppercase, normalizes line endings,
/// identifies comments starting with `: ` after optional indentation, escapes
/// non-alphanumeric/non-symbol characters within comments as `:CHAR:<dec>:`,
/// and returns an error for any disallowed byte outside of comments.
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
                write!(&mut output, ":CHAR:{c}:")?;
            } else {
                bail!("Disallowed byte in StageL v2 source: {c}");
            }
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
    fn test_normalize_sl2_fixtures() {
        let input = get_stagel_data("fixtures/sl2/input.sl")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sl2/input.sl"));
        let expected = get_stagel_data("fixtures/sl2/intermediate.sli")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sl2/intermediate.sli")
            });

        let normalized = normalize(&input).unwrap();
        assert_eq!(normalized, expected);
    }
}