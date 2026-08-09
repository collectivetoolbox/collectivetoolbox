/* SPDX-License-Identifier: MIT */
// See full license details in COPYING in the `ctb-formats-pan` crate source directory.

//! Additional Pan string utilities.

use ctb_formats_applescript::escape_string;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Escapes a string to use as an AppleScript string literal.
///
/// The result is wrapped in double quotes, and any `"` or `\` characters are
/// escaped with a leading backslash.
pub fn applescriptstring(text: &str) -> String {
    escape_string(text)
}

/// Concatenates `prefix` and `suffix` with `connector` when both exist.
pub fn connect(prefix: &str, connector: &str, suffix: &str) -> String {
    match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => String::new(),
        (true, false) => suffix.to_owned(),
        (false, true) => prefix.to_owned(),
        (false, false) => {
            let mut out = String::with_capacity(
                prefix
                    .len()
                    .saturating_add(connector.len())
                    .saturating_add(suffix.len()),
            );
            out.push_str(prefix);
            out.push_str(connector);
            out.push_str(suffix);
            out
        }
    }
}

/// Like `connect`, but with different parameter naming.
pub fn yoke(prefix: &str, joiner: &str, suffix: &str) -> String {
    connect(prefix, joiner, suffix)
}

/// Replaces carriage returns with vertical tabs.
pub fn crtovtab(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\r' { '\u{000B}' } else { c })
        .collect()
}

/// Replaces vertical tabs with carriage returns.
pub fn vtabtocr(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\u{000B}' { '\r' } else { c })
        .collect()
}

/// Returns `default` when `text` is empty.
pub fn defaulttext(text: &str, default: &str) -> String {
    if text.is_empty() {
        default.to_owned()
    } else {
        text.to_owned()
    }
}

/// Extracts the `item`-th element from an array.
///
/// If `item` exceeds the number of elements, an empty string `""` is returned.
pub fn extract(text: &str, item: i64, separator: char) -> Result<String> {
    if item == -1 {
        let count = if text.is_empty() {
            0
        } else {
            text.split(separator).count()
        };
        return Ok(count.to_string());
    }

    if item < 1 {
        bail!("extract(): item must be >= 1, or -1 to request the item count");
    }

    let idx_i64 = item.saturating_sub(1);
    let idx =
        usize::try_from(idx_i64).context("extract(): item index overflow")?;
    Ok(text
        .split(separator)
        .nth(idx)
        // Reason for fallback: requested item index out of bounds in separator split defaults to empty string (as per official Pan docs and docblock here)
        .unwrap_or_default()
        .to_owned())
}

/// Pads or truncates `text` to exactly `width` characters.
pub fn fixedwidth(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut chars: Vec<char> = text.chars().collect();
    if chars.len() > width {
        chars.truncate(width);
        return chars.into_iter().collect();
    }

    let pad = width.saturating_sub(chars.len());
    let mut out: String = chars.into_iter().collect();
    out.extend(std::iter::repeat_n(' ', pad));
    out
}

/// Right-aligns `text` to `width`, truncating from the left if needed.
#[allow(
    clippy::expect_used,
    reason = "start = chars.len() - width < chars.len() guaranteed by chars.len() > width"
)]
pub fn fixedwidthright(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() > width {
        let start = chars.len().saturating_sub(width);
        return chars
            .get(start..)
            .expect("start < chars.len() guaranteed by chars.len() > width")
            .iter()
            .collect();
    }

    let pad = width.saturating_sub(chars.len());
    let mut out = String::new();
    out.extend(std::iter::repeat_n(' ', pad));
    out.extend(chars);
    out
}

#[allow(
    clippy::expect_used,
    reason = "start = chars.len() - width < chars.len() guaranteed by chars.len() > width"
)]
pub fn padzero(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() > width {
        let start = chars.len().saturating_sub(width);
        return chars
            .get(start..)
            .expect("start < chars.len() guaranteed by chars.len() > width")
            .iter()
            .collect();
    }

    let pad = width.saturating_sub(chars.len());
    let mut out = String::new();
    out.extend(std::iter::repeat_n('0', pad));
    out.extend(chars);
    out
}

