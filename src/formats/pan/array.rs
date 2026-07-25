/* SPDX-License-Identifier: MIT */
//! Pan-style array manipulation helpers.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};

/// Returns the 1-based `item` from `text` split by `sep`.
///
/// Returns an empty string when the item is out of range.
pub fn array(text: &str, item: usize, sep: char) -> Result<String> {
    if item == 0 {
        bail!("array item numbers start at 1");
    }

    let idx = item.saturating_sub(1);
    Ok(text.split(sep).nth(idx).unwrap_or("").to_string())
}

/// Returns true if any element equals `needle`.
pub fn arraycontains(text: &str, needle: &str, sep: char) -> Result<bool> {
    Ok(text.split(sep).any(|el| el == needle))
}

/// Replaces the 1-based `item` with `value` when it exists.
pub fn arraychange(
    text: &str,
    value: &str,
    item: usize,
    sep: char,
) -> Result<String> {
    if item == 0 {
        bail!("array item numbers start at 1");
    }

    let mut parts: Vec<&str> = text.split(sep).collect();
    let idx = item.saturating_sub(1);
    let Some(slot) = parts.get_mut(idx) else {
        return Ok(text.to_string());
    };

    *slot = value;
    Ok(join_parts(&parts, sep))
}

/// Deletes `count` items starting at the 1-based `item` index.
pub fn arraydelete(
    text: &str,
    item: usize,
    count: usize,
    sep: char,
) -> Result<String> {
    if item == 0 {
        bail!("array item numbers start at 1");
    }
    if count == 0 {
        return Ok(text.to_string());
    }

    let mut parts: Vec<&str> = text.split(sep).collect();
    let start = item.saturating_sub(1);
    if start >= parts.len() {
        return Ok(text.to_string());
    }

    let end_exclusive = start.saturating_add(count).min(parts.len());
    parts.drain(start..end_exclusive);
    Ok(join_parts(&parts, sep))
}

/// Removes duplicate elements and returns a sorted array.
pub fn arraydeduplicate(text: &str, sep: char) -> Result<String> {
    let set: BTreeSet<String> = text.split(sep).map(str::to_string).collect();
    let parts: Vec<&str> = set.iter().map(String::as_str).collect();
    Ok(join_parts(&parts, sep))
}

/// Returns unique elements present in both arrays.
///
/// The result preserves the order of `a1` and ignores empty elements.
pub fn arrayboth(a1: &str, a2: &str, sep: char) -> Result<String> {
    let b: HashSet<&str> = a2.split(sep).filter(|s| !s.is_empty()).collect();

    let mut out: Vec<&str> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for el in a1.split(sep).filter(|s| !s.is_empty()) {
        if b.contains(el) && seen.insert(el) {
            out.push(el);
        }
    }

    Ok(join_parts(&out, sep))
}

/// Returns unique elements in `a1` that are not in `a2`.
///
/// The result preserves the order of `a1` and ignores empty elements.
pub fn arraydifference(a1: &str, a2: &str, sep: char) -> Result<String> {
    let b: HashSet<&str> = a2.split(sep).filter(|s| !s.is_empty()).collect();

    let mut out: Vec<&str> = Vec::new();
    let mut seen: HashSet<&str> = HashSet::new();
    for el in a1.split(sep).filter(|s| !s.is_empty()) {
        if !b.contains(el) && seen.insert(el) {
            out.push(el);
        }
    }

    Ok(join_parts(&out, sep))
}

/// Returns the 1-based element index containing `position`.
///
/// The `position` is 1-based in character indices.
pub fn arrayelement(text: &str, position: usize, sep: char) -> Result<usize> {
    if position == 0 {
        bail!("character positions start at 1");
    }

    let mut elem_no: usize = 1;
    let mut char_pos: usize = 0;

    for ch in text.chars() {
        char_pos = char_pos.saturating_add(1);

        if char_pos == position {
            if ch == sep {
                return Ok(elem_no.saturating_add(1));
            }
            return Ok(elem_no);
        }

        if ch == sep {
            elem_no = elem_no.saturating_add(1);
        }
    }

    Ok(elem_no)
}

