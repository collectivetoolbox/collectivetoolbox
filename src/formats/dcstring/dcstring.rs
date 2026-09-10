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

//! Implements DcText and related formats DcList and DcUTF. These formats are
//! encodings for sequences of integer global graph IDs (which are long Dcs).
//! Format looks like (w/o backticks): `Unicode (UTF-8) text @123@miesu@214748364@@L662@`
//! where `Unicode text` is actual unicode text, and between each pair of @ signs, is a DcId. A DcId can be any int 128 bits (u128) in decimal, and it may have an `L` prefix.
//! Output format is sort of UTF-8 text. For normal Unicode input characters, the output character is the same. For DcIds less than or equal to 1114111 (the largest Unicode character, I believe), the output character is the corresponding "generalized UTF-8", the numeric value encoded in the same underlying algorithm as UTF-8. For DcIds greater than 1114111 and not prefixed with an L, the output character is the decimal DcId represented by extending the usual algorithm of UTF-8 encoding, but for those larger numbers. For DcIds prefixed with an L, the output is equivalent to @1114408@ (short Dc 296) followed by a Dc number for the number that followed the L (the L is just a shorthand for that 1114408 Dc). That is to say, it's not a true Unicode encoding, it's simply using an extension of the algorithm underlying UTF-8 as a convenient encoding of ints.
//! Currently, DcList is used as the internal format for pivoting between other formats, but DcUtf might make more sense eventually for space efficiency.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub use ctb_formats_utf_8e_128::{
    DcChar, DcCharIndices, DcChars, DcStr, DcString, DcUtfError,
    decode_utf_8e_128, encode_utf_8e_128_buf, validate_dcutf,
};
pub use ctb_formats_dc_data::dc::{
    GID_ESCAPE, GID_LONG_DC, SHORT_DC_ESCAPE, SHORT_DC_LONG_DC,
    SHORT_DC_REGION_END, SHORT_DC_REGION_START,
};
pub use ctb_formats_utilities::ConversionOutput;
use ctb_formats_utilities::FormatLog;

pub mod character_description;
pub mod cli;
pub mod cli_identifiers;
pub mod dc_number;
pub mod dcal;
pub mod utf8;

pub use dc_number::{
    GID_BASE64_END, GID_BASE64_PADDING, GID_BASE64_START, GID_BEGIN_NUMBER,
    GID_END_NUMBER, GID_FORMAT_199, GID_NEGATIVE, GID_POSITIVE,
    SHORT_DC_BASE64_END, SHORT_DC_BASE64_PADDING, SHORT_DC_BASE64_START,
    SHORT_DC_BEGIN_NUMBER, SHORT_DC_END_NUMBER, SHORT_DC_NEGATIVE,
    SHORT_DC_POSITIVE, SHORT_ID_FORMAT_199, base64_char_to_global_dc,
    base64_char_to_short_dc, base64_str_to_global_dcs, base64_str_to_short_dcs,
    global_dc_to_base64_char, global_dcs_to_base64_str,
    i128_to_dc_number_global, i128_to_dc_number_short,
    integer_to_dc_number_global, integer_to_dc_number_short,
    natural_to_dc_number_global, natural_to_dc_number_short,
    parse_dc_number_global, parse_dc_number_global_i128, parse_dc_number_short,
    parse_dc_number_short_i128, read_dc_number_global, read_dc_number_short,
    short_dc_to_base64_char, short_dcs_to_base64_str, u128_to_dc_number_global,
    u128_to_dc_number_short,
};

pub use character_description::{
    describe_dcal, describe_dclist, describe_graph_id,
};
pub use cli::{
    CharacterDescriptionArgs, CharacterDescriptionInputFormat,
    execute_cli_character_description,
};
pub use cli_identifiers::{
    GidArgs, ShortDcArgs, ShortFmtArgs, execute_cli_gid, execute_cli_short_dc,
    execute_cli_short_fmt, parse_graph_or_short_id,
};
pub use dcal::{dcal_to_dclist, dclist_to_dcal};
pub use utf8::{
    DcListUtf8Settings, dclist_from_utf8, dclist_to_utf8, utf8_to_dclist,
};