/// Removes empty lines and normalizes line breaks to `\n`.
#[allow(
    clippy::expect_used,
    reason = "Infallible byte indexing and ASCII character boundary slice ranges in linestrip"
)]
pub fn linestrip(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let b = *bytes
            .get(i)
            .expect("i < bytes.len() guaranteed by loop condition");
        let is_cr = b == b'\r';
        let is_lf = b == b'\n';
        if !is_cr && !is_lf {
            i = i.saturating_add(1);
            continue;
        }

        let line = text
            .get(start..i)
            .expect("start..i is a valid char boundary range <= text.len()");
        if !line.trim().is_empty() {
            out.push(line);
        }

        if is_cr && bytes.get(i.saturating_add(1)) == Some(&b'\n') {
            i = i.saturating_add(2);
        } else {
            i = i.saturating_add(1);
        }
        start = i;
    }

    let tail = text
        .get(start..)
        .expect("start is a valid char boundary <= text.len()");
    if !tail.trim().is_empty() {
        out.push(tail);
    }

    out.join("\n")
}

/// Converts all characters to lowercase.
pub fn lower(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).collect()
}

/// Converts all characters to uppercase.
pub fn upper(text: &str) -> String {
    text.chars().flat_map(char::to_uppercase).collect()
}

/// Uppercases the first character of each whitespace-delimited word.
pub fn upperword(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut new_word = true;

    for ch in text.chars() {
        if ch.is_whitespace() {
            new_word = true;
            out.push(ch);
            continue;
        }

        if new_word {
            out.extend(ch.to_uppercase());
            new_word = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }

    out
}

/// Replaces all but the last `keep_last` digits with `X`.
pub fn obscuredigits(text: &str, keep_last: usize) -> String {
    let total_digits = text.chars().filter(char::is_ascii_digit).count();
    let keep = total_digits.min(keep_last);

    let mut seen_from_end = 0usize;
    let mut out_rev = String::with_capacity(text.len());

    for ch in text.chars().rev() {
        if ch.is_ascii_digit() {
            if seen_from_end < keep {
                out_rev.push(ch);
            } else {
                out_rev.push('X');
            }
            seen_from_end = seen_from_end.saturating_add(1);
        } else {
            out_rev.push(ch);
        }
    }

    out_rev.chars().rev().collect()
}

/// Collapses runs of spaces into a single space and trims ends.
pub fn onespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = true;

    for ch in text.chars() {
        if ch == ' ' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }
        out.push(ch);
        prev_space = false;
    }

    out.trim_matches(' ').to_owned()
}

/// Collapses runs of whitespace into a single space and trims ends.
pub fn onewhitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = true;

    for ch in text.chars() {
        let mapped = if ch.is_whitespace() { ' ' } else { ch };
        if mapped == ' ' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }

        out.push(mapped);
        prev_space = false;
    }

    out.trim_matches(' ').to_owned()
}

/// Wraps text in double quotes, doubling any internal quotes.
pub fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('"');
    for ch in text.chars() {
        if ch == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out.push('"');
    out
}

/// Repeats `text` `count` times.
pub fn rep(text: &str, count: i64) -> Result<String> {
    if count < 0 {
        bail!("rep(): count must be non-negative");
    }
    let count_usize =
        usize::try_from(count).context("rep(): count overflow")?;

    let cap = text
        .len()
        .checked_mul(count_usize)
        .context("rep(): allocation size overflow")?;

    let mut out = String::with_capacity(cap);
    for _ in 0..count_usize {
        out.push_str(text);
    }
    Ok(out)
}

/// Replaces all occurrences of `search` with `replace_with`.
pub fn replace(text: &str, search: &str, replace_with: &str) -> Result<String> {
    if search.is_empty() {
        bail!("replace(): search string must not be empty");
    }
    Ok(text.replace(search, replace_with))
}

/// Replaces multiple search strings with their corresponding replacements.
pub fn replacemultiple(
    text: &str,
    search: &str,
    replace_with: &str,
    sep: char,
) -> Result<String> {
    let search_items: Vec<&str> = search.split(sep).collect();
    let replace_items: Vec<&str> = replace_with.split(sep).collect();

    if search_items.len() != replace_items.len() {
        bail!(
            "replacemultiple(): search/replace lists must have the same length"
        );
    }

    let mut out = text.to_owned();
    for (s, r) in search_items.into_iter().zip(replace_items) {
        if !s.is_empty() {
            out = out.replace(s, r);
        }
    }
    Ok(out)
}

