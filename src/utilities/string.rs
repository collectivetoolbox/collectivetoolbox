// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from dlint: MIT
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

// This file includes some code derived from Deno's dlint
// (https://github.com/denoland/deno_lint).
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

// See additional licensing details at end of file.

//! String manipulation, line slicing, and text transformation helpers.

use anyhow::{Result, anyhow, bail};

use crate::bail_if_none;

pub mod bytes;

/// Remove the line at the specified index from the given string.
pub fn remove_line(s: &str, idx_to_remove: usize) -> String {
    s.lines()
        .enumerate()
        .filter_map(
            |(i, line)| if i == idx_to_remove { None } else { Some(line) },
        )
        .collect::<Vec<_>>()
        .join("\n")
}

/// Removes the specified suffix from the string if it is present.
///
/// Returns the original string slice if the suffix is not found.
pub fn remove_suffix_unchecked<'a>(string: &'a str, suffix: &str) -> &'a str {
    // Reason for fallback: per docblock
    string.strip_suffix(suffix).unwrap_or(string)
}

/// Removes the specified suffix from the string, returning the result as a
/// new `String`.
///
/// Returns an error if the string does not end with the suffix.
pub fn remove_suffix(string: &str, suffix: &str) -> Result<String> {
    if let Some(s) = string.strip_suffix(suffix) {
        Ok(s.to_string())
    } else {
        Err(anyhow!("suffix not found"))
    }
}

/// Parses a hexadecimal string (with optional leading 0x/0X) into a `u32`.
pub fn parse_hex_u32(s: &str) -> Result<u32> {
    let trimmed = s.trim();
    let hex_part = if let Some(stripped) = trimmed.strip_prefix("0x") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("0X") {
        stripped
    } else {
        trimmed
    };
    u32::from_str_radix(hex_part, 16)
        .map_err(|e| anyhow!("Failed to parse hex '{s}': {e}"))
}

/// Parses a hexadecimal string (with optional leading 0x/0X) into a `u8`.
pub fn parse_hex_u8(s: &str) -> Result<u8> {
    let val = parse_hex_u32(s)?;
    u8::try_from(val).map_err(|e| anyhow!("Hex value '{s}' exceeds byte range: {e}"))
}

/// Parses space-separated hex Unicode codepoints into a vector of characters.
pub fn parse_hex_codepoints(s: &str) -> Result<Vec<char>> {
    let mut chars = Vec::new();
    for token in s.split_whitespace() {
        let code = parse_hex_u32(token)?;
        let ch = char::from_u32(code)
            .ok_or_else(|| anyhow!("Invalid Unicode code point '{token}'"))?;
        chars.push(ch);
    }
    Ok(chars)
}

/// Performs a compile-time (const) equality comparison between two string
/// slices.
#[expect(
    clippy::indexing_slicing,
    reason = "bracket indexing is necessary here as slice::get is not stable as const fn"
)]
pub(crate) const fn const_str_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut i = 0;
    while i < a_bytes.len() {
        if a_bytes[i] != b_bytes[i] {
            return false;
        }
        i = i.saturating_add(1);
    }
    true
}

/// Strips all ANSI escape codes (e.g., terminal color and styling codes)
/// from the string.
#[expect(
    clippy::expect_used,
    reason = "It is always the same regex, and has test coverage. Seems unlikely to fail I guess? Returning Result would be awkward."
)]
pub fn strip_ansi_codes(s: &str) -> String {
    static ANSI_RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| {
            regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]")
                .expect("Valid ANSI regex pattern")
        });
    ANSI_RE.replace_all(s, "").into_owned()
}

/// Checks if `haystack` contains `needle`, ignoring ASCII case.
pub fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

