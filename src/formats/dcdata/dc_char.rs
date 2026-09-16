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

//! Document Character (`DcChar`) representation and conversions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;

use anyhow::{anyhow, bail};
use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;
use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_END, FORMAT_REGION_START, SHORT_DC_REGION_END,
    SHORT_DC_REGION_START,
};


/// A single Document Character (Dc) or Unicode codepoint, represented as a
/// 128-bit integer.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DcChar(pub u128);

impl DcChar {
    /// Unicode replacement character (`U+FFFD`).
    pub const REPLACEMENT: Self = Self(0xFFFD);

    /// Creates a `DcChar` from a `u128` integer ID.
    #[must_use]
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }

    /// Returns the underlying `u128` integer ID.
    #[must_use]
    pub const fn to_u128(self) -> u128 {
        self.0
    }

    /// Creates a `DcChar` from a long (global graph) `u128` ID.
    #[must_use]
    pub const fn from_long(val: u128) -> Self {
        Self(val)
    }

    /// Returns the long (global graph) `u128` ID.
    #[must_use]
    pub const fn to_long(self) -> u128 {
        self.0
    }

    /// Creates a `DcChar` from a short Document Character ID (`u32`).
    ///
    /// Computes `SHORT_DC_REGION_START + short_id`.
    #[must_use]
    pub const fn from_short(short_id: u32) -> Self {
        let b = short_id.to_le_bytes();
        let id = u128::from_le_bytes([
            b[0], b[1], b[2], b[3], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        Self(SHORT_DC_REGION_START.saturating_add(id))
    }

    /// Converts this `DcChar` into a short Document Character ID (`u32`).
    ///
    /// # Errors
    /// Returns an error if this character is not in the short Document
    /// Character region (`SHORT_DC_REGION_START..=SHORT_DC_REGION_END`).
    pub fn to_short(self) -> Result<u32> {
        if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&self.0) {
            let diff = self.0.saturating_sub(SHORT_DC_REGION_START);
            u32::try_from(diff)
                .map_err(|e| anyhow!("Failed to convert short Dc offset to u32: {e}"))
        } else {
            bail!(
                "DcChar(0x{:X}) is not in short Document Character range ({SHORT_DC_REGION_START}..={SHORT_DC_REGION_END})",
                self.0
            )
        }
    }

    /// Converts this character into a sequence of short Document Character IDs.
    ///
    /// Infallible conversion using Dc 308 embedding for non-short characters:
    /// - Dc 308 itself is escaped as `[255, 308]`.
    /// - Dc 255 itself is escaped as `[255, 255]`.
    /// - Short Document Characters (`SHORT_DC_REGION_START..=SHORT_DC_REGION_END`)
    ///   map directly to `[short_id]`.
    /// - All other characters (Unicode codepoints and out-of-range IDs) are
    ///   embedded via Dc 308: `[308, 6, 199, ...digits..., 7]`.
    #[must_use]
    pub fn to_short_vec(self) -> Vec<u32> {
        let escape = crate::dc::DC_ESCAPE.to_short().unwrap_or(255);
        let long_dc = crate::dc::DC_LONG_DC.to_short().unwrap_or(308);

        if self == crate::dc::DC_LONG_DC {
            vec![escape, long_dc]
        } else if self == crate::dc::DC_ESCAPE {
            vec![escape, escape]
        } else if let Ok(short_id) = self.to_short() {
            vec![short_id]
        } else {
            let mut out = Vec::new();
            out.push(long_dc);
            out.extend(crate::dc_number_minimal::u128_to_dc_number_short(self.0));
            out
        }
    }

    /// Creates a `DcChar` from a short Format ID (`u32`).
    ///
    /// Computes `FORMAT_REGION_START + fmt_id`.
    #[must_use]
    pub const fn from_format(fmt_id: u32) -> Self {
        let b = fmt_id.to_le_bytes();
        let id = u128::from_le_bytes([
            b[0], b[1], b[2], b[3], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        Self(FORMAT_REGION_START.saturating_add(id))
    }

    /// Converts this `DcChar` into a short Format ID (`u32`).
    ///
    /// # Errors
    /// Returns an error if this character is not in the Format region
    /// (`FORMAT_REGION_START..=FORMAT_REGION_END`).
    pub fn to_format(self) -> Result<u32> {
        if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&self.0) {
            let diff = self.0.saturating_sub(FORMAT_REGION_START);
            u32::try_from(diff)
                .map_err(|e| anyhow!("Failed to convert format offset to u32: {e}"))
        } else {
            bail!(
                "DcChar(0x{:X}) is not in Format region ({FORMAT_REGION_START}..={FORMAT_REGION_END})",
                self.0
            )
        }
    }

    /// Returns `true` if this character is in the Format region.
    #[must_use]
    pub const fn is_format(self) -> bool {
        self.0 >= FORMAT_REGION_START && self.0 <= FORMAT_REGION_END
    }

    /// Creates a `DcChar` from a Unicode codepoint (`u32`).
    ///
    /// # Errors
    /// Returns an error if `cp` exceeds `0x10_FFFF`.
    pub fn from_unicode(cp: u32) -> Result<Self> {
        if cp <= 0x10_FFFF {
            let b = cp.to_le_bytes();
            let id = u128::from_le_bytes([
                b[0], b[1], b[2], b[3], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]);
            Ok(Self(id))
        } else {
            bail!("Codepoint 0x{cp:X} exceeds maximum Unicode range (0..=0x10FFFF)")
        }
    }

    /// Converts this `DcChar` into a Unicode codepoint (`u32`).
    ///
    /// # Errors
    /// Returns an error if this character exceeds `0x10_FFFF`.
    pub fn to_unicode(self) -> Result<u32> {
        if self.0 <= 0x10_FFFF {
            u32::try_from(self.0)
                .map_err(|e| anyhow!("Failed to convert Unicode codepoint to u32: {e}"))
        } else {
            bail!(
                "DcChar(0x{:X}) is not a Unicode codepoint (0..=0x10FFFF)",
                self.0
            )
        }
    }

    /// Creates a `DcChar` from a standard Unicode `char`.
    #[must_use]
    pub fn from_char(c: char) -> Self {
        Self(u128::from(u32::from(c)))
    }

    /// Attempts to convert this `DcChar` into a standard Unicode `char`.
    ///
    /// Returns `Some(char)` if this codepoint is a valid Unicode scalar value
    /// (`0..=0x10FFFF`, excluding surrogate halves `0xD800..=0xDFFF`).
    #[must_use]
    pub fn as_char(self) -> Option<char> {
        if self.0 <= 0x10_FFFF {
            let cp = u32::try_from(self.0).ok()?;
            char::from_u32(cp)
        } else {
            None
        }
    }

    /// Returns `true` if this codepoint is within the Unicode range
    /// (`0..=0x10FFFF`), including surrogate code points.
    #[must_use]
    pub const fn is_unicode(self) -> bool {
        self.0 <= 0x10_FFFF
    }

    /// Returns `true` if this codepoint is a valid Unicode scalar value
    /// (excluding surrogate code points `0xD800..=0xDFFF`).
    #[must_use]
    pub const fn is_unicode_scalar(self) -> bool {
        self.0 <= 0x10_FFFF && !(self.0 >= 0xD800 && self.0 <= 0xDFFF)
    }

    /// Returns `true` if this codepoint is within the standard ASCII range
    /// (`0..=0x7F`).
    #[must_use]
    pub const fn is_ascii(self) -> bool {
        self.0 <= 0x7F
    }

    /// Returns `true` if this character is a short Document Character
    /// (in range `SHORT_DC_REGION_START..=SHORT_DC_REGION_END`).
    #[must_use]
    pub const fn is_short_dc(self) -> bool {
        self.0 >= SHORT_DC_REGION_START && self.0 <= SHORT_DC_REGION_END
    }

    /// Converts this character to a short Document Character number if in range.
    #[must_use]
    pub fn to_short_dc(self) -> Option<u32> {
        self.to_short().ok()
    }

    /// Returns the number of bytes required to encode this `DcChar` in
    /// `UTF-8e-128` format (1..=24 bytes).
    #[must_use]
    pub fn len_utf_8e_128(self) -> usize {
        if self.0 <= 0x7F {
            1
        } else if self.0 <= 0x7FF {
            2
        } else if self.0 <= 0xFFFF {
            3
        } else if self.0 <= 0x10_FFFF {
            4
        } else {
            #[expect(
                clippy::expect_used,
                reason = "u128 leading zeros is at most 128, which fits in usize on all supported platforms"
            )]
            let leading_zeros =
                usize::try_from(self.0.leading_zeros()).expect("leading zeros <= 128 fits in usize");
            let bits = 128usize.saturating_sub(leading_zeros);
            let l = bits.div_ceil(6).max(1);
            2usize.saturating_add(l)
        }
    }

    /// Encodes this `DcChar` into the provided byte buffer (must be at least 24
    /// bytes) and returns the number of bytes written.
    pub fn encode_buf(self, buf: &mut [u8]) -> usize {
        encode_utf_8e_128_buf(buf, self.0)
    }

    /// Encodes this `DcChar` into a newly allocated `Vec<u8>`.
    #[must_use]
    pub fn encode(self) -> Vec<u8> {
        let mut buf = [0u8; 24];
        let n = self.encode_buf(&mut buf);
        #[expect(
            clippy::expect_used,
            reason = "encode_buf returns index within the 24-byte stack buffer"
        )]
        buf.get(..n)
            .expect("encode_buf returns valid slice of buffer")
            .to_vec()
    }
}

