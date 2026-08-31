// SPDX-License-Identifier: AGPL-3.0-or-later AND Unicode-3.0
// SPDX-License-Identifier for parts derived from Unicode data: Unicode-3.0
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

// See additional licensing details at end of file.

//! Independent reimplementation of Andrew West's "What Unicode Character is This" tool.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub use crate::data::UnicodeVersion;
use crate::data::{
    KHITAN_DATA, UNIHAN_DATA, find_block_with_version, get_tables,
    is_noncharacter,
};

const HANGUL_S_BASE: u32 = 0xAC00;
const HANGUL_T_COUNT: u32 = 28;
const HANGUL_N_COUNT: u32 = 588; // 21 * 28
const HANGUL_S_COUNT: u32 = 11172; // 19 * 588

const JAMO_L_TABLE: [&str; 19] = [
    "G", "GG", "N", "D", "DD", "R", "M", "B", "BB", "S", "SS", "", "J", "JJ",
    "C", "K", "T", "P", "H",
];
const JAMO_V_TABLE: [&str; 21] = [
    "A", "AE", "YA", "YAE", "EO", "E", "YEO", "YE", "O", "WA", "WAE", "OE",
    "YO", "U", "WEO", "WE", "WI", "YU", "EU", "YI", "I",
];
const JAMO_T_TABLE: [&str; 28] = [
    "", "G", "GG", "GS", "N", "NJ", "NH", "D", "L", "LG", "LM", "LB", "LS",
    "LT", "LP", "LH", "M", "B", "BS", "S", "SS", "NG", "J", "C", "K", "T", "P",
    "H",
];

/// Computes the algorithmic Hangul syllable name for a code point if applicable.
fn hangul_syllable_name(cp: u32) -> Option<String> {
    if (HANGUL_S_BASE..HANGUL_S_BASE.saturating_add(HANGUL_S_COUNT))
        .contains(&cp)
    {
        let s_index = cp.saturating_sub(HANGUL_S_BASE);
        let l_index =
            usize::try_from(s_index.checked_div(HANGUL_N_COUNT)?).ok()?;
        let v_index = usize::try_from(
            (s_index.checked_rem(HANGUL_N_COUNT)?)
                .checked_div(HANGUL_T_COUNT)?,
        )
        .ok()?;
        let t_index =
            usize::try_from(s_index.checked_rem(HANGUL_T_COUNT)?).ok()?;
        let l = JAMO_L_TABLE.get(l_index)?;
        let v = JAMO_V_TABLE.get(v_index)?;
        let t = JAMO_T_TABLE.get(t_index)?;
        Some(format!("HANGUL SYLLABLE {l}{v}{t}"))
    } else {
        None
    }
}

/// Checks if a codepoint is in a Private Use Area.
fn is_private_use(cp: u32) -> bool {
    (0xE000..=0xF8FF).contains(&cp)
        || (0xF0000..=0xFFFFD).contains(&cp)
        || (0x100000..=0x10FFFD).contains(&cp)
}

/// Description formatting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DescriptionMode {
    /// Standard enhanced format: concise reserved/PUA/surrogates, multi-alias annotations, Unihan readings.
    #[default]
    Standard,
    /// Exact WUC compatibility format (matches `unicode_untrimmed_descriptions.txt`).
    WucCompat,
}

/// Format for control character names and abbreviations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlNameFormat {
    /// Official UCD `NameAliases` (e.g. ALERT [BEL], END OF LINE [EOL]).
    #[default]
    NameAliases,
    /// Legacy `NamesList` names (e.g. BELL [BEL], LINE FEED [LF], [EOM]).
    NamesList,
    /// "What Unicode Character is This" format
    Wuc,
}

/// Options controlling character description generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DescriptionOptions {
    pub mode: DescriptionMode,
    pub control_name_format: ControlNameFormat,
    pub unicode_version: UnicodeVersion,
    pub include_unihan_readings: bool,
}

impl Default for DescriptionOptions {
    fn default() -> Self {
        Self {
            mode: DescriptionMode::Standard,
            control_name_format: ControlNameFormat::NameAliases,
            unicode_version: UnicodeVersion::V17_0,
            include_unihan_readings: true,
        }
    }
}

impl DescriptionOptions {
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            mode: DescriptionMode::Standard,
            control_name_format: ControlNameFormat::NameAliases,
            unicode_version: UnicodeVersion::V17_0,
            include_unihan_readings: true,
        }
    }

    #[must_use]
    pub const fn wuc_compat() -> Self {
        Self {
            mode: DescriptionMode::WucCompat,
            control_name_format: ControlNameFormat::Wuc,
            unicode_version: UnicodeVersion::V16_0,
            include_unihan_readings: false,
        }
    }
}

