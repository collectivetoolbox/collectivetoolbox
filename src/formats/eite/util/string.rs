// ---------------
// String char access / mutation
// ---------------

use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use crate::{
    encoding::ascii::stagel_char_from_byte, util::ascii::ascii_is_digit,
};

pub fn str_char(s: &str, index: usize) -> String {
    s.chars()
        .nth(index)
        .map(|c| c.to_string())
        .unwrap_or_default()
}
pub fn str_char_at_pos(s: &str, index: usize) -> String {
    str_char(s, index)
}
pub fn char_at_pos(s: &str, index: usize) -> String {
    str_char(s, index)
}
pub fn char_at(s: &str, index: usize) -> String {
    str_char(s, index)
}

pub fn set_char_at(s: &str, index: usize, ch: &str) -> String {
    assert!(ch.chars().count() == 1, "Replacement must be single char");
    let Some(replacement) = ch.chars().next() else {
        return s.to_string();
    };
    s.chars()
        .enumerate()
        .map(|(i, c)| if i == index { replacement } else { c })
        .collect()
}

pub fn reverse_str(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn char_to_upper(ch: &str) -> String {
    assert!(ch.chars().count() == 1);
    ch.to_ascii_uppercase()
}

pub fn str_to_upper(s: &str) -> String {
    s.chars().map(|c| c.to_ascii_uppercase()).collect()
}

pub fn char_to_lower(ch: &str) -> String {
    assert!(ch.chars().count() == 1);
    ch.to_ascii_lowercase()
}

pub fn str_to_lower(s: &str) -> String {
    s.chars().map(|c| c.to_ascii_lowercase()).collect()
}

pub fn str_empty(s: &str) -> bool {
    s.is_empty()
}
pub fn str_nonempty(s: &str) -> bool {
    !s.is_empty()
}

pub fn str_contains_only_int(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    s.chars().all(|c| c.is_ascii_digit())
}

/// Index into a substring by start and lengths. Supports negative indices, but
/// replicates an off-by-one error from the original StageL implementation: the
/// length has to be one more than seems intuitive.
/// `substring_bug_compatible("test", 0, -2)` would return "tes".
pub fn substring_bug_compatible(
    s: &str,
    start: usize,
    length: isize,
) -> Result<String> {
    if length < 0 {
        // Negative length: JS code adjusts to (str.length + 1 + length)
        // Example: length = -1 => take (len + 1 - 1) = len chars (effectively whole string)
        let len_isize = isize::try_from(s.len()).context("String length exceeds isize::MAX")?;
        let adj = usize::try_from(
            len_isize
                .saturating_add(1)
                .saturating_add(length)
                .max(0),
        )
        .context("Adjusted length failed conversion to usize")?;
        Ok(s.chars().skip(start).take(adj).collect())
    } else {
        let take_len = usize::try_from(length).context("Length failed conversion to usize")?;
        Ok(s.chars().skip(start).take(take_len).collect())
    }
}

pub fn len_str(s: &str) -> usize {
    s.len()
}

/// strReplace(str, find, replace) – JS version used String.replace with first occurrence.
/// We replicate first occurrence behavior.
pub fn str_replace_once(haystack: &str, find: &str, replace: &str) -> String {
    if find.is_empty() {
        // JS would replace empty string at position 0; we mimic inserting replace at start.
        format!("{replace}{haystack}")
    } else if let Some(pos) = haystack.find(find) {
        let mut out = String::with_capacity(
            haystack
                .len()
                .saturating_sub(find.len())
                .saturating_add(replace.len()),
        );
        out.push_str(haystack.get(..pos).unwrap_or(""));
        out.push_str(replace);
        out.push_str(
            haystack.get(pos.saturating_add(find.len())..).unwrap_or(""),
        );
        out
    } else {
        haystack.to_string()
    }
}

// ---------------
// Percentage formatting (3 decimals)
// ---------------

pub fn format_percentage(a: i32, b: i32) -> Result<String> {
    if b == 0 {
        bail!("Division by zero in percentage");
    }
    if a == 0 {
        return Ok("0.000".into());
    }
    let pct = (f64::from(a) / f64::from(b)) * 100.0;
    Ok(format!("{pct:.3}"))
}

/// Basic (non-escaped) split replicating the original StageL strSplit.
/// Unlike `str.split(sep)` in many languages, this custom implementation:
/// - Does not collapse consecutive separators
/// - Returns trailing empty element if the input ends with the separator
pub fn str_split(input: &str, separator: &str) -> Vec<String> {
    if separator.is_empty() {
        // Degenerate: behave like every char boundary (mimic original loop semantics)
        return input.chars().map(|c| c.to_string()).collect();
    }

    let mut out = Vec::new();
    let mut remaining = input;
    let sep_len = separator.len();

    while !remaining.is_empty() {
        if remaining.starts_with(separator) {
            // Push current collected (could be empty), then remove the separator
            out.push(String::new());
            remaining = remaining.get(sep_len..).unwrap_or("");
        } else {
            // Consume one character
            let mut char_indices = remaining.char_indices();
            let Some((_, _first_char)) = char_indices.next() else {
                // I don't think we could ever hit this branch because it's in a !remaining.is_empty() loop.
                unreachable!();
            };
            let next_index =
                char_indices.next().map_or(remaining.len(), |(i, _)| i);
            let ch_str = remaining.get(..next_index).unwrap_or("");
            // We append the char to a "current" token: emulate original incremental building.
            if let Some(last_token) = out.last_mut() {
                last_token.push_str(ch_str);
            } else {
                out.push(ch_str.to_string());
            }
            remaining = remaining.get(next_index..).unwrap_or("");
        }
    }

    // Ensure leading separator case:
    if input.starts_with(separator) {
        out.insert(0, String::new());
    }

    out
}

/// Escape-aware split:
///
/// Original logic (strSplitEscaped):
/// - First performs a plain split.
/// - Iterates segments; if a segment ends with a backslash (escape char) and there
///   is a following segment, it replaces the trailing backslash with the separator
///   and concatenates the next segment to it, continuing (effectively treating the
///   separator as a literal).
/// - A dangling trailing backslash (no following segment) is preserved as-is.
pub fn str_split_escaped(input: &str, separator: &str) -> Vec<String> {
    ctb_utilities::string::split_escaped(input, separator)
}

/// Convenience wrappers retained for compatibility naming.
pub fn str_split_esc(input: &str, separator: &str) -> Vec<String> {
    str_split_escaped(input, separator)
}

pub fn explode_esc(input: &str, separator: &str) -> Vec<String> {
    str_split_escaped(input, separator)
}
pub fn explode_escaped(input: &str, separator: &str) -> Vec<String> {
    str_split_escaped(input, separator)
}

/// Join with escaping: occurrences of the separator in any element are preceded
/// by a backslash. A separator is appended after each element (trailing).
pub fn str_join_escaped(parts: &[String], separator: &str) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for p in parts {
        let escaped = p.replace(separator, &format!("\\{separator}"));
        out.push_str(&escaped);
        out.push_str(separator);
    }
    out
}

