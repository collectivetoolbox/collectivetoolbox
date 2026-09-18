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

use std::collections::HashSet;
use std::sync::LazyLock;

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

pub use icu_properties::props::GeneralCategory;

/// Expand a Unicode general category abbreviation into a human-readable description.
#[must_use]
pub fn describe_general_category(type_code: &str) -> String {
    match type_code {
        "Cc" => "Control".to_string(),
        "Cf" => "Format".to_string(),
        "Cs" => "Surrogate".to_string(),
        "Co" => "Private Use".to_string(),
        "Cn" => "Noncharacter".to_string(),
        "Lu" => "Letter: Uppercase".to_string(),
        "Ll" => "Letter: Lowercase".to_string(),
        "Lt" => "Letter: Titlecase".to_string(),
        "Lm" => "Letter: Modifier".to_string(),
        "Lo" => "Letter: Other".to_string(),
        "Mn" => "Mark: Nonspacing".to_string(),
        "Mc" => "Mark: Spacing Combining".to_string(),
        "Me" => "Mark: Enclosing".to_string(),
        "Nd" => "Number: Decimal".to_string(),
        "Nl" => "Number: Letter".to_string(),
        "No" => "Number: Other".to_string(),
        "Pc" => "Punctuation: Connector".to_string(),
        "Pd" => "Punctuation: Dash".to_string(),
        "Ps" => "Punctuation: Open".to_string(),
        "Pe" => "Punctuation: Close".to_string(),
        "Pi" => "Punctuation: Initial Quote".to_string(),
        "Pf" => "Punctuation: Final Quote".to_string(),
        "Po" => "Punctuation: Other".to_string(),
        "Sm" => "Symbol: Math".to_string(),
        "Sc" => "Symbol: Currency".to_string(),
        "Sk" => "Symbol: Modifier".to_string(),
        "So" => "Symbol: Other".to_string(),
        "Zs" => "Separator: Space".to_string(),
        "Zl" => "Separator: Line".to_string(),
        "Zp" => "Separator: Paragraph".to_string(),
        other => other.to_string(),
    }
}

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

/// Returns the official canonical Unicode character name for a code point
/// for a specific Unicode version, including standard names, control character
/// names, and algorithmic names.
#[must_use]
pub fn get_unicode_name_for_version(
    cp: u32,
    version: UnicodeVersion,
) -> Option<String> {
    let tables = data::get_tables(version);
    character_description::get_unicode_character_name(
        tables,
        cp,
        DescriptionMode::Standard,
    )
}

/// Returns the official canonical Unicode character name for a code point
/// in the current supported Unicode standard, including standard names, control
/// character names, and algorithmic names.
#[must_use]
pub fn get_unicode_name(cp: u32) -> Option<String> {
    get_unicode_name_for_version(cp, UnicodeVersion::V17_0)
}

pub use icu_properties::props::{BidiClass, CanonicalCombiningClass};

/// Returns the 2-letter general category code for a code point (e.g. "Lu", "Cc").
#[must_use]
pub fn general_category_code(cp: u32) -> &'static str {
    match icu_properties::CodePointMapData::<GeneralCategory>::new().get32(cp) {
        GeneralCategory::UppercaseLetter => "Lu",
        GeneralCategory::LowercaseLetter => "Ll",
        GeneralCategory::TitlecaseLetter => "Lt",
        GeneralCategory::ModifierLetter => "Lm",
        GeneralCategory::OtherLetter => "Lo",
        GeneralCategory::NonspacingMark => "Mn",
        GeneralCategory::SpacingMark => "Mc",
        GeneralCategory::EnclosingMark => "Me",
        GeneralCategory::DecimalNumber => "Nd",
        GeneralCategory::LetterNumber => "Nl",
        GeneralCategory::OtherNumber => "No",
        GeneralCategory::SpaceSeparator => "Zs",
        GeneralCategory::LineSeparator => "Zl",
        GeneralCategory::ParagraphSeparator => "Zp",
        GeneralCategory::Control => "Cc",
        GeneralCategory::Format => "Cf",
        GeneralCategory::PrivateUse => "Co",
        GeneralCategory::Surrogate => "Cs",
        GeneralCategory::DashPunctuation => "Pd",
        GeneralCategory::OpenPunctuation => "Ps",
        GeneralCategory::ClosePunctuation => "Pe",
        GeneralCategory::ConnectorPunctuation => "Pc",
        GeneralCategory::InitialPunctuation => "Pi",
        GeneralCategory::FinalPunctuation => "Pf",
        GeneralCategory::OtherPunctuation => "Po",
        GeneralCategory::MathSymbol => "Sm",
        GeneralCategory::CurrencySymbol => "Sc",
        GeneralCategory::ModifierSymbol => "Sk",
        GeneralCategory::OtherSymbol => "So",
        GeneralCategory::Unassigned => "Cn",
    }
}

