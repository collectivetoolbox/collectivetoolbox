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

use boa_engine::Context as BoaContext;
use boa_engine::property::PropertyKey;
use ctb_utilities::Context;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use boa_engine::object::builtins::JsRegExp;
use boa_string::{JsStr, JsString};

pub fn js_string(s: &str) -> JsString {
    JsString::from(s)
}

pub fn js_regex(regex: &str) -> Result<JsRegExp> {
    let mut context = BoaContext::default();
    JsRegExp::new(js_string(regex), "".into(), &mut context)
        .map_err(|e| anyhow::anyhow!("Failed to create regex: {e}"))
}

/// Returns the UTF-16 code unit at `index`, matching JS `charCodeAt`.
///
/// In JavaScript, `String.prototype.charCodeAt()` returns `NaN` when the
/// index is out of bounds. This helper returns `None` in that case.
///
/// `index` is interpreted as a UTF-16 code-unit index.
pub fn char_code_at_j(s: &JsString, index: i64) -> Option<u16> {
    let index = usize::try_from(index).ok()?;
    s.code_unit_at(index)
}

/// Returns the UTF-16 code unit at `index`, matching JS `charCodeAt`.
///
/// `index` is interpreted as a UTF-16 code-unit index.
pub fn char_code_at(s: &str, index: i64) -> Option<u16> {
    char_code_at_j(&JsString::from(s), index)
}

/// Take a single “character” at `index`, keeping surrogate pairs together.
///
/// This is the surrogate-pair-aware equivalent of the JS snippet in the
/// prompt: if `index` points at a high surrogate and the next code unit is a
/// low surrogate, returns both code units; otherwise returns one code unit.
///
/// Returns `None` if `index` is out of bounds.
pub fn take_char_at_j(s: &JsString, index: i64) -> Option<JsString> {
    let first = char_code_at_j(s, index)?;

    if (first & 0xfc00) == 0xd800 {
        let next = char_code_at_j(s, index.checked_add(1)?)?;
        if (next & 0xfc00) == 0xdc00 {
            let pair = [first, next];
            return Some(JsString::from(&pair[..]));
        }
    }

    let one = [first];
    Some(JsString::from(&one[..]))
}

/// Take a single “character” at `index`, keeping surrogate pairs together.
///
/// Returns `None` if `index` is out of bounds.
pub fn take_char_at(s: &str, index: i64) -> Option<String> {
    let taken = take_char_at_j(&JsString::from(s), index)?;
    Some(taken.to_std_string_lossy().clone())
}

fn clamp_i64_to_range(value: i64, min: i64, max: i64) -> i64 {
    // ...existing code... (none)
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

fn len_as_i64(s: &JsString) -> Result<i64> {
    let len = s.len();
    i64::try_from(len).context("JsString length does not fit in i64")
}

/// JavaScript-like `slice(start, end)` using UTF-16 code-unit indices.
///
/// Negative indices are relative to the end of the string. Indices are
/// clamped into `[0, len]`. If `end` is `None`, it defaults to `len`.
/// https://tc39.es/ecma262/#sec-string.prototype.slice
pub fn slice_j(s: &JsString, start: i64, end: Option<i64>) -> Result<JsString> {
    let len = len_as_i64(s)?;

    let mut start = if start < 0 {
        #[expect(
            clippy::expect_used,
            reason = "len >= 0 and start < 0, so adding them moves value towards 0 and cannot overflow i64"
        )]
        len.checked_add(start)
            .expect("len >= 0 and start < 0, addition cannot overflow")
    } else {
        start
    };
    start = clamp_i64_to_range(start, 0, len);

    let mut end = match end {
        Some(end) => {
            if end < 0 {
                #[expect(
                    clippy::expect_used,
                    reason = "len >= 0 and end < 0, so adding them moves value towards 0 and cannot overflow i64"
                )]
                len.checked_add(end)
                    .expect("len >= 0 and end < 0, addition cannot overflow")
            } else {
                end
            }
        }
        None => len,
    };
    end = clamp_i64_to_range(end, 0, len);

    if end <= start {
        return Ok(JsString::default());
    }

    let start =
        usize::try_from(start).context("slice start does not fit in usize")?;
    let end =
        usize::try_from(end).context("slice end does not fit in usize")?;

    let Some(part) = s.get(start..end) else {
        // Defensive: computed indices should always be in-bounds.
        return Ok(JsString::default());
    };

    Ok(part)
}