/// Extracts a column from a row/column delimited table.
pub fn arraycolumn(
    text: &str,
    colnum: usize,
    rowsep: char,
    colsep: char,
) -> Result<String> {
    if colnum == 0 {
        bail!("array column numbers start at 1");
    }

    let idx = colnum.saturating_sub(1);

    let mut out: Vec<String> = Vec::new();
    for row in text.split(rowsep) {
        let cell = row.split(colsep).nth(idx).unwrap_or("");
        out.push(cell.to_string());
    }

    let parts: Vec<&str> = out.iter().map(String::as_str).collect();
    Ok(join_parts(&parts, rowsep))
}

/// Removes alternating matches of `value`.
///
/// Consecutive matches preserve every second element.
pub fn arraydeletevalue(text: &str, value: &str, sep: char) -> Result<String> {
    let mut out: Vec<&str> = Vec::new();

    // When matches are consecutive, leave the second occurrence behind.
    let mut deleted_previous_match = false;

    for el in text.split(sep) {
        if el == value {
            if deleted_previous_match {
                out.push(el);
                deleted_previous_match = false;
            } else {
                deleted_previous_match = true;
            }
            continue;
        }

        out.push(el);
        deleted_previous_match = false;
    }

    Ok(join_parts(&out, sep))
}

/// Returns a slice from `first` to `last` (inclusive), 1-based.
pub fn arrayrange(
    text: &str,
    first: usize,
    last: usize,
    sep: char,
) -> Result<String> {
    if first == 0 || last == 0 {
        bail!("array item numbers start at 1");
    }
    if first > last {
        return Ok(String::new());
    }

    let parts: Vec<&str> = text.split(sep).collect();

    let start = first.saturating_sub(1);
    if start >= parts.len() {
        return Ok(String::new());
    }

    let last_idx = last.saturating_sub(1);
    let end_exclusive = last_idx.saturating_add(1).min(parts.len());

    Ok(join_parts(parts.get(start..end_exclusive).context("Invalid slice range")?, sep))
}

/// Returns the first element in the array.
pub fn arrayfirst(text: &str, sep: char) -> Result<String> {
    array(text, 1, sep)
}

/// Returns the last element in the array.
pub fn arraylast(text: &str, sep: char) -> Result<String> {
    Ok(text.split(sep).next_back().unwrap_or("").to_string())
}

/// Inserts `count` empty elements at the 1-based `item` position.
pub fn arrayinsert(
    text: &str,
    item: usize,
    count: usize,
    sep: char,
) -> Result<String> {
    if item == 0 {
        bail!("array item numbers start at 1");
    }
    if count == 0 {
        return Ok(text.to_string());
    }

    let mut parts: Vec<String> = text.split(sep).map(str::to_string).collect();
    let insert_at = (item.saturating_sub(1)).min(parts.len());
    for _ in 0..count {
        parts.insert(insert_at, String::new());
    }

    Ok(join_strings(&parts, sep))
}

/// Removes `count` elements from the start of the array.
pub fn arraylefttrim(text: &str, count: usize, sep: char) -> Result<String> {
    if count == 0 {
        return Ok(text.to_string());
    }

    let mut parts: Vec<&str> = text.split(sep).collect();
    let end_exclusive = count.min(parts.len());
    parts.drain(0..end_exclusive);
    Ok(join_parts(&parts, sep))
}

/// Removes `count` elements from the end of the array.
pub fn arraytrim(text: &str, count: usize, sep: char) -> Result<String> {
    if count == 0 {
        return Ok(text.to_string());
    }

    let mut parts: Vec<&str> = text.split(sep).collect();
    let len = parts.len();
    let start = len.saturating_sub(count);
    parts.drain(start..len);
    Ok(join_parts(&parts, sep))
}

/// Looks up `key` in key/value rows and returns its value.
///
/// Rows are split by `mainsep`, keys and values by `subsep`.
pub fn arraylookup(
    text: &str,
    key: &str,
    mainsep: char,
    subsep: char,
    default: &str,
) -> Result<String> {
    for line in text.split(mainsep) {
        let (k, v) = split_kv(line, subsep);
        if k == key {
            return Ok(v.to_string());
        }
    }
    Ok(default.to_string())
}

/// Looks up `key` as a value and returns its row key.
///
/// Rows are split by `mainsep`, keys and values by `subsep`.
pub fn arrayreverselookup(
    text: &str,
    key: &str,
    mainsep: char,
    subsep: char,
    default: &str,
) -> Result<String> {
    for line in text.split(mainsep) {
        let (k, v) = split_kv(line, subsep);
        if v == key {
            return Ok(k.to_string());
        }
    }
    Ok(default.to_string())
}

