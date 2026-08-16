#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use std::collections::HashMap;
use std::sync::LazyLock;

/// Returns the string contents of a file in the UCD asset directory.
fn get_ucd_file(name: &str) -> Option<String> {
    ctb_storage::get_asset_utf8(&format!("data/Unicode/UCD/{name}")).ok()
}

/// Returns the string contents of a file in the dictionaries asset directory.
fn get_dict_file(name: &str) -> Option<String> {
    ctb_storage::get_asset_utf8(&format!("data/dictionaries/{name}")).ok()
}


/// A parsed Unicode block range.
#[derive(Debug, Clone)]
pub struct UnicodeBlock {
    pub start: u32,
    pub end: u32,
    pub name: String,
}

/// Table of all Unicode blocks from Blocks.txt.
pub static BLOCKS: LazyLock<Vec<UnicodeBlock>> = LazyLock::new(|| {
    let Some(content) = get_ucd_file("Blocks.txt") else {
        return Vec::new();
    };
    let mut blocks = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((range_part, name_part)) = trimmed.split_once(';') else {
            continue;
        };
        let Some((start_hex, end_hex)) = range_part.trim().split_once("..")
        else {
            continue;
        };
        let Ok(start) = u32::from_str_radix(start_hex.trim(), 16) else {
            continue;
        };
        let Ok(end) = u32::from_str_radix(end_hex.trim(), 16) else {
            continue;
        };
        blocks.push(UnicodeBlock {
            start,
            end,
            name: name_part.trim().to_string(),
        });
    }
    blocks
});

/// Name aliases from NameAliases.txt.
#[derive(Debug, Default, Clone)]
pub struct NameAliasEntry {
    pub correction: Option<String>,
    pub control: Option<String>,
    pub abbreviation: Option<String>,
    pub alternate: Option<String>,
    pub figment: Option<String>,
}

/// Table of name aliases mapped by codepoint.
pub static NAME_ALIASES: LazyLock<HashMap<u32, NameAliasEntry>> =
    LazyLock::new(|| {
        let Some(content) = get_ucd_file("NameAliases.txt") else {
            return HashMap::new();
        };
        let mut map: HashMap<u32, NameAliasEntry> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(';').collect();
            if parts.len() < 3 {
                continue;
            }
            let (Some(&cp_hex), Some(&alias), Some(&kind)) =
                (parts.first(), parts.get(1), parts.get(2))
            else {
                continue;
            };
            let Ok(cp) = u32::from_str_radix(cp_hex.trim(), 16) else {
                continue;
            };
            let entry = map.entry(cp).or_default();
            match kind.trim() {
                "correction" => entry.correction = Some(alias.trim().to_string()),
                "control" => entry.control = Some(alias.trim().to_string()),
                "abbreviation" => {
                    entry.abbreviation = Some(alias.trim().to_string());
                }
                "alternate" => entry.alternate = Some(alias.trim().to_string()),
                "figment" => entry.figment = Some(alias.trim().to_string()),
                _ => {}
            }
        }
        map
    });

/// Parsed character entry from UnicodeData.txt / NamesList.txt.
#[derive(Debug, Default, Clone)]
pub struct UnicodeCharInfo {
    pub name: String,
    pub informative_alias: Option<String>,
    pub is_control: bool,
}

