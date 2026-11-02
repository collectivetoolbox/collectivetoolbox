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

//! Specification-oriented DCE 3.0a / 3.01a / 3.02a and CDCE converters.
//!
//! Unlike the legacy modules (`cdce`, `dce3_0a`, `dce3_01a`, `unicode`), which are
//! preserved to match `old/libdce-2.51/` as closely as possible for comparison,
//! this module implements the full DCE specification from `docs/` with fixes for
//! upstream table and state-machine bugs:
//!
//! - **DCE 3.0a (`3_0a`, `3_0a_raw`, `hex_3_0a`, `hex_3_0a_raw`)**:
//!   Losslessly encodes and decodes all mapped short Dcs in `0..=115`
//!   ([`MAX_DCE_3_0A_SHORT_DC`], i.e. `0`, `1`, and `13..=115`; short Dcs `2..=12`
//!   had no byte mappings in 3.0a and are logged as unmappable in `FormatLog` and omitted).
//! - **DCE 3.01a (`3_01a`, `3_01a_raw`, `hex_3_01a`, `hex_3_01a_raw`)**:
//!   Losslessly encodes and decodes all allocated short Dcs in `0..=199`
//!   ([`MAX_DCE_3_01A_SHORT_DC`]), unlocking Base64 encapsulation
//!   (`Dc 191` .. `Dc 127..=190` / `Dc 195` .. `Dc 192`) for arbitrary UTF-8 and data.
//! - **DCE 3.02a (`3_02a`, `3_02a_raw`, `hex_3_02a`, `hex_3_02a_raw`)**:
//!   Stubbed (returns an error when invoked). Allocates short Dcs `0..=202`
//!   ([`MAX_DCE_3_02A_SHORT_DC`]), adding keyed embed framing (`Dc 200..=202`) on
//!   `Core:C9..=CB`, but the 18 October 2013 specification left its version selector
//!   in `DCE_Version_Selectors` and corresponding short Dc unassigned (marked as TBD;
//!   `Dc 205` is `DCE Version Selector 3.03a`, so position `0x03` is not used here to
//!   avoid colliding with 3.03a).
//! - **Corrected Base64 digit 34 (`Dc 161`, `'i'`) vs digit 8 (`Dc 135`, `'I'`)**:
//!   The upstream `.ods` spreadsheet auto-capitalized lowercase `'i'` to `'I'`,
//!   corrupting `Base64_to_Dc` and `Dc_to_Base64`.
//! - **Corrected `Dc 113` (`End DCE content`) mapping (`Core:03`)**:
//!   The upstream `DCE Character List by ID` sheet had a typo `Core:01` instead of `Core:03`.
//! - **Unicode mappings** follow the specification's character list, including
//!   control whitespace, decimal-point projection, and variation selectors.
//!   Unmapped Unicode is preserved using UTF-8 encapsulation on capable targets.
//!
//! The 3.02a version selector is left marked as TBD because its spreadsheet leaves
//! the selector unassigned and `Dc 205` is allocated to DCE 3.03a. Legacy one-step and
//! HTML translators still delegate to the PHP-compatible implementation; see
//! `porting-notes.md` for audit limits.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow, bail};
use ctb_formats_hexdump::hex2bin;
use ctb_formats_utilities::FormatLog;

use crate::legacy;
use crate::tools::explode_escaped;

/// Maximum short Dc allocated in DCE 3.0a (`0..=115`).
pub const MAX_DCE_3_0A_SHORT_DC: u32 = 115;

/// Maximum short Dc allocated in DCE 3.01a (`0..=199`).
pub const MAX_DCE_3_01A_SHORT_DC: u32 = 199;

/// Maximum short Dc allocated in DCE 3.02a (`0..=202`).
pub const MAX_DCE_3_02A_SHORT_DC: u32 = 202;

/// DCE specification version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DceVersion {
    /// DCE 3.0a (version byte `0x01`, short Dcs `0..=115`).
    V3_0a,
    /// DCE 3.01a (version byte `0x02`, short Dcs `0..=199`, Base64 encapsulation).
    V3_01a,
    /// DCE 3.02a (stubbed; version byte TBD, short Dcs `0..=202`, keyed embeds + Base64 encapsulation).
    V3_02a,
}

impl DceVersion {
    /// Returns the DCE header version selector byte (`Some(0x01)` for 3.0a, `Some(0x02)` for 3.01a,
    /// or `None` for 3.02a whose version selector byte in `DCE_Version_Selectors` is TBD).
    #[must_use]
    pub const fn version_byte(self) -> Option<u8> {
        match self {
            Self::V3_0a => Some(0x01),
            Self::V3_01a => Some(0x02),
            // TBD: the 18 October 2013 DCE 3.02a specification left its version selector
            // position in DCE_Version_Selectors unassigned (position 0x03 may belong to 3.03a).
            Self::V3_02a => None,
        }
    }

    /// Returns the `dceutils` format code for this version (`"3_0a"`, `"3_01a"`, `"3_02a"`).
    #[must_use]
    pub const fn as_format_str(self) -> &'static str {
        match self {
            Self::V3_0a => "3_0a",
            Self::V3_01a => "3_01a",
            Self::V3_02a => "3_02a",
        }
    }

    /// Returns the maximum contiguous short Dc ID allocated in this version of DCE.
    #[must_use]
    pub const fn max_short_dc(self) -> u32 {
        match self {
            Self::V3_0a => MAX_DCE_3_0A_SHORT_DC,
            Self::V3_01a => MAX_DCE_3_01A_SHORT_DC,
            Self::V3_02a => MAX_DCE_3_02A_SHORT_DC,
        }
    }

    /// Returns `true` if `short_dc` can be represented in this version of DCE.
    #[must_use]
    pub const fn supports_short_dc(self, short_dc: u32) -> bool {
        match self {
            Self::V3_0a => {
                short_dc <= MAX_DCE_3_0A_SHORT_DC && !matches!(short_dc, 2..=12)
            }
            Self::V3_01a => short_dc <= MAX_DCE_3_01A_SHORT_DC,
            Self::V3_02a => {
                // TBD: version selector short Dc for 3.02a was left unassigned in the spec
                // (Dc 205 is DCE Version Selector 3.03a).
                short_dc <= MAX_DCE_3_02A_SHORT_DC
            }
        }
    }

    /// Returns `true` if this DCE version supports Base64 encapsulation (`Dc 127..=192`, `Dc 195`).
    #[must_use]
    pub const fn supports_base64_encapsulation(self) -> bool {
        matches!(self, Self::V3_01a | Self::V3_02a)
    }
}

/// Active DCE code sheet state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DceSheet {
    Core,
    DceVersionSelectors,
    VariantSelectors,
    SemanticRecords,
    Mathematics,
    PunctuationAndWhitespace,
}

impl DceSheet {
    const fn name(self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::DceVersionSelectors => "DCE_Version_Selectors",
            Self::VariantSelectors => "Variant_Selectors",
            Self::SemanticRecords => "Semantic_Records",
            Self::Mathematics => "Mathematics",
            Self::PunctuationAndWhitespace => "Punctuation_and_Whitespace",
        }
    }

    const fn switch_byte_from_core(self) -> Option<u8> {
        match self {
            Self::Core => None,
            Self::DceVersionSelectors => Some(0x01),
            Self::VariantSelectors => Some(0xC5),
            Self::SemanticRecords => Some(0xC6),
            Self::Mathematics => Some(0xC7),
            Self::PunctuationAndWhitespace => Some(0xC8),
        }
    }
}

/// Returns `true` if `short_dc` is a DCE stateful-encoding control character or
/// unbounded-content framing marker that has no glyph representation in text/CDCE.
#[must_use]
pub const fn is_dce_encoding_control_dc(short_dc: u32) -> bool {
    matches!(
        short_dc,
        13..=16 | 112..=115 | 194 | 196..=199 | 205
    )
}

/// Maps a standard Base64 ASCII byte (`A-Z`, `a-z`, `0-9`, `+`, `/`, `=`) to its
/// DCE 3.01a+ short Dc (`127..=190`, `195`).
///
/// Correctly maps `'I'` (`0x49`) to `135` and `'i'` (`0x69`) to `161`.
#[must_use]
pub const fn base64_char_to_short_dc(ch: u8) -> Option<u32> {
    match ch {
        b'A'..=b'Z' => {
            let offset = ch.saturating_sub(b'A');
            Some(127_u32.saturating_add(u32::from_le_bytes([offset, 0, 0, 0])))
        }
        b'a'..=b'z' => {
            let offset = ch.saturating_sub(b'a');
            Some(153_u32.saturating_add(u32::from_le_bytes([offset, 0, 0, 0])))
        }
        b'0'..=b'9' => {
            let offset = ch.saturating_sub(b'0');
            Some(179_u32.saturating_add(u32::from_le_bytes([offset, 0, 0, 0])))
        }
        b'+' => Some(189),
        b'/' => Some(190),
        b'=' => Some(195),
        _ => None,
    }
}