/// Performs replacements based on a separator-delimited mapping table.
pub fn batchreplace(
    text: &str,
    array: &str,
    sep: char,
    subsep: char,
) -> Result<String> {
    let mut out = text.to_owned();

    for row in array.split(sep) {
        if row.is_empty() {
            continue;
        }
        let mut parts = row.splitn(2, subsep);
        #[allow(
            clippy::expect_used,
            reason = "row is non-empty so splitn yields at least one part"
        )]
        let before = parts
            .next()
            .expect("row is non-empty so splitn yields at least one part");
        let after = parts
            .next()
            .context("batchreplace(): row missing subseparator")?;

        if before.is_empty() {
            continue;
        }
        out = out.replace(before, after);
    }

    Ok(out)
}

/// Wraps `root` with `prefix` and `suffix` when `root` is non-empty.
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

/// Trims whitespace and control characters from both ends.
pub fn strip(text: &str) -> String {
    text.trim_matches(|c: char| c.is_whitespace() || u32::from(c) < 32)
        .to_owned()
}

/// Keeps only characters within the inclusive `range` pairs.
pub fn stripchar(text: &str, range: &str) -> Result<String> {
    let rchars: Vec<char> = range.chars().collect();
    if !rchars.len().is_multiple_of(2) {
        bail!("stripchar(): range must contain an even number of characters");
    }

    let mut pairs = Vec::new();
    for chunk in rchars.chunks(2) {
        let a = *chunk.first().context("Missing start range character")?;
        let b = *chunk.get(1).context("Missing end range character")?;
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        pairs.push((lo, hi));
    }

    let mut out = String::with_capacity(text.len());
    'outer: for ch in text.chars() {
        for (lo, hi) in &pairs {
            if *lo <= ch && ch <= *hi {
                out.push(ch);
                continue 'outer;
            }
        }
    }
    Ok(out)
}

/// Removes simple HTML tags delimited by `<` and `>`.
pub fn striphtmltags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Removes non-printable ASCII control characters.
pub fn stripprintable(text: &str) -> String {
    text.chars().filter(|&c| u32::from(c) >= 32).collect()
}

/// Keeps only alphabetic characters.
pub fn striptoalpha(text: &str) -> String {
    text.chars().filter(|c| c.is_alphabetic()).collect()
}

/// Keeps only ASCII digits.
pub fn striptonum(text: &str) -> String {
    text.chars().filter(char::is_ascii_digit).collect()
}

/// Returns a random ASCII letter.
///
/// Use `U` for uppercase, `L` for lowercase, or anything else for either.
pub fn randomletter(option: &str) -> Result<char> {
    use rand::Rng;

    let mut rng = rand::rng();
    let choice: i32 = match option {
        "U" => rng.random_range(0..26),
        "L" => rng.random_range(26..52),
        _ => rng.random_range(0..52),
    };

    let base = if choice < 26 { b'A' } else { b'a' };
    let offset = if choice < 26 {
        choice
    } else {
        choice.saturating_sub(26)
    };

    let offset_u8 =
        u8::try_from(offset).context("randomletter(): offset overflow")?;
    let b = base
        .checked_add(offset_u8)
        .context("randomletter(): byte overflow")?;

    Ok(char::from(b))
}

/// Returns a random non-empty line from `text`.
pub fn randomline(text: &str) -> Result<String> {
    use rand::Rng;

    let mut lines: Vec<&str> = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let b = *bytes.get(i).context("Invalid byte index")?;
        let is_cr = b == b'\r';
        let is_lf = b == b'\n';
        if !is_cr && !is_lf {
            i = i.saturating_add(1);
            continue;
        }

        let line = text.get(start..i).context("Invalid line range")?;
        if !line.trim().is_empty() {
            lines.push(line);
        }

        if is_cr && bytes.get(i.saturating_add(1)) == Some(&b'\n') {
            i = i.saturating_add(2);
        } else {
            i = i.saturating_add(1);
        }
        start = i;
    }

    let tail = text.get(start..).context("Invalid tail range")?;
    if !tail.trim().is_empty() {
        lines.push(tail);
    }

    if lines.is_empty() {
        return Ok(String::new());
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..lines.len());
    Ok(lines
        .get(idx)
        .context("Random index out of bounds")?
        .to_string())
}