/// Primary names and informative aliases from NamesList.txt and UnicodeData.txt.
pub static CHAR_DATA: LazyLock<HashMap<u32, UnicodeCharInfo>> =
    LazyLock::new(|| {
        let mut map: HashMap<u32, UnicodeCharInfo> = HashMap::new();

        // Parse UnicodeData.txt first
        if let Some(content) = get_ucd_file("UnicodeData.txt") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split(';').collect();
                if parts.len() < 3 {
                    continue;
                }
                let (Some(&cp_hex), Some(&name), Some(&cat)) =
                    (parts.first(), parts.get(1), parts.get(2))
                else {
                    continue;
                };
                let Ok(cp) = u32::from_str_radix(cp_hex.trim(), 16) else {
                    continue;
                };
                let is_control = cat.trim() == "Cc" || name.trim() == "<control>";
                map.insert(
                    cp,
                    UnicodeCharInfo {
                        name: name.trim().to_string(),
                        informative_alias: None,
                        is_control,
                    },
                );
            }
        }

        // Parse NamesList.txt for character names and informative aliases (= ...)
        if let Some(content) = get_ucd_file("NamesList.txt") {
            let mut current_cp: Option<u32> = None;
            for line in content.lines() {
                if line.starts_with('\t') {
                    if let Some(cp) = current_cp {
                        let rest = line.trim_start_matches('\t');
                        if let Some(alias) = rest.strip_prefix("= ") {
                            let entry = map.entry(cp).or_default();
                            if entry.informative_alias.is_none() {
                                entry.informative_alias =
                                    Some(alias.trim().to_string());
                            }
                        }
                    }
                } else if !line.starts_with('@') && !line.is_empty() {
                    let trimmed = line.trim();
                    if let Some((hex_str, name)) = trimmed.split_once('\t') {
                        if let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) {
                            current_cp = Some(cp);
                            let entry = map.entry(cp).or_default();
                            if entry.name.is_empty()
                                || entry.name.starts_with('<')
                            {
                                entry.name = name.trim().to_string();
                            }
                        } else {
                            current_cp = None;
                        }
                    } else {
                        current_cp = None;
                    }
                }
            }
        }

        map
    });

/// Egyptian Hieroglyph metadata from Unikemet.txt.
#[derive(Debug, Default, Clone)]
pub struct UnikemetEntry {
    pub unik_code: Option<String>,
    pub description: Option<String>,
}

/// Table of Unikemet data mapped by codepoint.
pub static UNIKEMET_DATA: LazyLock<HashMap<u32, UnikemetEntry>> =
    LazyLock::new(|| {
        let Some(content) = get_ucd_file("Unikemet.txt") else {
            return HashMap::new();
        };
        let mut map: HashMap<u32, UnikemetEntry> = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() < 3 {
                continue;
            }
            let (Some(&cp_str), Some(&key), Some(&val)) =
                (parts.first(), parts.get(1), parts.get(2))
            else {
                continue;
            };
            let Some(hex_str) = cp_str.strip_prefix("U+") else {
                continue;
            };
            let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) else {
                continue;
            };
            let entry = map.entry(cp).or_default();
            match key.trim() {
                "kEH_UniK" => entry.unik_code = Some(val.trim().to_string()),
                "kEH_Desc" => {
                    let desc = val.trim().trim_end_matches('.');
                    entry.description = Some(desc.to_string());
                }
                _ => {}
            }
        }
        map
    });

/// Tangut character metadata (source reference and English gloss).
#[derive(Debug, Default, Clone)]
pub struct TangutEntry {
    pub source_ref: Option<String>,
    pub meaning: Option<String>,
}

/// Table of Tangut data mapped by codepoint.
pub static TANGUT_DATA: LazyLock<HashMap<u32, TangutEntry>> =
    LazyLock::new(|| {
        let mut map: HashMap<u32, TangutEntry> = HashMap::new();

        // 1. TangutSources.txt
        if let Some(content) = get_ucd_file("TangutSources.txt") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = trimmed.split('\t').collect();
                if parts.len() < 3 {
                    continue;
                }
                let (Some(&cp_str), Some(&key), Some(&val)) =
                    (parts.first(), parts.get(1), parts.get(2))
                else {
                    continue;
                };
                let Some(hex_str) = cp_str.strip_prefix("U+") else {
                    continue;
                };
                let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) else {
                    continue;
                };
                if key.trim() == "kTGT_MergedSrc" {
                    let entry = map.entry(cp).or_default();
                    entry.source_ref = Some(val.trim().to_string());
                }
            }
        }

        // 2. TangutMeanings.csv
        if let Some(content) = get_dict_file("TangutMeanings.csv") {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(content.as_bytes());
            for result in rdr.records() {
                let Ok(record) = result else {
                    continue;
                };
                let (Some(hex_str), Some(def)) = (record.get(0), record.get(1))
                else {
                    continue;
                };
                let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) else {
                    continue;
                };
                if !def.trim().is_empty() {
                    let entry = map.entry(cp).or_default();
                    entry.meaning = Some(def.trim().to_string());
                }
            }
        }

        // 3. TangutSupplementMeanings.csv
        if let Some(content) = get_dict_file("TangutSupplementMeanings.csv") {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(content.as_bytes());
            for result in rdr.records() {
                let Ok(record) = result else {
                    continue;
                };
                let (Some(hex_str), Some(def)) = (record.get(0), record.get(1))
                else {
                    continue;
                };
                let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) else {
                    continue;
                };
                let entry = map.entry(cp).or_default();
                if !def.trim().is_empty() && entry.meaning.is_none() {
                    entry.meaning = Some(def.trim().to_string());
                }
                if let Some(src) = record.get(5) {
                    if !src.trim().is_empty() && entry.source_ref.is_none() {
                        entry.source_ref = Some(src.trim().to_string());
                    }
                }
            }
        }

        map
    });

