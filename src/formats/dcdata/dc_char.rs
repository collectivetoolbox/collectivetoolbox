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

use ctb_formats_utf_8e_128::{
    encode_dctf, encode_dctf_buf, len_dctf, len_utf_8e_128,
};
pub use ctb_utilities::dc_char::DcChar;
use crate::syntax::SyntaxRuleResolver;

/// Returns whether the given token (u128 character code) represents a type
/// token in Document Character syntax.
#[must_use]
pub fn is_type_token(token: u128) -> bool {
    crate::syntax::default_syntax_resolver()
        .token_fills_role(token, crate::syntax::DcRole::TypeToken)
}

/// Extension methods for [`DcChar`] specific to document formats and serialization.
pub trait DcCharExt {
    /// Returns whether this character represents a type token.
    #[must_use]
    fn is_type_token(self) -> bool;

    /// Returns whether this character fills the specified syntactic or semantic role
    /// using the default compiled syntax resolver.
    #[must_use]
    fn fills_role(self, role: crate::syntax::DcRole) -> bool;

    /// Returns whether this character fills the specified syntactic or semantic role
    /// according to the provided resolver.
    #[must_use]
    fn fills_role_with(
        self,
        role: crate::syntax::DcRole,
        resolver: &dyn crate::syntax::SyntaxRuleResolver,
    ) -> bool;

    /// Converts this character into a sequence of short Document Character IDs.
    #[must_use]
    fn to_short_vec(self) -> Vec<u32>;

    /// Returns the number of bytes required to encode this `DcChar` in
    /// `DcTF` (`.dctf`, `f641`) format (1..=23 bytes).
    #[must_use]
    fn len_dctf(self) -> usize;

    /// Returns the number of bytes required to encode this `DcChar` in legacy
    /// `UTF-8e-128` (`DcUtf`, `f89`) format (1..=24 bytes).
    #[must_use]
    fn len_utf_8e_128(self) -> usize;

    /// Encodes this `DcChar` in `DcTF` into the provided byte buffer (must be
    /// at least 23 bytes) and returns the number of bytes written.
    fn encode_buf(self, buf: &mut [u8]) -> usize;

    /// Encodes this `DcChar` in `DcTF` into a newly allocated `Vec<u8>`.
    #[must_use]
    fn encode(self) -> Vec<u8>;
}

