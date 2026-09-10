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

//! Utilities for Unicode, including:
//! - Character descriptions, annotations, aliases, and meanings
//! - Conversion of scalars to surrogates and vice versa
//! - UCS-2 encoding and decoding from scalars

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod character_description;
pub mod cli;
pub(crate) mod data;

pub use character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
    describe, describe_codepoint, describe_codepoint_with_options,
    describe_with_options,
};
pub use data::{
    UnicodeDataTables, find_block, find_block_with_version, get_tables,
    is_assigned_unicode,
};

// Re-export all Unicode scalar/surrogate and UCS-2 helpers inlined for rustdoc.
#[doc(inline)]
pub use ctb_utilities::circular_dep_unicode::*;

pub use ctb_formats_utilities::describe_general_category;
pub use icu_properties::props::GeneralCategory;

/// Returns the Unicode General Category for a character using compiled ICU4X data.
#[must_use]
pub fn general_category(c: char) -> GeneralCategory {
    icu_properties::CodePointMapData::<GeneralCategory>::new().get(c)
}

/// Returns true if the character belongs to General Category Cf (Format).
#[must_use]
pub fn is_format_char(c: char) -> bool {
    general_category(c) == GeneralCategory::Format
}

/// Returns true if the character is marked deprecated in the Unicode standard.
#[must_use]
pub fn is_deprecated_unicode(cp: u32) -> bool {
    icu_properties::CodePointSetData::new::<icu_properties::props::Deprecated>()
        .contains32(cp)
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
    fn test_format_char_detection() {
        // Cf characters
        assert!(is_format_char('\u{00AD}')); // Soft hyphen
        assert!(is_format_char('\u{200E}')); // LRM
        assert!(is_format_char('\u{200F}')); // RLM
        assert!(is_format_char('\u{061C}')); // ALM
        assert!(is_format_char('\u{200B}')); // ZWSP
        assert!(is_format_char('\u{202A}')); // LRE
        assert!(is_format_char('\u{2060}')); // Word joiner

        // Non-Cf characters
        assert!(!is_format_char('a'));
        assert!(!is_format_char(' '));
        assert!(!is_format_char('\n'));
        assert!(!is_format_char('1'));
        assert!(!is_format_char('\u{2028}')); // Line separator (Zl)
    }

    #[crate::ctb_test]
    fn test_general_category_lookup() {
        assert_eq!(general_category('A'), GeneralCategory::UppercaseLetter);
        assert_eq!(general_category('a'), GeneralCategory::LowercaseLetter);
        assert_eq!(general_category('1'), GeneralCategory::DecimalNumber);
        assert_eq!(general_category('\u{200E}'), GeneralCategory::Format);
        assert_eq!(general_category('\n'), GeneralCategory::Control);
    }

    #[crate::ctb_test]
    fn test_deprecated_unicode_detection() {
        // Deprecated characters in Unicode standard (from PropList.txt Deprecated property)
        assert!(is_deprecated_unicode(0x0149)); // U+0149 LATIN SMALL LETTER N PRECEDED BY APOSTROPHE
        assert!(is_deprecated_unicode(0x17A3)); // U+17A3 KHMER INDEPENDENT VOWEL QAQ
        assert!(is_deprecated_unicode(0x206A)); // U+206A INHIBIT SYMMETRIC SWAPPING
        assert!(is_deprecated_unicode(0x2329)); // U+2329 LEFT-POINTING ANGLE BRACKET

        // Non-deprecated characters
        assert!(!is_deprecated_unicode(0x0041)); // 'A'
        assert!(!is_deprecated_unicode(0x0020)); // Space
        assert!(!is_deprecated_unicode(0x2212)); // Minus
        assert!(!is_deprecated_unicode(0xFFFD)); // Replacement character
    }
}