/// Returns a random whitespace-delimited word from `wordlist`.
pub fn randomword(wordlist: &str) -> Result<String> {
    use rand::Rng;

    let words: Vec<&str> = wordlist.split_whitespace().collect();
    if words.is_empty() {
        return Ok(String::new());
    }

    let mut rng = rand::rng();
    let idx = rng.random_range(0..words.len());
    Ok(words
        .get(idx)
        .context("Random index out of bounds")?
        .to_string())
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
    fn test_basic_joining() {
        assert_eq!(connect("City", ", ", "State"), "City, State");
        assert_eq!(connect("", ", ", "State"), "State");
        assert_eq!(connect("City", ", ", ""), "City");
        assert_eq!(connect("", ", ", ""), "");

        assert_eq!(yoke("a", "-", "b"), "a-b");
        assert_eq!(sandwich(" (", "Mgr", ")"), " (Mgr)");
        assert_eq!(sandwich(" (", "", ")"), "");
    }

    #[crate::ctb_test]
    fn test_cr_vtab_roundtrip() {
        let s = "a\rb";
        assert_eq!(crtovtab(s), "a\u{000B}b");
        assert_eq!(vtabtocr(&crtovtab(s)), s);
    }

    #[crate::ctb_test]
    fn test_defaulttext_extract() -> Result<()> {
        assert_eq!(defaulttext("", "x"), "x");
        assert_eq!(defaulttext("y", "x"), "y");

        let t = "a,b,,d,";
        assert_eq!(extract(t, ',', 1)?, "a");
        assert_eq!(extract(t, ',', 2)?, "b");
        assert_eq!(extract(t, ',', 3)?, "");
        assert_eq!(extract(t, ',', 5)?, "");
        assert_eq!(extract(t, ',', -1)?, "5");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_fixedwidths() {
        assert_eq!(fixedwidth("ab", 5), "ab   ");
        assert_eq!(fixedwidth("abcdef", 3), "abc");
        assert_eq!(fixedwidthright("ab", 5), "   ab");
        assert_eq!(fixedwidthright("abcdef", 3), "def");
        assert_eq!(padzero("7", 3), "007");
        assert_eq!(padzero("12345", 3), "345");
    }

    #[crate::ctb_test]
    fn test_linestrip() {
        let t = "a\r\r b \n\nc\r\n  \r\nd";
        assert_eq!(linestrip(t), "a\n b \nc\nd");
    }

    #[crate::ctb_test]
    fn test_case_and_words() {
        assert_eq!(lower("ABC 123"), "abc 123");
        assert_eq!(upper("a.f.k."), "A.F.K.");
        assert_eq!(upperword("new york"), "New York");
        assert_eq!(upperword("TEST"), "Test");
    }

    #[crate::ctb_test]
    fn test_obscuredigits() {
        let t = "1234-1234-1234-1234";
        assert_eq!(obscuredigits(t, 4), "XXXX-XXXX-XXXX-1234");
        assert_eq!(obscuredigits("abc", 4), "abc");
    }

    #[crate::ctb_test]
    fn test_spacing() {
        assert_eq!(onespace("  a   b  c "), "a b c");
        assert_eq!(onewhitespace(" \ta\r\nb  \nc "), "a b c");
    }

    #[crate::ctb_test]
    fn test_quotes() {
        assert_eq!(quoted(r#"a"b"#), r#""a""b""#);
        assert_eq!(applescriptstring(r#"a"b\c"#), r#""a\"b\\c""#);
    }

    #[crate::ctb_test]
    fn test_replace_rep_multi_batch() -> Result<()> {
        assert_eq!(rep("*", 5)?, "*****");
        assert_eq!(replace("aabb", "aa", "x")?, "xbb");

        let out = replacemultiple(
            "Street City County State",
            "Street,City,County,State",
            "St,City,Cty,NJ",
            ',',
        )?;
        assert_eq!(out, "St City Cty NJ");

        let array = "foo-bar|baz-qux";
        let out2 = batchreplace("foo baz foo", array, '|', '-')?;
        assert_eq!(out2, "bar qux bar");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_strip_and_strip_variants() -> Result<()> {
        assert_eq!(strip("\t  hi \r\n"), "hi");
        assert_eq!(stripprintable("a\u{0001}b"), "ab");
        assert_eq!(striptoalpha("a1 b!ç"), "abç");
        assert_eq!(striptonum("a1 b2!3"), "123");

        assert_eq!(striphtmltags("a<b>c</b>d"), "acd");

        assert_eq!(stripchar("AZaz09..-_*", "AZaz09..")?, "AZaz09..");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_random_sanity() -> Result<()> {
        let c = randomletter("U")?;
        assert!(c.is_ascii_uppercase());

        let line = randomline("a\n\nb\n")?;
        assert!(line == "a" || line == "b");

        let w = randomword("one two three")?;
        assert!(w == "one" || w == "two" || w == "three");
        Ok(())
    }
}
