// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from text-clipper: MIT
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

// Copyright (c) 2016-2024 Arend van Beelen jr., Speakap B.V.

// See additional licensing details at end of file.

//! Text clipping utility for HTML and plain text.
//!
//! LLM-assisted port of <https://github.com/arendjr/text-clipper>

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub fn truncate_html(html: &str, max_len: u64) -> String {
    clip(
        html,
        max_len,
        Some(&html_options(ClipHtmlOptions::default())),
    )
}

pub fn truncate_text(text: &str, max_len: u64) -> String {
    clip(text, max_len, None)
}

/// Clips a string to a maximum length (in UTF-16 code units), optionally
/// preserving valid HTML when clipping HTML input.
pub fn clip(text: &str, len: u64, options: Option<&ClipOptions>) -> String {
    if text.is_empty() {
        return String::new();
    }

    let Ok(max_len) = usize::try_from(len) else {
        warn_fmt!("clip: len {len} does not fit in usize; returning input");
        return text.to_string();
    };

    let (force_html, common, html_options) = match options {
        Some(ClipOptions::Html(html)) => (true, &html.common, Some(html)),
        Some(ClipOptions::Plain(plain)) => (false, &plain.common, None),
        None => (false, &CommonClipOptions::default(), None),
    };

    let mut html_mode = force_html;
    if !html_mode {
        // Heuristic: treat strings containing '<' as HTML unless explicitly
        // forced to plain text via ClipOptions::Plain.
        html_mode = text.contains('<') || looks_like_html_entity_encoded(text);
    }
    if let Some(true) = common.html {
        html_mode = true;
    }

    // Reason for fallback: when optional indicator string is omitted, unicode ellipsis "…" is used as default truncation indicator.
    let indicator = common
        .indicator
        .clone()
        .unwrap_or_else(|| "\u{2026}".to_string());
    // Reason for fallback: when optional insert_indicator_at_linebreak option is omitted, true is default behavior.
    let insert_indicator_at_linebreak =
        common.insert_indicator_at_linebreak.unwrap_or(true);
    // Reason for fallback: when optional break_words option is omitted, false is default behavior.
    let break_words = common.break_words.unwrap_or(false);
    // Reason for fallback: when optional max_lines option is omitted, usize::MAX represents unlimited lines.
    let max_lines = common
        .max_lines
        .and_then(|v| usize::try_from(v).ok())
        .unwrap_or(usize::MAX);

    if html_mode {
        // Reason for fallback: when optional image_weight option is omitted, 2 is standard HTML element weight default.
        let image_weight =
            html_options.and_then(|h| h.image_weight).unwrap_or(2);
        let strip_tags = html_options.and_then(|h| h.strip_tags.clone());

        if let Ok(result) = clip_html(
            text,
            max_len,
            &indicator,
            insert_indicator_at_linebreak,
            max_lines,
            break_words,
            image_weight,
            strip_tags,
        ) {
            result
        } else {
            clip_plain_text(
                text,
                max_len,
                &indicator,
                insert_indicator_at_linebreak,
                max_lines,
                break_words,
            )
        }
    } else {
        clip_plain_text(
            text,
            max_len,
            &indicator,
            insert_indicator_at_linebreak,
            max_lines,
            break_words,
        )
    }
}

fn looks_like_html_entity_encoded(text: &str) -> bool {
    // Detect HTML-encoded entities like "&lt;" / "&amp;" so that we count them
    // as a single character and preserve encoding.
    //
    // This is intentionally conservative: it requires a ';' terminator and at
    // least one ASCII alphanumeric between '&' and ';'.
    let bytes = text.as_bytes();
    let mut i: usize = 0;
    while let Some(&byte) = bytes.get(i) {
        if byte != b'&' {
            i = i.saturating_add(1);
            continue;
        }

        let mut j = i.saturating_add(1);
        let mut saw_alnum = false;
        while let Some(&b) = bytes.get(j) {
            if b == b';' {
                if saw_alnum {
                    return true;
                }
                break;
            }
            if b.is_ascii_digit()
                || b.is_ascii_uppercase()
                || b.is_ascii_lowercase()
            {
                saw_alnum = true;
                j = j.saturating_add(1);
                continue;
            }
            break;
        }

        i = i.saturating_add(1);
    }
    false
}

const NEWLINE_CHAR_CODE: u16 = 10; // '\n'
const EXCLAMATION_CHAR_CODE: u16 = 33; // '!'
const DOUBLE_QUOTE_CHAR_CODE: u16 = 34; // '"'
const AMPERSAND_CHAR_CODE: u16 = 38; // '&'
const SINGLE_QUOTE_CHAR_CODE: u16 = 39; // '\''
const FORWARD_SLASH_CHAR_CODE: u16 = 47; // '/'
const SEMICOLON_CHAR_CODE: u16 = 59; // ';'
const TAG_OPEN_CHAR_CODE: u16 = 60; // '<'
const EQUAL_SIGN_CHAR_CODE: u16 = 61; // '='
const TAG_CLOSE_CHAR_CODE: u16 = 62; // '>'

const COMMENT_START: &[u16] = &[60, 33, 45, 45]; // "<!--"
const COMMENT_END: &[u16] = &[45, 45, 62]; // "-->"
const CDATA_START: &[u16] = &[60, 33, 91, 67, 68, 65, 84, 65, 91]; // "<![CDATA["
const CDATA_END: &[u16] = &[93, 93, 62]; // "]] >"

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input",
    "keygen", "link", "meta", "param", "source", "track", "wbr",
];

const BLOCK_ELEMENTS: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "canvas",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "li",
    "main",
    "nav",
    "noscript",
    "ol",
    "output",
    "p",
    "pre",
    "section",
    "table",
    "tbody",
    "tfoot",
    "thead",
    "tr",
    "ul",
    "video",
];

const UNBREAKABLE_ELEMENTS: &[&str] = &["audio", "math", "svg", "video"];

fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag)
}

fn is_block_element(tag: &str) -> bool {
    BLOCK_ELEMENTS.contains(&tag)
}

fn is_unbreakable_element(tag: &str) -> bool {
    UNBREAKABLE_ELEMENTS.contains(&tag)
}

fn any_unbreakable_open(tag_stack: &[String]) -> bool {
    UNBREAKABLE_ELEMENTS
        .iter()
        .any(|t| tag_stack.iter().any(|s| s == t))
}

fn is_high_surrogate(unit: u16) -> bool {
    (unit & 0xfc00) == 0xd800
}

fn is_low_surrogate(unit: u16) -> bool {
    (unit & 0xfc00) == 0xdc00
}

fn is_white_space(unit: u16) -> bool {
    unit == 9 || unit == 10 || unit == 12 || unit == 13 || unit == 32
}

fn is_character_reference_character(unit: u16) -> bool {
    (48..=57).contains(&unit)
        || (65..=90).contains(&unit)
        || (97..=122).contains(&unit)
}

fn encode_utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn units_to_string(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

fn find_subsequence(
    haystack: &[u16],
    needle: &[u16],
    from: usize,
) -> Option<usize> {
    if needle.is_empty() {
        return Some(from.min(haystack.len()));
    }
    if from > haystack.len() {
        return None;
    }
    let remaining = haystack.len().saturating_sub(from);
    if needle.len() > remaining {
        return None;
    }
    let end = haystack.len().saturating_sub(needle.len());
    let mut i = from;
    while i <= end {
        if haystack.get(i..i.saturating_add(needle.len())) == Some(needle) {
            return Some(i);
        }
        i = i.saturating_add(1);
    }
    None
}

fn index_of_white_space(units: &[u16], from: usize) -> usize {
    let mut i = from;
    while let Some(&u) = units.get(i) {
        if is_white_space(u) {
            return i;
        }
        i = i.saturating_add(1);
    }
    units.len()
}

fn should_simplify_white_space(tag_stack: &[String]) -> bool {
    for tag in tag_stack.iter().rev() {
        if tag == "li" || tag == "td" {
            return false;
        }
        if tag == "ol" || tag == "table" || tag == "ul" {
            return true;
        }
    }
    false
}

fn simplify_white_space(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut in_ws = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(ch);
            in_ws = false;
        }
    }
    out
}

