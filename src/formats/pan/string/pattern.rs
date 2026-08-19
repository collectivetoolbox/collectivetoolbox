/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Pan numeric pattern formatting.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use number_to_words::number_to_words;
use pluralizer::pluralize;
use regex::Regex;

use std::str::FromStr;

/// Formats `number` using the Pan pattern syntax.
///
/// Use `#` for digits, `§` for words, `¢` for cents digits, `~` for
/// pluralization, and an optional trailing `-` or parentheses to control
/// negative formatting.
pub fn pattern(number: f64, pat: &str) -> Result<String> {
    if !number.is_finite() {
        bail!("pattern(): number must be finite");
    }

    let (abs_value, core_pat, neg_style) = extract_negative_style(number, pat)?;

    let (rendered, is_one) = if core_pat.contains('§') {
        (
            format_words(abs_value, &core_pat)?,
            is_one_words(abs_value, &core_pat)?,
        )
    } else {
        let (prefix, numeric_pat, suffix) =
            split_prefix_numeric_suffix(&core_pat)?;

        if is_scientific_marker(&numeric_pat) {
            let precision = scientific_precision(&numeric_pat)?;
            let s = format_scientific(abs_value, precision)?;
            (format!("{prefix}{s}{suffix}"), is_one_from_rendered(&s))
        } else if numeric_pat.contains('.') || numeric_pat.contains(',') {
            let s = format_normal_numeric(abs_value, &numeric_pat)?;
            (format!("{prefix}{s}{suffix}"), is_one_from_rendered(&s))
        } else if numeric_pat.chars().all(|c| c == '#') {
            // A plain run of hashes (e.g. "#" or "#####") is a normal numeric
            // pattern: it may pad with leading zeros, but it must not fail when the
            // value has more digits than the pattern length.
            let s = format_normal_numeric(abs_value, &numeric_pat)?;
            (format!("{prefix}{s}{suffix}"), is_one_from_rendered(&s))
        } else {
            // Mixed literal/component patterns like "###-####".
            let s = format_component(abs_value, &numeric_pat)?;
            (format!("{prefix}{s}{suffix}"), is_one_from_rendered(&s))
        }
    };

    let mut out = apply_plural_suffix(rendered, is_one);

    out = match neg_style {
        NegStyle::Parens if number.is_sign_negative() && number != 0.0 => {
            format!("({out})")
        }
        NegStyle::TrailingMinus
            if number.is_sign_negative() && number != 0.0 =>
        {
            let mut s = out;
            s.push('-');
            s
        }
        NegStyle::Default if number.is_sign_negative() && number != 0.0 => {
            format!("-{out}")
        }
        _ => out,
    };

    Ok(out)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NegStyle {
    Default,
    TrailingMinus,
    Parens,
}

fn extract_negative_style(
    number: f64,
    pat: &str,
) -> Result<(f64, String, NegStyle)> {
    let abs_value = number.abs();
    let mut p = pat.to_string();

    // Parentheses indicate negative-only wrapping: (#.##)
    if let Some((open, close)) = find_wrapping_parens(&p) {
        let inner = p
            .get(open.saturating_add(1)..close)
            .context("Invalid parentheses range")?;
        let contains_placeholder =
            inner.chars().any(|c| matches!(c, '#' | '§' | '¢'));
        if contains_placeholder {
            let mut stripped = String::new();
            stripped.push_str(p.get(..open).context("Invalid prefix range")?);
            stripped.push_str(inner);
            stripped.push_str(
                p.get(close.saturating_add(1)..)
                    .context("Invalid suffix range")?,
            );
            return Ok((abs_value, stripped, NegStyle::Parens));
        }
    }

    // Trailing minus: #.##-
    if p.ends_with('-') && p.chars().any(|c| matches!(c, '#' | '§' | '¢')) {
        p.pop();
        return Ok((abs_value, p, NegStyle::TrailingMinus));
    }

    Ok((abs_value, p, NegStyle::Default))
}

fn find_wrapping_parens(s: &str) -> Option<(usize, usize)> {
    let open = s.find('(')?;
    let close_rel = s.get(open..)?.find(')')?;
    let close = open.saturating_add(close_rel);
    Some((open, close))
}

fn split_prefix_numeric_suffix(pat: &str) -> Result<(String, String, String)> {
    let first = pat
        .char_indices()
        .find(|(_, c)| matches!(c, '#' | '§' | '¢'))
        .map(|(i, _)| i)
        .context("pattern(): missing placeholder (#, §, or ¢)")?;

    let last_hash = pat
        .char_indices()
        .rev()
        .find(|(_, c)| matches!(c, '#' | '§' | '¢'))
        .map(|(i, _)| i)
        .context("pattern(): missing placeholder (#, §, or ¢)")?;

    // Extend numeric region by a single scientific marker if it immediately
    // follows the last placeholder, e.g. "#.#E kg".
    let after_last = last_hash
        .checked_add(1)
        .context("pattern(): invalid placeholder index")?;
    let mut end = last_hash;

    if let Some(next) = pat
        .get(after_last..)
        .context("Invalid index")?
        .chars()
        .next()
    {
        if matches!(next, 'e' | 'E') {
            end = after_last;
        }
    }

    let prefix = pat
        .get(..first)
        .context("Invalid prefix index")?
        .to_string();
    let numeric = pat
        .get(first..=end)
        .context("Invalid numeric index")?
        .to_string();
    // Reason for fallback: pattern without trailing suffix after numeric part defaults suffix string to empty
    let suffix = pat.get(end.saturating_add(1)..).unwrap_or("").to_string();

    Ok((prefix, numeric, suffix))
}

fn apply_plural_suffix(s: String, is_one: bool) -> String {
    if !s.contains('~') {
        return s;
    }

    let count = if is_one { 1 } else { 2 };

    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(tilde_pos) = rest.find('~') {
        let (before_tilde, with_tilde) = rest.split_at(tilde_pos);

        // Reason for fallback: word start search without preceding whitespace defaults word boundary offset to 0
        let word_start = before_tilde
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |i| i.saturating_add(1));
        let (prefix, word) = before_tilde.split_at(word_start);

        out.push_str(prefix);
        if !word.is_empty() {
            out.push_str(&pluralize(word, count, false));
        }

        // Skip the tilde.
        // Reason for fallback: rest slice stripping tilde at end of string defaults remaining rest to empty string
        rest = with_tilde.get(1..).unwrap_or("");
    }

    out.push_str(rest);
    out
}