/// Maps a DCE 3.01a+ Base64 short Dc (`127..=190`, `195`) to its standard Base64 character.
///
/// Correctly maps `135` to `'I'` and `161` to `'i'`.
#[must_use]
pub const fn short_dc_to_base64_char(short_dc: u32) -> Option<char> {
    match short_dc {
        127..=152 => {
            let cp = short_dc.saturating_sub(127).saturating_add(0x41);
            char::from_u32(cp)
        }
        153..=178 => {
            let cp = short_dc.saturating_sub(153).saturating_add(0x61);
            char::from_u32(cp)
        }
        179..=188 => {
            let cp = short_dc.saturating_sub(179).saturating_add(0x30);
            char::from_u32(cp)
        }
        189 => Some('+'),
        190 => Some('/'),
        195 => Some('='),
        _ => None,
    }
}

/// Returns `true` if `short_dc` is a Base64 digit (`127..=190`) or padding (`195`).
#[must_use]
pub const fn is_base64_payload_short_dc(short_dc: u32) -> bool {
    matches!(short_dc, 127..=190 | 195)
}

/// Encodes arbitrary bytes into DCE 3.01a+ Base64 short Dcs (`127..=190`, `195`).
#[must_use]
pub fn encode_bytes_to_dc64(bytes: &[u8]) -> Vec<u32> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let b64 = ctb_formats_base64::base64_encode(bytes);
    let mut out = Vec::with_capacity(b64.len());
    for &b in b64.as_bytes() {
        if let Some(dc) = base64_char_to_short_dc(b) {
            out.push(dc);
        }
    }
    out
}

/// Encodes UTF-8 bytes into a DCE 3.01a+ Base64-encapsulated short Dc sequence
/// (`Dc 191`, Base64 Dcs `127..=190` / `195`, `Dc 192`).
#[must_use]
pub fn encode_utf8_to_dc64_encapsulated(bytes: &[u8]) -> Vec<u32> {
    let payload = encode_bytes_to_dc64(bytes);
    let mut out = Vec::with_capacity(payload.len().saturating_add(2));
    out.push(191);
    out.extend(payload);
    out.push(192);
    out
}

/// Decodes a slice of DCE 3.01a+ Base64 short Dcs (`127..=190`, `195`) into raw bytes.
///
/// # Errors
/// Returns an error if `dcs` contains non-Base64 short Dcs or invalid Base64 data.
pub fn decode_dc64_to_bytes(
    dcs: &[u32],
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let mut b64_str = String::with_capacity(dcs.len());
    for &dc in dcs {
        if let Some(ch) = short_dc_to_base64_char(dc) {
            b64_str.push(ch);
        } else {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        }
    }
    ctb_formats_base64::base64_decode(&b64_str).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })
}

/// Maps a Unicode codepoint to its direct DCE short Dc (`0`, `1`, `18..=111`,
/// `116..=126`, `193`), if a lossless direct mapping exists in the DCE specification.
#[must_use]
pub const fn unicode_codepoint_to_short_dc(cp: u32) -> Option<u32> {
    match cp {
        0x00 => Some(0),
        0x08 => Some(116),
        0x09 => Some(117),
        0x0A => Some(120),
        0x0B => Some(118),
        0x0C => Some(193),
        0x0D => Some(121),
        0x20..=0x2C => Some(cp.saturating_sub(0x20).saturating_add(18)),
        0x2E..=0x3F => Some(cp.saturating_sub(0x20).saturating_add(18)),
        0x40 => Some(1),
        0x41..=0x7E => Some(cp.saturating_sub(0x41).saturating_add(50)),
        0x2010 => Some(31),
        0xFE00..=0xFE04 => Some(cp.saturating_sub(0xFE00).saturating_add(122)),
        _ => None,
    }
}

/// Maps a Unicode codepoint to its DCE short Dc using the lossy mapping where
/// `U+002D` (`HYPHEN-MINUS`) maps to `Dc 31` (`U+2010 HYPHEN`).
#[must_use]
pub const fn unicode_codepoint_to_short_dc_lossy(cp: u32) -> Option<u32> {
    if cp == 0x2D {
        Some(31)
    } else {
        unicode_codepoint_to_short_dc(cp)
    }
}

/// Maps a DCE short Dc (`0`, `1`, `18..=111`, `116..=126`, `193`) to its Unicode codepoint.
#[must_use]
pub const fn short_dc_to_unicode_codepoint(short_dc: u32) -> Option<u32> {
    match short_dc {
        0 => Some(0x00),
        1 => Some(0x40),
        12 => Some(0x2E),
        18..=30 => Some(short_dc.saturating_sub(18).saturating_add(0x20)),
        31 => Some(0x2010),
        32..=49 => Some(short_dc.saturating_sub(18).saturating_add(0x20)),
        50..=111 => Some(short_dc.saturating_sub(50).saturating_add(0x41)),
        116 => Some(0x08),
        117 => Some(0x09),
        118 => Some(0x0B),
        119 => Some(0x0A),
        120 => Some(0x0A),
        121 => Some(0x0D),
        122..=126 => Some(short_dc.saturating_sub(122).saturating_add(0xFE00)),
        193 => Some(0x0C),
        _ => None,
    }
}

/// Maps a short Dc ID to its target `(DceSheet, byte)` for the given `DceVersion`.
const fn encode_short_dc_in_version(
    short_dc: u32,
    ver: DceVersion,
) -> Option<(DceSheet, u8)> {
    if !ver.supports_short_dc(short_dc) {
        return None;
    }
    match short_dc {
        0 => Some((DceSheet::Core, 0x00)),
        1 => Some((DceSheet::Core, 0x40)),
        2..=11 => {
            let [low, _, _, _] = short_dc.saturating_sub(1).to_le_bytes();
            Some((DceSheet::SemanticRecords, low))
        }
        12 => Some((DceSheet::Mathematics, 0x01)),
        13 => Some((DceSheet::Core, 0xFD)),
        14 => Some((DceSheet::Core, 0xFE)),
        15 => Some((DceSheet::Core, 0x02)),
        16 => Some((DceSheet::Core, 0x01)),
        17 => Some((DceSheet::Core, 0xFF)),
        18..=49 => {
            let [low, _, _, _] = short_dc
                .saturating_sub(18)
                .saturating_add(0x20)
                .to_le_bytes();
            Some((DceSheet::Core, low))
        }
        50..=111 => {
            let [low, _, _, _] = short_dc
                .saturating_sub(50)
                .saturating_add(0x41)
                .to_le_bytes();
            Some((DceSheet::Core, low))
        }
        112 => Some((DceSheet::DceVersionSelectors, 0x01)),
        113 => Some((DceSheet::Core, 0x03)),
        114 => Some((DceSheet::Core, 0x80)),
        115 => Some((DceSheet::Core, 0x81)),
        116 => Some((DceSheet::Core, 0x08)),
        117 => Some((DceSheet::Core, 0x09)),
        118 => Some((DceSheet::Core, 0x0B)),
        119 => Some((DceSheet::PunctuationAndWhitespace, 0x01)),
        120 => Some((DceSheet::Core, 0x0A)),
        121 => Some((DceSheet::Core, 0x0D)),
        122..=126 => {
            let [low, _, _, _] = short_dc.saturating_sub(121).to_le_bytes();
            Some((DceSheet::VariantSelectors, low))
        }
        127..=190 => {
            let [low, _, _, _] = short_dc
                .saturating_sub(127)
                .saturating_add(0x82)
                .to_le_bytes();
            Some((DceSheet::Core, low))
        }
        191 => Some((DceSheet::Core, 0xC3)),
        192 => Some((DceSheet::Core, 0xC4)),
        193 => Some((DceSheet::Core, 0x0C)),
        194 => Some((DceSheet::DceVersionSelectors, 0x02)),
        195 => Some((DceSheet::Core, 0xC2)),
        196 => Some((DceSheet::Core, 0xC5)),
        197 => Some((DceSheet::Core, 0xC6)),
        198 => Some((DceSheet::Core, 0xC7)),
        199 => Some((DceSheet::Core, 0xC8)),
        200 => Some((DceSheet::Core, 0xC9)),
        201 => Some((DceSheet::Core, 0xCB)),
        202 => Some((DceSheet::Core, 0xCA)),
        // TBD: DCE 3.02a version selector short Dc and DCE_Version_Selectors byte
        // (Dc 205 is DCE Version Selector 3.03a).
        _ => None,
    }
}

