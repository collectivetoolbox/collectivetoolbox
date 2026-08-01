#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, ensure};

pub fn byte_from_stagel_char(s: &str) -> Result<u8> {
    let c = s
        .chars()
        .next()
        .context("empty string for byte_from_char")?;
    let code = u32::from(c);
    ensure!((32..127).contains(&code), "Non-ASCII supported range");
    u8::try_from(code).context("Failed to convert code to u8")
}

pub fn stagel_char_from_byte(b: u8) -> Result<String> {
    ensure!((0x20..=0x7E).contains(&b), "Out of visible ASCII range");
    Ok((char::from(b)).to_string())
}
