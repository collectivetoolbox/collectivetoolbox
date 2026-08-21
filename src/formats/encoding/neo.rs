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

//! Neo character encoding and decoding utilities.
//!
//! Provides bidirectional character mappings for AlphaSmart Neo devices with
//! US, Ukrainian Mac, and Ukrainian PC layouts. By default, the low character
//! area (0x00..=0x1F) uses graphical symbols (the common device character set),
//! with an optional control character mode available. Documentation of the
//! control character encoding (with graphical 0x0) was found commented out in
//! the AlphaSync driver; not sure what its use (if any) was.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::LazyLock;
use ctb_utilities::anyhow::anyhow;

pub use crate::mapping::{LowArea, NeoRegion, SingleByteMapping};

/// Alias for `SingleByteMapping` in the Neo encoding context.
pub type NeoMapping = SingleByteMapping;

fn parse_hex_codepoints(bytes: &[u8]) -> Result<Vec<char>> {
    let s = std::str::from_utf8(bytes)?;
    let mut chars = Vec::new();
    for token in s.split_whitespace() {
        let code = u32::from_str_radix(token, 16)
            .map_err(|e| anyhow!("Failed to parse hex codepoint '{token}': {e}"))?;
        let ch = char::from_u32(code)
            .ok_or_else(|| anyhow!("Invalid Unicode codepoint '{token}'"))?;
        chars.push(ch);
    }
    Ok(chars)
}

/// Loads a `NeoMapping` table for the specified region and low-area mode.
pub fn try_load_mapping(region: NeoRegion, low_area: LowArea) -> Result<NeoMapping> {
    let low_file = match low_area {
        LowArea::Graphical => "neo/low-graphical.csv",
        LowArea::Control => "neo/low-control.csv",
    };
    let high_file = match region {
        NeoRegion::Us => "neo/us.csv",
        NeoRegion::UaMac => "neo/ua-mac.csv",
        NeoRegion::UaPc => "neo/ua-pc.csv",
    };

    let low_bytes = crate::get_encoding_data(low_file)
        .ok_or_else(|| anyhow!("Missing Neo low-area data file: {low_file}"))?;
    let high_bytes = crate::get_encoding_data(high_file)
        .ok_or_else(|| anyhow!("Missing Neo high-area data file: {high_file}"))?;

    let low_chars = parse_hex_codepoints(&low_bytes)?;
    let high_chars = parse_hex_codepoints(&high_bytes)?;

    if low_chars.len() != 32 {
        return Err(anyhow!(
            "Expected 32 low-area characters, got {}",
            low_chars.len()
        ));
    }
    if high_chars.len() != 224 {
        return Err(anyhow!(
            "Expected 224 high-area characters, got {}",
            high_chars.len()
        ));
    }

    let mut decode_table = ['\0'; 256];
    let mut encode_table = HashMap::new();

    for (i, &ch) in low_chars.iter().enumerate() {
        let byte = u8::try_from(i)?;
        if let Some(slot) = decode_table.get_mut(usize::from(byte)) {
            *slot = ch;
        }
        encode_table.entry(ch).or_insert(byte);
    }

    for (i, &ch) in high_chars.iter().enumerate() {
        let byte = u8::try_from(i.saturating_add(32))?;
        if let Some(slot) = decode_table.get_mut(usize::from(byte)) {
            *slot = ch;
        }
        encode_table.entry(ch).or_insert(byte);
    }

    Ok(SingleByteMapping::from_raw(decode_table, encode_table))
}

/// Static instance for Neo US layout with graphical low area (the default/usual encoding).
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_US: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::Us, LowArea::Graphical)
        .expect("Failed to load Neo US (Graphical) mapping")
});

/// Static instance for Neo US layout with control low area (graphical 0x0 byte).
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_US_CONTROL: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::Us, LowArea::Control)
        .expect("Failed to load Neo US (Control) mapping")
});

/// Static instance for Neo Ukrainian Mac layout with graphical low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_UA_MAC: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::UaMac, LowArea::Graphical)
        .expect("Failed to load Neo UA-Mac (Graphical) mapping")
});

/// Static instance for Neo Ukrainian Mac layout with control low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_UA_MAC_CONTROL: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::UaMac, LowArea::Control)
        .expect("Failed to load Neo UA-Mac (Control) mapping")
});

/// Static instance for Neo Ukrainian PC layout with graphical low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_UA_PC: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::UaPc, LowArea::Graphical)
        .expect("Failed to load Neo UA-PC (Graphical) mapping")
});

/// Static instance for Neo Ukrainian PC layout with control low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub static NEO_UA_PC_CONTROL: LazyLock<NeoMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::UaPc, LowArea::Control)
        .expect("Failed to load Neo UA-PC (Control) mapping")
});

