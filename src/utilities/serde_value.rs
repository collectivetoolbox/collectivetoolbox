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

//! Utilities for working with `serde_json::Value` objects, including insertion,
//! extraction, and conversion operations.

use anyhow::{Result, anyhow};
use serde::Serialize;
use serde_json::{Value, to_value};

use crate::circular_dep_base64::standard_base64_to_bytes;
use crate::json;

/// Inserts a key-value pair into a JSON object. If the input is an object, it adds
/// the key directly. If not, it wraps the input in a new object under "original"
/// and adds the new key. Returns the modified `Value`.
pub fn insert_key<T: Serialize>(input: T, key: &str, value: Value) -> Result<Value> {
    let mut val = to_value(input)?;
    if let Value::Object(map) = &mut val {
        map.insert(key.to_string(), value);
        Ok(val)
    } else {
        // If it's not an object, wrap it in a new object with the new key
        let mut map = serde_json::Map::new();
        map.insert(key.to_string(), value);
        map.insert("original".to_string(), val);
        Ok(Value::Object(map))
    }
}

/// Extracts bytes from a `Value`. Supports arrays of u64 (converted to u8) or
/// base64-encoded strings. Returns `None` for unsupported types. Differs from
/// `get_as_bytes` by operating directly on the `Value` rather than extracting
/// from an object key.
pub fn get_bytes(val: &Value) -> Option<Vec<u8>> {
    let value_bytes: Vec<u8> = match val {
        Value::Array(arr) => {
            let mut bytes = Vec::with_capacity(arr.len());
            for v in arr {
                let num = v.as_u64()?;
                let byte = u8::try_from(num).ok()?;
                bytes.push(byte);
            }
            bytes
        }
        Value::String(s) => {
            standard_base64_to_bytes(s.clone()).ok()?
        }
        _ => return None,
    };
    Some(value_bytes)
}

/// Retrieves the value associated with a key from a JSON object. Returns `None`
/// if the `Value` is not an object or the key doesn't exist. Differs from
/// type-specific getters like `get_as_string` by returning the raw `Value`.
pub fn get_key(val: &Value, key: &str) -> Option<Value> {
    if let Value::Object(map) = val {
        map.get(key).cloned()
    } else {
        None
    }
}

/// Retrieve a key and convert it to a String. Returns `None` if the key
/// doesn't exist or is not a string.
pub fn get_string(val: &Value, key: &str) -> Option<String> {
    let val = get_key(val, key)?;
    val.as_str().map(std::string::ToString::to_string)
}

/// Retrieve a key and convert it to a JSON string. Returns `None` if the key
/// doesn't exist.
pub fn get_as_json_string(val: &Value, key: &str) -> Option<String> {
    let val = get_key(val, key)?;
    Some(json!(val))
}

/// Converts a `Value` to a plain string representation. Supports String, Number,
/// Bool, and Null types. Returns an error for unsupported types like Array or
/// Object. Differs from `to_html_form_string` by providing a generic string
/// conversion without HTML-specific handling.
pub fn to_plain_string(val: &Value) -> Result<String> {
    match &val {
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(anyhow!("Unsupported value type for {val:?}")),
    }
}

/// Converts a `Value` to a string suitable for HTML form fields. Handles Bool
/// as "on" for true or None for false, and supports String, Number, and Null.
/// Returns an error for unsupported types. Differs from `to_plain_string` by
/// applying HTML form conventions (e.g., bool handling).
pub fn to_html_form_string(val: &Value) -> Result<Option<String>> {
    match &val {
        serde_json::Value::String(s) => Ok(Some(s.clone())),
        serde_json::Value::Number(n) => Ok(Some(n.to_string())),
        serde_json::Value::Bool(b) => {
            if *b {
                Ok(Some("on".to_string()))
            } else {
                Ok(None)
            }
        }
        serde_json::Value::Null => Ok(Some(String::new())),
        _ => Err(anyhow!("Unsupported value type for {val:?}")),
    }
}

/// Retrieves bytes from a JSON object under a specific key. Combines `get_key`
/// and `get_bytes` to extract and convert the value. Returns `None` if the key
/// doesn't exist or conversion fails.
pub fn get_as_bytes(val: &Value, key: &str) -> Option<Vec<u8>> {
    if let Value::Object(map) = val {
        if let Some(v) = map.get(key) {
            get_bytes(v)
        } else {
            None
        }
    } else {
        None
    }
}

/// Retrieves the u64 value associated with a key from a JSON object.
/// Returns `None` if the `Value` is not an object, the key doesn't exist, or
/// the value is not a u64.
pub fn get_as_u64(val: &Value, key: &str) -> Option<u64> {
    if let Value::Object(map) = val {
        map.get(key).and_then(serde_json::Value::as_u64)
    } else {
        None
    }
}

/// Parses a JSON string into a `serde_json::Value`. Returns an error if parsing
/// fails. This is a convenience function for deserializing JSON strings into
/// `Value` objects.
pub fn from_json_string(s: &str) -> Result<Value> {
    let v: Value = serde_json::from_str(s)?;
    Ok(v)
}

/// Retrieves the value associated with a key from a JSON string representing
/// an object. Combines `from_json_string` and `get_key` to parse and extract
/// the value. Returns an error if parsing fails.
pub fn get_key_from_json_string(s: &str, key: &str) -> Result<Option<Value>> {
    let v: Value = from_json_string(s)?;
    Ok(get_key(&v, key))
}

