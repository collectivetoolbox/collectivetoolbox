use ctb_utilities::string::to_char;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow};

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

pub fn to_fancy_hex_dump(data: &[u8]) -> Result<String> {
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
            out.push(byte_to_graphical(b)?);
        }
        out.push_str("|\n");
    }
    Ok(out)
}

pub fn byte_to_graphical(b: u8) -> Result<char> {
    if b == 0 {
        Ok('␀')
    } else if b == 1 {
        Ok('␁')
    } else if b == 2 {
        Ok('␂')
    } else if b == 3 {
        Ok('␃')
    } else if b == 4 {
        Ok('␄')
    } else if b == 5 {
        Ok('␅')
    } else if b == 6 {
        Ok('␆')
    } else if b == 7 {
        Ok('␇')
    } else if b == 8 {
        Ok('␈')
    } else if b == 9 {
        Ok('␉')
    } else if b == 10 {
        Ok('␊')
    } else if b == 11 {
        Ok('␋')
    } else if b == 12 {
        Ok('␌')
    } else if b == 13 {
        Ok('␍')
    } else if b == 14 {
        Ok('␎')
    } else if b == 15 {
        Ok('␏')
    } else if b == 16 {
        Ok('␐')
    } else if b == 17 {
        Ok('␑')
    } else if b == 18 {
        Ok('␒')
    } else if b == 19 {
        Ok('␓')
    } else if b == 20 {
        Ok('␔')
    } else if b == 21 {
        Ok('␕')
    } else if b == 22 {
        Ok('␖')
    } else if b == 23 {
        Ok('␗')
    } else if b == 24 {
        Ok('␘')
    } else if b == 25 {
        Ok('␙')
    } else if b == 26 {
        Ok('␚')
    } else if b == 27 {
        Ok('␛')
    } else if b == 28 {
        Ok('␜')
    } else if b == 29 {
        Ok('␝')
    } else if b == 30 {
        Ok('␞')
    } else if b == 31 {
        Ok('␟')
    } else if b == 32 {
        Ok('␠')
    } else if (33..=126).contains(&b) {
        Ok(char::from(b))
    } else if b == 127 {
        Ok('␡')
    } else if b < 255 {
        ctb_formats_encoding::cp437::chr_char(b)
    } else {
        Ok('⍽')
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
    fn test_traditional_hexdump() {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_hex_dump(data);
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello, World!...|\n";
        assert_eq!(dump, expected);
    }

    #[crate::ctb_test]
    fn test_fancy_hexdump() -> Result<()> {
        let data = b"Hello, World!\x00\x01\xff";
        let dump = to_fancy_hex_dump(data)?;
        // Note: 'H' is 'H' (72), '\x00' is '␀', '\x01' is '␁', '\xff' is ' ' (NBSP in CP437, code 160)
        let expected = "00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 21 00 01 ff  |Hello,␠World!␀␁⍽|\n";
        assert_eq!(dump, expected);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_byte_to_graphical_all_bytes() -> Result<()> {
        // Verify every byte 0..=255 returns a unique character
        let mut seen = std::collections::HashSet::new();
        for b in 0u8..=255 {
            let c = byte_to_graphical(b)?;
            assert!(seen.insert(c), "Duplicate glyph found for byte {b}: {c}");
        }
        assert_eq!(seen.len(), 256);
        Ok(())
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
        hex2bin("not a hex string").unwrap_err();
    }
}