fn search_next_char_of_interest(
    units: &[u16],
    from: usize,
    include_newline: bool,
) -> Option<usize> {
    let mut i = from;
    while let Some(&u) = units.get(i) {
        if u == TAG_OPEN_CHAR_CODE || u == AMPERSAND_CHAR_CODE {
            return Some(i);
        }
        if include_newline && u == NEWLINE_CHAR_CODE {
            return Some(i);
        }
        if is_high_surrogate(u) {
            return Some(i);
        }
        i = i.saturating_add(1);
    }
    None
}

fn has_non_whitespace(units: &[u16], from: usize) -> bool {
    let mut i = from;
    while let Some(&u) = units.get(i) {
        if !is_white_space(u) {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}

fn has_html_non_whitespace(units: &[u16], from: usize) -> bool {
    let mut i = from;
    while let Some(&u) = units.get(i) {
        if u == TAG_OPEN_CHAR_CODE {
            i = i.saturating_add(1);
            while units.get(i).is_some_and(|&x| is_white_space(x)) {
                i = i.saturating_add(1);
            }

            if units.get(i) == Some(&FORWARD_SLASH_CHAR_CODE) {
                i = i.saturating_add(1);
                while units.get(i).is_some_and(|&x| x != TAG_CLOSE_CHAR_CODE) {
                    i = i.saturating_add(1);
                }
            } else {
                return true;
            }
        } else if !is_white_space(u) {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}

fn is_line_break(units: &[u16], index: usize) -> bool {
    let Some(&u) = units.get(index) else {
        return false;
    };
    match u {
        NEWLINE_CHAR_CODE => true,
        TAG_OPEN_CHAR_CODE => {
            // JS: /^<(block|br)[\t\n\f\r ]*\/?>/i
            let mut i = index.saturating_add(1);
            if i >= units.len() {
                return false;
            }

            // Read tag name until whitespace, '/', or '>'.
            let name_start = i;
            while let Some(&u) = units.get(i) {
                if is_white_space(u)
                    || u == FORWARD_SLASH_CHAR_CODE
                    || u == TAG_CLOSE_CHAR_CODE
                {
                    break;
                }
                i = i.saturating_add(1);
            }
            if i == name_start {
                return false;
            }
            #[allow(
                clippy::expect_used,
                reason = "name_start < i <= units.len() guaranteed by loop termination"
            )]
            let tag = units
                .get(name_start..i)
                .map(units_to_string)
                .expect("name_start < i <= units.len() is in bounds")
                .to_ascii_lowercase();
            if !(is_block_element(&tag) || tag == "br") {
                return false;
            }

            while units.get(i).is_some_and(|&u| is_white_space(u)) {
                i = i.saturating_add(1);
            }
            if units.get(i) == Some(&FORWARD_SLASH_CHAR_CODE) {
                i = i.saturating_add(1);
            }
            units.get(i) == Some(&TAG_CLOSE_CHAR_CODE)
        }
        _ => false,
    }
}

fn take_char_at(units: &[u16], index: usize) -> Vec<u16> {
    let Some(&first) = units.get(index) else {
        return Vec::new();
    };
    if is_high_surrogate(first) {
        if let Some(&next) = units.get(index.saturating_add(1)) {
            if is_low_surrogate(next) {
                return vec![first, next];
            }
        }
    }
    vec![first]
}

fn take_html_char_at(units: &[u16], index: usize) -> Vec<u16> {
    let mut taken = take_char_at(units, index);
    if taken.first() == Some(&AMPERSAND_CHAR_CODE) {
        let mut j = index.saturating_add(1);
        while let Some(&u) = units.get(j) {
            if is_character_reference_character(u) {
                taken.push(u);
                j = j.saturating_add(1);
            } else if u == SEMICOLON_CHAR_CODE {
                taken.push(u);
                break;
            } else {
                break;
            }
        }
    }
    taken
}

fn should_strip(strip_tags: Option<&StripTags>, tag_name: &str) -> bool {
    let Some(strip_tags) = strip_tags else {
        return false;
    };
    match strip_tags {
        StripTags::All(true) => true,
        StripTags::All(false) => false,
        StripTags::Tags(tags) => {
            if tags.is_empty() {
                return false;
            }
            tags.iter().any(|t| t == tag_name)
        }
    }
}

fn pop_tag_stack(
    mut result: String,
    tag_stack: &mut Vec<String>,
    strip_tags: Option<&StripTags>,
) -> String {
    while let Some(tag) = tag_stack.pop() {
        if !should_strip(strip_tags, &tag) {
            result.push_str("</");
            result.push_str(&tag);
            result.push('>');
        }
    }
    result
}

#[expect(clippy::too_many_lines, reason = "+/- more readable")]
fn clip_plain_text(
    text: &str,
    max_len: usize,
    indicator: &str,
    insert_indicator_at_linebreak: bool,
    max_lines: usize,
    break_words: bool,
) -> String {
    let units = encode_utf16(text);
    let indicator_units = encode_utf16(indicator);

    let mut num_chars = indicator_units.len();
    let mut num_lines: usize = 1;

    let mut i: usize = 0;
    while i < units.len() {
        num_chars = num_chars.saturating_add(1);
        if num_chars > max_len {
            break;
        }

        let Some(&u) = units.get(i) else {
            break;
        };
        if u == NEWLINE_CHAR_CODE {
            num_lines = num_lines.saturating_add(1);
            if num_lines > max_lines {
                break;
            }
        } else if is_high_surrogate(u) {
            if let Some(&next) = units.get(i.saturating_add(1)) {
                if is_low_surrogate(next) {
                    i = i.saturating_add(1);
                }
            }
        }

        i = i.saturating_add(1);
    }

    if num_chars > max_len {
        let mut next_char = take_char_at(&units, i);
        if !indicator_units.is_empty() {
            let peek_index = i.saturating_add(next_char.len());
            if peek_index == units.len() {
                return text.to_string();
            }
            if let Some(&u) = units.get(peek_index) {
                if u == NEWLINE_CHAR_CODE {
                    let insert_indicator = insert_indicator_at_linebreak
                        && has_non_whitespace(&units, peek_index);
                    #[allow(
                        clippy::expect_used,
                        reason = "peek_index < units.len() checked on line 543"
                    )]
                    let mut out = units
                        .get(..peek_index)
                        .expect("peek_index < units.len() is in bounds")
                        .to_vec();
                    if insert_indicator {
                        out.extend_from_slice(&indicator_units);
                    }
                    return units_to_string(&out);
                }
            }
        }

        if !break_words {
            let backtrack_start = i.saturating_sub(indicator_units.len());
            let mut j = backtrack_start;
            loop {
                let Some(&u) = units.get(j) else {
                    break;
                };
                if u == NEWLINE_CHAR_CODE {
                    i = j;
                    next_char = vec![NEWLINE_CHAR_CODE];
                    break;
                }
                if is_white_space(u) {
                    i = j.saturating_add(usize::from(
                        !indicator_units.is_empty(),
                    ));
                    break;
                }
                if j == 0 {
                    break;
                }
                j = j.saturating_sub(1);
            }
        }

        let next_is_newline = next_char.first() == Some(&NEWLINE_CHAR_CODE);
        let insert_indicator = (insert_indicator_at_linebreak
            || !next_is_newline)
            && has_non_whitespace(&units, i);

        while i > 0 {
            if let Some(&prev) = units.get(i.saturating_sub(1)) {
                if is_white_space(prev) {
                    i = i.saturating_sub(1);
                    continue;
                }
            }
            break;
        }

        #[allow(
            clippy::expect_used,
            reason = "i <= units.len() invariant in loop"
        )]
        let mut out = units
            .get(..i)
            .expect("i <= units.len() is in bounds")
            .to_vec();
        if insert_indicator {
            out.extend_from_slice(&indicator_units);
        }
        return units_to_string(&out);
    }

    if num_lines > max_lines {
        let insert_indicator =
            insert_indicator_at_linebreak && has_non_whitespace(&units, i);

        while i > 0 {
            if let Some(&prev) = units.get(i.saturating_sub(1)) {
                if is_white_space(prev) {
                    i = i.saturating_sub(1);
                    continue;
                }
            }
            break;
        }

        #[allow(
            clippy::expect_used,
            reason = "i <= units.len() invariant in loop"
        )]
        let mut out = units
            .get(..i)
            .expect("i <= units.len() is in bounds")
            .to_vec();
        if insert_indicator {
            out.extend_from_slice(&indicator_units);
        }
        return units_to_string(&out);
    }

    text.to_string()
}