/// JavaScript-like `slice(start, end)` using UTF-16 code-unit indices.
///
/// Negative indices are relative to the end of the string. Indices are
/// clamped into `[0, len]`. If `end` is `None`, it defaults to `len`.
pub fn slice(s: &str, start: i64, end: Option<i64>) -> Result<String> {
    let out = slice_j(&JsString::from(s), start, end)?;
    Ok(out.to_std_string_lossy().clone())
}

/// JavaScript-like `substr(start, length)` using UTF-16 code-unit indices.
///
/// `start` may be negative (relative to the end). `length` is clamped to
/// `>= 0` and truncated to the remaining string length.
pub fn substr_j(
    s: &JsString,
    start: i64,
    length: Option<i64>,
) -> Result<JsString> {
    let len = len_as_i64(s)?;

    let mut start = if start < 0 {
        #[expect(
            clippy::expect_used,
            reason = "len >= 0 and start < 0, so adding them moves value towards 0 and cannot overflow i64"
        )]
        len.checked_add(start)
            .expect("len >= 0 and start < 0, addition cannot overflow")
    } else {
        start
    };
    start = clamp_i64_to_range(start, 0, len);

    let length = match length {
        Some(length) if length > 0 => length,
        Some(_) => 0,
        None => len.saturating_sub(start),
    };

    let end = start
        .checked_add(length)
        // Reason for fallback: 0 <= start <= len and 0 <= length so checked_add is bounded by positive values; defaulting to len matches ECMAScript spec requiring length to be clamped to string end.
        .map_or(len, |v| clamp_i64_to_range(v, 0, len));

    slice_j(s, start, Some(end))
}

/// JavaScript-like `substr(start, length)` using UTF-16 code-unit indices.
///
/// `start` may be negative (relative to the end). `length` is clamped to
/// `>= 0` and truncated to the remaining string length.
pub fn substr(s: &str, start: i64, length: Option<i64>) -> Result<String> {
    let out = substr_j(&JsString::from(s), start, length)?;
    Ok(out.to_std_string_lossy().clone())
}

/// JavaScript-like `indexOf(search, fromIndex)` using UTF-16 code-unit indices.
///
/// Returns the first match index, or `-1` if not found.
///
/// Note: JS returns `min(fromIndex, len)` when `search` is the empty string.
pub fn index_of_j(
    haystack: &JsString,
    needle: JsStr<'_>,
    from_index: i64,
) -> Result<i64> {
    let len = len_as_i64(haystack)?;
    let from_index = clamp_i64_to_range(from_index, 0, len);
    let from_index_usize = usize::try_from(from_index)
        .context("from_index does not fit in usize")?;

    if needle.is_empty() {
        return Ok(from_index);
    }

    let Some(pos) = haystack.index_of(needle, from_index_usize) else {
        return Ok(-1);
    };

    i64::try_from(pos).context("match index does not fit in i64")
}

/// JavaScript-like `indexOf(search, fromIndex)` using UTF-16 code-unit indices.
///
/// Returns the first match index, or `-1` if not found.
///
/// Note: JS returns `min(fromIndex, len)` when `search` is the empty string.
pub fn index_of(haystack: &str, needle: &str, from_index: i64) -> Result<i64> {
    let j_haystack = JsString::from(haystack);
    let j_needle = JsString::from(needle);
    index_of_j(&j_haystack, j_needle.as_str(), from_index)
}

/// JavaScript-like `search(regex)` using UTF-16 code-unit indices.
///
/// Returns the first match index, or `-1` if not found.
pub fn search_j(haystack: JsString, regex: &JsRegExp) -> Result<i64> {
    let context = &mut BoaContext::default();

    let exec_result = regex
        .exec(haystack, context)
        .map_err(|e| anyhow::anyhow!("Regex execution failed: {e}"))?;

    let Some(exec_result) = exec_result else {
        return Ok(-1);
    };

    let index_value = exec_result
        .get(PropertyKey::String(js_string("index")), context)
        .map_err(|e| anyhow::anyhow!("Regex result access failed: {e}"))?;
    let index_i32 = index_value.to_i32(context).map_err(|e| {
        anyhow::anyhow!("Regex match index conversion failed: {e}")
    })?;

    Ok(i64::from(index_i32))
}