fn is_scientific_marker(numeric_pat: &str) -> bool {
    numeric_pat.ends_with('e') || numeric_pat.ends_with('E')
}

fn scientific_precision(numeric_pat: &str) -> Result<usize> {
    let mut chars = numeric_pat.chars().peekable();

    // Count # after '.' until the trailing e/E (if any).
    for c in chars.by_ref() {
        if c == '.' {
            break;
        }
    }
    let mut n = 0usize;
    while let Some(&c) = chars.peek() {
        if matches!(c, 'e' | 'E') {
            break;
        }
        if c == '#' {
            n = n.saturating_add(1);
        }
        chars.next();
    }
    Ok(n)
}

fn format_scientific(value: f64, precision: usize) -> Result<String> {
    let raw = format!("{value:.precision$e}");
    let (mantissa, exp_part) = raw
        .split_once('e')
        .context("pattern(): unexpected scientific formatting")?;

    let exp_i32 = i32::from_str(exp_part)
        .context("pattern(): invalid exponent in scientific formatting")?;

    let exp = if exp_i32 >= 0 {
        format!("+{exp_i32}")
    } else {
        exp_i32.to_string()
    };

    Ok(format!("{mantissa}e{exp}"))
}

fn format_normal_numeric(value: f64, numeric_pat: &str) -> Result<String> {
    let has_comma = numeric_pat.contains(',');

    let (min_int_digits, decimals) = min_int_and_decimals(numeric_pat)?;
    let formatted = format!("{value:.decimals$}");
    let (mut int_part, frac_part) = split_int_frac(&formatted, decimals)?;

    if int_part.len() < min_int_digits {
        let mut padded = String::new();
        padded.extend(std::iter::repeat_n(
            '0',
            min_int_digits.saturating_sub(int_part.len()),
        ));
        padded.push_str(&int_part);
        int_part = padded;
    }

    if has_comma {
        int_part = add_commas(&int_part);
    }

    if decimals == 0 {
        return Ok(int_part);
    }

    Ok(format!("{int_part}.{frac_part}"))
}