#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    reason = "complex HTML structure parsing logic"
)]
fn clip_html(
    text: &str,
    max_len: usize,
    indicator: &str,
    insert_indicator_at_linebreak: bool,
    max_lines: usize,
    break_words: bool,
    image_weight: u32,
    strip_tags: Option<StripTags>,
) -> Result<String> {
    let indicator_units = encode_utf16(indicator);
    let mut units = encode_utf16(text);

    let mut num_chars = indicator_units.len();
    let mut num_lines: usize = 1;
    let image_weight_usize = usize::try_from(image_weight)
        .context("image_weight does not fit in usize")?;

    let mut tag_stack: Vec<String> = Vec::new();
    let mut i: usize = 0;
    let mut unbreakable_element_index: Option<usize> = None;

    while i < units.len() {
        let will_simplify = should_simplify_white_space(&tag_stack);
        let include_newline =
            unbreakable_element_index.is_none() && !will_simplify;
        let next_index =
            search_next_char_of_interest(&units, i, include_newline);

        let next_block_size = match next_index {
            Some(idx) => idx.saturating_sub(i),
            None => units.len().saturating_sub(i),
        };

        if unbreakable_element_index.is_none() {
            if will_simplify {
                let block_units = if next_index.is_none() {
                    &units[i..]
                } else {
                    &units[i..i.saturating_add(next_block_size)]
                };
                let mut simplified =
                    simplify_white_space(&units_to_string(block_units));

                if let Some(current_tag) = tag_stack.last() {
                    if should_strip(strip_tags.as_ref(), current_tag) {
                        let insert_space_before = i > 0
                            && units
                                .get(i.saturating_sub(1))
                                .copied()
                                .is_some_and(|u| !is_white_space(u));
                        let insert_space_after = units
                            .get(i.saturating_add(next_block_size))
                            .copied()
                            .is_some_and(|u| !is_white_space(u));

                        if !simplified.is_empty() {
                            if insert_space_before {
                                simplified = format!(" {simplified}");
                            }
                            if insert_space_after {
                                simplified.push(' ');
                            }
                        } else if insert_space_before && insert_space_after {
                            simplified = " ".to_string();
                        }

                        let simplified_units = encode_utf16(&simplified);
                        let replace_end = i.saturating_add(next_block_size);
                        units.splice(
                            i..replace_end,
                            simplified_units.iter().copied(),
                        );
                        // Length changed; next_block_size becomes simplified length.
                        let new_block_size = simplified.encode_utf16().count();
                        num_chars = num_chars.saturating_add(new_block_size);
                        if num_chars > max_len {
                            break;
                        }
                        i = i.saturating_add(new_block_size);
                    } else {
                        let simplified_len = simplified.encode_utf16().count();
                        num_chars = num_chars.saturating_add(simplified_len);
                        if num_chars > max_len {
                            break;
                        }
                        i = i.saturating_add(next_block_size);
                    }
                } else {
                    let simplified_len = simplified.encode_utf16().count();
                    num_chars = num_chars.saturating_add(simplified_len);
                    if num_chars > max_len {
                        break;
                    }
                    i = i.saturating_add(next_block_size);
                }
            } else {
                num_chars = num_chars.saturating_add(next_block_size);
                if num_chars > max_len {
                    let next_block_size_i64 = i64::try_from(next_block_size)
                        .context("next_block_size does not fit in i64")?;
                    let num_chars_i64 = i64::try_from(num_chars)
                        .context("num_chars does not fit in i64")?;
                    let max_len_i64 = i64::try_from(max_len)
                        .context("max_len does not fit in i64")?;
                    // Reason for fallback: if arithmetic overflow occurs during clip index calculation, defaulting to 0 safely resets clip position to start of string.
                    let new_i = i64::try_from(i)?
                        .checked_add(next_block_size_i64)
                        .and_then(|v| v.checked_sub(num_chars_i64))
                        .and_then(|v| v.checked_add(max_len_i64))
                        .unwrap_or(0);
                    i = usize::try_from(new_i.max(0))
                        .context("clip index does not fit in usize")?;
                    break;
                }
                i = i.saturating_add(next_block_size);
            }
        } else {
            i = i.saturating_add(next_block_size);
        }

        if next_index.is_none() {
            break;
        }

        let Some(&u) = units.get(i) else {
            break;
        };
        if u == TAG_OPEN_CHAR_CODE {
            // Reason for fallback: if i + 1 is past the end of units vector, 0 is returned which is not '!' (EXCLAMATION_CHAR_CODE), avoiding false positive special tag match.
            let next_u = units.get(i.saturating_add(1)).copied().unwrap_or(0);
            let is_special_tag = next_u == EXCLAMATION_CHAR_CODE;
            if is_special_tag {
                // Comment: <!-- ... -->
                if units
                    .get(i..i.saturating_add(COMMENT_START.len()))
                    .is_some_and(|s| s == COMMENT_START)
                {
                    let search_from = i.saturating_add(COMMENT_START.len());
                    let Some(end_idx) =
                        find_subsequence(&units, COMMENT_END, search_from)
                    else {
                        bail!("Invalid HTML: {text}");
                    };
                    i = end_idx.saturating_add(COMMENT_END.len());
                    continue;
                }

                // CDATA: <![CDATA[ ... ]]>
                if units
                    .get(i..i.saturating_add(CDATA_START.len()))
                    .is_some_and(|s| s == CDATA_START)
                {
                    let search_from = i.saturating_add(CDATA_START.len());
                    let Some(end_idx) =
                        find_subsequence(&units, CDATA_END, search_from)
                    else {
                        bail!("Invalid HTML: {text}");
                    };
                    i = end_idx.saturating_add(CDATA_END.len());
                    continue;
                }
            }

            {
                let is_end_tag = next_u == FORWARD_SLASH_CHAR_CODE;
                if num_chars == max_len && !is_end_tag {
                    num_chars = num_chars.saturating_add(1);
                    break;
                }

                let mut attribute_quote: u16 = 0;
                let mut end_index = i;
                let mut is_attribute_value = false;

                loop {
                    end_index = end_index.saturating_add(1);
                    if end_index >= units.len() {
                        bail!("Invalid HTML: {text}");
                    }

                    let Some(&cu) = units.get(end_index) else {
                        break;
                    };
                    if is_attribute_value {
                        if attribute_quote != 0 {
                            if cu == attribute_quote {
                                is_attribute_value = false;
                            }
                        } else if is_white_space(cu) {
                            is_attribute_value = false;
                        } else if cu == TAG_CLOSE_CHAR_CODE {
                            is_attribute_value = false;
                            end_index = end_index.saturating_sub(1);
                        }
                    } else if cu == EQUAL_SIGN_CHAR_CODE {
                        while units
                            .get(end_index.saturating_add(1))
                            .copied()
                            .is_some_and(is_white_space)
                        {
                            end_index = end_index.saturating_add(1);
                        }
                        is_attribute_value = true;

                        // Reason for fallback: if end_index + 1 is past end of units, 0 is returned which does not match quote characters.
                        let first_attr = units
                            .get(end_index.saturating_add(1))
                            .copied()
                            .unwrap_or(0);
                        if first_attr == DOUBLE_QUOTE_CHAR_CODE
                            || first_attr == SINGLE_QUOTE_CHAR_CODE
                        {
                            attribute_quote = first_attr;
                            end_index = end_index.saturating_add(1);
                        } else {
                            attribute_quote = 0;
                        }
                    } else if cu == TAG_CLOSE_CHAR_CODE {
                        let tag_name_start =
                            i.saturating_add(if is_end_tag { 2 } else { 1 });
                        let tag_name_end =
                            index_of_white_space(&units, tag_name_start)
                                .min(end_index);
                        // Reason for fallback: malformed HTML tag syntax with invalid range bounds defaults tag name to empty string, causing unparseable tags to be skipped safely
                        let mut tag_name = units
                            .get(tag_name_start..tag_name_end)
                            .map(units_to_string)
                            .unwrap_or_default()
                            .to_ascii_lowercase();
                        if let Some(last) = tag_name.as_bytes().last().copied()
                        {
                            if last == b'/' {
                                tag_name.pop();
                            }
                        }

                        let strip =
                            should_strip(strip_tags.as_ref(), &tag_name);

                        if is_end_tag {
                            let current = tag_stack.pop();
                            if current.as_deref() != Some(tag_name.as_str()) {
                                return Err(anyhow::anyhow!(
                                    "Invalid HTML: {text}"
                                ));
                            }

                            if is_unbreakable_element(&tag_name) {
                                if any_unbreakable_open(&tag_stack) {
                                    // nested unbreakable element
                                } else if strip {
                                    if let Some(start) =
                                        unbreakable_element_index
                                    {
                                        i = start;
                                    }
                                    unbreakable_element_index = None;
                                } else {
                                    unbreakable_element_index = None;
                                    num_chars = num_chars
                                        .saturating_add(image_weight_usize);
                                    if num_chars > max_len {
                                        break;
                                    }
                                }
                            }

                            if is_block_element(&tag_name)
                                && unbreakable_element_index.is_none()
                                && !strip
                            {
                                num_lines = num_lines.saturating_add(1);
                                if num_lines > max_lines {
                                    tag_stack.push(tag_name);
                                    break;
                                }
                            }
                        } else if is_void_element(&tag_name)
                            // Reason for fallback: if end_index is 0, 0 is returned which does not match '/' (FORWARD_SLASH_CHAR_CODE).
                            || units
                                .get(end_index.saturating_sub(1))
                                .copied()
                                .unwrap_or(0)
                                == FORWARD_SLASH_CHAR_CODE
                        {
                            if strip {
                                // stripped elements aren't counted
                            } else if tag_name == "br" {
                                num_lines = num_lines.saturating_add(1);
                                if num_lines > max_lines {
                                    break;
                                }
                            } else if tag_name == "img" {
                                num_chars = num_chars
                                    .saturating_add(image_weight_usize);
                                if num_chars > max_len {
                                    break;
                                }
                            }
                        } else {
                            if any_unbreakable_open(&tag_stack) {
                                // nested
                            } else if is_unbreakable_element(&tag_name) {
                                unbreakable_element_index = Some(i);
                            }
                            tag_stack.push(tag_name.clone());
                        }

                        if strip && unbreakable_element_index.is_none() {
                            units.drain(i..=end_index);
                            i = i.saturating_sub(1);
                        } else {
                            i = end_index;
                        }

                        break;
                    }
                }

                if num_chars > max_len || num_lines > max_lines {
                    break;
                }
            }
        } else if u == AMPERSAND_CHAR_CODE {
            let mut end_index = i.saturating_add(1);
            let mut is_char_ref = true;
            while let Some(&cu) = units.get(end_index) {
                if is_character_reference_character(cu) {
                    end_index = end_index.saturating_add(1);
                } else if cu == SEMICOLON_CHAR_CODE {
                    break;
                } else {
                    is_char_ref = false;
                    break;
                }
            }

            if unbreakable_element_index.is_none() {
                num_chars = num_chars.saturating_add(1);
                if num_chars > max_len {
                    break;
                }
            }

            if is_char_ref
                && end_index < units.len()
                && units.get(end_index) == Some(&SEMICOLON_CHAR_CODE)
            {
                i = end_index;
            }
        } else if u == NEWLINE_CHAR_CODE {
            num_chars = num_chars.saturating_add(1);
            if num_chars > max_len {
                break;
            }
            num_lines = num_lines.saturating_add(1);
            if num_lines > max_lines {
                break;
            }
        } else {
            if unbreakable_element_index.is_none() {
                num_chars = num_chars.saturating_add(1);
                if num_chars > max_len {
                    break;
                }
            }
            if is_high_surrogate(u) {
                if let Some(&next) = units.get(i.saturating_add(1)) {
                    if is_low_surrogate(next) {
                        i = i.saturating_add(1);
                    }
                }
            }
        }

        i = i.saturating_add(1);
    }

    if num_chars > max_len {
        let mut next_char = take_html_char_at(&units, i);

        if !indicator_units.is_empty() {
            let mut peek_index = i.saturating_add(next_char.len());
            while units.get(peek_index) == Some(&TAG_OPEN_CHAR_CODE)
                && units.get(peek_index.saturating_add(1))
                    == Some(&FORWARD_SLASH_CHAR_CODE)
            {
                let Some(tag_end) = units
                    .get(peek_index.saturating_add(2)..)
                    .and_then(|slice| {
                        slice.iter().position(|u| *u == TAG_CLOSE_CHAR_CODE)
                    })
                else {
                    break;
                };
                let next_peek = peek_index
                    .saturating_add(2)
                    .saturating_add(tag_end)
                    .saturating_add(1);
                peek_index = next_peek;
            }

            if peek_index == units.len() || is_line_break(&units, peek_index) {
                i = i.saturating_add(next_char.len());
                // Reason for fallback: when index i reaches or exceeds units length, empty vector indicates end of input stream.
                next_char =
                    units.get(i).copied().map(|u| vec![u]).unwrap_or_default();
            }
        }

        while next_char.first() == Some(&TAG_OPEN_CHAR_CODE)
            && units.get(i.saturating_add(1)) == Some(&FORWARD_SLASH_CHAR_CODE)
        {
            let Some(tag_name) = tag_stack.pop() else {
                break;
            };
            let close_pat = TAG_CLOSE_CHAR_CODE;
            let mut tag_end_index: Option<usize> = None;
            let mut j = i.saturating_add(2);
            while j < units.len() {
                if units.get(j) == Some(&close_pat) {
                    tag_end_index = Some(j);
                    break;
                }
                j = j.saturating_add(1);
            }
            let Some(tag_end_index) = tag_end_index else {
                bail!("Invalid HTML: {text}");
            };

            let Some(between_units) = units.get(i.saturating_add(2)..tag_end_index) else {
                bail!("Invalid HTML: {text}");
            };
            let between = units_to_string(between_units).trim().to_string();
            if between != tag_name {
                bail!("Invalid HTML: {text}");
            }

            if should_strip(strip_tags.as_ref(), &tag_name) {
                units.drain(i..=tag_end_index);
            } else {
                i = tag_end_index.saturating_add(1);
            }

            // Reason for fallback: when index i reaches or exceeds units length, empty vector indicates end of input stream.
            next_char =
                units.get(i).copied().map(|u| vec![u]).unwrap_or_default();
        }

        if i < units.len() {
            if !break_words {
                let backtrack_start = i.saturating_sub(indicator_units.len());
                let mut j = backtrack_start;
                loop {
                    let Some(&cu) = units.get(j) else {
                        break;
                    };
                    if cu == TAG_CLOSE_CHAR_CODE || cu == SEMICOLON_CHAR_CODE {
                        break;
                    }
                    if cu == NEWLINE_CHAR_CODE || cu == TAG_OPEN_CHAR_CODE {
                        i = j;
                        break;
                    }
                    if is_white_space(cu) {
                        i = j.saturating_add(usize::from(
                            !indicator_units.is_empty(),
                        ));
                        break;
                    }
                    if j == 0 {
                        break;
                    }
                    j = j.saturating_sub(1);
                }
            }

            let insert_indicator = (insert_indicator_at_linebreak
                || !is_line_break(&units, i))
                && has_html_non_whitespace(&units, i);

            while i > 0 {
                if let Some(&prev) = units.get(i.saturating_sub(1)) {
                    if is_white_space(prev) {
                        i = i.saturating_sub(1);
                        continue;
                    }
                }
                break;
            }

            #[allow(
                clippy::expect_used,
                reason = "i <= units.len() invariant in clip_html_text loop"
            )]
            let mut out_units = units
                .get(..i)
                .expect("i <= units.len() is in bounds")
                .to_vec();
            if insert_indicator {
                out_units.extend_from_slice(&indicator_units);
            }

            let out = units_to_string(&out_units);
            return Ok(pop_tag_stack(out, &mut tag_stack, strip_tags.as_ref()));
        }
    } else if num_lines > max_lines {
        let insert_indicator =
            insert_indicator_at_linebreak && has_html_non_whitespace(&units, i);

        while i > 0 {
            if let Some(&prev) = units.get(i.saturating_sub(1)) {
                if is_white_space(prev) {
                    i = i.saturating_sub(1);
                    continue;
                }
            }
            break;
        }

        #[allow(
            clippy::expect_used,
            reason = "i <= units.len() invariant in clip_html_text loop"
        )]
        let mut out_units = units
            .get(..i)
            .expect("i <= units.len() is in bounds")
            .to_vec();
        if insert_indicator {
            out_units.extend_from_slice(&indicator_units);
        }
        let out = units_to_string(&out_units);
        return Ok(pop_tag_stack(out, &mut tag_stack, strip_tags.as_ref()));
    }

    Ok(units_to_string(&units))
}

