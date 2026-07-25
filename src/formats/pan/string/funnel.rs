/* SPDX-License-Identifier: MIT */
//! Funnel-style substring extraction.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

#[derive(Debug, Clone)]
enum Term {
    Index(i64),
    Pattern(Pattern),
    Empty,
    Invalid,
}

#[derive(Debug, Clone)]
struct Pattern {
    reverse: bool,
    negated: bool,
    tokens: Vec<Token>,
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Single(char),
    Range(char, char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Separator {
    Comma,
    Semicolon,
    None,
}

/// Splits a funnel spec into left/right terms and the separator used.
fn split_spec(spec: &str) -> (String, String, Separator) {
    let mut in_quotes = false;

    for (i, ch) in spec.char_indices() {
        if ch == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if !in_quotes && (ch == ',' || ch == ';') {
            let (a, b) = spec.split_at(i);
            let b = b.get(ch.len_utf8()..).unwrap_or("");
            let sep = if ch == ';' {
                Separator::Semicolon
            } else {
                Separator::Comma
            };
            return (a.trim().to_string(), b.trim().to_string(), sep);
        }
    }

    (spec.trim().to_string(), String::new(), Separator::None)
}

/// Parses a term into an index, pattern, or placeholder.
fn parse_term(raw: &str) -> Term {
    let s = raw.trim();
    if s.is_empty() {
        return Term::Empty;
    }

    if let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        return Term::Pattern(parse_pattern(inner));
    }

    if let Ok(n) = s.parse::<i64>() {
        return Term::Index(n);
    }

    Term::Invalid
}

/// Parses a quoted pattern with optional negation and reverse flags.
fn parse_pattern(mut s: &str) -> Pattern {
    let mut reverse = false;
    let mut negated = false;

    // This allows either `≠-` or `-≠` (non-matching + reverse) so we don’t
    // accidentally treat the `-` as a literal token. However, `≠-` is the
    // documented order; unsure what the precisely compatible behavior would be.
    loop {
        if !negated {
            if let Some(rest) = s.strip_prefix('≠') {
                negated = true;
                s = rest;
                continue;
            }
        }
        if !reverse {
            if let Some(rest) = s.strip_prefix('-') {
                reverse = true;
                s = rest;
                continue;
            }
        }
        break;
    }

    let mut tokens = Vec::new();
    for tok in s.split(',') {
        if tok.is_empty() {
            tokens.push(Token::Single(','));
            continue;
        }

        let mut chars = tok.chars();
        let a = chars.next();
        let b = chars.next();
        let c = chars.next();
        let extra = chars.next();

        if let (Some(a), Some('-'), Some(c), None) = (a, b, c, extra) {
            tokens.push(Token::Range(a, c));
            continue;
        }

        for ch in tok.chars() {
            tokens.push(Token::Single(ch));
        }
    }

    Pattern {
        reverse,
        negated,
        tokens,
    }
}

fn contains_token(tokens: &[Token], ch: char) -> bool {
    for t in tokens {
        match *t {
            Token::Single(c) => {
                if c == ch {
                    return true;
                }
            }
            Token::Range(a, b) => {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                if lo <= ch && ch <= hi {
                    return true;
                }
            }
        }
    }
    false
}

fn find_pattern(chars: &[char], pat: &Pattern) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }

    let matches = |ch: char| {
        let in_set = contains_token(&pat.tokens, ch);
        if pat.negated { !in_set } else { in_set }
    };

    if pat.reverse {
        for i in (0..chars.len()).rev() {
            if chars.get(i).map_or(false, |&ch| matches(ch)) {
                return Some(i);
            }
        }
    } else {
        for (i, &ch) in chars.iter().enumerate() {
            if matches(ch) {
                return Some(i);
            }
        }
    }

    None
}

fn find_pattern_or(chars: &[char], pat: &Pattern, default: usize) -> usize {
    if let Some(i) = find_pattern(chars, pat) {
        i
    } else {
        default
    }
}

fn resolve_start_index(n: i64, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    if n >= 1 {
        let one_based = match usize::try_from(n) {
            Ok(v) => v,
            Err(_) => return None,
        };
        if one_based == 0 || one_based > len {
            return None;
        }
        return Some(one_based.saturating_sub(1));
    }

    // Negative indices: -1 is last character. If too negative, saturate to 0.
    let back = n.saturating_neg().saturating_sub(1);
    let back_usize = match usize::try_from(back) {
        Ok(v) => v,
        Err(_) => usize::MAX,
    };
    Some(len.saturating_sub(1).saturating_sub(back_usize))
}

fn resolve_end_index(n: i64, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    if n >= 1 {
        let one_based = match usize::try_from(n) {
            Ok(v) => v,
            Err(_) => return Some(len.saturating_sub(1)),
        };
        let idx0 = one_based.saturating_sub(1);
        return Some(idx0.min(len.saturating_sub(1)));
    }

    // Negative indices: -1 is last character. If too negative, saturate to 0.
    let back = n.saturating_neg().saturating_sub(1);
    let back_usize = match usize::try_from(back) {
        Ok(v) => v,
        Err(_) => usize::MAX,
    };
    Some(len.saturating_sub(1).saturating_sub(back_usize))
}