/// Detects the DCE format version (`"3_0a"` or `"3_01a"`) of `data`.
#[must_use]
pub fn get_dce_version(data: &[u8]) -> String {
    let mut log = FormatLog::default();
    let res = get_dce_version_with_log(data, &mut log);
    log.auto_log();
    res
}

/// Detects the DCE format version (`"3_0a"` or `"3_01a"`) of `data`,
/// recording any errors in `log`.
pub fn get_dce_version_with_log(data: &[u8], log: &mut FormatLog) -> String {
    if data.len() < 7 || data.get(0..6) != Some(b"DCEe\x02\x01") {
        log.error("This document does not appear to be stored using DCE.");
        return "This document does not appear to be stored using DCE."
            .to_string();
    }
    match data.get(6) {
        Some(&1) => "3_0a".to_string(),
        Some(&2) => "3_01a".to_string(),
        // TBD: DCE 3.02a version selector byte is unassigned (position 0x03 may belong to 3.03a).
        _ => {
            log.error(
                "This document is not stored using a supported version of DCE.",
            );
            "This document is not stored using a supported version of DCE."
                .to_string()
        }
    }
}

/// Selects the minimum implemented [`DceVersion`] (`V3_0a` or `V3_01a`) required to represent `dcs`.
///
/// Note: [`DceVersion::V3_02a`] (`Dc 200..=202`) is stubbed until its version selector
/// in `DCE_Version_Selectors` is determined (TBD).
#[must_use]
pub fn select_dce_version_for_short_dcs(dcs: &[u32]) -> DceVersion {
    let mut ver = DceVersion::V3_0a;
    for &dc in dcs {
        if matches!(dc, 2..=12 | 116..=202) {
            ver = DceVersion::V3_01a;
        }
    }
    ver
}

/// Decodes a DCE binary or hex-encoded stream (`dce`, `3_0a`, `3_0a_raw`,
/// `3_01a`, `3_01a_raw`, `3_02a`, `3_02a_raw`, and their `hex_*` variants)
/// into a sequence of short Dc IDs while preserving all stateful DCE control
/// characters (`13..=16`, `112..=115`, `194`, `196..=199`, `205`) as-is.
///
/// # Errors
/// Returns an error if the input format is unknown or the header does not match
/// the specified DCE version.
pub fn decode_dce_preserving_controls(
    data: &[u8],
    input_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    let (raw_bytes, base_fmt) = match input_format {
        "hex_dce" | "dce_hex" => (decode_hex_payload(data, log)?, "dce"),
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => {
            (decode_hex_payload(data, log)?, "3_0a")
        }
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            (decode_hex_payload(data, log)?, "3_0a_raw")
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => {
            (decode_hex_payload(data, log)?, "3_01a")
        }
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            (decode_hex_payload(data, log)?, "3_01a_raw")
        }
        "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex" => {
            (decode_hex_payload(data, log)?, "3_02a")
        }
        "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            (decode_hex_payload(data, log)?, "3_02a_raw")
        }
        "3_0a" | "dce_3_0a" => (data.to_vec(), "3_0a"),
        "3_0a_raw" | "dce_3_0a_raw" => (data.to_vec(), "3_0a_raw"),
        "3_01a" | "dce_3_01a" => (data.to_vec(), "3_01a"),
        "3_01a_raw" | "dce_3_01a_raw" => (data.to_vec(), "3_01a_raw"),
        "3_02a" | "dce_3_02a" => (data.to_vec(), "3_02a"),
        "3_02a_raw" | "dce_3_02a_raw" => (data.to_vec(), "3_02a_raw"),
        "dce" => (data.to_vec(), "dce"),
        _ => {
            log.error("Unknown DCE input format.");
            bail!("Unknown DCE input format: {input_format}");
        }
    };

    let (ver, start_offset) = match base_fmt {
        "3_0a_raw" => {
            if raw_bytes.first() != Some(&0x80) {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            (DceVersion::V3_0a, 0_usize)
        }
        "3_01a_raw" => {
            if raw_bytes.first() != Some(&0x80) {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            (DceVersion::V3_01a, 0_usize)
        }
        "3_02a" | "3_02a_raw" => {
            // Stubbed for later: the 18 October 2013 DCE 3.02a specification left its
            // version selector in DCE_Version_Selectors unassigned (TBD; position 0x03
            // may belong to DCE 3.03a / Dc 205).
            let msg = "DCE 3.02a is not yet implemented (version selector in DCE_Version_Selectors is TBD).";
            log.error(msg);
            bail!("{msg}");
        }
        "3_0a" | "3_01a" | "dce" => {
            if raw_bytes.len() < 7
                || raw_bytes.get(0..6) != Some(b"DCEe\x02\x01")
            {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            let ver_byte = *raw_bytes.get(6).context("missing version byte")?;
            match (base_fmt, ver_byte) {
                ("3_0a" | "dce", 1) => (DceVersion::V3_0a, 4_usize),
                ("3_01a" | "dce", 2) => (DceVersion::V3_01a, 4_usize),
                // TBD: DCE 3.02a version selector byte in DCE_Version_Selectors is unassigned.
                ("3_0a", _) => {
                    log.error(
                        "This document is not stored using the specified format.",
                    );
                    bail!(
                        "This document is not stored using the specified format."
                    );
                }
                ("3_01a", _) => {
                    log.error(
                        "This document is not stored using the specified version of DCE.",
                    );
                    bail!(
                        "This document is not stored using the specified version of DCE."
                    );
                }
                _ => {
                    log.error(
                        "This document is not stored using a supported version of DCE.",
                    );
                    bail!(
                        "This document is not stored using a supported version of DCE."
                    );
                }
            }
        }
        _ => unreachable!("base_fmt is constrained above"),
    };

    let mut out = Vec::new();
    let mut state = DceSheet::Core;
    let valid_envelope = if start_offset == 0 {
        raw_bytes.last() == Some(&0x81)
    } else {
        raw_bytes.get(7) == Some(&0xFD) && raw_bytes.ends_with(&[0xFD, 0x03])
    };
    if !valid_envelope {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
    let slice = raw_bytes
        .get(start_offset..)
        .context("raw_bytes shorter than start offset")?;

    for (idx, &byte) in slice.iter().enumerate() {
        let idx_u64 = u64::try_from(idx.saturating_add(start_offset))?;
        match state {
            DceSheet::Core => match byte {
                0x00 => out.push(0),
                0x01 => {
                    out.push(16);
                    state = DceSheet::DceVersionSelectors;
                }
                0x02 => out.push(15),
                0x03 => out.push(113),
                0x08 if ver >= DceVersion::V3_01a => out.push(116),
                0x09 if ver >= DceVersion::V3_01a => out.push(117),
                0x0A if ver >= DceVersion::V3_01a => out.push(120),
                0x0B if ver >= DceVersion::V3_01a => out.push(118),
                0x0C if ver >= DceVersion::V3_01a => out.push(193),
                0x0D if ver >= DceVersion::V3_01a => out.push(121),
                0x20..=0x3F => {
                    out.push(
                        u32::from(byte).saturating_sub(0x20).saturating_add(18),
                    );
                }
                0x40 => out.push(1),
                0x41..=0x7E => {
                    out.push(
                        u32::from(byte).saturating_sub(0x41).saturating_add(50),
                    );
                }
                0x80 => out.push(114),
                0x81 => out.push(115),
                0x82..=0xC1 if ver >= DceVersion::V3_01a => {
                    out.push(
                        u32::from(byte)
                            .saturating_sub(0x82)
                            .saturating_add(127),
                    );
                }
                0xC2 if ver >= DceVersion::V3_01a => out.push(195),
                0xC3 if ver >= DceVersion::V3_01a => out.push(191),
                0xC4 if ver >= DceVersion::V3_01a => out.push(192),
                0xC5 if ver >= DceVersion::V3_01a => {
                    out.push(196);
                    state = DceSheet::VariantSelectors;
                }
                0xC6 if ver >= DceVersion::V3_01a => {
                    out.push(197);
                    state = DceSheet::SemanticRecords;
                }
                0xC7 if ver >= DceVersion::V3_01a => {
                    out.push(198);
                    state = DceSheet::Mathematics;
                }
                0xC8 if ver >= DceVersion::V3_01a => {
                    out.push(199);
                    state = DceSheet::PunctuationAndWhitespace;
                }
                0xC9 if ver >= DceVersion::V3_02a => out.push(200),
                0xCA if ver >= DceVersion::V3_02a => out.push(202),
                0xCB if ver >= DceVersion::V3_02a => out.push(201),
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE Core byte 0x{byte:02X} dropped during import"
                        ),
                    );
                }
            },
            DceSheet::DceVersionSelectors => match byte {
                0x00 => out.push(0),
                0x01 => out.push(112),
                0x02 if ver >= DceVersion::V3_01a => out.push(194),
                // TBD: DCE 3.02a version selector byte and short Dc (position 0x03 is unassigned
                // in the 3.02a specification and may belong to DCE 3.03a / Dc 205).
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE_Version_Selectors byte 0x{byte:02X} dropped during import"
                        ),
                    );
                }
            },
            DceSheet::VariantSelectors => match byte {
                0x00 => out.push(0),
                0x01..=0x05 => {
                    out.push(u32::from(byte).saturating_add(121));
                }
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE Variant_Selectors byte 0x{byte:02X} dropped during import"
                        ),
                    );
                }
            },
            DceSheet::SemanticRecords => match byte {
                0x00 => out.push(0),
                0x01..=0x0A => {
                    out.push(u32::from(byte).saturating_add(1));
                }
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE Semantic_Records byte 0x{byte:02X} dropped during import"
                        ),
                    );
                }
            },
            DceSheet::Mathematics => match byte {
                0x00 => out.push(0),
                0x01 => out.push(12),
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE Mathematics byte 0x{byte:02X} dropped during import"
                        ),
                    );
                }
            },
            DceSheet::PunctuationAndWhitespace => match byte {
                0x00 => out.push(0),
                0x01 => out.push(119),
                0xFD => {
                    out.push(13);
                    state = DceSheet::Core;
                }
                0xFE => {
                    out.push(14);
                    state = DceSheet::Core;
                }
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE {} byte 0x{byte:02X} dropped during import",
                            state.name()
                        ),
                    );
                }
            },
        }
    }

    Ok(out)
}