/// Common options shared by both plain-text and HTML clipping.
#[derive(Debug, Clone, Default)]
pub struct CommonClipOptions {
    /// By default, we try to break only at word boundaries. Set to true if this is undesired.
    pub break_words: Option<bool>,

    /// Set to `true` if the string is HTML-encoded. If so, this method will take extra care to make
    /// sure the HTML-encoding is correctly maintained.
    pub html: Option<bool>,

    /// The string to insert to indicate clipping. Default: "…".
    pub indicator: Option<String>,

    /// Whether the indicator should be inserted when the text is clipped at a linebreak.
    /// Default: `true`.
    pub insert_indicator_at_linebreak: Option<bool>,

    /// Maximum amount of lines allowed.
    pub max_lines: Option<u32>,
}

/// Options for clipping plain text (html = false).
#[derive(Debug, Clone, Default)]
pub struct ClipPlainTextOptions {
    /// Embed common options; `html` should be false or omitted.
    pub common: CommonClipOptions,
}

/// Options for clipping HTML (html = true).
#[derive(Debug, Clone, Default)]
pub struct ClipHtmlOptions {
    /// Embed common options; `html` should be true.
    pub common: CommonClipOptions,

    /// The amount of characters to assume for images. Default: 2.
    pub image_weight: Option<u32>,

