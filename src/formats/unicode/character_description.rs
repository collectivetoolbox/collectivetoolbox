#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::data::{
    CHAR_DATA, KHITAN_DATA, NAME_ALIASES, STANDARDIZED_VARIANTS, TANGUT_DATA,
    UNIKEMET_DATA, find_block, is_noncharacter,
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
    "LT", "LP", "LH", "M", "B", "BS", "S", "SS", "NG", "J", "C", "K", "T",
    "P", "H",
];

/// Computes the algorithmic Hangul syllable name for a code point if applicable.
fn hangul_syllable_name(cp: u32) -> Option<String> {
    if (HANGUL_S_BASE..HANGUL_S_BASE.saturating_add(HANGUL_S_COUNT)).contains(&cp)
    {
        let s_index = cp.saturating_sub(HANGUL_S_BASE);
        let l_index = usize::try_from(s_index.checked_div(HANGUL_N_COUNT)?).ok()?;
        let v_index = usize::try_from(
            (s_index.checked_rem(HANGUL_N_COUNT)?).checked_div(HANGUL_T_COUNT)?,
        )
        .ok()?;
        let t_index = usize::try_from(s_index.checked_rem(HANGUL_T_COUNT)?).ok()?;
        let l = JAMO_L_TABLE.get(l_index)?;
        let v = JAMO_V_TABLE.get(v_index)?;
        let t = JAMO_T_TABLE.get(t_index)?;
        Some(format!("HANGUL SYLLABLE {l}{v}{t}"))
    } else {
        None
    }
}

use icu_properties::props::{Script, UnifiedIdeograph};
use icu_properties::{CodePointMapData, CodePointSetData};

/// Checks if a codepoint is a CJK Unified Ideograph using ICU4X Unified_Ideograph property.
fn is_cjk_unified_ideograph(cp: u32) -> bool {
    CodePointSetData::new::<UnifiedIdeograph>().contains32(cp)
}

/// Checks if a codepoint is a Tangut Ideograph using ICU4X Script property.
fn is_tangut_ideograph(cp: u32) -> bool {
    CodePointMapData::<Script>::new().get32(cp) == Script::Tangut
        && !find_block(cp).is_some_and(|b| b.contains("Components"))
}

/// Checks if a codepoint is a Khitan Small Script character using ICU4X Script property.
fn is_khitan_small_script(cp: u32) -> bool {
    CodePointMapData::<Script>::new().get32(cp) == Script::KhitanSmallScript
}


