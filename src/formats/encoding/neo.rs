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
//! the `AlphaSync` driver.
//!
//! `AlphaWord` uses a more complex file format/encoding. FIXME update this comment with details from `AlphaSync`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_utilities::anyhow::anyhow;
use ctb_utilities::string::parse_hex_codepoints;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::mapping::SingleByteMapping;
pub use ctb_formats_utilities::encoding::{LowArea, NeoRegion};

/// Loads a `SingleByteMapping` table for the specified region and low-area mode.
pub fn try_load_mapping(
    region: NeoRegion,
    low_area: LowArea,
) -> Result<SingleByteMapping> {
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
    let high_bytes = crate::get_encoding_data(high_file).ok_or_else(|| {
        anyhow!("Missing Neo high-area data file: {high_file}")
    })?;

    let low_str = std::str::from_utf8(&low_bytes)?;
    let high_str = std::str::from_utf8(&high_bytes)?;

    let low_chars = parse_hex_codepoints(low_str)?;
    let high_chars = parse_hex_codepoints(high_str)?;

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
pub(crate) static NEO_US: LazyLock<SingleByteMapping> = LazyLock::new(|| {
    try_load_mapping(NeoRegion::Us, LowArea::Graphical)
        .expect("Failed to load Neo US (Graphical) mapping")
});

/// Static instance for Neo US layout with control low area (graphical 0x0 byte).
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static NEO_US_CONTROL: LazyLock<SingleByteMapping> =
    LazyLock::new(|| {
        try_load_mapping(NeoRegion::Us, LowArea::Control)
            .expect("Failed to load Neo US (Control) mapping")
    });

/// Static instance for Neo Ukrainian Mac layout with graphical low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static NEO_UA_MAC: LazyLock<SingleByteMapping> =
    LazyLock::new(|| {
        try_load_mapping(NeoRegion::UaMac, LowArea::Graphical)
            .expect("Failed to load Neo UA-Mac (Graphical) mapping")
    });

/// Static instance for Neo Ukrainian Mac layout with control low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static NEO_UA_MAC_CONTROL: LazyLock<SingleByteMapping> =
    LazyLock::new(|| {
        try_load_mapping(NeoRegion::UaMac, LowArea::Control)
            .expect("Failed to load Neo UA-Mac (Control) mapping")
    });

/// Static instance for Neo Ukrainian PC layout with graphical low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static NEO_UA_PC: LazyLock<SingleByteMapping> =
    LazyLock::new(|| {
        try_load_mapping(NeoRegion::UaPc, LowArea::Graphical)
            .expect("Failed to load Neo UA-PC (Graphical) mapping")
    });

/// Static instance for Neo Ukrainian PC layout with control low area.
#[expect(clippy::expect_used, reason = "Better to fail early here")]
pub(crate) static NEO_UA_PC_CONTROL: LazyLock<SingleByteMapping> =
    LazyLock::new(|| {
        try_load_mapping(NeoRegion::UaPc, LowArea::Control)
            .expect("Failed to load Neo UA-PC (Control) mapping")
    });

/// Returns the static `SingleByteMapping` for the specified region and low area.
#[must_use]
pub(crate) fn get_mapping(
    region: NeoRegion,
    low_area: LowArea,
) -> &'static SingleByteMapping {
    match (region, low_area) {
        (NeoRegion::Us, LowArea::Graphical) => &NEO_US,
        (NeoRegion::Us, LowArea::Control) => &NEO_US_CONTROL,
        (NeoRegion::UaMac, LowArea::Graphical) => &NEO_UA_MAC,
        (NeoRegion::UaMac, LowArea::Control) => &NEO_UA_MAC_CONTROL,
        (NeoRegion::UaPc, LowArea::Graphical) => &NEO_UA_PC,
        (NeoRegion::UaPc, LowArea::Control) => &NEO_UA_PC_CONTROL,
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
    use ctb_formats_utilities::encoding::CharEncoding;

    #[crate::ctb_test]
    fn test_neo_us_graphical_low_area() -> Result<()> {
        let enc = CharEncoding::neo_us();
        // In graphical mode, 0x00 is ■ (0x25a0), 0x01 is δ (0x03b4), 0x02 is Δ (0x0394)
        assert_eq!(crate::chr(enc, 0x00), "■");
        assert_eq!(crate::chr(enc, 0x01), "δ");
        assert_eq!(crate::chr(enc, 0x02), "Δ");

        assert_eq!(crate::asc(enc, "■"), Some(0x00));
        assert_eq!(crate::asc(enc, "δ"), Some(0x01));
        assert_eq!(crate::asc(enc, "Δ"), Some(0x02));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_us_control_low_area() -> Result<()> {
        let enc = CharEncoding::neo(NeoRegion::Us, LowArea::Control);
        // In control mode, 0x00 is ■ (0x25a0), 0x01 is '\u{1}', 0x02 is '\u{2}'
        assert_eq!(crate::chr(enc, 0x00), "■");
        assert_eq!(crate::chr(enc, 0x01), "\u{1}");
        assert_eq!(crate::chr(enc, 0x02), "\u{2}");

        assert_eq!(crate::asc(enc, "\u{1}"), Some(0x01));
        assert_eq!(crate::asc(enc, "\u{2}"), Some(0x02));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_us_roundtrip() -> Result<()> {
        let enc = CharEncoding::neo_us();
        let text = "Hello, World! 123 - α β Ω";
        let encoded = crate::encode(enc, text)?;
        let decoded = crate::decode(enc, &encoded)?;
        assert_eq!(decoded, text);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_neo_ua_mac_and_pc() -> Result<()> {
        let ua_mac = CharEncoding::neo(NeoRegion::UaMac, LowArea::Graphical);
        let ua_pc = CharEncoding::neo(NeoRegion::UaPc, LowArea::Graphical);

        // Byte 0x22 in UA Mac: 0x0404 ('Є')
        assert_eq!(crate::chr(ua_mac, 0x22), "Є");
        assert_eq!(crate::asc(ua_mac, "Є"), Some(0x22));

        // Byte 0x22 in UA PC: 0x0404 ('Є')
        assert_eq!(crate::chr(ua_pc, 0x22), "Є");
        assert_eq!(crate::asc(ua_pc, "Є"), Some(0x22));

        // Roundtrip encode/decode on UA Mac
        let ua_text = "Привіт, світ!";
        let encoded = crate::encode(ua_mac, ua_text)?;
        let decoded = crate::decode(ua_mac, &encoded)?;
        assert_eq!(decoded, ua_text);

        // Roundtrip encode/decode on UA PC
        let encoded_pc = crate::encode(ua_pc, ua_text)?;
        let decoded_pc = crate::decode(ua_pc, &encoded_pc)?;
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