    /// Optional list of tags to be stripped from the output. If `true`, all tags are stripped.
    /// Tag names must be specified in lowercase.
    pub strip_tags: Option<StripTags>,
}

/// Represents either an explicit list of tags or the boolean `true` meaning "strip all tags".
#[derive(Debug, Clone)]
pub enum StripTags {
    All(bool),         // true -> strip all tags
    Tags(Vec<String>), // list of tag names (lowercase)
}

impl Default for StripTags {
    fn default() -> Self {
        Self::Tags(Vec::new())
    }
}

/// Discriminated union of allowed clip options.
#[derive(Debug, Clone)]
pub enum ClipOptions {
    Plain(ClipPlainTextOptions),
    Html(ClipHtmlOptions),
}

impl Default for ClipOptions {
    fn default() -> Self {
        Self::Plain(ClipPlainTextOptions::default())
    }
}

pub fn plain_options(common: CommonClipOptions) -> ClipOptions {
    ClipOptions::Plain(ClipPlainTextOptions { common })
}

pub fn html_options(mut options: ClipHtmlOptions) -> ClipOptions {
    if options.common.html.is_none() {
        options.common.html = Some(true);
    }
    ClipOptions::Html(options)
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

    fn get_html_table() -> String {
        r#"<table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>
        </tr>
        <tr>
            <td>intel</td>
            <td>amazon</td>
        </tr>
    </tbody>
</table>"#
            .to_string()
    }

    fn get_html_with_svg() -> String {
        "<p>\
        <svg width=\"100%\" height=\"100%\" viewBox=\"0 0 100 100\" \
        xmlns=\"http://www.w3.org/2000/svg\">\n\
        <style>\n\
        /* <![CDATA[ */\n\
        circle {\n\
        fill: orange;\n\
        stroke: black;\n\
        stroke-width: 10px; \
        // Note that the value of a pixel depend on the viewBox\n\
        }\n\
        /* ]]> */\n\
        </style>\n\
        \n\
        <circle cx=\"50\" cy=\"50\" r=\"40\" />\n\
        </svg>test\n\
        </p>"
            .to_string()
    }

    #[crate::ctb_test]
    fn test_examples() {
        let opt: Option<&ClipOptions> = None;
        assert_eq!(clip("foo", 3, opt), "foo");
        assert_eq!(clip("foo", 2, opt), "f…");
        assert_eq!(clip("foo bar", 5, opt), "foo…");
        assert_eq!(clip("foo\nbar", 5, opt), "foo…");
    }

    #[crate::ctb_test]
    fn test_basic_html() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("<p>Lorum ipsum</p>", 5, options), "<p>Loru…</p>");
        assert_eq!(
            clip("<p><i>Lorum</i> <i>ipsum</i></p>", 5, options),
            "<p><i>Loru…</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i> <i>ipsum</i></p>", 6, options),
            "<p><i>Lorum</i>…</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i> <i>ipsum</i></p>", 7, options),
            "<p><i>Lorum</i>…</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i>\n<i>ipsum</i></p>", 5, options),
            "<p><i>Lorum</i>…</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i><br><i>ipsum</i></p>", 5, options),
            "<p><i>Lorum</i>…</p>"
        );

        assert_eq!(
            clip("<p><i>Lorum</i></p>", 5, options),
            "<p><i>Lorum</i></p>"
        );

        assert_eq!(
            clip("<p><i>Lorum</i>a</p>", 5, options),
            "<p><i>Loru…</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>a", 5, options),
            "<p><i>Loru…</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i>a</p>", 6, options),
            "<p><i>Lorum</i>a</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>a", 6, options),
            "<p><i>Lorum</i></p>a"
        );
        assert_eq!(
            clip("<p><i>Lorum</i>aA</p>", 6, options),
            "<p><i>Lorum</i>…</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>aA", 6, options),
            "<p><i>Lorum</i></p>…"
        );
        assert_eq!(
            clip("<p><i>Lorum</i>a</p>", 7, options),
            "<p><i>Lorum</i>a</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>a", 7, options),
            "<p><i>Lorum</i></p>a"
        );
        assert_eq!(
            clip("<p><i>Lorum</i>aA</p>", 7, options),
            "<p><i>Lorum</i>aA</p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>aA", 7, options),
            "<p><i>Lorum</i></p>aA"
        );

        assert_eq!(
            clip("<p><i>Lorum</i> </p>", 5, options),
            "<p><i>Loru…</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p> ", 5, options),
            "<p><i>Loru…</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i> </p>", 6, options),
            "<p><i>Lorum</i> </p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p> ", 6, options),
            "<p><i>Lorum</i></p> "
        );
        assert_eq!(
            clip("<p><i>Lorum</i>  </p>", 6, options),
            "<p><i>Lorum</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>  ", 6, options),
            "<p><i>Lorum</i></p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i> </p>", 7, options),
            "<p><i>Lorum</i> </p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p> ", 7, options),
            "<p><i>Lorum</i></p> "
        );
        assert_eq!(
            clip("<p><i>Lorum</i>  </p>", 7, options),
            "<p><i>Lorum</i>  </p>"
        );
        assert_eq!(
            clip("<p><i>Lorum</i></p>  ", 7, options),
            "<p><i>Lorum</i></p>  "
        );

        assert_eq!(clip("Lo<ins>rum</ins>", 4, options), "Lo<ins>r…</ins>");
        assert_eq!(clip("Lo<del>rum</del>", 4, options), "Lo<del>r…</del>");

        assert_eq!(
            clip(
                "<a href=\"http://just-a-link.com\">Just a link</a>",
                8,
                options
            ),
            "<a href=\"http://just-a-link.com\">Just a…</a>"
        );

        assert_eq!(
            clip(
                "<a href=\"http://just-a-link.com\">Just a link</a>, yo",
                13,
                options
            ),
            "<a href=\"http://just-a-link.com\">Just a link</a>,…"
        );
    }

    #[crate::ctb_test]
    fn test_html_comments() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(
            clip("<b><!-- this is bold -->bold</b>", 4, options),
            "<b><!-- this is bold -->bold</b>"
        );

        assert_eq!(
            clip("<b><!-- this is bold -->bold</b>", 3, options),
            "<b><!-- this is bold -->bo…</b>"
        );
    }

    #[crate::ctb_test]
    fn test_special_characters_in_attribute_values() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(
            clip("<b class=\"<i>\">bold</b>", 4, options),
            "<b class=\"<i>\">bold</b>"
        );
        assert_eq!(
            clip("<b class=\"<i>\">bold</b>", 3, options),
            "<b class=\"<i>\">bo…</b>"
        );
        assert_eq!(
            clip("<b class=\"'test'\">bold</b>", 4, options),
            "<b class=\"'test'\">bold</b>"
        );
        assert_eq!(
            clip("<b class=\"'test'\">bold</b>", 3, options),
            "<b class=\"'test'\">bo…</b>"
        );

        assert_eq!(
            clip("<b class='javascript:alert(\"hoi\");'>bold</b>", 4, options),
            "<b class='javascript:alert(\"hoi\");'>bold</b>"
        );

        assert_eq!(
            clip("<b class='javascript:alert(\"hoi\");'>bold</b>", 3, options),
            "<b class='javascript:alert(\"hoi\");'>bo…</b>"
        );
    }

    #[crate::ctb_test]
    fn test_embedded_svg() {
        let options = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                ..Default::default()
            },
            image_weight: Some(5),
            ..Default::default()
        }));

        let input = get_html_with_svg();

        // This is mostly the same but lacks the newline before </p>".
        let expected = "<p>\
        <svg width=\"100%\" height=\"100%\" viewBox=\"0 0 100 100\" \
        xmlns=\"http://www.w3.org/2000/svg\">\n\
        <style>\n\
        /* <![CDATA[ */\n\
        circle {\n\
        fill: orange;\n\
        stroke: black;\n\
        stroke-width: 10px; \
        // Note that the value of a pixel depend on the viewBox\n\
        }\n\
        /* ]]> */\n\
        </style>\n\
        \n\
        <circle cx=\"50\" cy=\"50\" r=\"40\" />\n\
        </svg>test\
        </p>"
            .to_string();

        // Note: this test is now valid Rust syntax; behavior will be addressed
        // once `clip` is implemented.
        assert_eq!(clip(&input.clone(), 9, options), expected);
    }

    #[crate::ctb_test]
    fn test_unicode_surrogate_pairs() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("Lorum 𝌆", 7, options), "Lorum 𝌆");
        assert_eq!(clip("𝌆𝌆𝌆𝌆", 4, options), "𝌆𝌆𝌆𝌆");
        assert_eq!(clip("𝌆𝌆𝌆𝌆", 3, options), "𝌆𝌆…");
        assert_eq!(clip("😔🙏👙😃🏧", 6, options), "😔🙏👙😃🏧");
        assert_eq!(clip("😔🙏👙😃🏧", 5, options), "😔🙏👙😃🏧");
        assert_eq!(clip("😔🙏👙😃🏧", 4, options), "😔🙏👙…");
        assert_eq!(clip("😔🙏👙😃🏧", 3, options), "😔🙏…");
    }

    #[crate::ctb_test]
    fn test_plain_text() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("Lorum ipsum", 5, options), "Loru…");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");

        assert_eq!(clip("Lorum\nipsum", 10, options), "Lorum…");

        assert_eq!(clip("Lorum i", 7, options), "Lorum i");
        assert_eq!(clip("Lorum …", 7, options), "Lorum …");
    }

    #[crate::ctb_test]
    fn test_word_breaking() {
        let options = Some(&plain_options(CommonClipOptions {
            break_words: Some(true),
            ..Default::default()
        }));

        assert_eq!(clip("Lorum ipsum", 5, options), "Loru…");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum i…");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum ip…");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum ips…");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");
    }

    #[crate::ctb_test]
    fn test_word_breaking_without_indicator() {
        let options = Some(&plain_options(CommonClipOptions {
            break_words: Some(true),
            indicator: Some(String::new()),
            ..Default::default()
        }));
        assert_eq!(clip("Lorum ipsum", 5, options), "Lorum");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum i");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum ip");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum ips");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum ipsu");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");
    }

    #[crate::ctb_test]
    fn test_max_lines() {
        let options_2 = Some(&plain_options(CommonClipOptions {
            max_lines: Some(2),
            ..Default::default()
        }));
        let options_1 = Some(&plain_options(CommonClipOptions {
            max_lines: Some(1),
            ..Default::default()
        }));

        assert_eq!(clip("Lorum\nipsum", 100, options_2), "Lorum\nipsum");
        assert_eq!(clip("Lorum\nipsum", 100, options_1), "Lorum…");
        assert_eq!(clip("Lorum\n\nipsum", 100, options_2), "Lorum…");

        // If there is *only* whitespace left, we don't insert an indicator regardless.
        assert_eq!(clip("Lorum\nipsum\n", 100, options_2), "Lorum\nipsum");
        assert_eq!(clip("Lorum\nipsum\n\n", 100, options_2), "Lorum\nipsum");

        let html_2 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                max_lines: Some(2),
                ..Default::default()
            },
            ..Default::default()
        }));
        let html_1 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                max_lines: Some(1),
                ..Default::default()
            },
            ..Default::default()
        }));

        assert_eq!(
            clip("<p>Lorem ipsum</p><p>Lorem ipsum</p>", 100, html_2),
            "<p>Lorem ipsum</p><p>Lorem ipsum</p>"
        );
        assert_eq!(
            clip("<p>Lorem ipsum</p><p>Lorem ipsum</p>", 100, html_1),
            "<p>Lorem ipsum…</p>"
        );
        assert_eq!(
            clip("<div>Lorem ipsum</div><div>Lorem ipsum</div>", 100, html_2),
            "<div>Lorem ipsum</div><div>Lorem ipsum</div>"
        );
        assert_eq!(
            clip("<div>Lorem ipsum</div><div>Lorem ipsum</div>", 100, html_1),
            "<div>Lorem ipsum…</div>"
        );
    }

    #[crate::ctb_test]
    fn test_odd_html() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("<p>foo > bar</p>", 9, options), "<p>foo > bar</p>");
        assert_eq!(
            clip("<p><i>Lorum>>></i> <i>ipsum</i></p>", 7, options),
            "<p><i>Lorum>…</i></p>"
        );
    }

    #[crate::ctb_test]
    fn test_ampersand() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("&", 1, options), "&");
        assert_eq!(clip("&", 2, options), "&");
        assert_eq!(clip("&lt;", 1, options), "&lt;");
        assert_eq!(clip("&lt;", 2, options), "&lt;");
        assert_eq!(clip("&amp;", 1, options), "&amp;");
        assert_eq!(clip("&amp;", 2, options), "&amp;");
        assert_eq!(clip("<p>&</p>", 1, options), "…");
        assert_eq!(clip("<p>&</p>", 2, options), "<p>&</p>");
        assert_eq!(clip("<p>&lt;</p>", 1, options), "…");
        assert_eq!(clip("<p>&lt;</p>", 2, options), "<p>&lt;</p>");
        assert_eq!(clip("<p>&amp;</p>", 1, options), "…");
        assert_eq!(clip("<p>&amp;</p>", 2, options), "<p>&amp;</p>");

        assert_eq!(clip("foo & bar", 5, options), "foo…");
        assert_eq!(clip("foo & bar", 9, options), "foo & bar");
        assert_eq!(clip("foo&<i>bar</i>", 5, options), "foo&…");
        assert_eq!(clip("foo&<i>bar</i>", 7, options), "foo&<i>bar</i>");
        assert_eq!(clip("foo&&& bar", 5, options), "foo&…");
        assert_eq!(clip("foo&&& bar", 10, options), "foo&&& bar");

        assert_eq!(
            clip(
                "<a href=\"http://example.com/?x=1&y=2\">foo</a>",
                3,
                options
            ),
            "<a href=\"http://example.com/?x=1&y=2\">foo</a>"
        );
        assert_eq!(clip("&123", 4, options), "&123");
        assert_eq!(clip("&abc", 4, options), "&abc");
        assert_eq!(clip("foo &0 bar", 10, options), "foo &0 bar");
        assert_eq!(clip("foo &lolwat bar", 15, options), "foo &lolwat bar");
    }

    #[crate::ctb_test]
    fn test_ampersand_without_indicator() {
        let options = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                indicator: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        }));

        assert_eq!(clip("&", 1, options), "&");
        assert_eq!(clip("&", 2, options), "&");
        assert_eq!(clip("&lt;", 1, options), "&lt;");
        assert_eq!(clip("&lt;", 2, options), "&lt;");
        assert_eq!(clip("&amp;", 1, options), "&amp;");
        assert_eq!(clip("&amp;", 2, options), "&amp;");
        assert_eq!(clip("<p>&</p>", 1, options), "<p>&</p>");
        assert_eq!(clip("<p>&</p>", 2, options), "<p>&</p>");
        assert_eq!(clip("<p>&lt;</p>", 1, options), "<p>&lt;</p>");
        assert_eq!(clip("<p>&lt;</p>", 2, options), "<p>&lt;</p>");
        assert_eq!(clip("<p>&amp;</p>", 1, options), "<p>&amp;</p>");
        assert_eq!(clip("<p>&amp;</p>", 2, options), "<p>&amp;</p>");

        assert_eq!(clip("foo & bar", 5, options), "foo &");
        assert_eq!(clip("foo & bar", 9, options), "foo & bar");
        // Ideally "bar" wouldn't have been broken, but we accept this
        // limitation when encountering tags during backtracking:
        assert_eq!(clip("foo&<i>bar</i>", 5, options), "foo&<i>b</i>");
        assert_eq!(clip("foo&<i>bar</i>", 7, options), "foo&<i>bar</i>");
        assert_eq!(clip("foo&&& bar", 5, options), "foo&&");
        assert_eq!(clip("foo&&& bar", 10, options), "foo&&& bar");

        assert_eq!(
            clip(
                "<a href=\"http://example.com/?x=1&y=2\">foo</a>",
                3,
                options
            ),
            "<a href=\"http://example.com/?x=1&y=2\">foo</a>"
        );
        assert_eq!(clip("&123", 4, options), "&123");
        assert_eq!(clip("&abc", 4, options), "&abc");
        assert_eq!(clip("foo &0 bar", 10, options), "foo &0 bar");
        assert_eq!(clip("foo &lolwat bar", 15, options), "foo &lolwat bar");
    }

    #[crate::ctb_test]
    fn test_ampersand_without_indicator_and_break_words() {
        let options = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                break_words: Some(true),
                html: Some(true),
                indicator: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        }));

        assert_eq!(clip("&", 1, options), "&");
        assert_eq!(clip("&", 2, options), "&");
        assert_eq!(clip("&lt;", 1, options), "&lt;");
        assert_eq!(clip("&lt;", 2, options), "&lt;");
        assert_eq!(clip("&amp;", 1, options), "&amp;");
        assert_eq!(clip("&amp;", 2, options), "&amp;");
        assert_eq!(clip("<p>&</p>", 1, options), "<p>&</p>");
        assert_eq!(clip("<p>&</p>", 2, options), "<p>&</p>");
        assert_eq!(clip("<p>&lt;</p>", 1, options), "<p>&lt;</p>");
        assert_eq!(clip("<p>&lt;</p>", 2, options), "<p>&lt;</p>");
        assert_eq!(clip("<p>&amp;</p>", 1, options), "<p>&amp;</p>");
        assert_eq!(clip("<p>&amp;</p>", 2, options), "<p>&amp;</p>");

        assert_eq!(clip("foo & bar", 5, options), "foo &");
        assert_eq!(clip("foo & bar", 9, options), "foo & bar");
        assert_eq!(clip("foo&<i>bar</i>", 5, options), "foo&<i>b</i>");
        assert_eq!(clip("foo&<i>bar</i>", 7, options), "foo&<i>bar</i>");
        assert_eq!(clip("foo&&& bar", 5, options), "foo&&");
        assert_eq!(clip("foo&&& bar", 10, options), "foo&&& bar");

        assert_eq!(
            clip(
                "<a href=\"http://example.com/?x=1&y=2\">foo</a>",
                3,
                options
            ),
            "<a href=\"http://example.com/?x=1&y=2\">foo</a>"
        );
        assert_eq!(clip("&123", 4, options), "&123");
        assert_eq!(clip("&abc", 4, options), "&abc");
        assert_eq!(clip("foo &0 bar", 10, options), "foo &0 bar");
        assert_eq!(clip("foo &lolwat bar", 15, options), "foo &lolwat bar");
    }

    #[crate::ctb_test]
    fn test_edge_cases() {
        let options = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                break_words: Some(true),
                html: Some(true),
                indicator: Some("...".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }));

        assert_eq!(
            clip(
                "one <a href=\"#\">two - three <br>four</a> five",
                0,
                options
            ),
            "..."
        );
        assert_eq!(
            clip(
                "<p>one <a href=\"#\">two - three <br>four</a> five</p>",
                0,
                options
            ),
            "..."
        );
        assert_eq!(
            clip(
                "<p>one <a href=\"#\">two - three <br>four</a> five</p>",
                6,
                options
            ),
            "<p>one...</p>"
        );
    }

    #[crate::ctb_test]
    fn test_upstream_issue_12_split_tables() {
        let html = get_html_table();

        let options_26 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                break_words: Some(true),
                ..Default::default()
            },
            ..Default::default()
        }));
        let options_25 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                break_words: Some(true),
                ..Default::default()
            },
            ..Default::default()
        }));
        let options_25_2 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                break_words: Some(true),
                max_lines: Some(2),
                ..Default::default()
            },
            ..Default::default()
        }));

        assert_eq!(
            clip(&html.clone(), 26, options_26),
            r#"<table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>
        </tr>
        <tr>
            <td>intel</td>…</tr></tbody></table>"#
                .to_string()
        );

        assert_eq!(
            clip(&html.clone(), 25, options_25),
            r#"<table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>
        </tr>
        <tr>
            <td>int…</td></tr></tbody></table>"#
                .to_string()
        );

        assert_eq!(
            clip(&html, 25, options_25_2),
            r#"<table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>…</tr></tbody></table>"#
                .to_string()
        );
    }

    #[crate::ctb_test]
    fn test_strip_tags() {
        // Basic stripping of tags:
        let html_with_image =
            "<p>Image <img alt=\"blup\" src=\"#\"> and such</p>";

        let options_strip_none = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(Vec::new())),
            ..Default::default()
        }));
        let options_default: Option<&ClipOptions> = None;
        assert_eq!(
            clip(html_with_image, 12, options_strip_none),
            clip(html_with_image, 12, options_default)
        );

        let options_strip_img = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(vec!["img".to_string()])),
            ..Default::default()
        }));
        assert_eq!(
            clip(html_with_image, 12, options_strip_img),
            "<p>Image  and…</p>"
        );

        let options_strip_img_p = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(vec![
                "img".to_string(),
                "p".to_string(),
            ])),
            ..Default::default()
        }));
        assert_eq!(
            clip(html_with_image, 12, options_strip_img_p),
            "Image  and…"
        );

        let options_strip_all = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::All(true)),
            ..Default::default()
        }));
        assert_eq!(clip(html_with_image, 12, options_strip_all), "Image  and…");

        assert_eq!(
            clip(html_with_image, 15, options_strip_img),
            "<p>Image  and such</p>"
        );

        // Links are stripped (but content is preserved):
        let html_with_link = "<a href=\"http://example.com/?x=1&y=2\">foo</a>";
        let options_strip_a = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(vec!["a".to_string()])),
            ..Default::default()
        }));
        assert_eq!(clip(html_with_link, 3, options_strip_a), "foo");

        let options_strip_b = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(vec!["b".to_string()])),
            ..Default::default()
        }));
        assert_eq!(clip(html_with_link, 3, options_strip_b), html_with_link);

        // Same for tables, but whitespace is also simplified:
        let html_with_table = r#"hello <table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>
        </tr>
        <tr>
            <td>intel</td>
            <td>amazon</td>
        </tr>
    </tbody>