/// Zips two arrays, joining elements with `joiner`.
///
/// The `joiner` must be 1 to 10 characters long.
pub fn arraymerge(
    array1: &str,
    array2: &str,
    separator: char,
    joiner: &str,
) -> Result<String> {
    let joiner_len = joiner.chars().count();
    if joiner_len == 0 || joiner_len > 10 {
        bail!("arraymerge joiner must be 1 to 10 characters");
    }

    let a1: Vec<&str> = array1.split(separator).collect();
    let a2: Vec<&str> = array2.split(separator).collect();
    let len = a1.len().max(a2.len());

    let mut out: Vec<String> = Vec::with_capacity(len);
    for i in 0..len {
        let left = a1.get(i).copied().unwrap_or("");
        let right = a2.get(i).copied().unwrap_or("");
        out.push(format!("{left}{joiner}{right}"));
    }

    Ok(join_strings(&out, separator))
}

/// Returns true if no element equals `needle`.
pub fn arraynotcontains(text: &str, needle: &str, sep: char) -> Result<bool> {
    Ok(!text.split(sep).any(|el| el == needle))
}

/// Returns the array sorted lexicographically.
pub fn arraysort(text: &str, separator: char) -> Result<String> {
    let mut parts: Vec<&str> = text.split(separator).collect();
    parts.sort_unstable();
    Ok(join_parts(&parts, separator))
}

/// Returns the array sorted by numeric value.
///
/// Returns an error if any element is non-numeric or non-finite.
pub fn arraynumericsort(text: &str, separator: char) -> Result<String> {
    let mut parts: Vec<(f64, &str)> = Vec::new();
    for el in text.split(separator) {
        let n: f64 = el.parse().with_context(|| {
            format!("arraynumericsort element is not numeric: {el:?}")
        })?;
        if !n.is_finite() {
            bail!("arraynumericsort element is not finite: {el:?}");
        }
        parts.push((n, el));
    }

    parts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    let out: Vec<&str> = parts.into_iter().map(|(_, s)| s).collect();
    Ok(join_parts(&out, separator))
}

/// Sums numeric elements of the array.
///
/// Returns an error if any element is non-numeric or non-finite.
pub fn arraynumerictotal(text: &str, separator: char) -> Result<f64> {
    let mut sum = 0.0f64;
    for el in text.split(separator) {
        let n: f64 = el.parse().with_context(|| {
            format!("arraynumerictotal element is not numeric: {el:?}")
        })?;
        if !n.is_finite() {
            bail!("arraynumerictotal element is not finite: {el:?}");
        }
        sum += n;
    }
    Ok(sum)
}

/// Deterministically shuffles elements based on the input.
pub fn arrayrandomize(text: &str, separator: char) -> Result<String> {
    let mut parts: Vec<String> =
        text.split(separator).map(str::to_string).collect();
    if parts.len() <= 1 {
        return Ok(text.to_string());
    }

    let seed = stable_hash64(text) ^ stable_hash64(&separator.to_string());
    shuffle_deterministic(&mut parts, seed)?;

    // Avoid an unhelpful "randomization" that returns the original order.
    if join_strings(&parts, separator) == text {
        parts.rotate_left(1);
    }

    Ok(join_strings(&parts, separator))
}

/// Replaces alternating matches of `oldvalue` with `newvalue`.
///
/// Consecutive matches preserve every second element.
pub fn arrayreplacevalue(
    text: &str,
    oldvalue: &str,
    newvalue: &str,
    sep: char,
) -> Result<String> {
    let mut out: Vec<&str> = Vec::new();

    let mut replaced_previous_match = false;
    for el in text.split(sep) {
        if el == oldvalue {
            if replaced_previous_match {
                out.push(el);
                replaced_previous_match = false;
            } else {
                out.push(newvalue);
                replaced_previous_match = true;
            }
            continue;
        }

        out.push(el);
        replaced_previous_match = false;
    }

    Ok(join_parts(&out, sep))
}

/// Reverses the order of the array elements.
pub fn arrayreverse(text: &str, sep: char) -> Result<String> {
    let mut parts: Vec<&str> = text.split(sep).collect();
    parts.reverse();
    Ok(join_parts(&parts, sep))
}

