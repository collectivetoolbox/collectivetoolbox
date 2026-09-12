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

//! String parsing, CamelCase word splitting, and text formatting helpers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

/// Splits a CamelCase string into its constituent subwords.
#[must_use]
pub fn split_camel_case(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len == 0 {
        return Vec::new();
    }

    let mut words = Vec::new();
    let mut start = 0;
    for i in 1..len {
        let Some(&prev) = chars.get(i.saturating_sub(1)) else {
            continue;
        };
        let Some(&curr) = chars.get(i) else {
            continue;
        };
        let next = chars.get(i.saturating_add(1)).copied();

        let is_lower_to_upper = prev.is_lowercase() && curr.is_uppercase();
        let is_upper_to_upper_then_lower = prev.is_uppercase()
            && curr.is_uppercase()
            && next.is_some_and(|n| n.is_lowercase());
        let is_letter_to_digit = prev.is_alphabetic() && curr.is_ascii_digit();
        let is_digit_to_letter = prev.is_ascii_digit() && curr.is_alphabetic();

        if is_lower_to_upper
            || is_upper_to_upper_then_lower
            || is_letter_to_digit
            || is_digit_to_letter
        {
            if let Some(slice) = chars.get(start..i) {
                let word: String = slice.iter().collect();
                if !word.is_empty() {
                    words.push(word);
                }
            }
            start = i;
        }
    }
    if let Some(slice) = chars.get(start..len) {
        let last: String = slice.iter().collect();
        if !last.is_empty() {
            words.push(last);
        }
    }
    words
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

    #[crate::ctb_test]
    fn test_split_camel_case() {
        assert_eq!(split_camel_case(""), Vec::<String>::new());
        assert_eq!(split_camel_case("Foo"), vec!["Foo"]);
        assert_eq!(split_camel_case("FooBar"), vec!["Foo", "Bar"]);
        assert_eq!(
            split_camel_case("FooBarRegular"),
            vec!["Foo", "Bar", "Regular"]
        );
        assert_eq!(split_camel_case("XMLParser"), vec!["XML", "Parser"]);
        assert_eq!(split_camel_case("utf8String"), vec!["utf", "8", "String"]);
    }
}