/// Base offset for short Document Characters in the global graph layout.
/// Short Dc 0 starts at 1114112 (0x110000).
pub use dc_data::layout::SHORT_DC_OFFSET: u128 = 1_114_112;

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
    fn test_dc_char() {
        let c_ascii = DcChar::from_char('A');
        assert_eq!(c_ascii.as_char(), Some('A'));
        assert!(c_ascii.is_ascii());
        assert!(c_ascii.is_unicode());
        assert!(c_ascii.is_unicode_scalar());
        assert!(!c_ascii.is_short_dc());
        assert_eq!(c_ascii.to_short_dc(), None);
        assert_eq!(format!("{c_ascii}"), "A");

        let c_at = DcChar::from_char('@');
        assert_eq!(format!("{c_at}"), "@@");

        let c_short = DcChar(SHORT_DC_OFFSET + 42);
        assert_eq!(c_short.as_char(), None);
        assert!(!c_short.is_ascii());
        assert!(!c_short.is_unicode());
        assert!(c_short.is_short_dc());
        assert_eq!(c_short.to_short_dc(), Some(42));
        assert_eq!(format!("{c_short}"), format!("@{}@", SHORT_DC_OFFSET + 42));

        // Surrogate codepoint (allowed in DcUtf as a Unicode codepoint, but not a scalar char)
        let c_surrogate = DcChar(0xDBFF);
        assert_eq!(c_surrogate.as_char(), None);
        assert!(c_surrogate.is_unicode());
        assert!(!c_surrogate.is_unicode_scalar());
    }

    #[crate::ctb_test]
    fn test_dc_string_and_dc_str() {
        let mut s = DcString::new();
        s.push('h');
        s.push('i');
        s.push_str(" ");
        s.push(DcChar(SHORT_DC_OFFSET));

        assert_eq!(
            s.len(),
            3 + DcChar(SHORT_DC_OFFSET).len_utf_8e_128()
        );
        assert!(!s.is_empty());
        assert!(s.is_char_boundary(0));
        assert!(s.is_char_boundary(1));
        assert!(s.is_char_boundary(2));
        assert!(s.is_char_boundary(3));
        assert!(!s.is_char_boundary(4)); // interior byte of 0xFF sequence
        assert!(s.is_char_boundary(s.len()));

        // Slicing
        let sub = &s[0..2];
        assert_eq!(sub.as_bytes(), b"hi");
        assert_eq!(sub.as_str(), Some("hi"));
        assert_eq!(s.as_str(), None); // contains SHORT_DC_OFFSET (0xFF)

        // Iteration
        let chars: Vec<DcChar> = s.chars().collect();
        assert_eq!(
            chars,
            vec![
                DcChar::from_char('h'),
                DcChar::from_char('i'),
                DcChar::from_char(' '),
                DcChar(SHORT_DC_OFFSET)
            ]
        );

        // Reverse iteration
        let rev_chars: Vec<DcChar> = s.chars().rev().collect();
        assert_eq!(
            rev_chars,
            vec![
                DcChar(SHORT_DC_OFFSET),
                DcChar::from_char(' '),
                DcChar::from_char('i'),
                DcChar::from_char('h')
            ]
        );

        // Character indices
        let indices: Vec<(usize, DcChar)> = s.char_indices().collect();
        assert_eq!(
            indices,
            vec![
                (0, DcChar::from_char('h')),
                (1, DcChar::from_char('i')),
                (2, DcChar::from_char(' ')),
                (3, DcChar(SHORT_DC_OFFSET))
            ]
        );

        // Mutation: pop
        let popped = s.pop();
        assert_eq!(popped, Some(DcChar(SHORT_DC_OFFSET)));
        assert_eq!(s.as_str(), Some("hi "));

        // Mutation: truncate
        s.truncate(2);
        assert_eq!(s.as_str(), Some("hi"));

        // Mutation: retain
        s.retain(|c| c != DcChar::from_char('i'));
        assert_eq!(s.as_str(), Some("h"));
    }

    #[crate::ctb_test]
    fn test_validation_and_errors() {
        assert!(validate_dcutf(b"hello world").is_ok());

        let mut buf = Vec::new();
        buf.extend_from_slice(b"abc");
        let mut dc_buf = [0u8; 24];
        let n = encode_utf_8e_128_buf(&mut dc_buf, SHORT_DC_OFFSET + 100);
        buf.extend_from_slice(&dc_buf[..n]);
        assert!(validate_dcutf(&buf).is_ok());

        // Invalid continuation byte as start
        let err = validate_dcutf(b"abc\x80def").unwrap_err();
        assert_eq!(err.valid_up_to(), 3);
        assert_eq!(err.error_len(), Some(1));

        // Truncated extended sequence
        let err = validate_dcutf(b"\xFF\x84\x81").unwrap_err();
        assert_eq!(err.valid_up_to(), 0);
        assert_eq!(err.error_len(), None);

        // Safe conversion via from_bytes
        let dc_str = DcStr::from_bytes(b"hello").unwrap();
        assert_eq!(dc_str.as_str(), Some("hello"));

        let dc_string = DcString::from_dcutf(buf).unwrap();
        assert_eq!(dc_string.chars().count(), 4);
    }
}
