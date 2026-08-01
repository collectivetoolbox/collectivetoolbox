#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::collections::HashMap;

use crate::bail_if_none;

/// Converts bytes to standard RFC 4648 base64 representation.
/// <https://datatracker.ietf.org/doc/html/rfc4648#section-4>
pub fn bytes_to_standard_base64(bytes: &[u8]) -> String {
    BASE64_STANDARD.encode(bytes)
}
pub fn base64_encode(bytes: &[u8]) -> String {
    bytes_to_standard_base64(bytes)
}

pub fn standard_base64_to_bytes(base64: &str) -> Result<Vec<u8>> {
    BASE64_STANDARD
        .decode(base64)
        .map_err(|e| anyhow!("Failed to decode base64: {e}"))
}
pub fn base64_decode(base64: &str) -> Result<Vec<u8>> {
    standard_base64_to_bytes(base64)
}

/// Converts base64 input string to decimal representation. Padding becomes 64.
pub fn standard_base64_to_decimal(base64: String) -> Result<Vec<u8>> {
    let alphabet = base64::alphabet::STANDARD.as_str();

    // build an index of the base64 alphabet
    let index: HashMap<u8, u8> = alphabet
        .chars()
        .enumerate()
        .map(|(i, c)| (u8::try_from(c).unwrap(), u8::try_from(i).unwrap()))
        .collect();

    let mut out = Vec::new();
    // convert the decoded bytes to decimal using the index
    for byte in base64.bytes() {
        if byte == 61 {
            out.push(64);
        } else {
            out.push(bail_if_none!(index.get(&byte).copied()));
        }
    }
    Ok(out)
}

/// Converts decimal representation (0 = A, 1 = B and so on) to standard RFC
/// 4648 base64 string. Padding is represented by 64.
pub fn decimal_to_standard_base64(decimal: Vec<u8>) -> Result<String> {
    let alphabet = base64::alphabet::STANDARD.as_str();
    let mut out = String::new();

    for &value in &decimal {
        let v = value;
        if v == 64 {
            out.push('=');
            continue;
        }
        out.push(bail_if_none!(alphabet.chars().nth(usize::from(v))));
    }

    Ok(out)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use crate::utilities::{assert_vec_u8_eq, assert_vec_u8_ok_eq};
    use ctb_formats_utilities::assert_string_ok_eq;

    use super::*;

    #[crate::ctb_test]
    fn test_bytes_to_standard_base64() {
        let input = b"Hello, world!";
        let expected = "SGVsbG8sIHdvcmxkIQ==";
        let result = bytes_to_standard_base64(input);
        assert_eq!(result, expected);
    }

    #[crate::ctb_test]
    fn test_standard_base64_to_bytes() {
        let input = "SGVsbG8sIHdvcmxkIQ==".to_string();
        let expected = b"Hello, world!";
        let result = standard_base64_to_bytes(&input).unwrap();
        assert_vec_u8_eq(expected, &result);
    }

    #[crate::ctb_test]
    fn test_base64_to_decimal() {
        let input = "SGVsbG8sIHdvcmxkIQ==";
        let expected = vec![
            18, 6, 21, 44, 27, 6, 60, 44, 8, 7, 29, 47, 28, 38, 49, 36, 8, 16,
            64, 64,
        ];
        let result = standard_base64_to_decimal(input.to_string());
        assert_vec_u8_ok_eq(&expected, result);
    }

    #[crate::ctb_test]
    fn test_decimal_to_base64() {
        let input = vec![
            18, 6, 21, 44, 27, 6, 60, 44, 8, 7, 29, 47, 28, 38, 49, 36, 8, 16,
            64, 64,
        ];
        let expected = "SGVsbG8sIHdvcmxkIQ==";
        let result = decimal_to_standard_base64(input);
        assert_string_ok_eq(expected, result);
    }
}