/// Encodes a sequence of short Dc IDs into a DCE binary or hex-encoded format
/// (`dce`, `3_0a`, `3_0a_raw`, `3_01a`, `3_01a_raw`, `3_02a`, `3_02a_raw`, and
/// their `hex_*` variants), respecting any explicit DCE sheet/control Dcs
/// already present in `dcs` and automatically inserting sheet transitions or
/// framing when absent.
///
/// # Errors
/// Returns an error if `output_format` is not a recognized DCE format.
pub fn encode_dce_preserving_controls(
    dcs: &[u32],
    output_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let (base_fmt, is_hex) = match output_format {
        "hex_dce" | "dce_hex" => ("dce", true),
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => ("3_0a", true),
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            ("3_0a_raw", true)
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => ("3_01a", true),
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            ("3_01a_raw", true)
        }
        "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex" => ("3_02a", true),
        "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            ("3_02a_raw", true)
        }
        "dce" | "3_0a" | "3_0a_raw" | "3_01a" | "3_01a_raw" | "3_02a"
        | "3_02a_raw" => (output_format, false),
        "dce_3_0a" => ("3_0a", false),
        "dce_3_0a_raw" => ("3_0a_raw", false),
        "dce_3_01a" => ("3_01a", false),
        "dce_3_01a_raw" => ("3_01a_raw", false),
        "dce_3_02a" => ("3_02a", false),
        "dce_3_02a_raw" => ("3_02a_raw", false),
        _ => {
            log.error("Unknown DCE output format.");
            bail!("Unknown DCE output format: {output_format}");
        }
    };

    let is_raw = matches!(base_fmt, "3_0a_raw" | "3_01a_raw" | "3_02a_raw");

    // Strip outer DCE file envelope (`[15, 16, 112|194, 13|14, ..., 13|14, 113]`;
    // 3.02a version selector Dc is TBD) only when both header and trailer are present.
    let (inner, inner_offset) = if dcs.len() >= 6
        && dcs.first() == Some(&15)
        && dcs.get(1) == Some(&16)
        && matches!(dcs.get(2), Some(&112 | &194))
        && matches!(dcs.get(3), Some(&13 | &14))
        && matches!(dcs.get(dcs.len().saturating_sub(2)), Some(&13 | &14))
        && dcs.get(dcs.len().saturating_sub(1)) == Some(&113)
    {
        (
            dcs.get(4..dcs.len().saturating_sub(2))
                .context("dcs envelope slice out of bounds")?,
            4_usize,
        )
    } else {
        (dcs, 0_usize)
    };

    // Strip outer unbounded-content markers (`[114, ..., 115]`) only when both are present.
    let (body, body_offset) = if inner.len() >= 2
        && inner.first() == Some(&114)
        && inner.last() == Some(&115)
    {
        (
            inner
                .get(1..inner.len().saturating_sub(1))
                .context("inner unbounded slice out of bounds")?,
            inner_offset.saturating_add(1),
        )
    } else {
        (inner, inner_offset)
    };

    let ver = match base_fmt {
        "3_0a" | "3_0a_raw" => DceVersion::V3_0a,
        "3_01a" | "3_01a_raw" => DceVersion::V3_01a,
        "3_02a" | "3_02a_raw" => {
            // Stubbed for later: the 18 October 2013 DCE 3.02a specification left its
            // version selector in DCE_Version_Selectors unassigned (TBD; position 0x03
            // may belong to DCE 3.03a / Dc 205).
            let msg = "DCE 3.02a is not yet implemented (version selector in DCE_Version_Selectors is TBD).";
            log.error(msg);
            bail!("{msg}");
        }
        "dce" => select_dce_version_for_short_dcs(body),
        _ => unreachable!("base_fmt is constrained above"),
    };

    let mut raw_out = Vec::with_capacity(body.len().saturating_add(16));
    if !is_raw {
        let Some(ver_byte) = ver.version_byte() else {
            let msg = "DCE 3.02a is not yet implemented (version selector in DCE_Version_Selectors is TBD).";
            log.error(msg);
            bail!("{msg}");
        };
        raw_out.extend_from_slice(b"DCEe\x02\x01");
        raw_out.push(ver_byte);
        raw_out.push(0xFD);
    }

    // Begin unbounded content (Dc 114 = 0x80)
    raw_out.push(0x80);

    let mut state = DceSheet::Core;
    encode_short_dcs_into_stream(
        body,
        body_offset,
        ver,
        output_format,
        &mut state,
        &mut raw_out,
        log,
    )?;

    if state != DceSheet::Core {
        raw_out.push(0xFD);
    }

    // End unbounded content (Dc 115 = 0x81)
    raw_out.push(0x81);

    if !is_raw {
        raw_out.extend_from_slice(&[0xFD, 0x03]);
    }

    if is_hex {
        Ok(bin2hex(raw_out).to_uppercase().into_bytes())
    } else {
        Ok(raw_out)
    }
}

fn encode_short_dcs_into_stream(
    body: &[u32],
    body_offset: usize,
    ver: DceVersion,
    output_format: &str,
    state: &mut DceSheet,
    raw_out: &mut Vec<u8>,
    log: &mut FormatLog,
) -> Result<()> {
    for (idx, &dc) in body.iter().enumerate() {
        let idx_u64 = u64::try_from(idx.saturating_add(body_offset))?;
        match dc {
            13 => {
                raw_out.push(0xFD);
                *state = DceSheet::Core;
                continue;
            }
            14 => {
                raw_out.push(0xFE);
                *state = DceSheet::Core;
                continue;
            }
            16 => {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                }
                raw_out.push(0x01);
                *state = DceSheet::DceVersionSelectors;
                continue;
            }
            196 if ver >= DceVersion::V3_01a => {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                }
                raw_out.push(0xC5);
                *state = DceSheet::VariantSelectors;
                continue;
            }
            197 if ver >= DceVersion::V3_01a => {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                }
                raw_out.push(0xC6);
                *state = DceSheet::SemanticRecords;
                continue;
            }
            198 if ver >= DceVersion::V3_01a => {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                }
                raw_out.push(0xC7);
                *state = DceSheet::Mathematics;
                continue;
            }
            199 if ver >= DceVersion::V3_01a => {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                }
                raw_out.push(0xC8);
                *state = DceSheet::PunctuationAndWhitespace;
                continue;
            }
            _ => {}
        }

        if let Some((sheet_needed, byte)) = encode_short_dc_in_version(dc, ver)
        {
            if sheet_needed != *state {
                if *state != DceSheet::Core {
                    raw_out.push(0xFD);
                    *state = DceSheet::Core;
                }
                if let Some(sw_byte) = sheet_needed.switch_byte_from_core() {
                    raw_out.push(sw_byte);
                    *state = sheet_needed;
                }
            }
            raw_out.push(byte);
        } else {
            log.export_warning_unmappable(idx_u64, dc, output_format);
        }
    }
    Ok(())
}