/// Formats a single Unicode code point into a detailed description string using default options.
pub fn describe_codepoint(cp: u32) -> String {
    describe_codepoint_with_options(cp, DescriptionOptions::default())
}

/// Formats a single Unicode code point with the given options.
pub fn describe_codepoint_with_options(
    cp: u32,
    options: DescriptionOptions,
) -> String {
    // 1. High and Low Surrogates
    if (0xD800..=0xDBFF).contains(&cp) {
        if options.mode == DescriptionMode::WucCompat {
            return format!("U+{cp:04X} : <surrogate>");
        }
        return format!("U+{cp:04X} : <surrogate> (high surrogate)");
    }
    if (0xDC00..=0xDFFF).contains(&cp) {
        if options.mode == DescriptionMode::WucCompat {
            return format!("U+{cp:04X} : <surrogate>");
        }
        return format!("U+{cp:04X} : <surrogate> (low surrogate)");
    }

    // 2. Noncharacter Code Points
    if is_noncharacter(cp) {
        return format!("U+{cp:04X} : <noncharacter>");
    }

    // 3. Private Use Area
    if is_private_use(cp) {
        if options.mode == DescriptionMode::WucCompat {
            return format!("U+{cp:04X} : <private-use>");
        }
        if let Some(block) =
            find_block_with_version(options.unicode_version, cp)
        {
            return format!("U+{cp:04X} : <private-use> ({block} block)");
        }
        return format!("U+{cp:04X} : <private-use>");
    }

    let tables = get_tables(options.unicode_version);

    // In WUC compatibility mode, characters newly assigned in Unicode 17.0+ (and late 16.0 additions) were reserved
    if options.mode == DescriptionMode::WucCompat
        && (get_tables(UnicodeVersion::V17_0).is_v17_or_later(cp)
            || matches!(cp, 0x1F8B2..=0x1F8BB | 0x1F8C0..=0x1F8C1 | 0x2B739))
    {
        return format!("U+{cp:04X} : <reserved>");
    }

    let char_info = tables.char_data.get(&cp);
    let alias_entry = tables.name_aliases.get(&cp);

    // 4. Control Characters
    let is_ctrl = char_info.is_some_and(|info| info.is_control)
        || (0x0000..=0x001F).contains(&cp)
        || (0x007F..=0x009F).contains(&cp);

    if is_ctrl {
        let (control_name, abbr) = match options.control_name_format {
            ControlNameFormat::Wuc => {
                let name = char_info
                    .and_then(|info| info.nameslist_control_name.as_deref());
                let ab =
                    if matches!(cp, 0x000B | 0x001C | 0x001D | 0x001E | 0x001F)
                    {
                        None
                    } else if cp == 0x0009 {
                        Some("TAB")
                    } else if cp == 0x0019 {
                        Some("EOM")
                    } else {
                        char_info
                            .and_then(|info| {
                                info.nameslist_control_abbr.as_deref()
                            })
                            .or_else(|| {
                                alias_entry
                                    .and_then(|a| a.abbreviation.as_deref())
                            })
                    };
                (name, ab)
            }
            ControlNameFormat::NamesList => {
                let name = char_info
                    .and_then(|info| info.nameslist_control_name.as_deref());
                let ab = char_info
                    .and_then(|info| info.nameslist_control_abbr.as_deref())
                    .or_else(|| {
                        alias_entry.and_then(|a| a.abbreviation.as_deref())
                    });
                (name, ab)
            }
            ControlNameFormat::NameAliases => {
                let name = alias_entry.and_then(|a| a.control.as_deref());
                let ab = alias_entry.and_then(|a| a.abbreviation.as_deref());
                (name, ab)
            }
        };

        let mut res = match (control_name, abbr) {
            (Some(name), Some(ab)) => {
                format!("U+{cp:04X} : <control> {name} [{ab}]")
            }
            (Some(name), None) => format!("U+{cp:04X} : <control> {name}"),
            (None, Some(ab)) => {
                if options.mode == DescriptionMode::WucCompat {
                    format!("U+{cp:04X} : <control>")
                } else {
                    format!("U+{cp:04X} : <control> [{ab}]")
                }
            }
            (None, None) => format!("U+{cp:04X} : <control>"),
        };

        if let Some(info) = char_info {
            if !info.informative_aliases.is_empty() {
                let joined = info.informative_aliases.join("; ");
                res.push_str(&format!(" {{{joined}}}"));
            }
        }
        return res;
    }

    // 5. Named Unicode Characters & Algorithmic Ranges
    let mut primary_name: Option<String> = None;

    if let Some(hangul) = hangul_syllable_name(cp) {
        primary_name = Some(hangul);
    } else if tables.is_cjk_unified_ideograph(cp) {
        if options.mode == DescriptionMode::WucCompat
            && ((0xF900..=0xFAFF).contains(&cp)
                || (0x2F800..=0x2FA1F).contains(&cp))
        {
            primary_name =
                Some(format!("CJK COMPATIBILITY IDEOGRAPH-{cp:04X}"));
        } else {
            primary_name = Some(format!("CJK UNIFIED IDEOGRAPH-{cp:04X}"));
        }
    } else if tables.is_tangut_ideograph(cp) {
        primary_name = Some(format!("TANGUT IDEOGRAPH-{cp:04X}"));
    } else if tables.is_khitan_character(cp) {
        primary_name = Some(format!("KHITAN SMALL SCRIPT CHARACTER-{cp:04X}"));
    } else if (0x13460..=0x143FA).contains(&cp) {
        if options.mode == DescriptionMode::Standard {
            if let Some(entry) = tables.unikemet_data.get(&cp) {
                if let Some(ref unik) = entry.unik_code {
                    primary_name = Some(format!("EGYPTIAN HIEROGLYPH {unik}"));
                }
            }
        }
        if primary_name.is_none() {
            if let Some(info) = char_info {
                if !info.name.is_empty() && !info.name.starts_with('<') {
                    primary_name = Some(info.name.clone());
                }
            }
        }
    } else if let Some(info) = char_info {
        if !info.name.is_empty() && !info.name.starts_with('<') {
            primary_name = Some(info.name.clone());
        }
    }

    // 6. Unassigned / Reserved Code Points
    let Some(mut name) = primary_name else {
        if options.mode == DescriptionMode::WucCompat {
            return format!("U+{cp:04X} : <reserved>");
        }
        if let Some(block) =
            find_block_with_version(options.unicode_version, cp)
        {
            return format!("U+{cp:04X} : <reserved> ({block} block)");
        }
        return format!("U+{cp:04X} : <reserved> (unassigned region)");
    };

    if options.mode == DescriptionMode::WucCompat {
        // Name adjustments (omitted hyphens in Arabic/Uyghur/Cypro-Minoan/Minnan/Znamenny/X-Ray names)
        if matches!(
            cp,
            0x0898
                | 0x10F72
                | 0x12F90..=0x12FF2
                | 0x1AFF0..=0x1AFF3
                | 0x1AFF5..=0x1AFFB
                | 0x1AFFD..=0x1AFFE
                | 0x1CF42..=0x1CF43
                | 0x1FA7B
                | 0xFD46..=0xFD4E
        ) {
            name = name.replace('-', "");
        }
        // NamesList leading-space quirk in Devanagari, Kawi, Kaktovik, Cyrillic Ext-D, Nag Mundari
        if matches!(
            cp,
            0x11B00..=0x11B09
                | 0x11F00..=0x11F59
                | 0x1D2C0..=0x1D2D3
                | 0x1E030..=0x1E06D
                | 0x1E08F
                | 0x1E4D0..=0x1E4F9
        ) {
            name = format!(" {name}");
        }
        // Typo in Tamil sign - this was corrected to UZHAKKU very shortly before the Unicode 12 release in which it was added.
        if cp == 0x11FD8 {
            name = "TAMIL SIGN UZHAAKKU".to_string();
        }
    }

    let mut result = format!("U+{cp:04X} : {name}");

    // 7. Graphic Character Abbreviation from NameAliases.txt
    if let Some(abbr) = alias_entry.and_then(|a| a.abbreviation.as_deref()) {
        if !name.ends_with(&format!("[{abbr}]")) {
            result.push_str(&format!(" [{abbr}]"));
        }
    }

    // 8. Formal Correction Alias
    if options.mode == DescriptionMode::WucCompat {
        if cp == 0xFEFF {
            result.push_str(" (alias BYTE ORDER MARK [BOM])");
        } else if !matches!(
            cp,
            0x0616 | 0x1BBD | 0x12327 | 0x1680B | 0x1E899 | 0x1E89A
        ) {
            if let Some(alias) =
                alias_entry.and_then(|a| a.correction.as_deref())
            {
                result.push_str(&format!(" (alias {alias})"));
            }
        }
    } else if let Some(alias) =
        alias_entry.and_then(|a| a.correction.as_deref())
    {
        result.push_str(&format!(" (alias {alias})"));
    }

    // 9. Tangut Source Reference
    if let Some(src) = tables
        .tangut_data
        .get(&cp)
        .and_then(|t| t.source_ref.as_deref())
    {
        result.push_str(&format!(" ({src})"));
    }

    // 10. Standardized Variation Sequence (CJK Compatibility Ideographs)
    if let Some(variant) = tables.standardized_variants.get(&cp) {
        result.push_str(&format!(" {variant}"));
    }

    // 11. Informative Annotations & Meanings
    if options.mode == DescriptionMode::WucCompat {
        match cp {
            0x002C => result.push_str(" {decimal separator}"),
            0x0040 => result.push_str(" {at sign}"),
            0x0060 | 0x0195 | 0x025C | 0x2C6D | 0xA729 | 0xA7FB..=0xA7FD | 0x12326 | 0x187F0..=0x187F6 => {}
            0x018E => result.push_str(" {turned e}"),
            0x0190 => result.push_str(" {epsilon}"),
            0x01C2 => result.push_str(" {double-barred pipe; palatoalveolar click (IPA)}"),
            0x01C3 => result.push_str(" {(post)alveolar click (IPA)}"),
            0x0292 => result.push_str(" {dram}"),
            0x13158 => result.push_str(" {A saharan helmeted guinea fowl (Numida meleagris), with a lappet}"),
            0x131D8 => result.push_str(" {A forearm, with the palm of the hand facing upwards (D36), written over a desert plant, with four branches, with flowers on every branch, on a horizontal base (M26)}"),
            0x132D6 => result.push_str(" {The union of the red crown of Lower Egypt and the white crown of Upper Egypt, with the white crown worn within the red crown (i.e., the double crown)}"),
            0x132D7 => result.push_str(" {The union of the red crown of Lower Egypt and the white crown of Upper Egypt, with the white crown worn within the red crown (i.e., the double crown, S5), on top of a wickerwork basket (V30)}"),
            0x134BB => result.push_str(" {Man, naked, written horizontally, facing upwards, with the head looking downwards, with knees and hips bend, lower arm bend beside the body, elbow downwards, hand upwards, facing outwards, upper arm hanging loosely beside the body}"),
            0x13509 => result.push_str(" {Man, standing, both arms towards the front, holding an key for a tumbler lock consisting of a slightly bend vertical line with two shorter horizontal lines attached to the top akin to a flag, pointing outwards, as if to strike with it}"),
            0x135A5 => result.push_str(" {Man, seated on heel, right knee raised, with a feather on the head (angled backwards), right arm raised, left arm in front of body}"),
            0x135AD => result.push_str(" {Man, seated, both knees down, right arm forward, forearm horizontal, holding a bow or shield, left arm in front of the body, holding multiple sticks or arrows, angling backwards, leaning against the left shoulder}"),
            0x135DC => result.push_str(" {3 men, kneeling/walking on knees, gound together around the neck, first man, right knee before left knee, both arms raised in front, handpalms outward, second man, right knee just in front of left knee, both arms behind the back, as if bound, third man, right knee and foot in front of the left knee, left lower leg raised at 45°, right arm in front, hand against the head of the second man, left arm behind the back, hand visible in front of the waist}"),
            0x13977 => result.push_str(" {An eye with a painted lower lid, and two lines at the top forming a V shape}"),
            0x13CF1 => result.push_str(" {A saharan helmeted guinea fowl (Numida meleagris), without a lappet}"),
            0x13CF2 => result.push_str(" {A saharan helmeted guinea fowl (Numida meleagris), with a lappet, without the two protruding feathers on its head}"),
            0x13CF4 => result.push_str(" {A saharan helmeted guinea fowl (Numida meleagris), without a lappet, without the two protruding feathers on its head}"),
            0x13E06 => result.push_str(" {A tree without foliage, or a branch with many side branches}"),
            0x13E9B => result.push_str(" {A forearm, with the palm of the hand facing upwards (D36), written over a desert plant with a flower, without branches, }"),
            0x13FEB => result.push_str(" {A column resembling a stem of papyrus with a bud (M13), on a base, with a }"),
            0x140C6 => result.push_str(" { A brazier resembling a wide cup (W10), with a flame rising from it, flame angling towards the front}"),
            0x140C7 => result.push_str(" { A brazier with a flame rising from it, flame angling towards the front, represented as dots}"),
            0x140C8 => result.push_str(" { A brazier with a flame rising from it, flame curving forwards}"),
            0x1415B => result.push_str(" {The union of the red crown of Lower Egypt and the white crown of Upper Egypt, with the white crown worn within the red crown (i.e., the double crown), with an Uraeus at the front}"),
            0x14343 => result.push_str(" { A wickerwork basket with a handle at either side}"),
            _ => {
                if let Some(desc) = tables
                    .unikemet_data
                    .get(&cp)
                    .and_then(|u| u.description.as_deref())
                {
                    result.push_str(&format!(" {{{desc}}}"));
                } else if let Some(meaning) = tables
                    .tangut_data
                    .get(&cp)
                    .and_then(|t| t.meaning.as_deref())
                {
                    result.push_str(&format!(" {{{meaning}}}"));
                } else if let Some(meaning) = KHITAN_DATA.get(&cp) {
                    result.push_str(&format!(" {{{meaning}}}"));
                } else if let Some(info) = char_info {
                    if !info.informative_aliases.is_empty() {
                        let joined = info.informative_aliases.join("; ");
                        result.push_str(&format!(" {{{joined}}}"));
                    }
                }
            }
        }
    } else if let Some(desc) = tables
        .unikemet_data
        .get(&cp)
        .and_then(|u| u.description.as_deref())
    {
        result.push_str(&format!(" {{{desc}}}"));
    } else if let Some(meaning) = tables
        .tangut_data
        .get(&cp)
        .and_then(|t| t.meaning.as_deref())
    {
        result.push_str(&format!(" {{{meaning}}}"));
    } else if let Some(meaning) = KHITAN_DATA.get(&cp) {
        result.push_str(&format!(" {{{meaning}}}"));
    } else if let Some(info) = char_info {
        if !info.informative_aliases.is_empty() {
            let joined = info.informative_aliases.join("; ");
            result.push_str(&format!(" {{{joined}}}"));
        }
    }

    // 12. Unihan Readings & Definitions
    if options.include_unihan_readings {
        if let Some(unihan) = UNIHAN_DATA.get(&cp) {
            let mut parts = Vec::new();
            if let Some(ref def) = unihan.definition {
                parts.push(format!("def: {def}"));
            }
            if let Some(ref man) = unihan.mandarin {
                parts.push(format!("Mandarin: {man}"));
            }
            if let Some(ref can) = unihan.cantonese {
                parts.push(format!("Cantonese: {can}"));
            }
            if let Some(ref jap) = unihan.japanese {
                parts.push(format!("Japanese: {jap}"));
            }
            if let Some(ref kor) = unihan.korean {
                parts.push(format!("Korean: {kor}"));
            }
            if let Some(ref viet) = unihan.vietnamese {
                parts.push(format!("Vietnamese: {viet}"));
            }
            if !parts.is_empty() {
                let joined = parts.join("; ");
                result.push_str(&format!(" {{{joined}}}"));
            }
        }
    }

    result
}

