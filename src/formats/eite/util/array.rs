//! Array utilities

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

pub fn arr_empty<T>(a: &[T]) -> bool {
    a.is_empty()
}
pub fn arr_nonempty<T>(a: &[T]) -> bool {
    !a.is_empty()
}

pub fn arr_eq<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).all(|(x, y)| x == y)
}

/// count(array)
pub fn count<T>(a: &[T]) -> usize {
    a.len()
}

pub fn append_vec<T: Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut out = Vec::with_capacity(a.len().saturating_add(b.len()));
    out.extend_from_slice(a);
    out.extend_from_slice(b);
    out
}

pub fn js_compat_array_slice<T: Clone>(
    a: &[T],
    start: i64,
    end_exclusive: i64,
) -> Option<Vec<T>> {
    let len = i64::try_from(a.len()).expect("usize not fit in i64");

    // Convert negative indices relative to length
    let mut s = if start < 0 { len.saturating_add(start) } else { start };
    let mut e = if end_exclusive < 0 {
        len.saturating_add(end_exclusive)
    } else {
        end_exclusive
    };

    // Clamp to [0, len]
    if s < 0 {
        s = 0;
    }
    if e < 0 {
        e = 0;
    }
    if s > len {
        s = len;
    }
    if e > len {
        e = len;
    }

    // If end <= start return empty vec (JS slice returns empty array)
    if e <= s {
        return None;
    }

    let s = usize::try_from(s).ok()?;
    let e = usize::try_from(e).ok()?;
    let slice = a.get(s..e)?;

    Some(slice.to_vec())
}

/// Close translation of StageL anSubset.
pub fn subset<T: Clone>(
    a: &[T],
    mut start: i64,
    mut end: i64,
) -> Option<Vec<T>> {
    let mut count = i64::try_from(a.len()).expect("usize not fit in i64");

    if start < 0 {
        start = start.saturating_add(count);
    }
    if end < 0 {
        end = end.saturating_add(count);
    }

    /* Copilot suggests:
       if start < 0 || end > len || start > end {
           return None;
       }

       let mut res = Vec::new();
       let mut i = start;

       while i < end {
           // Safe to call `a.get(i as usize)` because of the bounds check above.
           if let Some(item) = a.get(i as usize) {
               res.push(item.clone());
           } else {
               // This should not happen because of the bounds check, but just in case
               return None;
           }
           i += 1;
       }

       Some(res)
    */

    count = end;
    let mut res: Vec<T> = Vec::new();

    let mut i = i128::from(start);
    let count_i128 = i128::from(count);
    while i <= count_i128 {
        if let Some(item) = a.get(usize::try_from(i).ok()?) {
            res.push(item.clone());
        } else {
            return None;
        }
        i = i.saturating_add(1);
    }

    Some(res)
}

/// JS pop(array) -> subset(array, 0, -2) meaning drop last element.
/// We replicate safe behavior (no negative wrap).
pub fn pop<T: Clone>(a: &[T]) -> Vec<T> {
    if a.is_empty() {
        Vec::new()
    } else {
        a.get(..a.len().saturating_sub(1)).unwrap_or(&[]).to_vec()
    }
}

/// shift(array) -> subset(array, 1, -1) => drop first element
pub fn shift<T: Clone>(a: &[T]) -> Vec<T> {
    if a.len() <= 1 {
        Vec::new()
    } else {
        a.get(1..).unwrap_or(&[]).to_vec()
    }
}

/// first(array)
pub fn first<T: Clone>(a: &[T]) -> Option<T> {
    a.first().cloned()
}

/// last(array)
pub fn last<T: Clone>(a: &[T]) -> Option<T> {
    a.last().cloned()
}

/// setElement(array, index, value) – panics if out of range except allowing
/// index == len (append) in the original? JS code forbade index > length.
/// If index == len we append.
pub fn set_element<T: Clone>(a: &mut Vec<T>, index: isize, value: T) {
    let len = isize::try_from(a.len()).expect("usize not fit in isize");
    let mut idx = index;
    if idx < 0 {
        idx = idx.saturating_add(len);
    }
    assert!(
        !(idx < 0 || idx > len),
        "Cannot insert at position {index} greater than appending to the length of the array."
    );
    if idx == len {
        a.push(value);
    } else if let Some(elem) = a.get_mut(usize::try_from(idx).unwrap_or(0)) {
        *elem = value;
    }
}

// ===============
// SUBSET HELPERS (abSubset / anSubset / asSubset)
// These treat the end as inclusive, and allow negative indexes.
// ===============

