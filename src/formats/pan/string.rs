/* SPDX-License-Identifier: MIT */
// See full license details in COPYING in the `ctb-formats-pan` crate source directory.

//! Pan string formatting and slicing helpers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Funnel-style substring extraction helpers.
pub mod funnel;
/// Numeric string helpers.
pub mod numeric;
/// Numeric formatting with Pan pattern syntax.
pub mod pattern;
/// Additional string manipulation helpers.
pub mod stringmod;

/// Concatenates two strings.
pub fn cat(left: &str, right: &str) -> String {
    let mut out = String::with_capacity(left.len().saturating_add(right.len()));
    out.push_str(left);
    out.push_str(right);
    out
}

/// Wraps `root` with `prefix` and `suffix` when `root` is non-empty.
/// For example: a, b, c -> "abc"; a, "", c -> "".
pub fn sandwich(prefix: &str, root: &str, suffix: &str) -> String {
    if root.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(
        prefix
            .len()
            .saturating_add(root.len())
            .saturating_add(suffix.len()),
    );
    out.push_str(prefix);
    out.push_str(root);
    out.push_str(suffix);
    out
}

/// Repeats `text` `count` times.
pub fn rep(text: &str, count: usize) -> String {
    let mut out = String::with_capacity(text.len().saturating_mul(count));
    for _ in 0..count {
        out.push_str(text);
    }
    out
}

fn byte_index_at_char(s: &str, char_pos: usize) -> usize {
    let mut it = s.char_indices();
    if let Some((idx, _)) = it.nth(char_pos) {
        return idx;
    }
    s.len()
}

fn slice_chars(s: &str, start_char: usize, len_chars: usize) -> String {
    if len_chars == 0 {
        return String::new();
    }

    let start_b = byte_index_at_char(s, start_char);
    let end_b = byte_index_at_char(s, start_char.saturating_add(len_chars));
    s.get(start_b..end_b).unwrap_or("").to_string()
}

fn slice_chars_from(s: &str, start_char: usize) -> String {
    let start_b = byte_index_at_char(s, start_char);
    s.get(start_b..).unwrap_or("").to_string()
}

fn slice_chars_to(s: &str, end_char: usize) -> String {
    let end_b = byte_index_at_char(s, end_char);
    s.get(..end_b).unwrap_or("").to_string()
}

/// Returns the substring after the first occurrence of `tag`.
///
/// Returns an empty string if `tag` is empty or not found.
pub fn after(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        return String::new();
    }
    let Some(pos) = text.find(tag) else {
        return String::new();
    };
    let start = pos.saturating_add(tag.len());
    text.get(start..).unwrap_or("").to_string()
}

/// Returns the substring before the first occurrence of `tag`.
///
/// Returns an empty string if `tag` is empty or not found.
pub fn before(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        return String::new();
    }
    let Some(pos) = text.find(tag) else {
        return String::new();
    };
    text.get(..pos).unwrap_or("").to_string()
}

/// Returns the first line split by `\n`.
pub fn firstline(text: &str) -> String {
    let mut it = text.split('\n');
    if let Some(first) = it.next() {
        return first.to_string();
    }
    String::new()
}

/// Returns the last line split by `\n`.
pub fn lastline(text: &str) -> String {
    let mut it = text.rsplit('\n');
    if let Some(last) = it.next() {
        return last.to_string();
    }
    String::new()
}

/// Returns the last whitespace-delimited word.
pub fn lastword(text: &str) -> String {
    let mut it = text.split_whitespace();
    if let Some(last) = it.next_back() {
        return last.to_string();
    }
    String::new()
}

/// Returns the leftmost `len` characters.
pub fn left(text: &str, len: usize) -> String {
    let total = text.chars().count();
    slice_chars(text, 0, len.min(total))
}

/// Returns `len` characters starting at the 1-based `start` position.
pub fn mid(text: &str, start: usize, len: usize) -> String {
    let Some(start0) = start.checked_sub(1) else {
        return String::new();
    };
    let total = text.chars().count();
    if start0 >= total {
        return String::new();
    }
    slice_chars(text, start0, len.min(total.saturating_sub(start0)))
}

/// Returns the 1-based line number `num`, or empty if missing.
pub fn nthline(text: &str, num: usize) -> String {
    if num == 0 {
        return String::new();
    }
    let mut it = text.split('\n');
    let idx = num.saturating_sub(1);
    if let Some(line) = it.nth(idx) {
        return line.to_string();
    }
    String::new()
}