/// Describes each character in a Unicode string line by line using default options.
pub fn describe(input: &str) -> String {
    describe_with_options(input, DescriptionOptions::default())
}

/// Describes each character in a Unicode string line by line with given options.
pub fn describe_with_options(
    input: &str,
    options: DescriptionOptions,
) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        let cp = u32::from(ch);
        let line = describe_codepoint_with_options(cp, options);
        out.push_str(&line);
        out.push('\n');
    }
    out
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
    fn can_describe_string_standard() {
        let input = "द्ध्र्य︘ꡀ𓉔A 😂\u{0004}𗿼선𘮝";
        let out = describe(input);
        assert!(out.contains("U+0926 : DEVANAGARI LETTER DA"));
        assert!(out.contains("U+094D : DEVANAGARI SIGN VIRAMA {halant (the preferred Hindi name)}"));
        assert!(out.contains("U+FE18 : PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRAKCET (alias PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET)"));
        assert!(out.contains("U+13254 : EGYPTIAN HIEROGLYPH O004 {A plan of a courtyard or reed shelter}"));
        assert!(out.contains("U+0041 : LATIN CAPITAL LETTER A"));
        assert!(out.contains("U+0020 : SPACE [SP]"));
        assert!(out.contains("U+1F602 : FACE WITH TEARS OF JOY"));
        assert!(out.contains("U+0004 : <control> END OF TRANSMISSION [EOT]"));
        assert!(
            out.contains(
                "U+17FFC : TANGUT IDEOGRAPH-17FFC (L2008-2262) {bird}"
            )
        );
        assert!(out.contains("U+C120 : HANGUL SYLLABLE SEON"));
        assert!(
            out.contains(
                "U+18B9D : KHITAN SMALL SCRIPT CHARACTER-18B9D {GOLD}"
            )
        );

        let input = "द्ध्र्य︘ꡀ𓉔A字😂\u{0004}𗿼선𘮝狀";
        let expected = "U+0926 : DEVANAGARI LETTER DA
U+094D : DEVANAGARI SIGN VIRAMA {halant (the preferred Hindi name)}
U+0927 : DEVANAGARI LETTER DHA
U+094D : DEVANAGARI SIGN VIRAMA {halant (the preferred Hindi name)}
U+0930 : DEVANAGARI LETTER RA
U+094D : DEVANAGARI SIGN VIRAMA {halant (the preferred Hindi name)}
U+092F : DEVANAGARI LETTER YA
U+FE18 : PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRAKCET (alias PRESENTATION FORM FOR VERTICAL RIGHT WHITE LENTICULAR BRACKET)
U+A840 : PHAGS-PA LETTER KA
U+13254 : EGYPTIAN HIEROGLYPH O004 {A plan of a courtyard or reed shelter}
U+0041 : LATIN CAPITAL LETTER A
U+5B57 : CJK UNIFIED IDEOGRAPH-5B57 {def: letter, character, word; Mandarin: zì; Cantonese: zi6; Japanese: ジ シ あざ あざな やしなう; Korean: 자:0E; Vietnamese: tự}
U+1F602 : FACE WITH TEARS OF JOY
U+0004 : <control> END OF TRANSMISSION [EOT]
U+17FFC : TANGUT IDEOGRAPH-17FFC (L2008-2262) {bird}
U+C120 : HANGUL SYLLABLE SEON
U+18B9D : KHITAN SMALL SCRIPT CHARACTER-18B9D {GOLD}
U+F9FA : CJK COMPATIBILITY IDEOGRAPH-F9FA = U+72C0 + VS1 {def: form; appearance; shape; official; Korean: 장:0}
";
        assert_eq!(describe(input), expected);
    }

    #[crate::ctb_test]
    fn can_describe_string_wuc_compat() {
        let options = DescriptionOptions::wuc_compat();
        let out = describe_with_options("!\u{0007}\u{0009}\u{000A}", options);
        assert!(out.contains("U+0021 : EXCLAMATION MARK {factorial; bang}"));
        assert!(out.contains("U+0007 : <control> BELL [BEL]"));
        assert!(out.contains("U+0009 : <control> CHARACTER TABULATION [TAB] {horizontal tabulation (HT); tab}"));
        assert!(out.contains("U+000A : <control> LINE FEED [LF] {new line (NL); end of line (EOL)}"));
    }

    #[crate::ctb_test]
    fn test_surrogates_and_noncharacters_and_reserved() {
        assert_eq!(
            describe_codepoint(0xD800),
            "U+D800 : <surrogate> (high surrogate)"
        );
        assert_eq!(
            describe_codepoint(0xDBFF),
            "U+DBFF : <surrogate> (high surrogate)"
        );
        assert_eq!(
            describe_codepoint(0xDC00),
            "U+DC00 : <surrogate> (low surrogate)"
        );
        assert_eq!(
            describe_codepoint(0xDFFF),
            "U+DFFF : <surrogate> (low surrogate)"
        );
        assert_eq!(describe_codepoint(0xFDD0), "U+FDD0 : <noncharacter>");
        assert_eq!(describe_codepoint(0xFFFF), "U+FFFF : <noncharacter>");
        assert_eq!(describe_codepoint(0x1FFFE), "U+1FFFE : <noncharacter>");
        assert_eq!(
            describe_codepoint(0x1BC9A),
            "U+1BC9A : <reserved> (Duployan block)"
        );
        assert_eq!(
            describe_codepoint(0x35000),
            "U+35000 : <reserved> (unassigned region)"
        );
        assert_eq!(
            describe_codepoint(0xE000),
            "U+E000 : <private-use> (Private Use Area block)"
        );
    }

    #[crate::ctb_test]
    fn test_wuc_mode_descriptions() {
        let wuc_opts = DescriptionOptions::wuc_compat();

        // 1. Control characters in legacy NamesList format
        assert_eq!(
            describe_codepoint_with_options(0x0000, wuc_opts),
            "U+0000 : <control> NULL [NUL]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0007, wuc_opts),
            "U+0007 : <control> BELL [BEL]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0009, wuc_opts),
            "U+0009 : <control> CHARACTER TABULATION [TAB] {horizontal tabulation (HT); tab}"
        );
        assert_eq!(
            describe_codepoint_with_options(0x000A, wuc_opts),
            "U+000A : <control> LINE FEED [LF] {new line (NL); end of line (EOL)}"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0019, wuc_opts),
            "U+0019 : <control> END OF MEDIUM [EOM]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0080, wuc_opts),
            "U+0080 : <control>"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0082, wuc_opts),
            "U+0082 : <control> BREAK PERMITTED HERE [BPH]"
        );

        // 2. Graphic abbreviations
        assert_eq!(
            describe_codepoint_with_options(0x0020, wuc_opts),
            "U+0020 : SPACE [SP]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x00A0, wuc_opts),
            "U+00A0 : NO-BREAK SPACE [NBSP]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x00AD, wuc_opts),
            "U+00AD : SOFT HYPHEN [SHY] {discretionary hyphen}"
        );

        // 3. Informative alias joining
        assert_eq!(
            describe_codepoint_with_options(0x0021, wuc_opts),
            "U+0021 : EXCLAMATION MARK {factorial; bang}"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0023, wuc_opts),
            "U+0023 : NUMBER SIGN {pound sign (weight); hashtag, hash; crosshatch, octothorpe}"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0027, wuc_opts),
            "U+0027 : APOSTROPHE {single quote; APL quote}"
        );

        // 4. Surrogates, Noncharacters, Reserved, PUA in WUC compat mode
        assert_eq!(
            describe_codepoint_with_options(0xD800, wuc_opts),
            "U+D800 : <surrogate>"
        );
        assert_eq!(
            describe_codepoint_with_options(0xDC00, wuc_opts),
            "U+DC00 : <surrogate>"
        );
        assert_eq!(
            describe_codepoint_with_options(0xFDD0, wuc_opts),
            "U+FDD0 : <noncharacter>"
        );
        assert_eq!(
            describe_codepoint_with_options(0x0378, wuc_opts),
            "U+0378 : <reserved>"
        );
        assert_eq!(
            describe_codepoint_with_options(0xE000, wuc_opts),
            "U+E000 : <private-use>"
        );
        assert_eq!(
            describe_codepoint_with_options(0xF0000, wuc_opts),
            "U+F0000 : <private-use>"
        );

        // 5. CJK Compatibility Ideographs in WUC compat mode
        assert_eq!(
            describe_codepoint_with_options(0xFA0E, wuc_opts),
            "U+FA0E : CJK COMPATIBILITY IDEOGRAPH-FA0E"
        );
    }

    #[crate::ctb_test]
    fn test_standard_mode_descriptions() {
        let std_opts = DescriptionOptions::standard();

        // 1. Surrogates with high/low designation
        assert_eq!(
            describe_codepoint_with_options(0xD800, std_opts),
            "U+D800 : <surrogate> (high surrogate)"
        );
        assert_eq!(
            describe_codepoint_with_options(0xDC00, std_opts),
            "U+DC00 : <surrogate> (low surrogate)"
        );

        // 2. Noncharacters
        assert_eq!(
            describe_codepoint_with_options(0xFDD0, std_opts),
            "U+FDD0 : <noncharacter>"
        );

        // 3. Reserved with block name or unassigned region
        assert_eq!(
            describe_codepoint_with_options(0x0378, std_opts),
            "U+0378 : <reserved> (Greek and Coptic block)"
        );
        assert_eq!(
            describe_codepoint_with_options(0x35000, std_opts),
            "U+35000 : <reserved> (unassigned region)"
        );

        // 4. Private Use Area with block name
        assert_eq!(
            describe_codepoint_with_options(0xE000, std_opts),
            "U+E000 : <private-use> (Private Use Area block)"
        );

        // 5. Official UCD control names
        assert_eq!(
            describe_codepoint_with_options(0x0007, std_opts),
            "U+0007 : <control> ALERT [BEL]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x000A, std_opts),
            "U+000A : <control> END OF LINE [EOL] {new line (NL); end of line (EOL)}"
        );

        // 6. CJK Unified Ideograph with Unihan readings
        let desc_3400 = describe_codepoint_with_options(0x3400, std_opts);
        assert!(desc_3400.starts_with("U+3400 : CJK UNIFIED IDEOGRAPH-3400"));
        assert!(desc_3400.contains("def: (same as 丘) hillock or mound"));
        assert!(desc_3400.contains("Mandarin: qiū"));
        assert!(desc_3400.contains("Cantonese: jau1"));

        // 7. CJK Unified Ideograph in compatibility range (U+FA0E)
        let desc_fa0e = describe_codepoint_with_options(0xFA0E, std_opts);
        assert!(desc_fa0e.starts_with("U+FA0E : CJK UNIFIED IDEOGRAPH-FA0E"));
    }

    #[crate::ctb_test]
    fn test_options_configurations() {
        let mut custom_opts = DescriptionOptions::standard();
        custom_opts.control_name_format = ControlNameFormat::NamesList;
        custom_opts.include_unihan_readings = false;

        assert_eq!(
            describe_codepoint_with_options(0x0007, custom_opts),
            "U+0007 : <control> BELL [BEL]"
        );
        assert_eq!(
            describe_codepoint_with_options(0x3400, custom_opts),
            "U+3400 : CJK UNIFIED IDEOGRAPH-3400"
        );

        let multiline = describe_with_options("\u{0007}A", custom_opts);
        assert_eq!(
            multiline,
            "U+0007 : <control> BELL [BEL]\nU+0041 : LATIN CAPITAL LETTER A\n"
        );

        let default_multiline = describe("A");
        assert_eq!(default_multiline, "U+0041 : LATIN CAPITAL LETTER A\n");

        // Test UnicodeDataTables derived_age & block lookup & dynamic ideograph lookup
        let tables_v17 = get_tables(UnicodeVersion::V17_0);
        assert!(tables_v17.is_age("1.1", 0x0041));
        assert!(tables_v17.is_age("16.0", 0x1FA89)); // HARP
        assert!(tables_v17.is_v17_or_later(0x088F));
        assert!(!tables_v17.is_v17_or_later(0x0041));
        assert!(tables_v17.is_cjk_unified_ideograph(0x3400));
        assert!(tables_v17.is_tangut_ideograph(0x17000));
        assert!(tables_v17.is_khitan_character(0x18B00));

        assert_eq!(crate::find_block(0x0041), Some("Basic Latin"));

        // Test V15.0 and V15.1 tables loading and data-derived ideographs
        let tables_v15 = get_tables(UnicodeVersion::V15_0);
        assert!(!tables_v15.blocks.is_empty());
        assert!(tables_v15.is_cjk_unified_ideograph(0x4E00));
        assert!(tables_v15.is_tangut_ideograph(0x17000));
        assert!(tables_v15.is_khitan_character(0x18B00));

        let tables_v15_1 = get_tables(UnicodeVersion::V15_1);
        assert!(!tables_v15_1.blocks.is_empty());
        assert!(tables_v15_1.is_cjk_unified_ideograph(0x4E00));
    }

    // #[crate::ctb_test]
    // fn test_full_file_regeneration_matches_fixture() {
    //     let manifest_dir = std::path::PathBuf::from(
    //         std::env::var("CARGO_MANIFEST_DIR")
    //             .unwrap_or_else(|_| "/workspaces/ctoolbox/src/formats/unicode".to_string()),
    //     );
    //     // This test should pass, but I'm not including the fixture now that it's working.
    //     let fixture_path = manifest_dir
    //         .join("tests")
    //         .join("fixtures")
    //         .join("unicode_untrimmed_descriptions.txt");

    //     let expected = std::fs::read_to_string(&fixture_path)
    //         .expect("read unicode_untrimmed_descriptions.txt fixture");
    //     let wuc_opts = DescriptionOptions::wuc_compat();

    //     let mut generated = String::with_capacity(expected.len());
    //     for cp in 0..=0x10FFFF {
    //         let line = describe_codepoint_with_options(cp, wuc_opts);
    //         generated.push_str(&line);
    //         generated.push('\n');
    //     }

    //     assert_eq!(generated, expected);
    // }

    #[crate::ctb_test]
    fn test_full_file_regeneration_matches_expected() {
        let expected =
            "2b1fc9126e2c9b2f7ee7398b96581db58d4f46bdc768264b198734d25d068be8";
        let wuc_opts = DescriptionOptions::wuc_compat();

        let mut generated = String::with_capacity(expected.len());
        for cp in 0..=0x10FFFF {
            let line = describe_codepoint_with_options(cp, wuc_opts);
            generated.push_str(&line);
            generated.push('\n');
        }

        assert_eq!(ctb_formats_checksum::sha256_hex(&generated), expected);
    }
}

/*

Unikemet descriptions are used under the following Unicode license terms:

```
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2026 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.
```
*/