/// Retrieves the string value associated with a key from a JSON string
/// representing an object. Combines `from_json_string` and `get_as_json_string`
/// to parse and extract the string. Returns an error if parsing fails.
pub fn get_as_json_string_from_json_string(
    s: &str,
    key: &str,
) -> Result<Option<String>> {
    let v: Value = from_json_string(s)?;
    if let Some(val) = get_as_json_string(&v, key) {
        Ok(Some(val.clone()))
    } else {
        Ok(None)
    }
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
    //! Tests for `serde_values` helpers.

    use crate::circular_dep_base64::bytes_to_standard_base64;
    use crate::json_value;

    use super::*;
    use serde::{Deserialize, Serialize};

    /// Simple struct for testing.
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        id: u32,
        name: String,
    }

    /// Test creating a value from a primitive and inserting a key.
    #[crate::ctb_test]
    fn test_insert_key_primitive() -> Result<()> {
        let val = 42u64;
        let v = json_value!(val);
        let v2 = insert_key(v, "extra", Value::String("hello".to_string()))?;
        assert!(get_key(&v2, "extra").is_some());
        assert_eq!(get_as_u64(&v2, "val"), None); // original key not present
        Ok(())
    }

    /// Test creating a value from a struct and extracting fields.
    #[crate::ctb_test]
    fn test_struct_value_and_getters() {
        let s = TestStruct {
            id: 7,
            name: "abc".to_string(),
        };
        let v = json_value!(s);
        assert_eq!(get_as_u64(&v, "id"), Some(7));
        assert_eq!(get_string(&v, "name"), Some("abc".to_string()));
    }

    /// Test inserting a key into a struct value.
    #[crate::ctb_test]
    fn test_insert_key_struct() -> Result<()> {
        let s = TestStruct {
            id: 1,
            name: "xyz".to_string(),
        };
        let v = json_value!(s);
        let v2 = insert_key(v, "extra", Value::Bool(true))?;
        assert_eq!(get_key(&v2, "extra"), Some(Value::Bool(true)));
        assert_eq!(get_string(&v2, "name"), Some("xyz".to_string()));
        Ok(())
    }

    /// Test extracting bytes from an array and a base64 string.
    #[crate::ctb_test]
    fn test_get_bytes_array_and_base64() {
        let arr = vec![1u8, 2, 3, 4];
        let v = json_value!(arr);
        let bytes = get_bytes(&v).unwrap();
        assert_eq!(bytes, vec![1, 2, 3, 4]);

        let base64_str = bytes_to_standard_base64(&[5u8, 6, 7]);
        let v2 = Value::String(base64_str.clone());
        let bytes2 = get_bytes(&v2).unwrap();
        assert_eq!(bytes2, vec![5, 6, 7]);
    }

    /// Test `get_as_bytes` from an object.
    #[crate::ctb_test]
    fn test_get_as_bytes_from_object() {
        let mut obj = serde_json::Map::new();
        obj.insert("data".to_string(), json_value!(vec![9u8, 8]));
        let v = Value::Object(obj);
        let bytes = get_as_bytes(&v, "data").unwrap();
        assert_eq!(bytes, vec![9, 8]);
    }

    /// Test `get_key` directly.
    #[crate::ctb_test]
    fn test_get_key() {
        let mut obj = serde_json::Map::new();
        obj.insert("key1".to_string(), Value::String("value".to_string()));
        let v = Value::Object(obj);
        assert_eq!(
            get_key(&v, "key1"),
            Some(Value::String("value".to_string()))
        );
        assert_eq!(get_key(&v, "missing"), None);
        assert_eq!(get_key(&Value::Null, "key1"), None);
    }

    /// Test `to_plain_string`.
    #[crate::ctb_test]
    fn test_to_plain_string() {
        assert_eq!(
            to_plain_string(&Value::String("test".to_string())).unwrap(),
            "test"
        );
        assert_eq!(to_plain_string(&Value::Number(42.into())).unwrap(), "42");
        assert_eq!(to_plain_string(&Value::Bool(true)).unwrap(), "true");
        assert_eq!(to_plain_string(&Value::Null).unwrap(), "");
        to_plain_string(&Value::Array(vec![])).unwrap_err();
    }

    /// Test `to_html_form_string`.
    #[crate::ctb_test]
    fn test_to_html_form_string() {
        assert_eq!(
            to_html_form_string(&Value::String("test".to_string())).unwrap(),
            Some("test".to_string())
        );
        assert_eq!(
            to_html_form_string(&Value::Number(42.into())).unwrap(),
            Some("42".to_string())
        );
        assert_eq!(
            to_html_form_string(&Value::Bool(true)).unwrap(),
            Some("on".to_string())
        );
        assert_eq!(to_html_form_string(&Value::Bool(false)).unwrap(), None);
        assert_eq!(
            to_html_form_string(&Value::Null).unwrap(),
            Some(String::new())
        );
        to_html_form_string(&Value::Array(vec![])).unwrap_err();
    }

    /// Test `get_as_u64`.
    #[crate::ctb_test]
    fn test_get_as_u64() {
        let mut obj = serde_json::Map::new();
        obj.insert("num".to_string(), Value::Number(123.into()));
        let v = Value::Object(obj);
        assert_eq!(get_as_u64(&v, "num"), Some(123));
        assert_eq!(get_as_u64(&v, "missing"), None);
        assert_eq!(get_as_u64(&Value::Null, "num"), None);
    }

    #[crate::ctb_test]
    fn test_from_json_string() {
        let json = r#"{"num": 123}"#;
        let v = from_json_string(json).unwrap();
        assert_eq!(get_as_u64(&v, "num"), Some(123));
        assert_eq!(get_as_u64(&v, "missing"), None);
        assert_eq!(get_as_u64(&Value::Null, "num"), None);
    }
}
