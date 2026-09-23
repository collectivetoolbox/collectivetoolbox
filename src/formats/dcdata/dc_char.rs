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

use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;
pub use ctb_utilities::dc_char::*;

/// Extension methods for [`DcChar`] specific to document formats and serialization.
pub trait DcCharExt {
    /// Converts this character into a sequence of short Document Character IDs.
    #[must_use]
    fn to_short_vec(self) -> Vec<u32>;

    /// Returns the number of bytes required to encode this `DcChar` in
    /// `UTF-8e-128` format (1..=24 bytes).
    #[must_use]
    fn len_utf_8e_128(self) -> usize;

    /// Encodes this `DcChar` into the provided byte buffer (must be at least 24
    /// bytes) and returns the number of bytes written.
    fn encode_buf(self, buf: &mut [u8]) -> usize;

    /// Encodes this `DcChar` into a newly allocated `Vec<u8>`.
    #[must_use]
    fn encode(self) -> Vec<u8>;
}

impl DcCharExt for DcChar {
    fn to_short_vec(self) -> Vec<u32> {
        // Reason for fallback: DC_ESCAPE is canonically Dc 255 which maps to short id 255
        let escape = crate::dc::DC_ESCAPE.to_short().unwrap_or(255);
        // Reason for fallback: DC_LONG_DC is canonically Dc 308 which maps to short id 308
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

    fn len_utf_8e_128(self) -> usize {
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

    fn encode_buf(self, buf: &mut [u8]) -> usize {
        encode_utf_8e_128_buf(buf, self.0)
    }

    fn encode(self) -> Vec<u8> {
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
        assert_eq!(dc_203.0, SHORT_DC_REGION_START.saturating_add(203));
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
        assert_eq!(fmt_199.0, FORMAT_REGION_START.saturating_add(199));
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
        assert_eq!(dc, SHORT_DC_REGION_START.saturating_add(203));
        assert!(dc.eq(&(SHORT_DC_REGION_START.saturating_add(203))));

        let uni = DcChar::from_char('Z');
        assert_eq!(uni, 'Z');
        assert!(uni.eq(&'Z'));
    }
}
