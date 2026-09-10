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

//! Document Character (`DcChar`) representation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;

use ctb_formats_dcdata::dc::SHORT_DC_REGION_START;
use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;

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
    /// (in range `SHORT_DC_REGION_START..SHORT_DC_REGION_START + 0x100000`).
    #[must_use]
    pub const fn is_short_dc(self) -> bool {
        self.0 >= SHORT_DC_REGION_START
            && self.0 < SHORT_DC_REGION_START.saturating_add(0x10_0000)
    }

    /// Converts this character to a short Document Character number if in range.
    #[must_use]
    pub fn to_short_dc(self) -> Option<u32> {
        if self.is_short_dc() {
            u32::try_from(self.0.saturating_sub(SHORT_DC_REGION_START)).ok()
        } else {
            None
        }
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
            let leading_zeros =
                usize::try_from(self.0.leading_zeros()).unwrap_or(0);
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
        buf.get(..n).map_or_else(Vec::new, <[u8]>::to_vec)
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
