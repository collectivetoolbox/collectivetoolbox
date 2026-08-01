// =====================
// KV ARRAY / KV STRING
// =====================
// Original JS KV array format: [ key, value, key, value, ... ]
// KV string format: "key:value,key2:value2," with (escaped) separators.
// Full escaping rules depend on strSplitEscaped / strJoinEsc / strJoinEscNoTrailing,
// which are not yet provided in the snippet. We therefore implement the
// array-level logic now and leave the string-level split/join until the
// escape utilities arrive.

use crate::utilities::*;

use anyhow::{Result, anyhow};

/// Check whether a slice of strings has even length.
fn is_kv_array(data: &[String]) -> bool {
    data.len().is_multiple_of(2)
}

pub fn kv_has_value(data: &[String], key: &str) -> bool {
    // Mimic original assertKvArray (would throw). We choose panic for now;
    // can be converted to Result once global error model decided.
    assert!(is_kv_array(data), "kv_has_value: invalid kv array length");
    // Walk even indices only
    let mut i: usize = 0;
    while i < data.len() {
        if data.get(i).is_some_and(|k| k == key) {
            return true;
        }
        i = i.saturating_add(2);
    }
    false
}

/// Returns value or empty string if not found (matching original forgiving behavior).
pub fn kv_get_value(data: &[String], key: &str) -> String {
    assert!(is_kv_array(data), "kv_get_value: invalid kv array length");
    let mut i: usize = 0;
    while i.saturating_add(1) < data.len() {
        if data.get(i).is_some_and(|k| k == key) {
            return data.get(i.saturating_add(1)).cloned().unwrap_or_default();
        }
        i = i.saturating_add(2);
    }
    String::new()
}

/// Like `kv_get_value` but asserts key exists (mirrors assertKvHasValue + retrieval).
pub fn kv_get_defined_value(data: &[String], key: &str) -> Result<String> {
    if kv_has_value(data, key) {
        Ok(kv_get_value(data, key))
    } else {
        Err(anyhow!("Key '{key}' not found"))
    }
}

/// Set (insert or replace) a key/value pair, returning a new `Vec<String>`.
pub fn kv_set_value(
    mut data: Vec<String>,
    key: &str,
    val: &str,
) -> Vec<String> {
    assert!(is_kv_array(&data), "kv_set_value: invalid kv array length");
    let mut i: usize = 0;
    while i.saturating_add(1) < data.len() {
        if data.get(i).is_some_and(|k| k == key) {
            if let Some(item) = data.get_mut(i.saturating_add(1)) {
                *item = val.to_string();
            }
            return data;
        }
        i = i.saturating_add(2);
    }
    data.push(key.to_string());
    data.push(val.to_string());
    data
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_kv_basic() {
        let data: Vec<String> = vec![];
        let data = kv_set_value(data, "a", "1");
        let data = kv_set_value(data, "b", "2");
        assert!(kv_has_value(&data, "a"));
        assert_eq!(kv_get_value(&data, "a"), "1");
        assert_eq!(kv_get_value(&data, "c"), "");
        let data = kv_set_value(data, "a", "3");
        assert_eq!(kv_get_value(&data, "a"), "3");
    }

    #[crate::ctb_test]
    fn test_kv_has_value_empty_and_odd_length() {
        let data: Vec<String> = vec![];
        assert!(!kv_has_value(&data, "x"));
        let data = vec!["a".to_string()];
        let result = std::panic::catch_unwind(|| kv_has_value(&data, "a"));
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_kv_get_value_empty_and_odd_length() {
        let data: Vec<String> = vec![];
        assert_eq!(kv_get_value(&data, "x"), "");
        let data = vec!["a".to_string()];
        let result = std::panic::catch_unwind(|| kv_get_value(&data, "a"));
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_kv_get_defined_value_found_and_not_found() {
        let mut data: Vec<String> = vec![];
        data = kv_set_value(data, "foo", "bar");
        assert_eq!(kv_get_defined_value(&data, "foo").unwrap(), "bar");
        let err = kv_get_defined_value(&data, "baz");
        assert!(err.is_err());
    }

    #[crate::ctb_test]
    fn test_kv_set_value_insert_and_replace() {
        let mut data: Vec<String> = vec![];
        data = kv_set_value(data, "x", "1");
        assert_eq!(data, vec!["x", "1"]);
        assert_eq!(kv_get_value(&data, "x"), "1");
        data = kv_set_value(data, "x", "2");
        assert_eq!(data, vec!["x", "2"]);
        assert_eq!(kv_get_value(&data, "x"), "2");
    }

    #[crate::ctb_test]
    fn test_kv_set_value_odd_length_panics() {
        let data = vec!["a".to_string()];
        let result = std::panic::catch_unwind(|| kv_set_value(data, "b", "2"));
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_kv_array_get_set_mid_elements() {
        let mut data = vec![
            "k1".to_string(),
            "v1".to_string(),
            "k2".to_string(),
            "v2".to_string(),
            "k3".to_string(),
            "v3".to_string(),
            "k4".to_string(),
            "v4".to_string(),
        ];
        // Get value for k2 and k3
        assert_eq!(kv_get_value(&data, "k2"), "v2");
        assert_eq!(kv_get_value(&data, "k3"), "v3");
        // Replace value for k2
        data = kv_set_value(data, "k2", "v2x");
        assert_eq!(kv_get_value(&data, "k2"), "v2x");
        // Insert new key-value pair
        data = kv_set_value(data, "k5", "v5");
        assert_eq!(kv_get_value(&data, "k5"), "v5");
        // Check full array
        assert_eq!(
            data,
            vec!["k1", "v1", "k2", "v2x", "k3", "v3", "k4", "v4", "k5", "v5"]
        );
    }

    #[crate::ctb_test]
    fn test_kv_array_get_set_first_and_last() {
        let mut data = vec![
            "a".to_string(),
            "1".to_string(),
            "b".to_string(),
            "2".to_string(),
            "c".to_string(),
            "3".to_string(),
            "d".to_string(),
            "4".to_string(),
        ];
        // Get first and last
        assert_eq!(kv_get_value(&data, "a"), "1");
        assert_eq!(kv_get_value(&data, "d"), "4");
        // Replace last
        data = kv_set_value(data, "d", "44");
        assert_eq!(kv_get_value(&data, "d"), "44");
        // Replace first
        data = kv_set_value(data, "a", "11");
        assert_eq!(kv_get_value(&data, "a"), "11");
        // Check full array
        assert_eq!(data, vec!["a", "11", "b", "2", "c", "3", "d", "44"]);
    }
}