/// Checks if the given string matches a pattern containing a custom wildcard
/// token.
///
/// Normalizes line endings (CRLF to LF) in both the pattern and the string
/// before comparison.
pub fn pattern_match_custom_wildcard(
    pattern: &str,
    s: &str,
    wildcard: &str,
) -> bool {
    // Normalize line endings
    let mut s = s.replace("\r\n", "\n");
    let pattern = pattern.replace("\r\n", "\n");

    if pattern == wildcard {
        return true;
    }

    let parts = pattern.split(wildcard).collect::<Vec<&str>>();
    let Some(first_part) = parts.first() else {
        return pattern == s;
    };
    if parts.len() == 1 {
        return pattern == s;
    }

    if !s.starts_with(first_part) {
        return false;
    }

    // If the first line of the pattern is just a wildcard the newline character
    // needs to be pre-pended so it can safely match anything or nothing and
    // continue matching.
    if pattern.lines().next() == Some(wildcard) {
        s.insert(0, '\n');
    }

    let mut t = s.split_at(first_part.len());

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            continue;
        }
        // dbg!(part, i);
        if i == parts.len().saturating_sub(1)
            && (part.is_empty() || *part == "\n")
        {
            // dbg!("exit 1 true", i);
            return true;
        }
        if let Some(found) = t.1.find(*part) {
            // dbg!("found ", found);
            t = t.1.split_at(found.saturating_add(part.len()));
        } else {
            // dbg!("exit false ", i);
            return false;
        }
    }

    // dbg!("end ", t.1.len());
    t.1.is_empty()
}

/// Binary find and replace, similar to `str_replace()` in PHP.
pub fn bytes_str_replace(input: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    if from.is_empty() {
        return input.to_vec();
    }
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    let from_len = from.len();
    while i <= input.len().saturating_sub(from_len) {
        if input.get(i..i.saturating_add(from_len)) == Some(from) {
            out.extend_from_slice(to);
            i = i.saturating_add(from_len);
        } else if let Some(&byte) = input.get(i) {
            out.push(byte);
            i = i.saturating_add(1);
        } else {
            break;
        }
    }
    // append remaining tail
    if let Some(tail) = input.get(i..) {
        out.extend_from_slice(tail);
    }
    out
}

pub fn to_char(input: String) -> Result<char> {
    let mut chars = input.chars();
    let first = bail_if_none!(chars.next(), "String is empty");
    if chars.next().is_some() {
        bail!("String has more than 1 char.");
    }
    Ok(first)
}

pub fn to_u128(input: &str) -> Result<u128> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("String is empty");
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        bail!("String contains non-digit characters");
    }
    trimmed
        .parse::<u128>()
        .map_err(|e| anyhow::anyhow!("Failed to parse u128: {e}"))
}

/// Formats a slice of bytes into its hexadecimal representation.
pub fn to_hex(bytes: &[u8]) -> String {
    crate::bin2hex(bytes)
}

/// Formats a slice of bytes into its hexadecimal representation, prefixed with
/// `0x`.
pub fn to_hex_0x(bytes: &[u8]) -> String {
    format!("0x{}", to_hex(bytes))
}

fn split_escaped_internal(
    s: &str,
    separator: &str,
    trim: bool,
    step_back_bug: bool,
) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    if separator.is_empty() {
        let mut result: Vec<String> =
            s.chars().map(|c| c.to_string()).collect();
        if trim {
            for item in &mut result {
                *item = item.trim().to_string();
            }
        }
        return result;
    }

    let mut exploded: Vec<String> = s
        .split(separator)
        .map(std::string::ToString::to_string)
        .collect();
    let mut fixed = Vec::new();
    let mut k = 0_usize;
    while k < exploded.len() {
        let Some(segment) = exploded.get(k) else {
            break;
        };
        if segment.ends_with('\\') {
            let next_idx = k.saturating_add(1);
            if next_idx >= exploded.len() {
                let seg_str = if trim { segment.trim() } else { segment };
                fixed.push(seg_str.to_string());
                break;
            }
            // Replace trailing '\' with separator
            #[allow(
                clippy::expect_used,
                reason = "segment ends with ASCII backslash '\\' so segment.len() - 1 is an infallible UTF-8 character boundary"
            )]
            let sub = segment
                .get(..segment.len().saturating_sub(1))
                .expect("segment ends with ASCII backslash '\\' so segment.len() - 1 is in bounds");
            let mut prefix = sub.to_string();
            prefix.push_str(separator);
            // Append next segment
            if let Some(next_seg) = exploded.get(next_idx) {
                prefix.push_str(next_seg);
            }
            if let Some(slot) = exploded.get_mut(k) {
                *slot = prefix;
            }
            if next_idx < exploded.len() {
                let _ = exploded.remove(next_idx);
            }
            if step_back_bug {
                k = k.saturating_sub(1);
                continue;
            }
            // Do not increment k, retry with the merged segment
        } else {
            let seg_str = if trim { segment.trim() } else { segment };
            fixed.push(seg_str.to_string());
            k = k.saturating_add(1);
        }
    }
    fixed
}

