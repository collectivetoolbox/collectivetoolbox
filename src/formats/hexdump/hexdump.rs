use ctb_utilities::string::to_char;

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow};
use std::collections::HashMap;

use crate::bail_if_none;

pub fn to_hex_dump(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}: ", i.saturating_mul(16)));
        for j in 0..16 {
            if let Some(&b) = chunk.get(j) {
                out.push_str(&format!("{b:02x} "));
            } else {
                out.push_str("   ");
            }
        }
        out.push_str(" |");
        for &b in chunk {
            if (0x20..=0x7e).contains(&b) {
                out.push(char::from(b));
            } else {
                out.push('.');
            }
        }
        out.push_str("|\n");
    }
    out
}

pub fn to_fancy_hex_dump(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        out.push_str(&format!("{:08x}: ", i.saturating_mul(16)));
        for j in 0..16 {
            if let Some(&b) = chunk.get(j) {
                out.push_str(&format!("{b:02x} "));
            } else {
                out.push_str("   ");
            }
        }
        out.push_str(" |");
        for &b in chunk {
            out.push(byte_to_graphical(b));
        }
        out.push_str("|\n");
    }
    out
}

pub fn byte_to_graphical(b: u8) -> char {
    if (b == 0) {
        return '␀';
    } else if (b == 1) {
        return '␁';
    } else if (b == 2) {
        return '␂';
    } else if (b == 3) {
        return '␃';
    } else if (b == 4) {
        return '␄';
    } else if (b == 5) {
        return '␅';
    } else if (b == 6) {
        return '␆';
    } else if (b == 7) {
        return '␇';
    } else if (b == 8) {
        return '␈';
    } else if (b == 9) {
        return '␉';
    } else if (b == 10) {
        return '␊';
    } else if (b == 11) {
        return '␋';
    } else if (b == 12) {
        return '␌';
    } else if (b == 13) {
        return '␍';
    } else if (b == 14) {
        return '␎';
    } else if (b == 15) {
        return '␏';
    } else if (b == 16) {
        return '␐';
    } else if (b == 17) {
        return '␑';
    } else if (b == 18) {
        return '␒';
    } else if (b == 19) {
        return '␓';
    } else if (b == 20) {
        return '␔';
    } else if (b == 21) {
        return '␕';
    } else if (b == 22) {
        return '␖';
    } else if (b == 23) {
        return '␗';
    } else if (b == 24) {
        return '␘';
    } else if (b == 25) {
        return '␙';
    } else if (b == 26) {
        return '␚';
    } else if (b == 27) {
        return '␛';
    } else if (b == 28) {
        return '␜';
    } else if (b == 29) {
        return '␝';
    } else if (b == 30) {
        return '␞';
    } else if (b == 31) {
        return '␟';
    } else if (b == 32) {
        return '␠';
    } else if (b >= 33 && b <=126) {
        return char::from(b);
    } else if (b == 127) {
        return '␡';
    } else if (b < 255) {
        return to_char(ctb_formats_encoding::cp437::chr(b)).expect("Should be infallible - codepage 437 decoding returned multiple characters for a single byte?");
    } else {
        return '⍽';
    }
}

/// Decodes a hexadecimal string into a byte vector.
///
/// Any prefix of "0x" or "0X" is stripped, and whitespace or colons are ignored.
pub fn hex2bin(s: &str) -> Result<Vec<u8>> {
    let mut s = s.trim();
    if let Some(stripped) = s.strip_prefix("0x") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("0X") {
        s = stripped;
    }
    let clean: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':')
        .collect();
    hex::decode(&clean).map_err(|e| anyhow!("Failed to decode hex string: {e}"))
}

/// Encodes binary data into a lowercase hexadecimal string.
pub fn bin2hex(data: &[u8]) -> String {
    utilities::bin2hex(data)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_traditional_hexdump() {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_hex_dump(data);
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello, World!...|\n";
        assert_eq!(dump, expected);
    }

    #[crate::ctb_test]
    fn test_fancy_hexdump() {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_fancy_hex_dump(data);
        // Note: 'H' is 'H' (72), '\x00' is '␀', '\x01' is '␁', '\xff' is ' ' (NBSP in CP437, code 160)
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello,␠World!␀␁⍽|\n";
        assert_eq!(dump, expected);
    }

    #[crate::ctb_test]
    fn test_byte_to_graphical_all_bytes() {
        // Verify every byte 0..=255 returns a unique character
        let mut seen = std::collections::HashSet::new();
        for b in 0u8..=255 {
            let c = byte_to_graphical(b);
            assert!(seen.insert(c), "Duplicate glyph found for byte {b}: {c}");
        }
        assert_eq!(seen.len(), 256);
    }

    #[crate::ctb_test]
    fn test_hex2bin_and_bin2hex() {
        let original = b"Hello, World!";
        let encoded = bin2hex(original);
        assert_eq!(encoded, "48656c6c6f2c20576f726c6421");

        let decoded = hex2bin(&encoded).unwrap();
        assert_eq!(decoded, original);

        // Test with 0x prefix, whitespace, and colons
        let complex = "0x48:65:6c:6c:6f 2c 20 57 6f 72 6c 64 21";
        let decoded_complex = hex2bin(complex).unwrap();
        assert_eq!(decoded_complex, original);

        // Test invalid hex input
        assert!(hex2bin("not a hex string").is_err());
    }
}