/// Converts between any `dceutils` / DCE / CDCE formats using the spec-compliant
/// implementations (supporting DCE 3.0a `0..=115`, DCE 3.01a `0..=199` with Base64
/// encapsulation, DCE 3.02a `0..=202`, and general/legacy CDCE).
///
/// # Errors
/// Returns an error if the input/output format is unknown or input validation fails.
pub fn dce_convert(
    data: &[u8],
    input_format: &str,
    output_format: &str,
) -> Result<Vec<u8>> {
    let (res, log) = dce_convert_with_log(data, input_format, output_format)?;
    log.auto_log();
    Ok(res)
}

/// Converts between any `dceutils` / DCE / CDCE formats using the spec-compliant
/// implementations, returning the converted bytes and [`FormatLog`].
///
/// # Errors
/// Returns an error if the input/output format is unknown or input validation fails.
pub fn dce_convert_with_log(
    data: &[u8],
    input_format: &str,
    output_format: &str,
) -> Result<(Vec<u8>, FormatLog)> {
    dce_convert_with_options(data, input_format, output_format, false)
}

/// Converts formats with optional replacement-character recovery for damaged
/// text, returning converted bytes and diagnostics.
///
/// # Errors
/// Returns an error for unsupported formats or unrecoverable input. In strict
/// mode, malformed encapsulated text also returns an error.
pub fn dce_convert_with_options(
    data: &[u8],
    input_format: &str,
    output_format: &str,
    permissive: bool,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut log = FormatLog::default();
    let res = dce_convert_internal(
        data, input_format, output_format, permissive, &mut log,
    )?;
    Ok((res, log))
}