/// Returns the standard Bidi class abbreviation for a code point (e.g. "L", "R", "BN").
#[must_use]
pub fn bidi_class_code(cp: u32) -> &'static str {
    match icu_properties::CodePointMapData::<BidiClass>::new().get32(cp) {
        BidiClass::LeftToRight => "L",
        BidiClass::RightToLeft => "R",
        BidiClass::EuropeanNumber => "EN",
        BidiClass::EuropeanSeparator => "ES",
        BidiClass::EuropeanTerminator => "ET",
        BidiClass::ArabicNumber => "AN",
        BidiClass::CommonSeparator => "CS",
        BidiClass::ParagraphSeparator => "B",
        BidiClass::SegmentSeparator => "S",
        BidiClass::WhiteSpace => "WS",
        BidiClass::OtherNeutral => "ON",
        BidiClass::LeftToRightEmbedding => "LRE",
        BidiClass::LeftToRightOverride => "LRO",
        BidiClass::ArabicLetter => "AL",
        BidiClass::RightToLeftEmbedding => "RLE",
        BidiClass::RightToLeftOverride => "RLO",
        BidiClass::PopDirectionalFormat => "PDF",
        BidiClass::NonspacingMark => "NSM",
        BidiClass::BoundaryNeutral => "BN",
        BidiClass::FirstStrongIsolate => "FSI",
        BidiClass::LeftToRightIsolate => "LRI",
        BidiClass::RightToLeftIsolate => "RLI",
        BidiClass::PopDirectionalIsolate => "PDI",
        _ => "ON",
    }
}

/// Returns the canonical combining class for a code point (0..=254).
#[must_use]
pub fn combining_class(cp: u32) -> u8 {
    icu_properties::CodePointMapData::<CanonicalCombiningClass>::new()
        .get32(cp)
        .to_icu4c_value()
}

/// Character metadata record for generating the unified Unicode character table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeCharRecord {
    pub cp: u32,
    pub name: String,
    pub combining_class: u8,
    pub bidi_class: &'static str,
    pub general_category: &'static str,
    pub script: String,
    pub aliases: String,
    pub description: String,
    pub is_deprecated: bool,
}

static ALL_UNICODE_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let tables = data::get_tables(UnicodeVersion::V17_0);
    let mut names = HashSet::new();

    for cp in 0..=0x10_FFFF {
        if let Some(n) = character_description::get_unicode_character_name(
            tables,
            cp,
            DescriptionMode::Standard,
        ) {
            names.insert(n.to_lowercase());
        }
        if let Some(entry) = tables.name_aliases.get(&cp) {
            for c in &entry.corrections {
                names.insert(c.to_lowercase());
            }
            for c in &entry.controls {
                names.insert(c.to_lowercase());
            }
            for a in &entry.alternates {
                names.insert(a.to_lowercase());
            }
            for f in &entry.figments {
                names.insert(f.to_lowercase());
            }
            for ab in &entry.abbreviations {
                names.insert(ab.to_lowercase());
            }
        }
    }
    names
});

