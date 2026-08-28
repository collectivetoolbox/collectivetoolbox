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

//! Altura Mac2Win character translation tables.
//!
//! Altura Mac2Win is/was a cross-platform compatibility runtime used by
//! applications such as Panorama for Windows. Internal data in `.pan` files is
//! stored in Mac OS Roman across platforms; when running on Windows, Altura
//! Mac2Win uses 128-byte lookup tables (covering bytes `0x80`..=`0xFF`) to
//! translate between Mac OS Roman and Windows ANSI (Windows-1252).
//! Some characters that are not shared between the encodings are changed to
//! different characters when files are transferred between operating systems.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::LazyLock;
use ctb_utilities::anyhow::anyhow;

fn parse_hex_table(file_path: &str) -> Result<[u8; 128]> {
    let bytes = crate::get_encoding_data(file_path)
        .ok_or_else(|| anyhow!("Missing Altura Mac2Win data file: {file_path}"))?;
    let text = String::from_utf8(bytes)
        .with_context(|| format!("Data file {file_path} is not valid UTF-8"))?;

    let mut table = [0u8; 128];
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    ensure!(
        tokens.len() == 128,
        "Expected 128 hex entries in {file_path}, found {}",
        tokens.len()
    );

    for (index, token) in tokens.iter().enumerate() {
        let val = u8::from_str_radix(token, 16)
            .with_context(|| format!("Invalid hex token '{token}' in {file_path}"))?;
        let slot = table
            .get_mut(index)
            .context("Table index out of bounds")?;
        *slot = val;
    }

    Ok(table)
}

static MAC_ROMAN_TO_ANSI: LazyLock<[u8; 128]> = LazyLock::new(|| {
    parse_hex_table("altura-mac2win/MacRoman to ANSI.csv")
        .expect("Failed to load Altura MacRoman to ANSI table")
});

static ANSI_TO_MAC_ROMAN: LazyLock<[u8; 128]> = LazyLock::new(|| {
    parse_hex_table("altura-mac2win/ANSI to MacRoman.csv")
        .expect("Failed to load Altura ANSI to MacRoman table")
});

/// Translates a single Mac OS Roman byte to Altura Windows ANSI.
#[inline]
#[must_use]
pub fn mac_roman_to_ansi_byte(b: u8) -> u8 {
    if b < 0x80 {
        b
    } else {
        let idx = usize::from(b.saturating_sub(0x80));
        MAC_ROMAN_TO_ANSI
            .get(idx)
            .copied()
            .unwrap_or(b)
    }
}

/// Translates a single Altura Windows ANSI byte to Mac OS Roman.
#[inline]
#[must_use]
pub fn ansi_to_mac_roman_byte(b: u8) -> u8 {
    if b < 0x80 {
        b
    } else {
        let idx = usize::from(b.saturating_sub(0x80));
        ANSI_TO_MAC_ROMAN
            .get(idx)
            .copied()
            .unwrap_or(b)
    }
}

/// Translates a slice of Mac OS Roman bytes to Altura Windows ANSI bytes.
#[must_use]
pub fn mac_roman_to_ansi(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|&b| mac_roman_to_ansi_byte(b))
        .collect()
}

/// Translates a slice of Altura Windows ANSI bytes to Mac OS Roman bytes.
#[must_use]
pub fn ansi_to_mac_roman(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .map(|&b| ansi_to_mac_roman_byte(b))
        .collect()
}

/// Converts Mac OS Roman bytes into a UTF-8 string from the Windows user
/// perspective (translating via the Altura ANSI table and decoding as
/// Windows-1252).
pub fn mac_roman_to_utf8_windows(bytes: &[u8]) -> Result<String> {
    let ansi_bytes = mac_roman_to_ansi(bytes);
    crate::standard::WINDOWS_1252_MAPPING.decode(&ansi_bytes)
}

/// Converts a UTF-8 string into Mac OS Roman bytes from the Windows user
/// perspective (encoding as Windows-1252 and translating via the Altura ANSI
/// table).
pub fn utf8_windows_to_mac_roman(text: &str) -> Result<Vec<u8>> {
    let ansi_bytes = crate::standard::WINDOWS_1252_MAPPING.encode(text)?;
    Ok(ansi_to_mac_roman(&ansi_bytes))
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
    fn test_altura_tables_load_and_have_128_entries() -> Result<()> {
        ensure!(MAC_ROMAN_TO_ANSI.len() == 128);
        ensure!(ANSI_TO_MAC_ROMAN.len() == 128);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_altura_mac_roman_to_ansi_known_mappings() -> Result<()> {
        // 0x80 (Ä in MacRoman) -> 0xC4 (Ä in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0x80) == 0xC4);
        // 0x85 (Ö in MacRoman) -> 0xD6 (Ö in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0x85) == 0xD6);
        // 0xA5 (• bullet in MacRoman) -> 0x95 (• bullet in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xA5) == 0x95);
        // 0xAA (™ trademark in MacRoman) -> 0x99 (™ trademark in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xAA) == 0x99);
        // 0xC9 (… ellipsis in MacRoman) -> 0x85 (… ellipsis in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xC9) == 0x85);
        // 0xD1 (— em dash in MacRoman) -> 0x97 (— em dash in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xD1) == 0x97);
        // 0xD5 (’ right single quote in MacRoman) -> 0x92 (’ in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xD5) == 0x92);
        // 0xB0 (∞ infinity sign in MacRoman) -> 0x81 (€ in Windows-1252)
        ensure!(mac_roman_to_ansi_byte(0xB0) == 0x81);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_altura_ansi_to_mac_roman_known_mappings() -> Result<()> {
        // 0xC4 (Ä in Windows-1252) -> 0x80 (Ä in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0xC4) == 0x80);
        // 0xD6 (Ö in Windows-1252) -> 0x85 (Ö in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0xD6) == 0x85);
        // 0x95 (• bullet in Windows-1252) -> 0xA5 (• bullet in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0x95) == 0xA5);
        // 0x99 (™ trademark in Windows-1252) -> 0xAA (™ trademark in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0x99) == 0xAA);
        // 0x85 (… ellipsis in Windows-1252) -> 0xC9 (… ellipsis in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0x85) == 0xC9);
        // 0x97 (— em dash in Windows-1252) -> 0xD1 (— em dash in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0x97) == 0xD1);
        // 0x92 (’ right single quote in Windows-1252) -> 0xD5 (’ in MacRoman)
        ensure!(ansi_to_mac_roman_byte(0x92) == 0xD5);
        // 0x81 (€ Euro sign in Windows-1252) -> 0xB0 (∞ in MacRoman)
        ensure!(mac_roman_to_ansi_byte(0xB0) == 0x81);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_altura_utf8_windows_conversions() -> Result<()> {
        let mac_bytes = [0x54, 0x65, 0x73, 0x74, 0xC9]; // "Test…" in MacRoman
        let utf8_win = mac_roman_to_utf8_windows(&mac_bytes)?;
        ensure!(utf8_win == "Test…");

        let roundtrip_mac = utf8_windows_to_mac_roman(&utf8_win)?;
        ensure!(roundtrip_mac == mac_bytes);
        Ok(())
    }
}