pub(crate) fn dce_convert_internal(
    data: &[u8],
    input_format: &str,
    output_format: &str,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    // One-step legacy translators
    match (input_format, output_format) {
        ("3_0a_old" | "dce_3_0a_old", "none") => {
            return legacy::onestep_3_0a_old_to_none(data);
        }
        ("dce2txt", "none") => return legacy::onestep_dce2txt_to_none(data),
        ("dce2hex", "none") => return legacy::onestep_dce2hex_to_none(data),
        ("hex2dce", "none") => return legacy::onestep_hex2dce_to_none(data),
        (
            "legacy_cdce" | "cdce_legacy",
            "html_snippet"
            | "html_snippet_l"
            | "html_s_legacy"
            | "html_legacy_cdce_snippet",
        ) => return legacy::onestep_legacy_cdce_to_html_snippet_l(data),
        (
            "legacy_cdce" | "cdce_legacy",
            "html" | "html_l" | "html_legacy" | "html_legacy_cdce",
        ) => return legacy::onestep_legacy_cdce_to_html_l(data),
        _ => {}
    }

    let target_supports_b64 = !matches!(
        output_format,
        "3_0a"
            | "dce_3_0a"
            | "3_0a_raw"
            | "dce_3_0a_raw"
            | "hex_3_0a"
            | "3_0a_hex"
            | "dce_3_0a_hex"
            | "hex_3_0a_raw"
            | "3_0a_raw_hex"
            | "dce_3_0a_raw_hex"
            | "legacy_cdce"
            | "cdce_legacy"
            | "html_l"
            | "html_legacy"
            | "html_legacy_cdce"
            | "html_snippet_l"
            | "html_s_legacy"
            | "html_legacy_cdce_snippet"
    );

    // Step 1: Decode input into short Dc list string (comma-separated)
    let dc_str = match input_format {
        "dc" | "dc_id_list" => std::str::from_utf8(data)
            .context("invalid UTF-8 in Dc list")?
            .to_string(),
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw" | "3_01a_raw_hex"
        | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a" | "3_02a_raw"
        | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex"
        | "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            let dcs = decode_dce_content_short_dcs(data, input_format, log)?;
            join_short_dcs(&dcs)
        }
        "cdce" => convert_cdce_to_dc_str(
            data,
            false,
            false,
            target_supports_b64,
            permissive,
            log,
        )?,
        "legacy_cdce" | "cdce_legacy" => {
            convert_cdce_to_dc_str(data, true, false, false, permissive, log)?
        }
        "cdce_lstrict" | "cdce_legacy_strict" => {
            convert_cdce_to_dc_str(data, true, true, false, permissive, log)?
        }
        "utf8" => convert_utf8_to_dc_str(data, target_supports_b64, permissive, log)?,
        "utf32" => {
            let utf8_bytes = ctb_formats_encoding::unicode::utf32be_to_utf8(
                data,
            )
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            convert_utf8_to_dc_str(&utf8_bytes, target_supports_b64, permissive, log)?
        }
        "utf8_base64" => {
            let b64_str = std::str::from_utf8(data).map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let decoded =
                ctb_formats_base64::base64_decode(b64_str).map_err(|_| {
                    log.error(
                        "This document is not stored using the specified format.",
                    );
                    anyhow!(
                        "This document is not stored using the specified format."
                    )
                })?;
            convert_utf8_to_dc_str(&decoded, target_supports_b64, permissive, log)?
        }
        "utf8_dc64" => {
            let decoded_utf8 = decode_utf8_dc64_csv_to_bytes(data, false, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        "utf8_dc64_enc" => {
            let decoded_utf8 = decode_utf8_dc64_csv_to_bytes(data, true, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        "utf8_dc64_bin" => {
            let decoded_utf8 =
                decode_utf8_dc64_bin_to_bytes(data, false, false, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        "utf8_dc64_bin_hex" => {
            let decoded_utf8 =
                decode_utf8_dc64_bin_to_bytes(data, false, true, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        "utf8_dc64_bin_enc" => {
            let decoded_utf8 =
                decode_utf8_dc64_bin_to_bytes(data, true, false, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        "utf8_dc64_bin_enc_hex" => {
            let decoded_utf8 =
                decode_utf8_dc64_bin_to_bytes(data, true, true, log)?;
            convert_utf8_to_dc_str(&decoded_utf8, target_supports_b64, permissive, log)?
        }
        _ => {
            log.error("Unknown input format.");
            bail!("Unknown input format.");
        }
    };

    if input_format == output_format
        && !matches!(input_format, "dc" | "dc_id_list")
    {
        return Ok(data.to_vec());
    }

    let mut dc_cleaned = dc_str.trim_end_matches(',').to_string();
    while dc_cleaned.contains(",,") {
        dc_cleaned = dc_cleaned.replace(",,", ",0,");
    }

    // Step 2: Convert short Dc list to output format
    match output_format {
        "dc" | "dc_id_list" => {
            let parsed = parse_dc_csv_list(&dc_cleaned, log)?;
            let dc_out =
                crate::convert_dc_to_dc_output(&join_short_dcs(&parsed), log);
            Ok(dc_out.into_bytes())
        }
        "cdce" => convert_dc_str_to_cdce(&dc_cleaned, false, permissive, log),
        "legacy_cdce" | "cdce_legacy" => {
            convert_dc_str_to_cdce(&dc_cleaned, true, permissive, log)
        }
        "html_l" | "html_legacy" | "html_legacy_cdce" => {
            let cdce_bytes = convert_dc_str_to_cdce(&dc_cleaned, true, permissive, log)?;
            legacy::onestep_legacy_cdce_to_html_l(&cdce_bytes)
        }
        "html_snippet_l" | "html_s_legacy" | "html_legacy_cdce_snippet" => {
            let cdce_bytes = convert_dc_str_to_cdce(&dc_cleaned, true, permissive, log)?;
            legacy::onestep_legacy_cdce_to_html_snippet_l(&cdce_bytes)
        }
        "dce" | "hex_dce" | "dce_hex" | "3_0a" | "dce_3_0a" | "3_0a_raw"
        | "dce_3_0a_raw" | "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex"
        | "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" | "3_01a"
        | "dce_3_01a" | "3_01a_raw" | "dce_3_01a_raw" | "hex_3_01a"
        | "3_01a_hex" | "dce_3_01a_hex" | "hex_3_01a_raw" | "3_01a_raw_hex"
        | "dce_3_01a_raw_hex" | "3_02a" | "dce_3_02a" | "3_02a_raw"
        | "dce_3_02a_raw" | "hex_3_02a" | "3_02a_hex" | "dce_3_02a_hex"
        | "hex_3_02a_raw" | "3_02a_raw_hex" | "dce_3_02a_raw_hex" => {
            let dcs = parse_dc_csv_list(&dc_cleaned, log)?;
            encode_dce_preserving_controls(&dcs, output_format, log)
        }
        "utf8" => convert_dc_str_to_utf8(&dc_cleaned, permissive, log),
        "utf32" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            ctb_formats_encoding::unicode::utf8_to_utf32be(&utf8_bytes)
        }
        "utf8_base64" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            Ok(ctb_formats_base64::base64_encode(&utf8_bytes).into_bytes())
        }
        "utf8_dc64" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_bytes_to_dc64(&utf8_bytes);
            Ok(join_short_dcs(&dcs).into_bytes())
        }
        "utf8_dc64_enc" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_utf8_to_dc64_encapsulated(&utf8_bytes);
            Ok(join_short_dcs(&dcs).into_bytes())
        }
        "utf8_dc64_bin" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_bytes_to_dc64(&utf8_bytes);
            encode_dc64_to_core_bytes(&dcs, false, false)
        }
        "utf8_dc64_bin_hex" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_bytes_to_dc64(&utf8_bytes);
            encode_dc64_to_core_bytes(&dcs, false, true)
        }
        "utf8_dc64_bin_enc" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_bytes_to_dc64(&utf8_bytes);
            encode_dc64_to_core_bytes(&dcs, true, false)
        }
        "utf8_dc64_bin_enc_hex" => {
            let utf8_bytes = convert_dc_str_to_utf8(&dc_cleaned, permissive, log)?;
            let dcs = encode_bytes_to_dc64(&utf8_bytes);
            encode_dc64_to_core_bytes(&dcs, true, true)
        }
        _ => {
            log.error("Unknown output format.");
            bail!("Unknown output format.");
        }
    }
}

/// Decodes a DCE binary/hex stream into its content short Dc IDs (bounded by `114` and `115`,
/// without the outer `[15, 16, ver, 13]` / `[13, 113]` file envelope or automatic sheet switches
/// unless those control Dcs were the sole content).
fn decode_dce_content_short_dcs(
    data: &[u8],
    input_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    let all = decode_dce_preserving_controls(data, input_format, log)?;
    let inner = if all.len() >= 6
        && all.first() == Some(&15)
        && all.get(1) == Some(&16)
        && matches!(all.get(2), Some(&112 | &194))
        && matches!(all.get(3), Some(&13 | &14))
        && matches!(all.get(all.len().saturating_sub(2)), Some(&13 | &14))
        && all.get(all.len().saturating_sub(1)) == Some(&113)
    {
        all.get(4..all.len().saturating_sub(2))
            .context("DCE envelope slice")?
    } else {
        all.as_slice()
    };

    // Strip sheet-switch control Dcs (`16`, `196..=199`, and `13`/`14` that follow a sheet switch)
    // when they were used to introduce a non-Core character, while preserving standalone control Dcs.
    let mut out = Vec::with_capacity(inner.len());
    let mut in_non_core_sheet = false;
    for (idx, &dc) in inner.iter().enumerate() {
        let next_dc = inner.get(idx.saturating_add(1)).copied();
        if matches!(dc, 16 | 196..=199) {
            let next_is_sheet_char = match (dc, next_dc) {
                // TBD: 3.02a version selector Dc in DCE_Version_Selectors is unassigned.
                (16, Some(112 | 194))
                | (196, Some(122..=126))
                | (197, Some(2..=11))
                | (198, Some(12))
                | (199, Some(119)) => true,
                _ => false,
            };
            if next_is_sheet_char {
                in_non_core_sheet = true;
                continue;
            }
            // Standalone sheet-switch Dc (followed immediately by 13/14/115)
            out.push(dc);
            in_non_core_sheet = true;
            continue;
        }
        if matches!(dc, 13 | 14) && in_non_core_sheet {
            in_non_core_sheet = false;
            continue;
        }
        out.push(dc);
    }

    Ok(out)
}

fn convert_cdce_to_dc_str(
    data: &[u8],
    legacy_only: bool,
    strict: bool,
    encapsulate_unmapped_utf8: bool,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<String> {
    let (doc, text_log) = ctb_formats_dcstring::converters::utf8::utf8_to_dcmstring_with_options(data, permissive)?;
    log.merge(&text_log);
    let codepoints: Vec<u32> = doc.to_dc_string()?.chars()
        .map(|ch| u32::try_from(ch.to_long()).context("UTF-8 codepoint fits u32"))
        .collect::<Result<_>>()?;

    let mut out_dcs: Vec<u32> = Vec::new();
    let mut unmapped_utf8_buf = String::new();
    let mut i = 0_usize;

    let flush_unmapped = |buf: &mut String, out: &mut Vec<u32>| {
        if !buf.is_empty() {
            out.extend(encode_utf8_to_dc64_encapsulated(buf.as_bytes()));
            buf.clear();
        }
    };

    while i < codepoints.len() {
        let cp = *codepoints
            .get(i)
            .context("codepoints index out of bounds")?;
        if cp == 0x40 {
            if let Some((dc_val, consumed)) =
                parse_at_escape(&codepoints, i, legacy_only)
            {
                flush_unmapped(&mut unmapped_utf8_buf, &mut out_dcs);
                out_dcs.push(dc_val);
                i = i.saturating_add(consumed);
                continue;
            }
            if strict && !permissive {
                let message = format!(
                    "CDCE decoding error: invalid escape sequence at codepoint index {i}"
                );
                log.error(&message);
                bail!("{message}");
            }
            log.warn(&format!(
                "Warning: Unicode recovery attempted for corrupted CDCE sequence at codepoint index {i}"
            ));
            flush_unmapped(&mut unmapped_utf8_buf, &mut out_dcs);
            out_dcs.push(1);
            i = i.saturating_add(1);
        } else if let Some(short_dc) = if encapsulate_unmapped_utf8 {
            unicode_codepoint_to_short_dc(cp)
        } else {
            unicode_codepoint_to_short_dc_lossy(cp)
        } {
            flush_unmapped(&mut unmapped_utf8_buf, &mut out_dcs);
            out_dcs.push(short_dc);
            i = i.saturating_add(1);
        } else if encapsulate_unmapped_utf8 && let Some(ch) = char::from_u32(cp)
        {
            unmapped_utf8_buf.push(ch);
            i = i.saturating_add(1);
        } else {
            log.import_warning(
                u64::try_from(i)?,
                &format!(
                    "Unmapped Unicode codepoint U+{cp:04X} dropped during lossy CDCE import"
                ),
            );
            i = i.saturating_add(1);
        }
    }

    flush_unmapped(&mut unmapped_utf8_buf, &mut out_dcs);
    Ok(join_short_dcs(&out_dcs))
}

fn parse_at_escape(
    codepoints: &[u32],
    start: usize,
    legacy_only: bool,
) -> Option<(u32, usize)> {
    let mut j = start.saturating_add(1);
    let mut val = 0_u32;
    let mut digits = 0_usize;
    while j < codepoints.len() {
        let cp = *codepoints.get(j)?;
        if cp == 0x40 {
            if digits == 0 {
                return None;
            }
            if legacy_only && !(1..=12).contains(&val) {
                return None;
            }
            return Some((val, digits.saturating_add(2)));
        }
        if (0x30..=0x39).contains(&cp) {
            let d = cp.saturating_sub(0x30);
            if digits == 0 && d == 0 && legacy_only {
                return None;
            }
            val = val.checked_mul(10)?.checked_add(d)?;
            digits = digits.saturating_add(1);
            if legacy_only && digits > 2 {
                return None;
            }
            j = j.saturating_add(1);
        } else {
            return None;
        }
    }
    None
}

fn convert_utf8_to_dc_str(
    data: &[u8],
    encapsulate_unmapped_utf8: bool,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<String> {
    let (doc, text_log) = ctb_formats_dcstring::converters::utf8::utf8_to_dcmstring_with_options(data, permissive)?;
    log.merge(&text_log);

    let mut out_dcs = Vec::new();
    let mut unmapped_utf8_buf = String::new();
    for (idx, ch) in doc.to_dc_string()?.chars().enumerate() {
        let cp = u32::try_from(ch.to_long()).context("UTF-8 codepoint fits u32")?;
        if let Some(short_dc) = if encapsulate_unmapped_utf8 {
            unicode_codepoint_to_short_dc(cp)
        } else {
            unicode_codepoint_to_short_dc_lossy(cp)
        } {
            if !unmapped_utf8_buf.is_empty() {
                out_dcs.extend(encode_utf8_to_dc64_encapsulated(
                    unmapped_utf8_buf.as_bytes(),
                ));
                unmapped_utf8_buf.clear();
            }
            out_dcs.push(short_dc);
        } else if encapsulate_unmapped_utf8 && let Some(ch) = char::from_u32(cp)
        {
            unmapped_utf8_buf.push(ch);
        } else {
            log.import_warning(
                u64::try_from(idx)?,
                &format!(
                    "Unmapped Unicode codepoint U+{cp:04X} dropped during lossy Unicode import"
                ),
            );
        }
    }

    if !unmapped_utf8_buf.is_empty() {
        out_dcs.extend(encode_utf8_to_dc64_encapsulated(
            unmapped_utf8_buf.as_bytes(),
        ));
    }

    Ok(join_short_dcs(&out_dcs))
}

fn decode_encapsulated_utf8(
    dcs: &[u32],
    start: usize,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<(Vec<u8>, usize)> {
    let remaining = dcs.get(start..).context("Missing UTF-8 payload")?;
    let length = if let Some(end) = remaining.iter().position(|&dc| dc == 192) {
        end.saturating_add(1)
    } else {
        remaining.len()
    };
    let payload = remaining
        .get(..length)
        .context("Invalid UTF-8 payload range")?;
    let decoded = short_dcs_to_utf8(payload, permissive, log)?;
    Ok((decoded, start.saturating_add(length)))
}

fn convert_dc_str_to_utf8(
    data: &str,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dcs = parse_dc_csv_list(data, log)?;
    short_dcs_to_utf8(&dcs, permissive, log)
}

fn short_dcs_to_utf8(
    dcs: &[u32],
    permissive: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    use ctb_formats_dcstring::converters::utf8::{
        DcListUtf8Settings, dcmstr_to_utf8_with_options,
    };
    use ctb_formats_dcstring::{DcChar, DcMstring};

    let mut doc = DcMstring::new();
    for &dc in dcs {
        if is_dce_encoding_control_dc(dc) {
            continue;
        }
        if dc == 119 {
            doc.push_char(DcChar::from('\n'));
        } else {
            doc.push_char(DcChar::from_short(dc));
        }
    }
    let settings = DcListUtf8Settings {
        canonicalize_equivalent_dcs: true,
        permissive,
        ..Default::default()
    };
    let (bytes, conversion_log) = dcmstr_to_utf8_with_options(&doc, &settings)?;
    log.merge(&conversion_log);
    Ok(bytes)
}

fn convert_dc_str_to_cdce(
    data: &str,
    legacy_only: bool,
    permissive: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dcs = parse_dc_csv_list(data, log)?;
    let mut out = Vec::new();
    let mut i = 0_usize;

    while i < dcs.len() {
        let dc = *dcs.get(i).context("dcs index out of bounds")?;
        if (1..=12).contains(&dc) {
            out.extend_from_slice(format!("@{dc}@").as_bytes());
            i = i.saturating_add(1);
            continue;
        }
        if dc == 191 {
            let (decoded, next) = decode_encapsulated_utf8(&dcs, i, permissive, log)?;
            for byte in decoded {
                if byte == b'@' {
                    out.extend_from_slice(b"@1@");
                } else {
                    out.push(byte);
                }
            }
            i = next;
            continue;
        }
        if let Some(cp) = short_dc_to_unicode_codepoint(dc)
            && let Some(ch) = char::from_u32(cp)
        {
            let mut buf = [0_u8; 4];
            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        } else if !is_dce_encoding_control_dc(dc) {
            if legacy_only {
                log.export_warning(
                    u64::try_from(i)?,
                    &format!(
                        "Short Dc {dc} has no Unicode mapping in legacy CDCE and was replaced with U+FFFD"
                    ),
                );
                out.extend_from_slice("\u{FFFD}".as_bytes());
            } else {
                out.extend_from_slice(format!("@{dc}@").as_bytes());
            }
        }
        i = i.saturating_add(1);
    }

    Ok(out)
}

fn decode_utf8_dc64_csv_to_bytes(
    data: &[u8],
    encapsulated: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let data_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    let trimmed = data_str.trim().trim_end_matches(',');
    let inner = if encapsulated && trimmed == "191,192" {
        ""
    } else if encapsulated {
        if let Some(rest) = trimmed.strip_prefix("191,")
            && let Some(core) = rest.strip_suffix(",192")
        {
            core
        } else {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        }
    } else {
        if trimmed.starts_with("191,") && trimmed.ends_with(",192") {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        }
        trimmed
    };

    let mut dcs = Vec::new();
    for piece in explode_escaped(',', inner) {
        let token = piece.trim();
        if token.is_empty() {
            continue;
        }
        let Ok(val) = token.parse::<u32>() else {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        };
        if !is_base64_payload_short_dc(val) {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        }
        dcs.push(val);
    }
    decode_dc64_to_bytes(&dcs, log)
}

fn decode_utf8_dc64_bin_to_bytes(
    data: &[u8],
    encapsulated: bool,
    is_hex: bool,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let raw_bytes = if is_hex {
        decode_hex_payload(data, log)?
    } else {
        data.to_vec()
    };

    let payload = if encapsulated {
        if raw_bytes.len() < 2
            || raw_bytes.first() != Some(&0xC3)
            || raw_bytes.last() != Some(&0xC4)
        {
            log.error(
                "This document is not stored using the specified format.",
            );
            bail!("This document is not stored using the specified format.");
        }
        raw_bytes
            .get(1..raw_bytes.len().saturating_sub(1))
            .context("encapsulated slice")?
    } else {
        raw_bytes.as_slice()
    };

    let mut dcs = Vec::with_capacity(payload.len());
    for &byte in payload {
        match byte {
            0x82..=0xC1 => {
                dcs.push(
                    u32::from(byte).saturating_sub(0x82).saturating_add(127),
                );
            }
            0xC2 => dcs.push(195),
            _ => {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
        }
    }
    decode_dc64_to_bytes(&dcs, log)
}

fn encode_dc64_to_core_bytes(
    dcs: &[u32],
    encapsulated: bool,
    is_hex: bool,
) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(dcs.len().saturating_add(2));
    if encapsulated {
        out.push(0xC3);
    }
    for &dc in dcs {
        let Some((DceSheet::Core, byte)) =
            encode_short_dc_in_version(dc, DceVersion::V3_01a)
        else {
            bail!("Invalid Base64 short Dc {dc}");
        };
        out.push(byte);
    }
    if encapsulated {
        out.push(0xC4);
    }
    if is_hex {
        Ok(bin2hex(out).to_uppercase().into_bytes())
    } else {
        Ok(out)
    }
}

fn parse_dc_csv_list(data: &str, log: &mut FormatLog) -> Result<Vec<u32>> {
    let trimmed = data.trim().trim_end_matches(',');
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for (idx, piece) in explode_escaped(',', trimmed).into_iter().enumerate() {
        let token = piece.trim();
        if token.is_empty() {
            out.push(0);
            continue;
        }
        match token.parse::<u32>() {
            Ok(val) => out.push(val),
            Err(_) => {
                log.import_error(
                    u64::try_from(idx)?,
                    &format!("Invalid short Dc ID token '{token}' in Dc list"),
                );
                bail!("Invalid short Dc ID token '{token}' in Dc list");
            }
        }
    }
    Ok(out)
}

fn join_short_dcs(dcs: &[u32]) -> String {
    dcs.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
}

fn decode_hex_payload(data: &[u8], log: &mut FormatLog) -> Result<Vec<u8>> {
    let hex_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    hex2bin(hex_str).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })
}

#[cfg(test)]
#[allow(
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
    fn test_new_converter_3_0a_all_short_dcs_0_to_115() {
        for dc in 0..=crate::MAX_DCE_3_0A_SHORT_DC {
            if (2..=12).contains(&dc) {
                // Short Dcs 2..=12 had no byte mappings in DCE 3.0a (added in 3.01a);
                // exporting to 3.0a must log an export warning in FormatLog and omit them.
                let mut enc_log = crate::FormatLog::default();
                let encoded = crate::converter::encode_dce_preserving_controls(
                    &[18, dc, 19],
                    "3_0a",
                    &mut enc_log,
                )
                .unwrap();
                assert!(
                    !enc_log.has_errors() && enc_log.has_warnings(),
                    "3_0a encoding of unmapped short Dc {dc} should log a warning: {enc_log:?}"
                );
                let mut dec_log = crate::FormatLog::default();
                let decoded = crate::converter::decode_dce_preserving_controls(
                    &encoded,
                    "3_0a",
                    &mut dec_log,
                )
                .unwrap();
                assert!(!decoded.contains(&dc));
                assert!(decoded.contains(&18) && decoded.contains(&19));
                continue;
            }

            for fmt in ["3_0a", "3_0a_raw", "hex_3_0a", "hex_3_0a_raw"] {
                let mut enc_log = crate::FormatLog::default();
                let encoded = crate::converter::encode_dce_preserving_controls(
                    &[dc],
                    fmt,
                    &mut enc_log,
                )
                .unwrap();
                assert!(
                    !enc_log.has_errors() && !enc_log.has_warnings(),
                    "3_0a ({fmt}) encoding of short Dc {dc} had warnings/errors: {enc_log:?}"
                );

                let mut dec_log = crate::FormatLog::default();
                let decoded = crate::converter::decode_dce_preserving_controls(
                    &encoded,
                    fmt,
                    &mut dec_log,
                )
                .unwrap();
                assert!(
                    !dec_log.has_errors() && !dec_log.has_warnings(),
                    "3_0a ({fmt}) decoding of short Dc {dc} had warnings/errors: {dec_log:?}"
                );
                assert!(
                    decoded.contains(&dc),
                    "3_0a ({fmt}) decoded sequence {decoded:?} did not contain short Dc {dc}"
                );

                // Re-encoding the decoded sequence must be byte-for-byte idempotent
                let mut reenc_log = crate::FormatLog::default();
                let reencoded =
                    crate::converter::encode_dce_preserving_controls(
                        &decoded,
                        fmt,
                        &mut reenc_log,
                    )
                    .unwrap();
                assert_eq!(
                    reencoded, encoded,
                    "3_0a ({fmt}) re-encode mismatch for short Dc {dc}"
                );
            }

            // Also verify dc -> 3_0a -> dc roundtrip via converter::dce_convert_with_log
            let dc_input = format!("114,{dc},115");
            let (dce_bytes, log_out) = crate::converter::dce_convert_with_log(
                dc_input.as_bytes(),
                "dc",
                "3_0a",
            )
            .unwrap();
            assert!(
                !log_out.has_errors() && !log_out.has_warnings(),
                "dce_convert dc -> 3_0a failed for Dc {dc}: {log_out:?}"
            );
            let (dc_back, log_in) = crate::converter::dce_convert_with_log(
                &dce_bytes, "3_0a", "dc",
            )
            .unwrap();
            assert!(
                !log_in.has_errors() && !log_in.has_warnings(),
                "dce_convert 3_0a -> dc failed for Dc {dc}: {log_in:?}"
            );
            assert_eq!(
                String::from_utf8(dc_back).unwrap(),
                dc_input,
                "dc -> 3_0a -> dc mismatch for short Dc {dc}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_new_converter_3_01a_all_short_dcs_0_to_199_and_base64() {
        for dc in 0..=crate::MAX_DCE_3_01A_SHORT_DC {
            for fmt in ["3_01a", "3_01a_raw", "hex_3_01a", "hex_3_01a_raw"] {
                let mut enc_log = crate::FormatLog::default();
                let encoded = crate::converter::encode_dce_preserving_controls(
                    &[dc],
                    fmt,
                    &mut enc_log,
                )
                .unwrap();
                assert!(
                    !enc_log.has_errors() && !enc_log.has_warnings(),
                    "3_01a ({fmt}) encoding of short Dc {dc} had warnings/errors: {enc_log:?}"
                );

                let mut dec_log = crate::FormatLog::default();
                let decoded = crate::converter::decode_dce_preserving_controls(
                    &encoded,
                    fmt,
                    &mut dec_log,
                )
                .unwrap();
                assert!(
                    !dec_log.has_errors() && !dec_log.has_warnings(),
                    "3_01a ({fmt}) decoding of short Dc {dc} had warnings/errors: {dec_log:?}"
                );
                assert!(
                    decoded.contains(&dc),
                    "3_01a ({fmt}) decoded sequence {decoded:?} did not contain short Dc {dc}"
                );

                let mut reenc_log = crate::FormatLog::default();
                let reencoded =
                    crate::converter::encode_dce_preserving_controls(
                        &decoded,
                        fmt,
                        &mut reenc_log,
                    )
                    .unwrap();
                assert_eq!(
                    reencoded, encoded,
                    "3_01a ({fmt}) re-encode mismatch for short Dc {dc}"
                );
            }

            let dc_input = format!("114,{dc},115");
            let (dce_bytes, log_out) = crate::converter::dce_convert_with_log(
                dc_input.as_bytes(),
                "dc",
                "3_01a",
            )
            .unwrap();
            assert!(
                !log_out.has_errors() && !log_out.has_warnings(),
                "dce_convert dc -> 3_01a failed for Dc {dc}: {log_out:?}"
            );
            let (dc_back, log_in) = crate::converter::dce_convert_with_log(
                &dce_bytes, "3_01a", "dc",
            )
            .unwrap();
            assert!(
                !log_in.has_errors() && !log_in.has_warnings(),
                "dce_convert 3_01a -> dc failed for Dc {dc}: {log_in:?}"
            );
            assert_eq!(
                String::from_utf8(dc_back).unwrap(),
                dc_input,
                "dc -> 3_01a -> dc mismatch for short Dc {dc}"
            );
        }

        // Verify Base64 'I' (Dc 135) and 'i' (Dc 161) are distinct and roundtrip properly
        assert_eq!(crate::converter::base64_char_to_short_dc(b'I'), Some(135));
        assert_eq!(crate::converter::base64_char_to_short_dc(b'i'), Some(161));
        assert_eq!(crate::converter::short_dc_to_base64_char(135), Some('I'));
        assert_eq!(crate::converter::short_dc_to_base64_char(161), Some('i'));

        // Payload whose Base64 encoding ("SGkh") contains 'i' (Dc 161) and ("I1 test") -> "STEgdGVzdA==" contains 'I'
        let utf8_with_b64_i_and_cap_i =
            "Hi! — 🌄 Testing Base64 'i' and 'I' encapsulation";
        for b64_fmt in [
            "utf8_dc64",
            "utf8_dc64_enc",
            "utf8_dc64_bin",
            "utf8_dc64_bin_hex",
            "utf8_dc64_bin_enc",
            "utf8_dc64_bin_enc_hex",
            "3_01a",
            "dce",
        ] {
            let (encoded, enc_log) = crate::converter::dce_convert_with_log(
                utf8_with_b64_i_and_cap_i.as_bytes(),
                "utf8",
                b64_fmt,
            )
            .unwrap();
            assert!(!enc_log.has_errors() && !enc_log.has_warnings());
            let (decoded, dec_log) = crate::converter::dce_convert_with_log(
                &encoded, b64_fmt, "utf8",
            )
            .unwrap();
            assert!(!dec_log.has_errors() && !dec_log.has_warnings());
            assert_eq!(
                String::from_utf8(decoded).unwrap(),
                utf8_with_b64_i_and_cap_i,
                "UTF-8 roundtrip through {b64_fmt} failed"
            );
        }
    }

    #[crate::ctb_test]
    fn test_new_converter_3_02a_stubbed_returns_error() {
        assert_eq!(crate::DceVersion::V3_02a.version_byte(), None);
        assert!(crate::DceVersion::V3_02a.supports_short_dc(202));
        assert!(!crate::DceVersion::V3_02a.supports_short_dc(205));

        for fmt in [
            "3_02a",
            "dce_3_02a",
            "3_02a_raw",
            "dce_3_02a_raw",
            "hex_3_02a",
            "3_02a_hex",
            "dce_3_02a_hex",
            "hex_3_02a_raw",
            "3_02a_raw_hex",
            "dce_3_02a_raw_hex",
        ] {
            let mut enc_log = crate::FormatLog::default();
            assert!(
                crate::converter::encode_dce_preserving_controls(
                    &[50],
                    fmt,
                    &mut enc_log,
                )
                .is_err(),
                "{fmt} encode should return an error while 3.02a is stubbed"
            );
            assert!(enc_log.has_errors());

            let mut dec_log = crate::FormatLog::default();
            let sample_input = if fmt.contains("hex") {
                &b"804181"[..]
            } else {
                &b"\x80A\x81"[..]
            };
            assert!(
                crate::converter::decode_dce_preserving_controls(
                    sample_input,
                    fmt,
                    &mut dec_log,
                )
                .is_err(),
                "{fmt} decode should return an error while 3.02a is stubbed"
            );
            assert!(dec_log.has_errors());

            assert!(
                crate::converter::dce_convert_with_log(
                    b"114,50,115",
                    "dc",
                    fmt,
                )
                .is_err(),
                "dce_convert dc -> {fmt} should return an error while 3.02a is stubbed"
            );
            assert!(
                crate::converter::dce_convert_with_log(
                    sample_input,
                    fmt,
                    "dc",
                )
                .is_err(),
                "dce_convert {fmt} -> dc should return an error while 3.02a is stubbed"
            );
        }
    }
}