/// Searches for a wildcard pattern and returns a 1-based index.
///
/// Returns 0 when no match is found.
pub fn arraysearch(
    array: &str,
    pattern: &str,
    start: usize,
    sep: char,
) -> Result<usize> {
    if start == 0 {
        bail!("array item numbers start at 1");
    }

    let parts: Vec<&str> = array.split(sep).collect();
    let start_idx = start.saturating_sub(1);
    if start_idx >= parts.len() {
        return Ok(0);
    }

    for (i, el) in parts.iter().enumerate().skip(start_idx) {
        if wildcard_match(pattern, el) {
            return Ok(i.saturating_add(1));
        }
    }

    Ok(0)
}

/// Removes empty elements from the array.
pub fn arraystrip(text: &str, sep: char) -> Result<String> {
    let out: Vec<&str> = text.split(sep).filter(|s| !s.is_empty()).collect();
    Ok(join_parts(&out, sep))
}

/// Replaces consecutive duplicates with empty elements.
pub fn arrayunpropagate(text: &str, separator: char) -> Result<String> {
    let parts: Vec<&str> = text.split(separator).collect();
    let mut out: Vec<&str> = Vec::with_capacity(parts.len());

    let mut prev: Option<&str> = None;
    for el in parts {
        if prev == Some(el) {
            out.push("");
        } else {
            out.push(el);
            prev = Some(el);
        }
    }

    Ok(join_parts(&out, separator))
}

/// Returns the number of elements, including empty ones.
pub fn arraysize(text: &str, sep: char) -> Result<usize> {
    Ok(text.split(sep).count())
}

/// Returns the value for `key` or the next greater table key.
pub fn arraytableceiling(
    array: &str,
    key: &str,
    sep: char,
    subsep: char,
    default: &str,
) -> Result<String> {
    arraytable_bound(array, key, sep, subsep, default, BoundDir::Ceiling)
}

/// Returns the value for `key` or the next lesser table key.
pub fn arraytablefloor(
    array: &str,
    key: &str,
    sep: char,
    subsep: char,
    default: &str,
) -> Result<String> {
    arraytable_bound(array, key, sep, subsep, default, BoundDir::Floor)
}

/// Builds a separator-joined sequence from `start` to `end` (inclusive).
pub fn makenumberedarray(sep: char, start: i64, end: i64) -> Result<String> {
    let mut out: Vec<String> = Vec::new();

    if start <= end {
        let mut n = start;
        while n <= end {
            out.push(n.to_string());
            n = n.saturating_add(1);
        }
    } else {
        let mut n = start;
        while n >= end {
            out.push(n.to_string());
            n = n.saturating_sub(1);
        }
    }

    Ok(join_strings(&out, sep))
}

/// Placeholder that requires export/arraybuild iteration context.
pub fn arrayscan(_field: &str, _sep: char) -> Result<String> {
    bail!("arrayscan requires export/arraybuild iteration context")
}

/// Placeholder that requires database scanning and formula evaluation.
pub fn arrayselectedbuild(
    _sep: char,
    _db: &str,
    _formula: &str,
) -> Result<String> {
    bail!(
        "arrayselectedbuild requires database scanning and formula evaluation"
    )
}

/// Placeholder that requires database field context.
pub fn lineitemarray(_field: &str, _separator: char) -> Result<String> {
    bail!("lineitemarray requires database field context")
}

/// Joins iterator items with `sep` into a single array string.
pub fn arraybuild_from_iter<I, S>(sep: char, items: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut out = String::new();
    let mut first = true;
    for item in items {
        if !first {
            out.push(sep);
        }
        first = false;
        out.push_str(item.as_ref());
    }
    Ok(out)
}

/// Placeholder that requires database scanning and formula evaluation.
pub fn arraybuild(_sep: char, _db: &str, _formula: &str) -> Result<String> {
    bail!("arraybuild requires database scanning and formula evaluation")
}

fn join_parts(parts: &[&str], sep: char) -> String {
    let mut out = String::new();
    let mut first = true;
    for p in parts {
        if !first {
            out.push(sep);
        }
        first = false;
        out.push_str(p);
    }
    out
}

fn join_strings(parts: &[String], sep: char) -> String {
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    join_parts(&parts, sep)
}

fn split_kv(line: &str, subsep: char) -> (&str, &str) {
    if subsep == '\0' {
        return (line, line);
    }

    let mut it = line.splitn(2, subsep);
    let k = it.next().unwrap_or("");
    let v = it.next().unwrap_or("");
    (k, v)
}