fn normalize_bounds(len: usize, start: isize, end: isize) -> (usize, usize) {
    let l = isize::try_from(len).expect("usize not fit in isize");
    let s = if start < 0 { l.saturating_add(start) } else { start };
    let e = if end < 0 { l.saturating_add(end) } else { end };
    let s = s.clamp(0, l.max(0));
    let e = e.clamp(0, l.max(0));
    (
        usize::try_from(s).expect("isize not fit in usize"),
        usize::try_from(e).expect("isize not fit in usize"),
    )
}

pub fn slice_inclusive_bool(a: &[bool], start: isize, end: isize) -> Vec<bool> {
    if a.is_empty() {
        return Vec::new();
    }
    let (s, e) = normalize_bounds(a.len(), start, end);
    if e < s {
        return Vec::new();
    }
    a.get(s..=e.min(a.len().saturating_sub(1))).unwrap_or(&[]).to_vec()
}

pub fn slice_inclusive_i32(a: &[i32], start: isize, end: isize) -> Vec<i32> {
    if a.is_empty() {
        return Vec::new();
    }
    let (s, e) = normalize_bounds(a.len(), start, end);
    if e < s {
        return Vec::new();
    }
    a.get(s..=e.min(a.len().saturating_sub(1))).unwrap_or(&[]).to_vec()
}

pub fn slice_inclusive_string(
    a: &[String],
    start: isize,
    end: isize,
) -> Vec<String> {
    if a.is_empty() {
        return Vec::new();
    }
    let (s, e) = normalize_bounds(a.len(), start, end);
    if e < s {
        return Vec::new();
    }
    a.get(s..=e.min(a.len().saturating_sub(1))).unwrap_or(&[]).to_vec()
}

/// Convert a single primitive into a one-element array (abFromB / anFromN / asFromS).
pub fn one_bool(b: bool) -> Vec<bool> {
    vec![b]
}
pub fn one_i32(n: i32) -> Vec<i32> {
    vec![n]
}
pub fn one_string(s: impl Into<String>) -> Vec<String> {
    vec![s.into()]
}

/// contains(genericArrayIn, genericValue) – specialized to strings & ints for now.
/// For a more dynamic approach we'd pass `Vec<Value>` and a Value.
/// We implement a generic helper where T: `PartialEq`.
pub fn contains<T: PartialEq>(a: &[T], needle: &T) -> bool {
    a.iter().any(|v| v == needle)
}

pub fn index_of<T: PartialEq>(a: &[T], needle: &T) -> isize {
    for (i, v) in a.iter().enumerate() {
        if v == needle {
            return isize::try_from(i).expect("usize not fit in isize");
        }
    }
    -1
}

// ---------------
// Printing arrays (space-delimited)
// ---------------

pub fn str_print_arr<T: ToString>(arr: &[T]) -> String {
    arr.iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Wrapper kept for compatibility: `print_array` -> `str_print_arr`
pub fn print_array<T: ToString>(arr: &[T]) -> String {
    str_print_arr(arr)
}

/// Wrapper kept for compatibility: `str_print_array` -> `str_print_arr`
pub fn str_print_array<T: ToString>(arr: &[T]) -> String {
    str_print_arr(arr)
}

/// Wrapper kept for compatibility: `print_arr` -> `str_print_arr`
pub fn print_arr<T: ToString>(arr: &[T]) -> String {
    str_print_arr(arr)
}

// ---------------
// Summation
// ---------------

pub fn sum_array(a: &[i32]) -> i32 {
    a.iter().copied().sum()
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_slice_inclusive() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(slice_inclusive_i32(&v, 1, 3), vec![2, 3, 4]);
        assert_eq!(slice_inclusive_i32(&v, -3, -1), vec![3, 4, 5]);
        assert_eq!(slice_inclusive_i32(&v, 0, 0), vec![1]);
    }

    #[crate::ctb_test]
    fn test_contains() {
        let v = vec![String::from("a"), String::from("b")];
        assert!(contains(&v, &"a".to_string()));
        assert!(!contains(&v, &"c".to_string()));
    }

    // PART 2

    #[crate::ctb_test]
    fn test_index_of() {
        let v = vec![1, 2, 3];
        assert_eq!(index_of(&v, &2), 1);
        assert_eq!(index_of(&v, &4), -1);
    }

    #[crate::ctb_test]
    fn test_arr_eq() {
        assert!(arr_eq(&[1, 2, 3], &[1, 2, 3]));
        assert!(!arr_eq(&[1, 2], &[1, 2, 3]));
    }

    #[crate::ctb_test]
    fn test_sum_array() {
        assert_eq!(sum_array(&[1, 2, 3, 4]), 10);
    }
}