/// Splits a string by a separator/delimiter, merging segments when the delimiter is escaped with a backslash.
pub fn split_escaped(s: &str, separator: &str) -> Vec<String> {
    split_escaped_internal(s, separator, false, false)
}

/// Splits a string by a separator/delimiter, merging segments when the delimiter is escaped with a backslash,
/// trimming whitespace from the resulting parts.
pub fn split_escaped_trim(s: &str, separator: &str) -> Vec<String> {
    split_escaped_internal(s, separator, true, false)
}

/// Splits a string by a separator/delimiter, merging segments when the delimiter is escaped with a backslash,
/// replicating a step-back logic bug from the original EITE implementation.
pub fn split_escaped_bug_compat(s: &str, separator: &str) -> Vec<String> {
    split_escaped_internal(s, separator, false, true)
}

/// Splits a string by a separator/delimiter, merging segments when the delimiter is escaped with a backslash,
/// with trimming enabled.
pub fn explode_escaped(s: &str, separator: &str) -> Vec<String> {
    split_escaped_trim(s, separator)
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
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_to_hex() {
        assert_eq!(to_hex(&[]), "");
        assert_eq!(to_hex(&[0]), "00");
        assert_eq!(to_hex(&[255]), "ff");
        assert_eq!(
            to_hex(&[0x12, 0x34, 0x56, 0xab, 0xcd, 0xef]),
            "123456abcdef"
        );
    }

    #[crate::ctb_test]
    fn test_to_u128() {
        assert_eq!(to_u128("123").unwrap(), 123);
        assert_eq!(to_u128("  456  ").unwrap(), 456);
        let _ = to_u128("").unwrap_err();
        let _ = to_u128("abc").unwrap_err();
        let _ = to_u128("12.3").unwrap_err();
        let _ = to_u128("-5").unwrap_err();
    }

    #[crate::ctb_test]
    fn test_remove_line() {
        let input = "Line 1\nLine 2\nLine 3\nLine 4";
        let expected = "Line 1\nLine 3\nLine 4";
        let result = remove_line(input, 1);
        assert_eq!(result, expected);
    }

    #[crate::ctb_test]
    fn test_split_escaped() {
        // Without trim
        assert_eq!(split_escaped("a, b, c", ","), vec!["a", " b", " c"]);
        assert_eq!(split_escaped("a\\,b, c", ","), vec!["a,b", " c"]);
        assert_eq!(split_escaped("a\\,b\\,c, d", ","), vec!["a,b,c", " d"]);
        assert_eq!(split_escaped("a\\", ","), vec!["a\\"]);

        // explode_escaped (with trim)
        assert_eq!(explode_escaped("a, b, c", ","), vec!["a", "b", "c"]);
        assert_eq!(explode_escaped("a\\,b, c", ","), vec!["a,b", "c"]);
        assert_eq!(explode_escaped("a\\,b\\,c, d", ","), vec!["a,b,c", "d"]);
        assert_eq!(explode_escaped("a\\", ","), vec!["a\\"]);

        // Verify the step-back logic bug in split_escaped_bug_compat
        assert_eq!(
            split_escaped_bug_compat("a,b\\,c", ","),
            vec!["a", "a", "b,c"]
        );
        assert_eq!(split_escaped("a,b\\,c", ","), vec!["a", "b,c"]);
    }

    #[crate::ctb_test]
    fn test_remove_suffix_unchecked() {
        assert_eq!(remove_suffix_unchecked("hello_world", "_world"), "hello");
        assert_eq!(remove_suffix_unchecked("hello_world", "world"), "hello_");
        assert_eq!(
            remove_suffix_unchecked("hello_world", "hello"),
            "hello_world"
        );
    }

    #[crate::ctb_test]
    fn test_remove_suffix() {
        assert_eq!(remove_suffix("hello_world", "_world").unwrap(), "hello");
        let _ = remove_suffix("hello_world", "hello").unwrap_err();
    }

    #[crate::ctb_test]
    fn test_const_str_eq() {
        assert!(const_str_eq("hello", "hello"));
        assert!(!const_str_eq("hello", "world"));
        assert!(!const_str_eq("hello", "hello_world"));
        assert!(!const_str_eq("hello_world", "hello"));

        const EQ: bool = const_str_eq("abc", "abc");
        const NEQ: bool = const_str_eq("abc", "def");
        assert!(EQ);
        assert!(!NEQ);
    }

    #[crate::ctb_test]
    fn test_strip_ansi_codes() {
        let colored = "\x1B[31mHello\x1B[0m \x1B[1mWorld\x1B[0m";
        assert_eq!(strip_ansi_codes(colored), "Hello World");

        let plain = "Hello World";
        assert_eq!(strip_ansi_codes(plain), "Hello World");
    }

    #[crate::ctb_test]
    fn test_pattern_match_custom_wildcard() {
        assert!(pattern_match_custom_wildcard(
            "hello * world",
            "hello brave new world",
            "*"
        ));
        assert!(pattern_match_custom_wildcard("*", "anything", "*"));
        assert!(pattern_match_custom_wildcard(
            "line1\r\n*",
            "line1\nline2",
            "*"
        ));
        assert!(!pattern_match_custom_wildcard(
            "hello * world",
            "hello brave new worlds",
            "*"
        ));
        assert!(pattern_match_custom_wildcard(
            "*\nworld",
            "hello\nworld",
            "*"
        ));

        assert!(pattern_match_custom_wildcard(
            "hello [WILDCARD] world",
            "hello brave new world",
            "[WILDCARD]"
        ));
        assert!(!pattern_match_custom_wildcard(
            "hello [WILDCARD] world",
            "hello brave new worlds",
            "[WILDCARD]"
        ));
        assert!(pattern_match_custom_wildcard(
            "[WILDCARD]",
            "anything",
            "[WILDCARD]"
        ));
    }

    #[crate::ctb_test]
    fn test_contains_ignore_ascii_case() {
        assert!(contains_ignore_ascii_case(b"gzip, deflate", b"GZIP"));
        assert!(contains_ignore_ascii_case(b"gzip, deflate", b"DEFLATE"));
        assert!(contains_ignore_ascii_case(b"gzip, deflate", b"gzip"));
        assert!(contains_ignore_ascii_case(b"hello world", b""));
        assert!(!contains_ignore_ascii_case(b"gzip, deflate", b"brotli"));
        assert!(!contains_ignore_ascii_case(b"short", b"very long needle"));
    }

    #[crate::ctb_test]
    fn test_hex_parsing() -> Result<()> {
        assert_eq!(parse_hex_u32("0x1F")?, 31);
        assert_eq!(parse_hex_u32("25a0")?, 0x25a0);
        assert_eq!(parse_hex_u8("0XFF")?, 255);
        assert!(parse_hex_u8("0x100").is_err());

        let codepoints = parse_hex_codepoints("25a0 03b4 0394")?;
        assert_eq!(codepoints, vec!['■', 'δ', 'Δ']);
        Ok(())
    }
}

/*
Code from dlint is used under the following license:
======

MIT License

Copyright (c) 2018-2024 the Deno authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