/// Convenience wrapper.
pub fn str_join_esc(parts: &[String], separator: &str) -> String {
    str_join_escaped(parts, separator)
}

/// Join escaped but remove trailing separator.
pub fn str_join_esc_no_trailing(parts: &[String], separator: &str) -> String {
    let joined = str_join_escaped(parts, separator);
    if joined.ends_with(separator) {
        let cut = joined.len().saturating_sub(separator.len());
        joined.get(..cut).unwrap_or("").to_string()
    } else {
        joined
    }
}

/// Parse a space-delimited string of decimal integers (the inverse of `str_print_arr<int>`).
/// Accepts optional trailing space. Rejects any non-digit / non-space characters.
pub fn int_arr_from_str_printed_arr(s: &str) -> Result<Vec<u32>> {
    let mut current = String::new();
    let mut out = Vec::new();

    for b in s.bytes() {
        if b == b' ' {
            if !current.is_empty() {
                out.push(
                    current
                        .parse::<u32>()
                        .map_err(|e| anyhow!("parse int: {e}"))?,
                );
                current.clear();
            }
        } else if ascii_is_digit(b) {
            let char = stagel_char_from_byte(b)?;
            current.push_str(&char.clone());
        } else {
            return Err(anyhow!(
                "Unexpected character (byte {b}) in int_arr_from_str_printed_arr"
            ));
        }
    }
    if !current.is_empty() {
        out.push(
            current
                .parse::<u32>()
                .map_err(|e| anyhow!("parse int: {e}"))?,
        );
    }
    Ok(out)
}

/// Convert string to vector of raw bytes (0..=255) treating each char's codepoint (0..=255).
/// For bytes above ASCII range, the direct low-byte value of the char is used (mirroring JS charCodeAt & 0xFF).
pub fn str_to_byte_array(s: &str) -> Result<Vec<u8>> {
    s.chars()
        .map(|c| {
            u8::try_from(u32::from(c) & 0xFF)
                .context("Failed to convert character codepoint to u8")
        })
        .collect()
}