</table> world"#;

        assert_eq!(clip(html_with_table, 10, options_strip_all), "hello fb…");
        assert_eq!(
            clip(html_with_table, 16, options_strip_all),
            "hello fb fbfbfb…"
        );
        assert_eq!(
            clip(html_with_table, 24, options_strip_all),
            "hello fb fbfbfb google…"
        );

        // SVG's `imageWeight` should not be counted when stripped:
        let html_with_svg = get_html_with_svg();
        let options_strip_svg = Some(&html_options(ClipHtmlOptions {
            strip_tags: Some(StripTags::Tags(vec!["svg".to_string()])),
            ..Default::default()
        }));
        assert_eq!(
            clip(&html_with_svg.clone(), 3, options_strip_svg),
            "<p>te…</p>"
        );
        assert_eq!(clip(&html_with_svg, 4, options_strip_svg), "<p>test</p>");
    }

    #[crate::ctb_test]
    fn test_disable_indicator_at_line_break() {
        let base = ClipHtmlOptions {
            common: CommonClipOptions {
                html: Some(true),
                insert_indicator_at_linebreak: Some(false),
                ..Default::default()
            },
            ..Default::default()
        };

        let options = Some(&html_options(base.clone()));
        assert_eq!(clip("Lorum\nipsum", 10, options), "Lorum");

        let options_max_lines_1 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                max_lines: Some(1),
                ..base.common.clone()
            },
            ..base.clone()
        }));
        assert_eq!(clip("Lorum\nipsum", 100, options_max_lines_1), "Lorum");

        assert_eq!(clip("Lorum<br/>ipsum", 100, options_max_lines_1), "Lorum");

        let options_max_lines_2 = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                max_lines: Some(2),
                ..base.common.clone()
            },
            ..base.clone()
        }));
        assert_eq!(clip("Lorum\n\nipsum", 100, options_max_lines_2), "Lorum");

        assert_eq!(clip("<p>&</p>", 1, options), "");
        assert_eq!(clip("<p>&lt;</p>", 1, options), "");

        assert_eq!(
            clip(
                "<p>Lorem ipsum</p><p>Lorem ipsum</p>",
                100,
                options_max_lines_1
            ),
            "<p>Lorem ipsum</p>"
        );

        let options_indicator_ellipsis = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                indicator: Some("...".to_string()),
                ..base.common.clone()
            },
            ..base.clone()
        }));
        assert_eq!(
            clip(
                "<p>one <a href=\"#\">two - three <br>four</a> five</p>",
                0,
                options_indicator_ellipsis
            ),
            ""
        );

        let options_break_words = Some(&html_options(ClipHtmlOptions {
            common: CommonClipOptions {
                break_words: Some(true),
                ..base.common.clone()
            },
            ..base.clone()
        }));
        assert_eq!(
            clip(&get_html_table(), 26, options_break_words),
            r#"<table border="1" cellpadding="1" cellspacing="1" style="width: 500px">
    <tbody>
        <tr>
            <td>fb</td>
            <td>fbfbfb</td>
        </tr>
        <tr>
            <td>google</td>
            <td>twitter</td>
        </tr>
        <tr>
            <td>intel</td></tr></tbody></table>"#
                .to_string()
        );

        let options_strip_tags_svg = Some(&html_options(ClipHtmlOptions {
            common: base.common.clone(),
            strip_tags: Some(StripTags::Tags(vec!["svg".to_string()])),
            ..base
        }));
        assert_eq!(
            clip(&get_html_with_svg(), 4, options_strip_tags_svg),
            "<p>test</p>"
        );
    }

    // Text-only variants (previously JS-style overloads) kept as Rust tests that
    // explicitly pass plain options.

    #[crate::ctb_test]
    fn test_text_basics() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("Lorum ipsum", 5, options), "Loru…");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");

        assert_eq!(clip("Lorum\nipsum", 10, options), "Lorum…");

        assert_eq!(clip("Lorum i", 7, options), "Lorum i");
        assert_eq!(clip("Lorum …", 7, options), "Lorum …");
    }

    #[crate::ctb_test]
    fn test_text_unicode_surrogate_pairs() {
        let options: Option<&ClipOptions> = None;

        assert_eq!(clip("Lorum 𝌆", 7, options), "Lorum 𝌆");
        assert_eq!(clip("𝌆𝌆𝌆𝌆", 4, options), "𝌆𝌆𝌆𝌆");
        assert_eq!(clip("𝌆𝌆𝌆𝌆", 3, options), "𝌆𝌆…");
        assert_eq!(clip("😔🙏👙😃🏧", 6, options), "😔🙏👙😃🏧");
        assert_eq!(clip("😔🙏👙😃🏧", 5, options), "😔🙏👙😃🏧");
        assert_eq!(clip("😔🙏👙😃🏧", 4, options), "😔🙏👙…");
        assert_eq!(clip("😔🙏👙😃🏧", 3, options), "😔🙏…");
    }

    #[crate::ctb_test]
    fn test_text_word_breaking() {
        let options = Some(&plain_options(CommonClipOptions {
            break_words: Some(true),
            ..Default::default()
        }));

        assert_eq!(clip("Lorum ipsum", 5, options), "Loru…");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum…");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum i…");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum ip…");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum ips…");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");
    }

    #[crate::ctb_test]
    fn test_text_word_breaking_without_indicator() {
        let options = Some(&plain_options(CommonClipOptions {
            break_words: Some(true),
            indicator: Some(String::new()),
            ..Default::default()
        }));
        assert_eq!(clip("Lorum ipsum", 5, options), "Lorum");
        assert_eq!(clip("Lorum ipsum", 6, options), "Lorum");
        assert_eq!(clip("Lorum ipsum", 7, options), "Lorum i");
        assert_eq!(clip("Lorum ipsum", 8, options), "Lorum ip");
        assert_eq!(clip("Lorum ipsum", 9, options), "Lorum ips");
        assert_eq!(clip("Lorum ipsum", 10, options), "Lorum ipsu");
        assert_eq!(clip("Lorum ipsum", 11, options), "Lorum ipsum");
    }

    #[crate::ctb_test]
    fn test_text_max_lines() {
        let options_2 = Some(&plain_options(CommonClipOptions {
            max_lines: Some(2),
            ..Default::default()
        }));
        let options_1 = Some(&plain_options(CommonClipOptions {
            max_lines: Some(1),
            ..Default::default()
        }));

        assert_eq!(clip("Lorum\nipsum", 100, options_2), "Lorum\nipsum");
        assert_eq!(clip("Lorum\nipsum", 100, options_1), "Lorum…");
        assert_eq!(clip("Lorum\n\nipsum", 100, options_2), "Lorum…");

        assert_eq!(clip("Lorum\nipsum\n", 100, options_2), "Lorum\nipsum");
        assert_eq!(clip("Lorum\nipsum\n\n", 100, options_2), "Lorum\nipsum");
    }

    #[crate::ctb_test]
    fn test_text_disable_indicator_at_line_break() {
        let options = Some(&plain_options(CommonClipOptions {
            insert_indicator_at_linebreak: Some(false),
            ..Default::default()
        }));

        assert_eq!(clip("Lorum\nipsum", 10, options), "Lorum");

        let options_max_lines_1 = Some(&plain_options(CommonClipOptions {
            insert_indicator_at_linebreak: Some(false),
            max_lines: Some(1),
            ..Default::default()
        }));
        assert_eq!(clip("Lorum\nipsum", 100, options_max_lines_1), "Lorum");

        let options_max_lines_2 = Some(&plain_options(CommonClipOptions {
            insert_indicator_at_linebreak: Some(false),
            max_lines: Some(2),
            ..Default::default()
        }));
        assert_eq!(clip("Lorum\n\nipsum", 100, options_max_lines_2), "Lorum");
    }

    #[crate::ctb_test]
    fn test_text_edge_cases() {
        let options = Some(&plain_options(CommonClipOptions {
            break_words: Some(true),
            indicator: Some("...".to_string()),
            ..Default::default()
        }));
        assert_eq!(clip("one two - three \nfour five", 0, options), "...");
        assert_eq!(clip("one two - three \nfour five", 6, options), "one...");
    }
}

/*

// This file is a port of text-clipper (https://github.com/arendjr/text-clipper):

The MIT License (MIT)

Copyright (c) 2016-2024 Arend van Beelen jr., Speakap B.V.

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