fn stable_hash64(s: &str) -> u64 {
    // 64-bit FNV-1a.
    let mut h = 0xcbf29ce484222325u64;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3u64);
    }
    h
}

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    if x == 0 {
        x = 0x9e3779b97f4a7c15u64;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn shuffle_deterministic(parts: &mut [String], seed: u64) -> Result<()> {
    let len = parts.len();
    let len_u64 =
        u64::try_from(len).context("arrayrandomize length overflow")?;
    if len <= 1 {
        return Ok(());
    }

    let mut state = seed;
    let mut i = len.saturating_sub(1);
    while i > 0 {
        let i1_u64 = u64::try_from(i.saturating_add(1))
            .context("arrayrandomize index overflow")?;
        let r = xorshift64(&mut state);
        let j_u64 = r
            .checked_rem(i1_u64)
            .context("modulo by zero in arrayrandomize")?;
        let j = usize::try_from(j_u64)
            .context("arrayrandomize index conversion failed")?;
        parts.swap(i, j);

        i = i.saturating_sub(1);
    }

    // Use len_u64 so it is referenced (helps keep this code obviously correct).
    let _ = len_u64;
    Ok(())
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();

    let mut pi = 0usize;
    let mut ti = 0usize;
    let mut star: Option<usize> = None;
    let mut match_ti = 0usize;

    while ti < t.len() {
        if pi < p.len() && (p.get(pi) == Some(&'?') || p.get(pi) == t.get(ti)) {
            pi = pi.saturating_add(1);
            ti = ti.saturating_add(1);
            continue;
        }

        if pi < p.len() && p.get(pi) == Some(&'*') {
            star = Some(pi);
            pi = pi.saturating_add(1);
            match_ti = ti;
            continue;
        }

        let Some(star_pi) = star else {
            return false;
        };

        pi = star_pi.saturating_add(1);
        match_ti = match_ti.saturating_add(1);
        ti = match_ti;
    }

    while pi < p.len() && p.get(pi) == Some(&'*') {
        pi = pi.saturating_add(1);
    }

    pi == p.len()
}

#[derive(Clone, Copy)]
enum BoundDir {
    Ceiling,
    Floor,
}

fn is_all_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut saw_digit = false;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if ch == '.' || ch == '-' || ch == '+' {
            continue;
        }
        return false;
    }

    saw_digit
}