impl From<char> for DcChar {
    fn from(c: char) -> Self {
        Self::from_char(c)
    }
}

impl From<u8> for DcChar {
    fn from(val: u8) -> Self {
        Self(u128::from(val))
    }
}

impl From<u16> for DcChar {
    fn from(val: u16) -> Self {
        Self(u128::from(val))
    }
}

impl From<u32> for DcChar {
    fn from(val: u32) -> Self {
        Self(u128::from(val))
    }
}

impl From<u64> for DcChar {
    fn from(val: u64) -> Self {
        Self(u128::from(val))
    }
}

impl From<u128> for DcChar {
    fn from(val: u128) -> Self {
        Self(val)
    }
}

impl From<DcChar> for u128 {
    fn from(dc: DcChar) -> Self {
        dc.0
    }
}

impl PartialEq<u128> for DcChar {
    fn eq(&self, other: &u128) -> bool {
        self.0 == *other
    }
}

impl PartialEq<DcChar> for u128 {
    fn eq(&self, other: &DcChar) -> bool {
        *self == other.0
    }
}

impl PartialEq<char> for DcChar {
    fn eq(&self, other: &char) -> bool {
        self.0 == u128::from(u32::from(*other))
    }
}

impl PartialEq<DcChar> for char {
    fn eq(&self, other: &DcChar) -> bool {
        u128::from(u32::from(*self)) == other.0
    }
}

