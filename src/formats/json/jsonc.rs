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
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub fn strip_jsonc_comments(json: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut chars = json.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_line_comment {
            if c == '\n' || c == '\r' {
                in_single_line_comment = false;
                result.push(c);
            }
        } else if in_multi_line_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_multi_line_comment = false;
            }
        } else if in_string {
            result.push(c);
            if c == '"' {
                in_string = false;
            } else if c == '\\' {
                if let Some(&next_c) = chars.peek() {
                    result.push(next_c);
                    chars.next();
                }
            }
        } else if c == '"' {
            in_string = true;
            result.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            in_single_line_comment = true;
            chars.next();
        } else if c == '/' && chars.peek() == Some(&'*') {
            in_multi_line_comment = true;
            chars.next();
        } else {
            result.push(c);
        }
    }
    result
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
mod tests {}