/// Build a string from raw bytes (each byte -> U+00XX).
pub fn str_from_byte_array(bytes: &[u8]) -> String {
    bytes.iter().map(|b| char::from(*b)).collect()
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
    fn test_substring_bug_compatible_behavior() -> Result<()> {
        let s = "abcdef";
        assert_eq!(substring_bug_compatible(s, 0, 3)?, "abc");
        assert_eq!(substring_bug_compatible(s, 2, 2)?, "cd");
        // Negative length example (len=6): length=-1 => take len+1-1 = 5 chars from start
        assert_eq!(substring_bug_compatible(s, 0, -1)?, "abcdef");
        assert_eq!(substring_bug_compatible(s, 0, -2)?, "abcde");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_char_mutation() {
        let s = "hello";
        let s2 = set_char_at(s, 1, "A");
        assert_eq!(s2, "hAllo");
        assert_eq!(reverse_str("abc"), "cba");
    }

    #[crate::ctb_test]
    fn test_case_conversion() {
        assert_eq!(char_to_upper("a"), "A");
        assert_eq!(str_to_upper("Abc!"), "ABC!");
        assert_eq!(char_to_lower("Q"), "q");
        assert_eq!(str_to_lower("AbC"), "abc");
    }

    #[crate::ctb_test]
    fn test_str_contains_only_int() {
        assert!(str_contains_only_int("12345"));
        assert!(!str_contains_only_int("12a45"));
    }

    #[crate::ctb_test]
    fn test_format_percentage() {
        let p = format_percentage(1, 2).unwrap();
        assert_eq!(p, "50.000"); // 1/2 * 100
        let p = format_percentage(1, 3).unwrap();
        assert_eq!(p, "33.333"); // 1/3 * 100
        let p = format_percentage(2, 3).unwrap();
        assert_eq!(p, "66.667"); // 2/3 * 100
    }

    #[crate::ctb_test]
    fn test_str_split_basic() {
        assert_eq!(
            str_split("a,b,c", ","),
            Vec::<String>::from(vec!["a".into(), "b".into(), "c".into()])
        );
        assert_eq!(
            str_split("a,b,c,", ","),
            Vec::<String>::from(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "".into()
            ])
        );
        assert_eq!(str_split("", ","), Vec::<String>::new());

        assert_eq!(
            str_split("aabbabc", "ab"),
            Vec::<String>::from(vec!["a".into(), "b".into(), "c".into()])
        );
        assert_eq!(
            str_split("aabbabcab", "ab"),
            Vec::<String>::from(vec![
                "a".into(),
                "b".into(),
                "c".into(),
                "".into()
            ])
        );
        assert_eq!(
            str_split("abc", "ab"),
            Vec::<String>::from(vec!["".into(), "c".into()])
        );
        assert_eq!(
            str_split("ababbaa", "ab"),
            Vec::<String>::from(vec!["".into(), "".into(), "baa".into()])
        );
        assert_eq!(
            str_split("aab", "ab"),
            Vec::<String>::from(vec!["a".into(), "".into()])
        );
        assert_eq!(
            str_split("abaab", "ab"),
            Vec::<String>::from(vec!["".into(), "a".into(), "".into()])
        );
        assert_eq!(
            str_split("abaabab", "ab"),
            Vec::<String>::from(vec![
                "".into(),
                "a".into(),
                "".into(),
                "".into()
            ])
        );
        assert_eq!(
            str_split("abab", "ab"),
            Vec::<String>::from(vec!["".into(), "".into(), "".into()])
        );
        assert_eq!(
            str_split("ab", "ab"),
            Vec::<String>::from(vec!["".into(), "".into()])
        );
        assert_eq!(
            str_split(str_split("abab", "ab").join("ab").as_str(), "ab"),
            Vec::<String>::from(vec!["".into(), "".into(), "".into()])
        );
    }

    #[crate::ctb_test]
    fn test_str_split_escaped() {
        // "a\,b" should decode to ["a,b"]
        let v = str_split_escaped(r"a\,b", ",");
        assert_eq!(v, vec!["a,b".to_string()]);
        // Mixed
        let v2 = str_split_escaped(r"a\,b,c", ",");
        assert_eq!(v2, vec!["a,b".to_string(), "c".to_string()]);
        // Escaped separator joining multiple times
        let v3 = str_split_escaped(r"a\,b\,c,d", ",");
        assert_eq!(v3, vec!["a,b,c".to_string(), "d".to_string()]);
    }

    #[crate::ctb_test]
    fn test_str_join_escaped() {
        let joined = str_join_escaped(&["a,b".into(), "c".into()], ",");
        // trailing comma per design
        assert_eq!(joined, r"a\,b,c,");

        let no_trailing =
            str_join_esc_no_trailing(&["a,b".into(), "c".into()], ",");
        assert_eq!(no_trailing, r"a\,b,c");
    }

    #[crate::ctb_test]
    fn test_int_arr_from_str_printed_arr() {
        let v = int_arr_from_str_printed_arr("1 2 3 ").unwrap();
        assert_eq!(v, vec![1, 2, 3]);

        let v2 = int_arr_from_str_printed_arr("10 20 30").unwrap();
        assert_eq!(v2, vec![10, 20, 30]);

        // invalid char
        assert!(int_arr_from_str_printed_arr("1 a 3").is_err());
    }

    #[crate::ctb_test]
    fn test_str_to_from_byte_array_roundtrip() -> Result<()> {
        let s = "ABC";
        let bytes = str_to_byte_array(s)?;
        assert_eq!(bytes, vec![65, 66, 67]);
        let s2 = str_from_byte_array(&bytes);
        assert_eq!(s2, s);
        Ok(())
    }
}