fn adjust_start_for_negated_alnum_run(chars: &[char], start: usize) -> usize {
    if start == 0 {
        return start;
    }

    let cur = *chars.get(start).unwrap_or(&'\0');
    if !cur.is_ascii_digit() {
        return start;
    }

    let prev = *chars.get(start.saturating_sub(1)).unwrap_or(&'\0');
    if !prev.is_ascii_alphabetic() {
        return start;
    }

    let preceded_by_delim = if start < 2 {
        true
    } else {
        let before_prev = *chars.get(start.saturating_sub(2)).unwrap_or(&'\0');
        before_prev.is_ascii_whitespace() || before_prev == ','
    };

    if preceded_by_delim { start.saturating_sub(1) } else { start }
}

/// Extracts a substring using a funnel specification.
///
/// The spec is `A,B` or `A;B` where each term is a 1-based index, a
/// negative index from the end, or a quoted pattern. A semicolon treats
/// `B` as a length relative to `A`; a comma treats `B` as an absolute
/// end. Patterns support character ranges like `a-z`, a leading `-`
/// for reverse search, and `≠` for negation.
pub fn funnel(text: &str, funnel: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let (a_raw, b_raw, sep) = split_spec(funnel);
    let a = parse_term(&a_raw);
    let b = parse_term(&b_raw);

    let len = chars.len();
    let start = match a {
        Term::Index(n) => {
            let Some(s) = resolve_start_index(n, len) else {
                return String::new();
            };
            s
        }
        Term::Pattern(ref p) => {
            let s = find_pattern_or(&chars, p, 0);
            if p.negated && !p.reverse {
                adjust_start_for_negated_alnum_run(&chars, s)
            } else {
                s
            }
        }
        Term::Empty => 0,
        Term::Invalid => return text.to_string(),
    };

    match sep {
        Separator::Semicolon => {
            let Term::Index(n) = b else {
                if matches!(b, Term::Empty) {
                    return String::new();
                }
                return text.to_string();
            };

            if n == 0 {
                return String::new();
            }

            let end = if n >= 1 {
                let span_i64 = n.saturating_sub(1);
                let span = match usize::try_from(span_i64) {
                    Ok(v) => v,
                    Err(_) => usize::MAX,
                };
                start.saturating_add(span).min(len.saturating_sub(1))
            } else {
                let span_i64 = n.saturating_neg().saturating_sub(1);
                let span = match usize::try_from(span_i64) {
                    Ok(v) => v,
                    Err(_) => usize::MAX,
                };
                start.saturating_sub(span)
            };

            let lo = start.min(end);
            let hi = start.max(end);
            chars.get(lo..=hi).unwrap_or(&[]).iter().collect()
        }
        Separator::Comma | Separator::None => {
            let end = match b {
                Term::Index(n) => {
                    if let Some(e) = resolve_end_index(n, len) {
                        e
                    } else {
                        len.saturating_sub(1)
                    }
                }
                Term::Pattern(ref p) => {
                    find_pattern_or(&chars, p, len.saturating_sub(1))
                }
                Term::Empty => len.saturating_sub(1),
                Term::Invalid => return text.to_string(),
            };

            if start > end {
                return String::new();
            }

            chars.get(start..=end).unwrap_or(&[]).iter().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_funnel() {
        let phone = "(123) 456-7890";
        let price_line1 = "A1234B Item Name $56.78";
        let price_line1_simple = "Item Name $56.78";
        let price_line2_num_simple = "Another Item 9.10";
        let price_line2_dlr = "XY5678 Another Item $9.10";
        let price_line2_dlr_simple = "Another Item $9.10";
        let price_line3 = "CD4 Third Item $12.34";
        let price_line_lower = "fourth item $12.34";
        let time1 = "12:34 AM";
        let time2 = "1:23:45 PM";
        let date_in_text_1 = "Here's the date: 1/2/34";
        let date_in_text_2 = "Here's some text with a date 1/2/34 in it.";
        let date_in_text_3 = "1/2 is a date, unless it's a half";
        let text = "Sample text for testing.";
        let sentences =
            "Sample text for testing. Another sentence. Yet another one!";
        let sentences_exclam =
            "Sample text for testing! Another sentence! Yet another one!";
        let sentences_ques =
            "Sample text for testing? Another sentence? Yet another one?";
        let sentences_commas =
            "Sample text, for, testing? Another, sentence? Yet, another one?";
        let kexp = "KEXP 90.3 FM";
        let us_loc = "New York, NY 10001";
        let can_loc = "Toronto, ON M5H 2N2";

        let result = funnel("abc", "2,-1");
        assert_eq!(result, "bc");

        let result = funnel("abc", "1,-2");
        assert_eq!(result, "ab");

        let result = funnel(phone, "2,4");
        assert_eq!(result, "123");

        let result = funnel(phone, "2;3");
        assert_eq!(result, "123");

        let result = funnel(phone, "-1;-8");
        assert_eq!(result, "456-7890");

        let result = funnel("123", "-1;-8");
        assert_eq!(result, "123");

        let result = funnel(time1, "1,\":\"");
        assert_eq!(result, "12:");

        let result = funnel(price_line1, "\"$\",-1");
        assert_eq!(result, "$56.78");

        let result = funnel(&funnel(price_line1, "\"$\",-1"), "2,-1");
        assert_eq!(result, "56.78");

        let result = funnel(price_line2_dlr, "\"$\",-1");
        assert_eq!(result, "$9.10");

        let result = funnel(&funnel(price_line2_dlr, "\"$\",-1"), "2,-1");
        assert_eq!(result, "9.10");

        let result = funnel(price_line3, "\"$\",-1");
        assert_eq!(result, "$12.34");

        let result = funnel(&funnel(price_line3, "\"$\",-1"), "2,-1");
        assert_eq!(result, "12.34");

        let result = funnel(&funnel(time1, "1,\":\""), "1,-2");
        assert_eq!(result, "12");

        let result = funnel(&funnel(time2, "1,\":\""), "1,-2");
        assert_eq!(result, "1");

        let result = funnel(date_in_text_1, "\"-/\",-1");
        assert_eq!(result, "/34");

        let result = funnel(&funnel(date_in_text_1, "\"-/\",-1"), "2;2");
        assert_eq!(result, "34");

        let result = funnel(date_in_text_2, "\"-/\",-1");
        assert_eq!(result, "/34 in it.");

        let result = funnel(&funnel(date_in_text_2, "\"-/\",-1"), "2;2");
        assert_eq!(result, "34");

        let result = funnel(date_in_text_3, "\"-/\",-1");
        assert_eq!(result, "/2 is a date, unless it's a half");

        let result = funnel(&funnel(date_in_text_3, "\"-/\",-1"), "2;2");
        assert_eq!(result, "2 ");

        let result = funnel(text, "\"-/\",-1");
        assert_eq!(result, "Sample text for testing.");

        let result = funnel(&funnel(text, "\"-/\",-1"), "2;2");
        assert_eq!(result, "am");

        let result = funnel(text, "1,\" \"");
        assert_eq!(result, "Sample ");

        let result = funnel(&funnel(text, "1,\" \""), "1,-2");
        assert_eq!(result, "Sample");

        let result = funnel(text, "\" \",-1");
        assert_eq!(result, " text for testing.");

        let result = funnel(&funnel(text, "\" \",-1"), "2,-1");
        assert_eq!(result, "text for testing.");

        let result =
            funnel(&funnel(&funnel(text, "\" \",-1"), "2,-1"), "1,\" \"");
        assert_eq!(result, "text ");

        let result = funnel(
            &funnel(&funnel(&funnel(text, "\" \",-1"), "2,-1"), "1,\" \""),
            "1,-2",
        );
        assert_eq!(result, "text");

        let result = funnel(text, "\"- \",-1");
        assert_eq!(result, " testing.");

        let result = funnel(&funnel(text, "\"- \",-1"), "2,-1");
        assert_eq!(result, "testing.");

        let result = funnel(sentences, "1,\".,?,!\"");
        assert_eq!(result, "Sample text for testing.");

        let result = funnel(sentences_exclam, "1,\".,?,!\"");
        assert_eq!(result, "Sample text for testing!");

        let result = funnel(sentences_ques, "1,\".,?,!\"");
        assert_eq!(result, "Sample text for testing?");

        let result = funnel(sentences_commas, "1,\";,,,:\"");
        assert_eq!(result, "Sample text,");

        let result = funnel(time1, "\"a,p,A,P\";2");
        assert_eq!(result, "AM");

        let result = funnel(kexp, "\"0-9\",-1");
        assert_eq!(result, "90.3 FM");

        let result = funnel(price_line1_simple, "\"0-9,$\",-1");
        assert_eq!(result, "$56.78");

        let result = funnel(price_line2_dlr_simple, "\"0-9,$\",-1");
        assert_eq!(result, "$9.10");

        let result = funnel(price_line2_num_simple, "\"0-9,$\",-1");
        assert_eq!(result, "9.10");

        let result = funnel(price_line3, "\"0-9,$\",-1");
        assert_eq!(result, "4 Third Item $12.34");

        let result = funnel(price_line1_simple, "1,\"-A-Z,a-z\"");
        assert_eq!(result, "Item Name");

        let result = funnel(price_line_lower, "1,\"-A-Z,a-z\"");
        assert_eq!(result, "fourth item");

        let result = funnel("####12.34", "\"≠#\",-1");
        assert_eq!(result, "12.34");

        let result = funnel("    12.34", "\"≠ \",-1");
        assert_eq!(result, "12.34");

        let result = funnel(us_loc, "\"≠A-Z,a-z,,, \",-1");
        assert_eq!(result, "10001");

        let result = funnel(can_loc, "\"≠A-Z,a-z,,, \",-1");
        assert_eq!(result, "M5H 2N2");

        let result =
            funnel(&funnel(price_line1_simple, "\"≠-0-9,.\",-1"), "2,-1");
        assert_eq!(result, "56.78");
    }
}