/// Returns the 1-based word number `num`, or empty if missing.
pub fn nthword(text: &str, num: usize) -> String {
    if num == 0 {
        return String::new();
    }
    let mut it = text.split_whitespace();
    let idx = num.saturating_sub(1);
    if let Some(word) = it.nth(idx) {
        return word.to_string();
    }
    String::new()
}

/// Removes `prefix` when present.
pub fn removeprefix(text: &str, prefix: &str) -> String {
    if let Some(rest) = text.strip_prefix(prefix) {
        return rest.to_string();
    }
    text.to_string()
}

/// Removes `suffix` when present.
pub fn removesuffix(text: &str, suffix: &str) -> String {
    if let Some(rest) = text.strip_suffix(suffix) {
        return rest.to_string();
    }
    text.to_string()
}

/// Returns the rightmost `len` characters.
pub fn right(text: &str, len: usize) -> String {
    let total = text.chars().count();
    if len >= total {
        return text.to_string();
    }
    let start = total.saturating_sub(len);
    slice_chars_from(text, start)
}

/// Removes a slice starting at the 0-based `startposition`.
///
/// A `count` of -1 removes to the end; non-positive counts are no-ops.
pub fn snip(text: &str, startposition: usize, count: i64) -> String {
    let total = text.chars().count();
    if startposition >= total {
        return text.to_string();
    }

    if count == -1 {
        return slice_chars_to(text, startposition);
    }
    if count <= 0 {
        return text.to_string();
    }

    let Ok(count_usize) = usize::try_from(count) else {
        return text.to_string();
    };

    let prefix = slice_chars_to(text, startposition);
    let suffix =
        slice_chars_from(text, startposition.saturating_add(count_usize));
    cat(&prefix, &suffix)
}

/// Returns the substring after `tag`, or `text` when not found.
pub fn textafter(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        return text.to_string();
    }
    let Some(pos) = text.find(tag) else {
        return text.to_string();
    };
    let start = pos.saturating_add(tag.len());
    text.get(start..).unwrap_or("").to_string()
}

/// Returns the substring before `tag`, or `text` when not found.
pub fn textbefore(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        return text.to_string();
    }
    let Some(pos) = text.find(tag) else {
        return text.to_string();
    };
    text.get(..pos).unwrap_or("").to_string()
}

/// Removes `len` characters from the end of `text`.
pub fn trim(text: &str, len: usize) -> String {
    let total = text.chars().count();
    if len >= total {
        return String::new();
    }
    slice_chars(text, 0, total.saturating_sub(len))
}

/// Removes `len` characters from the start of `text`.
pub fn trimleft(text: &str, len: usize) -> String {
    let total = text.chars().count();
    if len >= total {
        return String::new();
    }
    slice_chars_from(text, len)
}

/// Returns the number of Unicode scalar values in `text`.
pub fn length(text: &str) -> usize {
    text.chars().count()
}

/// Returns the number of `\n`-delimited lines.
pub fn linecount(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.split('\n').count()
}

fn parse_ranges(therange: &str) -> Vec<(char, char)> {
    let mut out = Vec::new();
    let mut it = therange.chars();
    loop {
        let Some(a) = it.next() else { break };
        let Some(b) = it.next() else { break };
        if a <= b {
            out.push((a, b));
        } else {
            out.push((b, a));
        }
    }
    out
}

/// Returns true if any character is within the range pairs.
/// For instance, "`AZaz`" means 'A'..='Z' and 'a'..='z'.
pub fn rangecontains(thetext: &str, therange: &str) -> bool {
    let ranges = parse_ranges(therange);
    for ch in thetext.chars() {
        for (lo, hi) in ranges.iter().copied() {
            if ch >= lo && ch <= hi {
                return true;
            }
        }
    }
    false
}

/// Returns true if every character is within the range pairs.
/// For instance, "`AZaz`" means 'A'..='Z' and 'a'..='z'.
pub fn rangematch(text: &str, therange: &str) -> bool {
    let ranges = parse_ranges(therange);
    for ch in text.chars() {
        let mut ok = false;
        for (lo, hi) in ranges.iter().copied() {
            if ch >= lo && ch <= hi {
                ok = true;
                break;
            }
        }
        if !ok {
            return false;
        }
    }
    true
}