impl DcCharExt for DcChar {
    fn is_type_token(self) -> bool {
        is_type_token(self.0)
    }
    fn fills_role(self, role: crate::syntax::DcRole) -> bool {
        crate::syntax::default_syntax_resolver().token_fills_role(self.0, role)
    }
    fn fills_role_with(
        self,
        role: crate::syntax::DcRole,
        resolver: &dyn crate::syntax::SyntaxRuleResolver,
    ) -> bool {
        resolver.token_fills_role(self.0, role)
    }
    fn to_short_vec(self) -> Vec<u32> {
        #[expect(
            clippy::expect_used,
            reason = "DC_ESCAPE and DC_LONG_DC are compile-time short Dc constants (Short Dc 255 and Short Dc 308)"
        )]
        let (escape, long_dc) = (
            crate::dc::DC_ESCAPE
                .to_short()
                .expect("DC_ESCAPE is Short Dc 255"),
            crate::dc::DC_LONG_DC
                .to_short()
                .expect("DC_LONG_DC is Short Dc 308"),
        );

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

    fn len_dctf(self) -> usize {
        len_dctf(self.0)
    }

    fn len_utf_8e_128(self) -> usize {
        len_utf_8e_128(self.0)
    }

    fn encode_buf(self, buf: &mut [u8]) -> usize {
        encode_dctf_buf(buf, self.0)
    }

    fn encode(self) -> Vec<u8> {
        encode_dctf(self.0)
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
    use ctb_utilities::dc_char::{FORMAT_REGION_START, SHORT_DC_REGION_START};

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

    #[crate::ctb_test]
    fn test_is_type_token() {
        assert!(is_type_token(DcChar::from_format(490).to_u128()));
        assert!(is_type_token(DcChar::from_format(0).to_u128()));
        assert!(DcChar::from_format(80).is_type_token());

        assert!(is_type_token(DcChar::from_short(275).to_u128()));
        assert!(is_type_token(DcChar::from_short(264).to_u128()));
        assert!(is_type_token(DcChar::from_short(278).to_u128()));
        assert!(is_type_token(DcChar::from_short(280).to_u128()));
        assert!(is_type_token(DcChar::from_short(306).to_u128()));
        assert!(is_type_token(DcChar::from_short(513).to_u128()));
        assert!(is_type_token(DcChar::from_short(300).to_u128()));
        assert!(is_type_token(DcChar::from_short(516).to_u128()));

        // EL Routines and Miscellaneous are NOT type tokens
        assert!(!is_type_token(DcChar::from_short(265).to_u128()));
        assert!(!is_type_token(DcChar::from_short(266).to_u128()));
        assert!(!is_type_token(DcChar::from_short(267).to_u128()));

        assert!(!is_type_token(DcChar::from_short(42).to_u128()));
        assert!(!is_type_token(DcChar::from_char('A').to_u128()));
        assert!(!DcChar::from_short(310).is_type_token());
    }

    #[crate::ctb_test]
    fn test_composite_role_canonical_matches() {
        use crate::syntax::DcRole;

        // TypeToken: .EL Types and format IDs match, routines do not
        assert!(DcRole::TypeToken.matches_canonical(DcChar::from_short(278).to_u128()));
        assert!(DcRole::TypeToken.matches_canonical(DcChar::from_short(280).to_u128()));
        assert!(DcRole::TypeToken.matches_canonical(DcChar::from_short(306).to_u128()));
        assert!(!DcRole::TypeToken.matches_canonical(DcChar::from_short(265).to_u128()));
        assert!(!DcRole::TypeToken.matches_canonical(DcChar::from_short(266).to_u128()));
        assert!(!DcRole::TypeToken.matches_canonical(DcChar::from_short(267).to_u128()));

        // IntegerFormatIndicator: Format 199 matches, short Dc 8 does not
        assert!(DcRole::IntegerFormatIndicator.matches_canonical(DcChar::from_format(199).to_u128()));
        assert!(!DcRole::IntegerFormatIndicator.matches_canonical(DcChar::from_short(8).to_u128()));

        // Base64Digit: 127..=190
        assert!(DcRole::Base64Digit.matches_canonical(DcChar::from_short(127).to_u128()));
        assert!(DcRole::Base64Digit.matches_canonical(DcChar::from_short(190).to_u128()));
        assert!(!DcRole::Base64Digit.matches_canonical(DcChar::from_short(191).to_u128()));
        assert!(!DcRole::Base64Digit.matches_canonical(DcChar::from_short(195).to_u128()));

        // DomainValueAtom: stream kinds, finder labels, file path delimiter, booleans, optional absent
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(379).to_u128()));
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(384).to_u128()));
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(482).to_u128()));
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(489).to_u128()));
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(513).to_u128()));
        assert!(DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(316).to_u128()));
        assert!(!DcRole::DomainValueAtom.matches_canonical(DcChar::from_short(405).to_u128()));

        // EvaluableExpressionStarter: .EL Routines and reference/invocation starters
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(256).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(265).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(276).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(279).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(540).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(397).to_u128()));
        assert!(DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(561).to_u128()));
        assert!(!DcRole::EvaluableExpressionStarter.matches_canonical(DcChar::from_short(6).to_u128()));

        // StructuralSeparator: newlines, paragraphs, list element separator
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(119).to_u128()));
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(120).to_u128()));
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(121).to_u128()));
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(294).to_u128()));
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(295).to_u128()));
        assert!(DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(312).to_u128()));
        assert!(!DcRole::StructuralSeparator.matches_canonical(DcChar::from_short(239).to_u128()));
    }
}