/// Returns true if a given name matches any canonical or aliased Unicode character name.
#[must_use]
pub fn is_known_unicode_name(name: &str) -> bool {
    ALL_UNICODE_NAMES.contains(&name.trim().to_lowercase())
}

/// Iterates over all assigned Unicode characters and produces structured records.
pub fn get_assigned_unicode_records() -> Vec<UnicodeCharRecord> {
    let tables = data::get_tables(UnicodeVersion::V17_0);
    let mut records = Vec::new();

    let gc_map = icu_properties::CodePointMapData::<GeneralCategory>::new();
    let bidi_map = icu_properties::CodePointMapData::<BidiClass>::new();
    let ccc_map =
        icu_properties::CodePointMapData::<CanonicalCombiningClass>::new();

    // Assert that compiled ICU Unicode properties match the inlined Unicode 17.0 tables
    let sample_17_cp = 0x088F; // ARABIC LETTER NOON WITH RING ABOVE, added in Unicode 17.0
    assert_ne!(
        gc_map.get32(sample_17_cp),
        GeneralCategory::Unassigned,
        "Compiled ICU Unicode properties must match inlined Unicode 17.0 tables"
    );

    for cp in 0..=0x10_FFFF {
        if !tables.is_assigned(cp) {
            continue;
        }

        let name = if let Some(n) =
            character_description::get_unicode_character_name(
                tables,
                cp,
                DescriptionMode::Standard,
            )
        {
            n
        } else if (0xE000..=0xF8FF).contains(&cp)
            || (0xF0000..=0xFFFFD).contains(&cp)
            || (0x100000..=0x10FFFD).contains(&cp)
        {
            format!("<private-use-{cp:04X}>")
        } else {
            continue;
        };

        let gc = match gc_map.get32(cp) {
            GeneralCategory::UppercaseLetter => "Lu",
            GeneralCategory::LowercaseLetter => "Ll",
            GeneralCategory::TitlecaseLetter => "Lt",
            GeneralCategory::ModifierLetter => "Lm",
            GeneralCategory::OtherLetter => "Lo",
            GeneralCategory::NonspacingMark => "Mn",
            GeneralCategory::SpacingMark => "Mc",
            GeneralCategory::EnclosingMark => "Me",
            GeneralCategory::DecimalNumber => "Nd",
            GeneralCategory::LetterNumber => "Nl",
            GeneralCategory::OtherNumber => "No",
            GeneralCategory::SpaceSeparator => "Zs",
            GeneralCategory::LineSeparator => "Zl",
            GeneralCategory::ParagraphSeparator => "Zp",
            GeneralCategory::Control => "Cc",
            GeneralCategory::Format => "Cf",
            GeneralCategory::PrivateUse => "Co",
            GeneralCategory::Surrogate => "Cs",
            GeneralCategory::DashPunctuation => "Pd",
            GeneralCategory::OpenPunctuation => "Ps",
            GeneralCategory::ClosePunctuation => "Pe",
            GeneralCategory::ConnectorPunctuation => "Pc",
            GeneralCategory::InitialPunctuation => "Pi",
            GeneralCategory::FinalPunctuation => "Pf",
            GeneralCategory::OtherPunctuation => "Po",
            GeneralCategory::MathSymbol => "Sm",
            GeneralCategory::CurrencySymbol => "Sc",
            GeneralCategory::ModifierSymbol => "Sk",
            GeneralCategory::OtherSymbol => "So",
            GeneralCategory::Unassigned => "Cn",
        };

        let bidi = match bidi_map.get32(cp) {
            BidiClass::LeftToRight => "L",
            BidiClass::RightToLeft => "R",
            BidiClass::EuropeanNumber => "EN",
            BidiClass::EuropeanSeparator => "ES",
            BidiClass::EuropeanTerminator => "ET",
            BidiClass::ArabicNumber => "AN",
            BidiClass::CommonSeparator => "CS",
            BidiClass::ParagraphSeparator => "B",
            BidiClass::SegmentSeparator => "S",
            BidiClass::WhiteSpace => "WS",
            BidiClass::OtherNeutral => "ON",
            BidiClass::LeftToRightEmbedding => "LRE",
            BidiClass::LeftToRightOverride => "LRO",
            BidiClass::ArabicLetter => "AL",
            BidiClass::RightToLeftEmbedding => "RLE",
            BidiClass::RightToLeftOverride => "RLO",
            BidiClass::PopDirectionalFormat => "PDF",
            BidiClass::NonspacingMark => "NSM",
            BidiClass::BoundaryNeutral => "BN",
            BidiClass::FirstStrongIsolate => "FSI",
            BidiClass::LeftToRightIsolate => "LRI",
            BidiClass::RightToLeftIsolate => "RLI",
            BidiClass::PopDirectionalIsolate => "PDI",
            _ => "ON",
        };

        let combining = ccc_map.get32(cp).to_icu4c_value();

        let script = data::find_block(cp)
            .unwrap_or("Unicode")
            .to_string();

        let mut alias_parts = Vec::new();
        if let Some(entry) = tables.name_aliases.get(&cp) {
            if let Some(ref c) = entry.correction {
                alias_parts.push(c.clone());
            }
            if let Some(ref a) = entry.abbreviation {
                alias_parts.push(a.clone());
            }
            if let Some(ref alt) = entry.alternate {
                alias_parts.push(alt.clone());
            }
        }
        let aliases = alias_parts.join(", ");

        let is_deprecated = is_deprecated_unicode(cp);

        records.push(UnicodeCharRecord {
            cp,
            name,
            combining_class: combining,
            bidi_class: bidi,
            general_category: gc,
            script,
            aliases,
            description: String::new(),
            is_deprecated,
        });
    }

    records
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

    #[crate::ctb_test]
    fn test_get_unicode_name() {
        assert_eq!(
            get_unicode_name(0x0000).as_deref(),
            Some("<control> NULL")
        );
        assert_eq!(
            get_unicode_name(0x000A).as_deref(),
            Some("<control, Unicode-semantic-defined> LINE FEED")
        );
        assert_eq!(
            get_unicode_name(0x000B).as_deref(),
            Some("<control, Unicode-semantic-defined> LINE TABULATION")
        );
        assert_eq!(
            get_unicode_name(0x000D).as_deref(),
            Some("<control, Unicode-semantic-defined> CARRIAGE RETURN")
        );
        assert_eq!(get_unicode_name(0x002D).as_deref(), Some("HYPHEN-MINUS"));
        assert_eq!(get_unicode_name(0x002E).as_deref(), Some("FULL STOP"));
        assert_eq!(
            get_unicode_name(0x0085).as_deref(),
            Some("<control, Unicode-semantic-defined> NEXT LINE")
        );
        assert_eq!(
            get_unicode_name(0x0041).as_deref(),
            Some("LATIN CAPITAL LETTER A")
        );
        assert_eq!(get_unicode_name(0xAC00).as_deref(), Some("HANGUL SYLLABLE GA"));
        assert_eq!(
            get_unicode_name(0x4E00).as_deref(),
            Some("CJK UNIFIED IDEOGRAPH-4E00")
        );
        assert_eq!(
            get_unicode_name(0x13460).as_deref(),
            Some("EGYPTIAN HIEROGLYPH A001F")
        );
        assert_eq!(get_unicode_name(0x11_0000), None);

        // Version-specific checks
        assert_eq!(
            get_unicode_name_for_version(0x0041, UnicodeVersion::V15_0)
                .as_deref(),
            Some("LATIN CAPITAL LETTER A")
        );
    }

    #[crate::ctb_test]
    fn test_icu_unicode_version_matches_tables() {
        let gc_map = icu_properties::CodePointMapData::<GeneralCategory>::new();
        assert_ne!(gc_map.get32(0x088F), GeneralCategory::Unassigned);
        assert_ne!(gc_map.get32(0x0C5C), GeneralCategory::Unassigned);
        assert_ne!(gc_map.get32(0x0CDC), GeneralCategory::Unassigned);
    }
}