fn min_int_and_decimals(numeric_pat: &str) -> Result<(usize, usize)> {
    let mut before_dot = 0usize;
    let mut after_dot = 0usize;
    let mut seen_dot = false;

    for c in numeric_pat.chars() {
        match c {
            '.' => seen_dot = true,
            '#' if !seen_dot => before_dot = before_dot.saturating_add(1),
            '#' if seen_dot => after_dot = after_dot.saturating_add(1),
            _ => {}
        }
    }

    Ok((before_dot, after_dot))
}

fn split_int_frac(s: &str, decimals: usize) -> Result<(String, String)> {
    if decimals == 0 {
        return Ok((s.to_string(), String::new()));
    }

    let (int_part, frac_part) = s
        .split_once('.')
        .context("pattern(): expected decimal point in formatted number")?;

    Ok((int_part.to_string(), frac_part.to_string()))
}

fn add_commas(int_part: &str) -> String {
    let mut groups: Vec<String> = Vec::new();
    let mut digits: Vec<char> = int_part.chars().collect();

    while !digits.is_empty() {
        let take = digits.len().min(3);
        let start = digits.len().saturating_sub(take);
        let chunk: String = digits.drain(start..).collect();
        groups.push(chunk);
    }

    groups.reverse();
    groups.join(",")
}

fn format_component(value: f64, numeric_pat: &str) -> Result<String> {
    let n_hashes = numeric_pat.chars().filter(|&c| c == '#').count();
    if n_hashes == 0 {
        return Ok(numeric_pat.to_string());
    }

    let digits = format!("{value:.0}");
    let mut padded = digits.clone();
    if padded.len() < n_hashes {
        let mut s = String::new();
        s.extend(std::iter::repeat_n(
            '0',
            n_hashes.saturating_sub(padded.len()),
        ));
        s.push_str(&padded);
        padded = s;
    }

    if padded.len() > n_hashes {
        bail!("pattern(): value has more digits than component pattern allows");
    }

    let mut it = padded.chars();
    let mut out = String::new();
    for c in numeric_pat.chars() {
        if c == '#' {
            let d = it
                .next()
                .context("pattern(): internal digit mapping error")?;
            out.push(d);
        } else {
            out.push(c);
        }
    }

    Ok(out)
}

fn is_one_from_rendered(rendered_numeric: &str) -> bool {
    // Normalize: trim and remove grouping commas.
    let s = rendered_numeric.trim().replace(',', "");

    // Exact integer 1.
    if s == "1" {
        return true;
    }

    // Decimal forms like "1.0", "1.00", or even "1." should be considered 1.
    if let Some(frac) = s.strip_prefix("1.") {
        // If everything after the decimal point is zeros (or empty), it's 1.
        return frac.chars().all(|c| c == '0');
    }

    false
}

fn is_one_words(value: f64, pat: &str) -> Result<bool> {
    let cents_run = cents_run_len(pat)?;
    if cents_run == 0 {
        return Ok(format!("{value:.0}") == "1");
    }

    let rounded = format!("{value:.cents_run$}");
    let (int_part, _) = split_int_frac(&rounded, cents_run)?;
    Ok(int_part == "1")
}

fn cents_run_len(pat: &str) -> Result<usize> {
    let mut run_len = 0usize;
    let mut seen_run = false;

    let mut it = pat.chars().peekable();
    while let Some(c) = it.next() {
        if c != '¢' {
            continue;
        }

        if seen_run && run_len == 0 {
            bail!("pattern(): multiple ¢ runs are not supported");
        }

        seen_run = true;
        run_len = 1;

        while matches!(it.peek(), Some('¢')) {
            it.next();
            run_len = run_len.saturating_add(1);
        }

        // If we find another run later, bail for now.
        for rest in it {
            if rest == '¢' {
                bail!("pattern(): multiple ¢ runs are not supported");
            }
        }
        break;
    }

    Ok(run_len)
}