impl TryFrom<DcChar> for char {
    type Error = anyhow::Error;

    fn try_from(dc: DcChar) -> Result<Self, Self::Error> {
        dc.as_char().ok_or_else(|| {
            anyhow::anyhow!(
                "DcChar(0x{:X}) cannot be converted to Unicode scalar char",
                dc.0
            )
        })
    }
}

impl fmt::Display for DcChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 64 {
            write!(f, "@@")
        } else if let Some(c) = self.as_char() {
            write!(f, "{c}")
        } else {
            write!(f, "@{dcid}@", dcid = self.0)
        }
    }
}

impl fmt::Debug for DcChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(c) = self.as_char() {
            write!(f, "DcChar({c:?})")
        } else {
            write!(f, "DcChar(@{}@)", self.0)
        }
    }
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
    fn test_from_short_and_to_short() {
        let dc_203 = DcChar::from_short(203);
        assert_eq!(dc_203.0, SHORT_DC_REGION_START + 203);
        assert_eq!(dc_203.to_short().unwrap(), 203);
        assert!(dc_203.is_short_dc());
        assert_eq!(dc_203.to_short_dc(), Some(203));

        let non_short = DcChar::from_long(42);
        assert!(!non_short.is_short_dc());
        assert!(non_short.to_short().is_err());
    }

    #[crate::ctb_test]
    fn test_from_format_and_to_format() {
        let fmt_199 = DcChar::from_format(199);
        assert_eq!(fmt_199.0, FORMAT_REGION_START + 199);
        assert!(fmt_199.is_format());
        assert_eq!(fmt_199.to_format().unwrap(), 199);

        let non_fmt = DcChar::from_short(10);
        assert!(!non_fmt.is_format());
        assert!(non_fmt.to_format().is_err());
    }

    #[crate::ctb_test]
    fn test_from_unicode_and_to_unicode() {
        let uni_a = DcChar::from_unicode(65).unwrap();
        assert_eq!(uni_a.to_unicode().unwrap(), 65);
        assert_eq!(uni_a.as_char(), Some('A'));
        assert_eq!(uni_a, 'A');

        assert!(DcChar::from_unicode(0x11_0000).is_err());
    }

    #[crate::ctb_test]
    fn test_to_short_vec_infallible() {
        // Short Dc
        let dc_42 = DcChar::from_short(42);
        assert_eq!(dc_42.to_short_vec(), vec![42]);

        // Dc 308 escaped
        let dc_308 = DcChar::from_short(308);
        assert_eq!(dc_308.to_short_vec(), vec![255, 308]);

        // Dc 255 escaped
        let dc_255 = DcChar::from_short(255);
        assert_eq!(dc_255.to_short_vec(), vec![255, 255]);

        // Unicode 'A' (65): 65 in base 64 is 1 * 64 + 1 -> 'B' (128), 'B' (128)
        let uni_a = DcChar::from_char('A');
        let short_vec = uni_a.to_short_vec();
        assert_eq!(short_vec.get(0..3), Some(&[308, 6, 199][..]));
        assert_eq!(short_vec.last(), Some(&7));
    }

    #[crate::ctb_test]
    fn test_partial_eq_comparisons() {
        let dc = DcChar::from_short(203);
        assert_eq!(dc, SHORT_DC_REGION_START + 203);
        assert!(dc.eq(&(SHORT_DC_REGION_START + 203)));

        let uni = DcChar::from_char('Z');
        assert_eq!(uni, 'Z');
        assert!(uni.eq(&'Z'));
    }
}