/// Formats a single Unicode code point into a detailed description string.
pub fn describe_codepoint(cp: u32) -> String {
    // 1. High and Low Surrogates
    if (0xD800..=0xDBFF).contains(&cp) {
        return format!("U+{cp:04X} is a high surrogate code point");
    }
    if (0xDC00..=0xDFFF).contains(&cp) {
        return format!("U+{cp:04X} is a low surrogate code point");
    }

    // 2. Noncharacter Code Points
    if is_noncharacter(cp) {
        return format!("U+{cp:04X} is not a character");
    }

    let char_info = CHAR_DATA.get(&cp);
    let alias_entry = NAME_ALIASES.get(&cp);

    // 3. Control Characters
    let is_ctrl = char_info.is_some_and(|info| info.is_control)
        || (0x0000..=0x001F).contains(&cp)
        || (0x007F..=0x009F).contains(&cp);

    if is_ctrl {
        let control_name = alias_entry.and_then(|a| a.control.as_deref());
        let abbr = alias_entry.and_then(|a| a.abbreviation.as_deref());
        match (control_name, abbr) {
            (Some(name), Some(ab)) => {
                return format!("U+{cp:04X} : <control> {name} [{ab}]");
            }
            (Some(name), None) => {
                return format!("U+{cp:04X} : <control> {name}");
            }
            (None, Some(ab)) => {
                return format!("U+{cp:04X} : <control> [{ab}]");
            }
            (None, None) => {
                return format!("U+{cp:04X} : <control>");
            }
        }
    }

    // 4. Primary Character Name Resolution
    let mut primary_name: Option<String> = None;

    if let Some(hangul) = hangul_syllable_name(cp) {
        primary_name = Some(hangul);
    } else if is_cjk_unified_ideograph(cp) {
        primary_name = Some(format!("CJK UNIFIED IDEOGRAPH-{cp:04X}"));
    } else if is_tangut_ideograph(cp) {
        primary_name = Some(format!("TANGUT IDEOGRAPH-{cp:04X}"));
    } else if is_khitan_small_script(cp) {
        primary_name = Some(format!("KHITAN SMALL SCRIPT CHARACTER-{cp:04X}"));
    } else if (0x13000..=0x143FF).contains(&cp) {
        // Egyptian Hieroglyphs
        if let Some(entry) = UNIKEMET_DATA.get(&cp) {
            if let Some(ref unik) = entry.unik_code {
                primary_name = Some(format!("EGYPTIAN HIEROGLYPH {unik}"));
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

    // 5. Unassigned / Reserved Code Points
    let Some(name) = primary_name else {
        if let Some(block) = find_block(cp) {
            return format!(
                "U+{cp:04X} is a reserved character in the {block} block"
            );
        }
        return format!(
            "U+{cp:04X} is a reserved character in an unassigned region"
        );
    };

    let mut result = format!("U+{cp:04X} : {name}");

    // 6. Formal Correction Alias
    if let Some(alias) = alias_entry.and_then(|a| a.correction.as_deref()) {
        result.push_str(&format!(" (alias {alias})"));
    }

    // 7. Tangut Source Reference
    if let Some(src) = TANGUT_DATA.get(&cp).and_then(|t| t.source_ref.as_deref())
    {
        result.push_str(&format!(" ({src})"));
    }

    // 8. Standardized Variation Sequence (CJK Compatibility Ideographs)
    if let Some(variant) = STANDARDIZED_VARIANTS.get(&cp) {
        result.push_str(&format!(" {variant}"));
    }

    // 9. Informative Annotations & Meanings
    if let Some(desc) =
        UNIKEMET_DATA.get(&cp).and_then(|u| u.description.as_deref())
    {
        result.push_str(&format!(" {{{desc}}}"));
    } else if let Some(meaning) =
        TANGUT_DATA.get(&cp).and_then(|t| t.meaning.as_deref())
    {
        result.push_str(&format!(" {{{meaning}}}"));
    } else if let Some(meaning) = KHITAN_DATA.get(&cp) {
        result.push_str(&format!(" {{{meaning}}}"));
    } else if let Some(alias) =
        char_info.and_then(|c| c.informative_alias.as_deref())
    {
        result.push_str(&format!(" {{{alias}}}"));
    }

    result
}

/// Describes each character in a Unicode string line by line.
pub fn describe(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        let cp = u32::from(ch);
        let line = describe_codepoint(cp);
        out.push_str(&line);
        out.push('\n');
    }
    out
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
    fn can_describe_string() {
        let input = "द्ध्र्य︘ꡀ𓉔A字😂\u{0004}𗿼선𘮝狀";
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
U+5B57 : CJK UNIFIED IDEOGRAPH-5B57
U+1F602 : FACE WITH TEARS OF JOY
U+0004 : <control> END OF TRANSMISSION [EOT]
U+17FFC : TANGUT IDEOGRAPH-17FFC (L2008-2262) {bird}
U+C120 : HANGUL SYLLABLE SEON
U+18B9D : KHITAN SMALL SCRIPT CHARACTER-18B9D {GOLD}
U+F9FA : CJK COMPATIBILITY IDEOGRAPH-F9FA = U+72C0 + VS1
";
        assert_eq!(describe(input), expected);
    }

    #[crate::ctb_test]
    fn test_surrogates_and_noncharacters_and_reserved() {
        assert_eq!(
            describe_codepoint(0xD800),
            "U+D800 is a high surrogate code point"
        );
        assert_eq!(
            describe_codepoint(0xDBFF),
            "U+DBFF is a high surrogate code point"
        );
        assert_eq!(
            describe_codepoint(0xDC00),
            "U+DC00 is a low surrogate code point"
        );
        assert_eq!(
            describe_codepoint(0xDFFF),
            "U+DFFF is a low surrogate code point"
        );
        assert_eq!(describe_codepoint(0xFDD0), "U+FDD0 is not a character");
        assert_eq!(describe_codepoint(0xFFFF), "U+FFFF is not a character");
        assert_eq!(describe_codepoint(0x1FFFE), "U+1FFFE is not a character");
        assert_eq!(
            describe_codepoint(0x1BC9A),
            "U+1BC9A is a reserved character in the Duployan block"
        );
        assert_eq!(
            describe_codepoint(0x35000),
            "U+35000 is a reserved character in an unassigned region"
        );
    }
}