fn format_words(value: f64, pat: &str) -> Result<String> {
    let cents_len = cents_run_len(pat)?;
    let rounded = if cents_len == 0 {
        format!("{value:.0}")
    } else {
        format!("{value:.cents_len$}")
    };

    let (int_part, frac_part) = if cents_len == 0 {
        (rounded, String::new())
    } else {
        split_int_frac(&rounded, cents_len)?
    };

    let int_f64 =
        f64::from_str(&int_part).context("pattern(): invalid integer part")?;
    let mut words = number_to_words(int_f64, true);
    words = normalize_words_output(&words)?;

    let mut out = pat.replacen('§', &words, 1);

    if cents_len > 0 {
        let cents_pat = "¢".repeat(cents_len);
        out = out.replace(&cents_pat, &frac_part);
    }

    Ok(out)
}

fn normalize_words_output(s: &str) -> Result<String> {
    let mut out = s.trim().to_string();

    // Some implementations add a trailing marker like "*".
    out = out.trim_end_matches('*').trim().to_string();

    // Use spaces rather than hyphens, and omit commas.
    out = out.replace('-', " ");
    out = out.replace(',', "");

    // If the words converter appends "and 00/100" for integer values, strip it.
    let re = Regex::new(r"\s+and\s+0+/100$")?;
    out = re.replace(&out, "").to_string();

    // Collapse whitespace.
    let ws = Regex::new(r"\s+")?;
    out = ws.replace_all(out.trim(), " ").to_string();

    Ok(out)
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
    fn examples_fixed_decimal_and_commas() -> Result<()> {
        assert_eq!(pattern(1234.56, "#")?, "1235");
        assert_eq!(pattern(1234.56, "#.#")?, "1234.6");
        assert_eq!(pattern(1234.56, "#.##")?, "1234.56");
        assert_eq!(pattern(1234.56, "#.####")?, "1234.5600");

        assert_eq!(pattern(1234.56, "#,.##")?, "1,234.56");
        assert_eq!(pattern(1234.56, "$#,.##")?, "$1,234.56");
        assert_eq!(pattern(1234.56, "#,.## kg")?, "1,234.56 kg");
        Ok(())
    }

    #[crate::ctb_test]
    fn examples_scientific() -> Result<()> {
        assert_eq!(pattern(1234.56, "#e")?, "1e+3");
        assert_eq!(pattern(1234.56, "#.#E")?, "1.2e+3");
        assert_eq!(pattern(1234.56, "#.######E")?, "1.234560e+3");
        assert_eq!(pattern(1234.56, "#.#E kg")?, "1.2e+3 kg");
        Ok(())
    }

    #[crate::ctb_test]
    fn examples_negative() -> Result<()> {
        assert_eq!(pattern(-1234.56, "#.##")?, "-1234.56");
        assert_eq!(pattern(-1234.56, "#.##-")?, "1234.56-");
        assert_eq!(pattern(-1234.56, "(#.##)")?, "(1234.56)");
        assert_eq!(pattern(1234.56, "(#.##)")?, "1234.56");
        Ok(())
    }

    #[crate::ctb_test]
    fn examples_leading_zeros_and_components() -> Result<()> {
        assert_eq!(pattern(123.0, "#####")?, "00123");
        assert_eq!(pattern(1234.0, "#####")?, "01234");
        assert_eq!(pattern(12345.0, "#####")?, "12345");

        assert_eq!(pattern(219_204_349.0, "###-##-####")?, "219-20-4349");
        assert_eq!(pattern(5_293_672.0, "###-####")?, "529-3672");
        assert_eq!(pattern(241_018.0, "L## R## L##")?, "L24 R10 L18");
        Ok(())
    }

    #[crate::ctb_test]
    fn examples_plural_suffix_and_words() -> Result<()> {
        assert_eq!(pattern(1.0, "# mile~")?, "1 mile");
        assert_eq!(pattern(5.0, "# mile~")?, "5 miles");
        assert_eq!(pattern(5.0, "# goose~")?, "5 geese");

        assert_eq!(pattern(312.0, "§")?, "Three hundred twelve");
        assert_eq!(
            pattern(42.29, "§ dollar~ and ¢¢/100")?,
            "Forty two dollars and 29/100"
        );
        assert_eq!(
            pattern(42.09, "§ dollar~ and ¢¢/100")?,
            "Forty two dollars and 09/100"
        );
        Ok(())
    }
}