fn arraytable_bound(
    array: &str,
    key: &str,
    sep: char,
    subsep: char,
    default: &str,
    dir: BoundDir,
) -> Result<String> {
    let key_num = if is_all_numeric(key) {
        let n: f64 = key.parse().with_context(|| {
            format!("arraytable key is not numeric: {key:?}")
        })?;
        if !n.is_finite() {
            bail!("arraytable key is not finite: {key:?}");
        }
        Some(n)
    } else {
        None
    };

    let mut best_key_num: Option<f64> = None;
    let mut best_key_txt: Option<&str> = None;
    let mut best_val: Option<&str> = None;

    for line in array.split(sep) {
        let (k, v) = split_kv(line, subsep);

        if let Some(kn) = key_num {
            let Ok(kf) = k.parse::<f64>() else {
                // If numeric mode is chosen, ignore non-numeric table rows.
                continue;
            };
            if !kf.is_finite() {
                continue;
            }

            if kf == kn {
                return Ok(v.to_string());
            }

            let better = match dir {
                BoundDir::Ceiling => {
                    kf > kn && best_key_num.is_none_or(|bk| kf < bk)
                }
                BoundDir::Floor => {
                    kf < kn && best_key_num.is_none_or(|bk| kf > bk)
                }
            };

            if better {
                best_key_num = Some(kf);
                best_val = Some(v);
            }
        } else {
            if k == key {
                return Ok(v.to_string());
            }

            let ord = k.cmp(key);
            let better = match dir {
                BoundDir::Ceiling => {
                    ord == Ordering::Greater
                        && best_key_txt.is_none_or(|bk| k < bk)
                }
                BoundDir::Floor => {
                    ord == Ordering::Less
                        && best_key_txt.is_none_or(|bk| k > bk)
                }
            };

            if better {
                best_key_txt = Some(k);
                best_val = Some(v);
            }
        }
    }

    Ok(best_val.unwrap_or(default).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_array() -> Result<()> {
        let list = ",ABCD,,EFGH,IJKL,,MNOP,,QRST,,UVWX,,YZZZ";
        assert_eq!(array(list, 7, ',')?, "MNOP");
        assert_eq!(array(list, 999, ',')?, "");
        assert!(array(list, 0, ',').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraycontains_exact_match() -> Result<()> {
        let colors = "it;was;a;dark;and;stormy;night";
        assert!(arraycontains(colors, "and", ';')?);
        assert!(!arraycontains(colors, "And", ';')?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraychange_existing_only() -> Result<()> {
        let colors = "it;was;a;dark;and;stormy;night";
        let changed = arraychange(colors, "Very Fun", 6, ';')?;
        assert_eq!(changed, "it;was;a;dark;and;Very Fun;night");

        let unchanged = arraychange(colors, "X", 99, ';')?;
        assert_eq!(unchanged, colors);

        assert!(arraychange(colors, "X", 0, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraydelete_preserves_empties() -> Result<()> {
        let s = ",a,,b,";
        // Elements are ["", "a", "", "b", ""]. Delete item 1 => ["a", "", "b", ""].
        assert_eq!(arraydelete(s, 1, 1, ',')?, "a,,b,");
        assert_eq!(arraydelete("a;b;c;d", 3, 1, ';')?, "a;b;d");

        // No-op cases.
        assert_eq!(arraydelete("a;b", 9, 1, ';')?, "a;b");
        assert_eq!(arraydelete("a;b", 1, 0, ';')?, "a;b");
        assert!(arraydelete("a;b", 0, 1, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraydeduplicate_sorts() -> Result<()> {
        assert_eq!(arraydeduplicate("b;a;b", ';')?, "a;b");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayboth_and_difference_ignore_empty() -> Result<()> {
        let a1 = "a;;b";
        let a2 = "b;c;";
        assert_eq!(arrayboth(a1, a2, ';')?, "b");
        assert_eq!(arraydifference(a1, a2, ';')?, "a");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayelement_position_to_element() -> Result<()> {
        let s = "dark;and;stormy";
        // Position 7 is 'n' in "and" => element 2.
        assert_eq!(arrayelement(s, 7, ';')?, 2);
        // If position is a separator, result is the element to the right.
        assert_eq!(arrayelement(s, 5, ';')?, 2);
        // Out of range positions clamp to the last element.
        assert_eq!(arrayelement(s, 999, ';')?, 3);
        assert!(arrayelement(s, 0, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraycolumn_extracts_missing_as_empty() -> Result<()> {
        let t = "a,b,c|d,e|x";
        assert_eq!(arraycolumn(t, 2, '|', ',')?, "b|e|");
        assert!(arraycolumn(t, 0, '|', ',').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraydeletevalue_deletes_with_consecutive_quirk() -> Result<()> {
        assert_eq!(arraydeletevalue("a;b;a", "a", ';')?, "b");
        assert_eq!(arraydeletevalue("a;a", "a", ';')?, "a");
        assert_eq!(arraydeletevalue("a;a;a", "a", ';')?, "a");
        // Deleting the empty element removes it, collapsing the array.
        assert_eq!(arraydeletevalue("x;;x", "", ';')?, "x;x");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayrange_inclusive_1_based() -> Result<()> {
        assert_eq!(arrayrange("a;b;c;d", 2, 3, ';')?, "b;c");
        assert_eq!(arrayrange("a;b;c;d", 1, 999, ';')?, "a;b;c;d");
        assert_eq!(arrayrange("a;b;c;d", 5, 6, ';')?, "");
        assert_eq!(arrayrange("a;b;c;d", 3, 2, ';')?, "");
        assert!(arrayrange("a;b", 0, 1, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayfirst_and_last() -> Result<()> {
        assert_eq!(arrayfirst(",a", ',')?, "");
        assert_eq!(arraylast(",a,", ',')?, "");
        assert_eq!(arraylast("x;y;z", ';')?, "z");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayinsert_inserts_blanks() -> Result<()> {
        assert_eq!(arrayinsert("a;b;c", 3, 2, ';')?, "a;b;;;c");
        assert_eq!(arrayinsert("a;b", 1, 1, ';')?, ";a;b");
        assert_eq!(arrayinsert("a;b", 99, 2, ';')?, "a;b;;");
        assert!(arrayinsert("a;b", 0, 1, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraylefttrim_and_trim() -> Result<()> {
        assert_eq!(arraylefttrim("a;b;c", 2, ';')?, "c");
        assert_eq!(arraylefttrim("a;b", 99, ';')?, "");
        assert_eq!(arraytrim("a;b;c;d", 2, ';')?, "a;b");
        assert_eq!(arraytrim("a;b", 99, ';')?, "");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraylookup_and_reverselookup() -> Result<()> {
        let t = "NJ.NEW JERSEY;NY.NEW YORK;MD.MARYLAND";
        assert_eq!(arraylookup(t, "NY", ';', '.', "")?, "NEW YORK");
        assert_eq!(arraylookup(t, "ZZ", ';', '.', "X")?, "X");
        assert_eq!(arrayreverselookup(t, "MARYLAND", ';', '.', "")?, "MD");

        let single = "10;20;30";
        assert_eq!(arraylookup(single, "20", ';', '\0', "")?, "20");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraymerge() -> Result<()> {
        assert_eq!(arraymerge("a;b;c", "1;2", ';', ":")?, "a:1;b:2;c:");
        assert!(arraymerge("a", "b", ';', "").is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraynotcontains() -> Result<()> {
        assert!(arraynotcontains("a;b;c", "x", ';')?);
        assert!(!arraynotcontains("a;b;c", "b", ';')?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraysort_and_strip_and_size() -> Result<()> {
        assert_eq!(arraysort("b;a;b", ';')?, "a;b;b");
        assert_eq!(arraystrip("a;;b;", ';')?, "a;b");
        assert_eq!(arraysize("", ';')?, 1);
        assert_eq!(arraysize("a;b", ';')?, 2);
        assert_eq!(arraysize(",", ',')?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraynumericsort_and_total() -> Result<()> {
        assert_eq!(arraynumericsort("10;2;1", ';')?, "1;2;10");
        assert_eq!(arraynumerictotal("1;2.5;-1", ';')?, 2.5);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayrandomize_is_permutation() -> Result<()> {
        let input = "a;b;c;d;e";
        let out = arrayrandomize(input, ';')?;
        assert_ne!(out, input);

        let mut a: Vec<&str> = input.split(';').collect();
        let mut b: Vec<&str> = out.split(';').collect();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayreplacevalue_consecutive_quirk() -> Result<()> {
        assert_eq!(arrayreplacevalue("a;a", "a", "x", ';')?, "x;a");
        assert_eq!(arrayreplacevalue("a;b;a", "a", "x", ';')?, "x;b;x");
        assert_eq!(arrayreplacevalue("x;;x", "", "_", ';')?, "x;_;x");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayreverse() -> Result<()> {
        assert_eq!(arrayreverse("1;2;3;4", ';')?, "4;3;2;1");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraysearch_wildcards() -> Result<()> {
        let s = "Luisa Lee;Jack Chan;Hattie Tyler";
        assert_eq!(arraysearch(s, "Susan*", 1, ';')?, 0);
        assert_eq!(arraysearch(s, "*Tyl*", 1, ';')?, 3);
        assert_eq!(arraysearch(s, "L?isa Lee", 1, ';')?, 1);
        assert_eq!(arraysearch(s, "Nope*", 1, ';')?, 0);
        assert!(arraysearch(s, "*", 0, ';').is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arraytableceiling_and_floor_numeric() -> Result<()> {
        let t = "10.A;20.B;30.C";
        assert_eq!(arraytableceiling(t, "15", ';', '.', "")?, "B");
        assert_eq!(arraytablefloor(t, "15", ';', '.', "")?, "A");
        assert_eq!(arraytableceiling(t, "20", ';', '.', "")?, "B");
        assert_eq!(arraytablefloor(t, "5", ';', '.', "D")?, "D");
        assert_eq!(arraytableceiling(t, "99", ';', '.', "D")?, "D");

        let single = "10;20;30";
        assert_eq!(arraytableceiling(single, "15", ';', '\0', "")?, "20");
        assert_eq!(arraytablefloor(single, "15", ';', '\0', "")?, "10");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_arrayunpropagate() -> Result<()> {
        assert_eq!(arrayunpropagate("a;a;a;b;b;c", ';')?, "a;;;b;;c");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_makenumberedarray() -> Result<()> {
        assert_eq!(makenumberedarray(',', 1, 5)?, "1,2,3,4,5");
        assert_eq!(makenumberedarray(',', 5, 3)?, "5,4,3");
        Ok(())
    }
}