/// Returns the 1-based character index of `phrase`, or 0 if missing.
pub fn search(text: &str, phrase: &str) -> usize {
    if phrase.is_empty() {
        return 0;
    }
    let Some(byte_pos) = text.find(phrase) else {
        return 0;
    };

    let prefix = text.get(..byte_pos).unwrap_or("");
    prefix.chars().count().saturating_add(1)
}

/// Returns the number of whitespace-delimited words.
pub fn wordcount(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Returns true if `substring` is present in `text`.
pub fn contains(text: &str, substring: &str) -> bool {
    text.contains(substring)
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
    fn test_string_utils() {
        // basic concatenation and repeat
        let result = cat("Hello, ", "world!");
        assert_eq!(result, "Hello, world!");
        assert_eq!("*****", rep("*", 5));

        // sandwich behavior
        assert_eq!(", foo", sandwich(", ", "foo", ""));
        assert_eq!("", sandwich(", ", "", ""));
        assert_eq!(" (foo)", sandwich(" (", "foo", ")"));
        assert_eq!("", sandwich(" (", "", ")"));

        // slices and lines
        assert_eq!("ef", after("abcdef", "cd"));
        assert_eq!("", after("abcdef", "")); // empty tag -> empty string
        assert_eq!("ab", before("abcdef", "cd"));
        assert_eq!("", before("abcdef", "")); // empty tag -> empty string
        assert_eq!("abc", firstline("abc\ndef\nghi"));
        assert_eq!("ghi", lastline("abc\ndef\nghi"));
        assert_eq!("ghi", lastword("abc\nde f\nghi"));

        // aware character operations
        assert_eq!("аб", left("абвг", 2)); // Cyrillic characters
        assert_eq!("fghi", mid("abcdefghijk", 6, 4));
        assert_eq!("", mid("abc", 0, 2)); // start is 1-based
        assert_eq!("def", nthline("abc\ndef\nghi", 2));
        assert_eq!("def", nthword("abc def ghi", 2));
        assert_eq!("c", removeprefix("abc", "ab"));
        assert_eq!("abc", removeprefix("abc", "f"));
        assert_eq!("ab", removesuffix("abc", "c"));
        assert_eq!("abc", removesuffix("abc", "f"));
        assert_eq!("bc", right("abc", 2));

        // snip behavior
        assert_eq!("abef", snip("abcdef", 2, 2));
        assert_eq!("ab", snip("abcdef", 2, -1)); // -1 removes to end
        assert_eq!("abcdef", snip("abcdef", 99, 2)); // start beyond end -> unchanged
        assert_eq!("abcdef", snip("abcdef", 2, 0)); // non-positive count -> unchanged

        // textafter/textbefore keep original if tag absent
        assert_eq!("abcdef", textafter("abcdef", "xx"));
        assert_eq!("abcdef", textbefore("abcdef", "xx"));
        assert_eq!("ef", textafter("abcdef", "cd"));
        assert_eq!("ab", textbefore("abcdef", "cd"));

        // trimming and counts
        assert_eq!("abcd", trim("abcdef", 2));
        assert_eq!("cdef", trimleft("abcdef", 2));
        assert_eq!(6, length("abcdef"));
        assert_eq!(3, linecount("abc\ndef\nghi"));

        // range parsing and matching
        assert!(rangecontains("AB1234CD", "AZaz"));
        assert!(rangecontains("AB1234CD", "09"));
        assert!(!rangematch("AB1234CD", "09"));
        assert!(rangematch("1234", "09"));
        // odd-length range string ignores trailing char
        assert!(rangecontains("a", "azb")); // parsed as ('a','z') and ('b',?) -> last ignored

        // search and wordcount
        assert_eq!(3, search("ABCD", "C"));
        assert_eq!(0, search("ABCD", "")); // empty phrase -> 0
        assert_eq!(3, wordcount("abc def ghi"));

        // contains helper
        assert!(contains("hello world", "world"));
        assert!(!contains("hello", "bye"));

        // internal helper coverage (private functions)
        assert_eq!(byte_index_at_char("aβc", 0), 0);
        assert!(byte_index_at_char("aβc", 99) >= "aβc".len().saturating_sub(1)); // out of range -> len()
        assert_eq!(slice_chars("aβcδ", 1, 2), "βc");
        assert_eq!(slice_chars_from("abc", 1), "bc");
        assert_eq!(slice_chars_to("abc", 2), "ab");
    }
}
