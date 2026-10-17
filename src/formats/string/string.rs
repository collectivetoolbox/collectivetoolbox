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

/// Formats arbitrary binary bytes into a human-readable, valid UTF-8 string,
/// preserving valid UTF-8 text and escaping invalid UTF-8 bytes and control
/// characters as `\xHH` or standard escape sequences.
#[must_use]
pub fn escape_bytes_human_readable(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for chunk in bytes.utf8_chunks() {
        for ch in chunk.valid().chars() {
            match ch {
                '\\' => out.push_str(r"\\"),
                '\n' => out.push_str(r"\n"),
                '\r' => out.push_str(r"\r"),
                '\t' => out.push_str(r"\t"),
                c if u32::from(c) < 0x20 || c == '\x7f' => {
                    let hex = format!("\\x{:02x}", u32::from(c));
                    out.push_str(&hex);
                }
                c => out.push(c),
            }
        }
        for &b in chunk.invalid() {
            let hex = format!("\\x{b:02x}");
            out.push_str(&hex);
        }
    }
    out
}

/// Formats a filesystem path into a human-readable, valid UTF-8 string,
/// escaping invalid UTF-8 bytes and control characters as `\xHH` hex escapes.
#[must_use]
pub fn escape_path_human_readable(path: &std::path::Path) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        escape_bytes_human_readable(path.as_os_str().as_bytes())
    }
    #[cfg(not(unix))]
    {
        escape_bytes_human_readable(path.to_string_lossy().as_bytes())
    }
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

    #[crate::ctb_test]
    fn test_escape_bytes_human_readable_ascii_and_unicode() {
        assert_eq!(escape_bytes_human_readable(b"hello world"), "hello world");
        assert_eq!(
            escape_bytes_human_readable("🦀 /path/to/ünicøde/日本語/file.txt".as_bytes()),
            "🦀 /path/to/ünicøde/日本語/file.txt"
        );
    }

    #[crate::ctb_test]
    fn test_escape_bytes_human_readable_controls_and_escapes() {
        assert_eq!(escape_bytes_human_readable(b"line1\nline2\r\ttab\\slash"), "line1\\nline2\\r\\ttab\\\\slash");
        assert_eq!(escape_bytes_human_readable(b"\x00null\x1funit"), "\\x00null\\x1funit");
    }

    #[crate::ctb_test]
    fn test_escape_bytes_human_readable_invalid_utf8() {
        // 0xe9 alone is invalid UTF-8 (e.g. Latin-1 'é')
        assert_eq!(
            escape_bytes_human_readable(b"/path/caf\xe9.txt"),
            "/path/caf\\xe9.txt"
        );
        // 0xff and 0xfe
        assert_eq!(
            escape_bytes_human_readable(b"\xff\xfe_raw"),
            "\\xff\\xfe_raw"
        );
    }

    #[crate::ctb_test]
    fn test_escape_path_human_readable() {
        let p = std::path::Path::new("/normal/path/file.txt");
        assert_eq!(escape_path_human_readable(p), "/normal/path/file.txt");
    }
}