/// Returns the static `NeoMapping` for the specified region and low area.
#[must_use]
pub fn get_mapping(region: NeoRegion, low_area: LowArea) -> &'static NeoMapping {
    match (region, low_area) {
        (NeoRegion::Us, LowArea::Graphical) => &NEO_US,
        (NeoRegion::Us, LowArea::Control) => &NEO_US_CONTROL,
        (NeoRegion::UaMac, LowArea::Graphical) => &NEO_UA_MAC,
        (NeoRegion::UaMac, LowArea::Control) => &NEO_UA_MAC_CONTROL,
        (NeoRegion::UaPc, LowArea::Graphical) => &NEO_UA_PC,
        (NeoRegion::UaPc, LowArea::Control) => &NEO_UA_PC_CONTROL,
    }
}

/// Returns the character string for a single byte in the default Neo US layout.
#[must_use]
pub fn chr(code: u8) -> String {
    NEO_US.chr(code)
}

/// Decodes a single byte to its Unicode character in the default Neo US layout.
pub fn chr_char(code: u8) -> Result<char> {
    NEO_US.decode_byte(code)
}

/// Returns the byte value for the first character of a string in default Neo US.
#[must_use]
pub fn asc(s: &str) -> Option<u8> {
    NEO_US.asc(s)
}

/// Encodes a Unicode string into bytes using default Neo US encoding.
pub fn encode(input: &str) -> Result<Vec<u8>> {
    NEO_US.encode(input)
}

/// Decodes bytes into a Unicode string using default Neo US encoding.
pub fn decode(input: &[u8]) -> Result<String> {
    NEO_US.decode(input)
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
    fn test_neo_us_graphical_low_area() -> Result<()> {
        // In graphical mode, 0x00 is ■ (0x25a0), 0x01 is δ (0x03b4), 0x02 is Δ (0x0394)
        assert_eq!(chr(0x00), "■");
        assert_eq!(chr(0x01), "δ");
        assert_eq!(chr(0x02), "Δ");

        assert_eq!(asc("■"), Some(0x00));
        assert_eq!(asc("δ"), Some(0x01));
        assert_eq!(asc("Δ"), Some(0x02));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_us_control_low_area() -> Result<()> {
        let control_mapping = &NEO_US_CONTROL;
        // In control mode, 0x00 is ■ (0x25a0), 0x01 is '\u{1}', 0x02 is '\u{2}'
        assert_eq!(control_mapping.chr(0x00), "■");
        assert_eq!(control_mapping.chr(0x01), "\u{1}");
        assert_eq!(control_mapping.chr(0x02), "\u{2}");

        assert_eq!(control_mapping.asc("\u{1}"), Some(0x01));
        assert_eq!(control_mapping.asc("\u{2}"), Some(0x02));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_us_roundtrip() -> Result<()> {
        let text = "Hello, World! 123 - α β Ω";
        let encoded = encode(text)?;
        let decoded = decode(&encoded)?;
        assert_eq!(decoded, text);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_ua_mac_and_pc() -> Result<()> {
        // Ukrainian Mac layout Cyrillic letters
        let ua_mac = &NEO_UA_MAC;
        let ua_pc = &NEO_UA_PC;

        // Byte 0x22 in UA Mac: 0x0404 ('Є')
        assert_eq!(ua_mac.chr(0x22), "Є");
        assert_eq!(ua_mac.asc("Є"), Some(0x22));

        // Byte 0x22 in UA PC: 0x0404 ('Є')
        assert_eq!(ua_pc.chr(0x22), "Є");
        assert_eq!(ua_pc.asc("Є"), Some(0x22));

        // Roundtrip encode/decode on UA Mac
        let ua_text = "Привіт, світ!";
        let encoded = ua_mac.encode(ua_text)?;
        let decoded = ua_mac.decode(&encoded)?;
        assert_eq!(decoded, ua_text);

        // Roundtrip encode/decode on UA PC
        let encoded_pc = ua_pc.encode(ua_text)?;
        let decoded_pc = ua_pc.decode(&encoded_pc)?;
        assert_eq!(decoded_pc, ua_text);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_remapping() -> Result<()> {
        let mut mapping = NEO_US.clone();
        // Remap byte 0x24 ($) to € (0x20ac)
        assert_eq!(mapping.chr(0x24), "$");
        mapping.remap(0x24, '€');
        assert_eq!(mapping.chr(0x24), "€");
        assert_eq!(mapping.asc("€"), Some(0x24));
        assert_eq!(mapping.decode(&[0x24])?, "€");
        assert_eq!(mapping.encode("€")?, vec![0x24]);
        Ok(())
    }
}
