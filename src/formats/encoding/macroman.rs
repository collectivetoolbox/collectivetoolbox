#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_utilities::anyhow::anyhow;

pub fn chr(code: u8) -> String {
    let bytes = [code];
    let (cow, _, _) = encoding_rs::MACINTOSH.decode(&bytes);
    cow.into_owned()
}

pub fn asc(s: &str) -> Option<u8> {
    let (cow, _, had_errors) = encoding_rs::MACINTOSH.encode(s);
    if had_errors {
        None
    } else {
        cow.first().copied()
    }
}

pub fn encode(input: &str) -> Result<Vec<u8>> {
    let (cow, _, had_errors) = encoding_rs::MACINTOSH.encode(input);
    if had_errors {
        Err(anyhow!("Encoding error: unmappable characters"))
    } else {
        Ok(cow.into_owned())
    }
}

pub fn decode(input: &[u8]) -> Result<String> {
    let (cow, _, had_errors) = encoding_rs::MACINTOSH.decode(input);
    if had_errors {
        Err(anyhow!("Decoding error: invalid bytes"))
    } else {
        Ok(cow.into_owned())
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
    use super::*;

    #[crate::ctb_test]
    fn test_macroman_encoding() {
        let original = "Hello, World! ñ ü á";
        let encoded = encode(original).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(original, decoded);

        assert_eq!(encode("caf\u{e9}").unwrap(), vec![99, 97, 102, 142]);
    }

    #[crate::ctb_test]
    fn test_chr_asc() {
        for code in 0u8..=255 {
            let character = chr(code);
            let retrieved_code = asc(&character).unwrap();
            assert_eq!(code, retrieved_code);
        }
        assert_eq!(65, asc("A").unwrap());
        assert_eq!("A", chr(65));
        assert_eq!(194, asc("¬").unwrap());
        assert_eq!("¬", chr(194));
    }
}