/// JavaScript-like `search(regex)` using UTF-16 code-unit indices, given Rust
/// string as input.
///
/// Returns the first match index, or `-1` if not found.
pub fn search(haystack: String, regex: String) -> Result<i64> {
    let js_haystack = JsString::from(haystack.as_str());
    let js_regex = js_regex(&regex)?;

    search_j(js_haystack, &js_regex)
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
    fn js_string_round_trips() -> Result<()> {
        let s = js_string("abc");
        assert_eq!(s.to_std_string_lossy(), "abc");
        Ok(())
    }

    #[crate::ctb_test]
    fn js_regex_compiles_simple_pattern() -> Result<()> {
        let _ = js_regex(r"\d+")?;
        Ok(())
    }

    #[crate::ctb_test]
    fn char_code_at_out_of_bounds_is_none() -> Result<()> {
        let s = JsString::from("abc");
        assert_eq!(char_code_at_j(&s, -1), None);
        assert_eq!(char_code_at_j(&s, 3), None);

        assert_eq!(char_code_at("abc", -1), None);
        assert_eq!(char_code_at("abc", 3), None);
        Ok(())
    }

    #[crate::ctb_test]
    fn take_char_at_keeps_surrogates_together() -> Result<()> {
        // U+1F604 😄 => UTF-16: D83D DE04
        let s = JsString::from("😄!");

        let taken0 = take_char_at_j(&s, 0).unwrap();
        assert_eq!(taken0.to_vec(), vec![0xd83d, 0xde04]);

        let taken1 = take_char_at_j(&s, 1).unwrap();
        assert_eq!(taken1.to_vec(), vec![0xde04]);

        let taken2 = take_char_at_j(&s, 2).unwrap();
        assert_eq!(taken2.to_std_string_lossy(), "!");

        assert!(take_char_at_j(&s, 3).is_none());

        assert_eq!(take_char_at("😄!", 0).as_deref(), Some("😄"));
        assert_eq!(take_char_at("😄!", 2).as_deref(), Some("!"));
        assert_eq!(take_char_at("😄!", 3), None);
        Ok(())
    }

    #[crate::ctb_test]
    fn slice_and_substr_use_utf16_indices() -> Result<()> {
        let s = JsString::from("abcd");
        assert_eq!(slice_j(&s, 1, Some(3))?.to_std_string_lossy(), "bc");
        assert_eq!(slice_j(&s, -2, None)?.to_std_string_lossy(), "cd");

        assert_eq!(substr_j(&s, 1, Some(2))?.to_std_string_lossy(), "bc");
        assert_eq!(substr_j(&s, -2, Some(1))?.to_std_string_lossy(), "c");

        assert_eq!(slice("abcd", 1, Some(3))?, "bc");
        assert_eq!(slice("abcd", -2, None)?, "cd");

        assert_eq!(substr("abcd", 1, Some(2))?, "bc");
        assert_eq!(substr("abcd", -2, Some(1))?, "c");
        Ok(())
    }

    #[crate::ctb_test]
    fn slice_and_substr_handle_surrogate_pairs_via_utf16_units() -> Result<()> {
        // "😄!" => UTF-16 units: [D83D, DE04, 0021]
        assert_eq!(slice("😄!", 0, Some(2))?, "😄");
        assert_eq!(slice("😄!", 2, None)?, "!");
        assert_eq!(substr("😄!", 0, Some(2))?, "😄");
        assert_eq!(substr("😄!", 2, Some(1))?, "!");
        Ok(())
    }

    #[crate::ctb_test]
    fn index_of_matches_js_empty_search_behavior() -> Result<()> {
        let s = JsString::from("abc");
        let empty = JsString::from("");
        assert_eq!(index_of_j(&s, empty.as_str(), 2)?, 2);
        assert_eq!(index_of_j(&s, empty.as_str(), 99)?, 3);

        assert_eq!(index_of("abc", "", 2)?, 2);
        assert_eq!(index_of("abc", "", 99)?, 3);
        Ok(())
    }

    #[crate::ctb_test]
    fn search_returns_match_index_or_minus_one() -> Result<()> {
        assert_eq!(search("abc123".to_string(), r"\d+".to_string())?, 3);
        assert_eq!(search("abcdef".to_string(), r"\d+".to_string())?, -1);
        Ok(())
    }

    #[crate::ctb_test]
    fn search_j_returns_match_index_or_minus_one() -> Result<()> {
        let js_haystack = JsString::from("abc123");
        let re = js_regex(r"\d+")?;
        assert_eq!(search_j(js_haystack, &re)?, 3);

        let js_haystack = JsString::from("abcdef");
        assert_eq!(search_j(js_haystack, &re)?, -1);
        Ok(())
    }
}