/// Table of Khitan Small Script character meanings from KhitanMeanings.csv.
pub static KHITAN_DATA: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    let Some(content) = get_dict_file("KhitanMeanings.csv") else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());
    for result in rdr.records() {
        let Ok(record) = result else {
            continue;
        };
        let (Some(hex_str), Some(def)) = (record.get(0), record.get(1)) else {
            continue;
        };
        let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) else {
            continue;
        };
        if !def.trim().is_empty() {
            map.insert(cp, def.trim().to_string());
        }
    }
    map
});

/// Standardized variation sequence mappings from StandardizedVariants.txt.
/// Maps compatibility ideograph / character to "= U+BASE + VS{N}".
pub static STANDARDIZED_VARIANTS: LazyLock<HashMap<u32, String>> =
    LazyLock::new(|| {
        let Some(content) = get_ucd_file("StandardizedVariants.txt") else {
            return HashMap::new();
        };
        let mut map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(';').collect();
            if parts.len() < 2 {
                continue;
            }
            let (Some(&seq_part), Some(&desc_part)) =
                (parts.first(), parts.get(1))
            else {
                continue;
            };
            let seq: Vec<&str> = seq_part.split_whitespace().collect();
            if seq.len() != 2 {
                continue;
            }
            let (Some(&base_hex), Some(&vs_hex)) = (seq.first(), seq.get(1))
            else {
                continue;
            };
            let (Ok(base_cp), Ok(vs_cp)) = (
                u32::from_str_radix(base_hex.trim(), 16),
                u32::from_str_radix(vs_hex.trim(), 16),
            ) else {
                continue;
            };

            let desc = desc_part.trim();
            // Check if this description names a CJK COMPATIBILITY IDEOGRAPH
            if let Some(comp_name) = desc.strip_prefix("CJK COMPATIBILITY IDEOGRAPH-") {
                let comp_hex = comp_name.trim().trim_end_matches(';');
                if let Ok(comp_cp) = u32::from_str_radix(comp_hex, 16) {
                    let vs_num = if (0xFE00..=0xFE0F).contains(&vs_cp) {
                        vs_cp.saturating_sub(0xFE00).saturating_add(1)
                    } else if (0xE0100..=0xE01EF).contains(&vs_cp) {
                        vs_cp.saturating_sub(0xE0100).saturating_add(17)
                    } else {
                        1
                    };
                    let formatted = format!("= U+{base_cp:04X} + VS{vs_num}");
                    map.insert(comp_cp, formatted);
                }
            }
        }
        map
    });

/// Checks if a codepoint is a Noncharacter according to Unicode PropList.txt.
pub fn is_noncharacter(cp: u32) -> bool {
    // Range U+FDD0..=U+FDEF in BMP
    if (0xFDD0..=0xFDEF).contains(&cp) {
        return true;
    }
    // End-of-plane noncharacters U+nFFFE and U+nFFFF for all 17 planes (0x00 to 0x10)
    let low16 = cp & 0xFFFF;
    (low16 == 0xFFFE || low16 == 0xFFFF) && cp <= 0x10FFFF
}

/// Finds the block name for a codepoint if it belongs to one.
pub fn find_block(cp: u32) -> Option<&'static str> {
    for b in BLOCKS.iter() {
        if cp >= b.start && cp <= b.end {
            return Some(&b.name);
        }
    }
    None
}
