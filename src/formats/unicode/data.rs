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

//! Unicode character database querying, property lookup, and dataset loader.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Unicode dataset version to query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnicodeVersion {
    /// Unicode 17.0 (default)
    #[default]
    V17_0,
    /// Unicode 16.0
    V16_0,
    /// Unicode 15.1
    V15_1,
    /// Unicode 15.0 (legacy JS compatibility)
    V15_0,
}

/// Returns the string contents of a file in the UCD asset directory for a given version.
fn get_ucd_file_for_version(
    version: UnicodeVersion,
    name: &str,
) -> Option<String> {
    match version {
        UnicodeVersion::V17_0 => ctb_storage_asset_bundle::get_asset_utf8(
            &format!("data/Unicode/Unicode-17.0.0/UCD/{name}"),
        )
        .ok(),
        UnicodeVersion::V16_0 => ctb_storage_asset_bundle::get_asset_utf8(
            &format!("data/Unicode/Unicode-16.0.0/UCD/{name}"),
        )
        .ok(),
        UnicodeVersion::V15_1 => ctb_storage_asset_bundle::get_asset_utf8(
            &format!("data/Unicode/Unicode-15.1.0/UCD/{name}"),
        )
        .ok(),
        UnicodeVersion::V15_0 => ctb_storage_asset_bundle::get_asset_utf8(
            &format!("data/Unicode/Unicode-15.0.0/UCD/{name}"),
        )
        .ok(),
    }
}

/// Returns the string contents of a file in the `character_descriptions` asset directory.
fn get_character_descriptions_file(name: &str) -> Option<String> {
    ctb_storage_asset_bundle::get_asset_utf8(&format!(
        "data/character_descriptions/{name}"
    ))
    .ok()
}

/// A parsed Unicode block range.
#[derive(Debug, Clone)]
pub struct UnicodeBlock {
    pub start: u32,
    pub end: u32,
    pub name: String,
}

/// Name aliases from NameAliases.txt.
#[derive(Debug, Default, Clone)]
pub struct NameAliasEntry {
    pub correction: Option<String>,
    pub control: Option<String>,
    pub abbreviation: Option<String>,
    pub alternate: Option<String>,
    pub figment: Option<String>,
}

/// Parsed character entry from UnicodeData.txt / NamesList.txt.
#[derive(Debug, Default, Clone)]
pub struct UnicodeCharInfo {
    pub name: String,
    pub informative_aliases: Vec<String>,
    pub is_control: bool,
    pub nameslist_control_name: Option<String>,
    pub nameslist_control_abbr: Option<String>,
}

/// Egyptian Hieroglyph metadata from Unikemet.txt.
#[derive(Debug, Default, Clone)]
pub struct UnikemetEntry {
    pub unik_code: Option<String>,
    pub description: Option<String>,
}

/// Tangut character metadata (source reference and English gloss).
#[derive(Debug, Default, Clone)]
pub struct TangutEntry {
    pub source_ref: Option<String>,
    pub meaning: Option<String>,
}

/// Complete dataset tables for a specific Unicode version.
pub struct UnicodeDataTables {
    pub blocks: Vec<UnicodeBlock>,
    pub name_aliases: HashMap<u32, NameAliasEntry>,
    pub char_data: HashMap<u32, UnicodeCharInfo>,
    pub standardized_variants: HashMap<u32, String>,
    pub unikemet_data: HashMap<u32, UnikemetEntry>,
    pub tangut_data: HashMap<u32, TangutEntry>,
    pub derived_age: HashMap<String, HashSet<u32>>,
    pub unified_ideographs: HashSet<u32>,
    pub tangut_ideographs: HashSet<u32>,
    pub khitan_characters: HashSet<u32>,
}

impl UnicodeDataTables {
    /// Returns true if a code point was assigned in a specific Unicode age (e.g. "16.0").
    #[must_use]
    pub fn is_age(&self, age: &str, cp: u32) -> bool {
        self.derived_age
            .get(age)
            .is_some_and(|set| set.contains(&cp))
    }

    /// Returns true if a code point was assigned in Unicode 17.0 or later.
    #[must_use]
    pub fn is_v17_or_later(&self, cp: u32) -> bool {
        self.derived_age
            .iter()
            .filter(|(ver, _)| ver.starts_with("17."))
            .any(|(_, set)| set.contains(&cp))
    }

    /// Returns true if a code point is a CJK Unified Ideograph in this dataset version.
    #[must_use]
    pub fn is_cjk_unified_ideograph(&self, cp: u32) -> bool {
        self.unified_ideographs.contains(&cp)
    }

    /// Returns true if a code point is a Tangut Ideograph in this dataset version.
    #[must_use]
    pub fn is_tangut_ideograph(&self, cp: u32) -> bool {
        self.tangut_ideographs.contains(&cp)
    }

    /// Returns true if a code point is a Khitan Small Script character in this dataset version.
    #[must_use]
    pub fn is_khitan_character(&self, cp: u32) -> bool {
        self.khitan_characters.contains(&cp)
    }

    /// Returns true if a code point is an assigned character in Unicode.
    #[must_use]
    pub fn is_assigned(&self, cp: u32) -> bool {
        if cp > 0x10_FFFF
            || is_noncharacter(cp)
            || (0xD800..=0xDFFF).contains(&cp)
        {
            return false;
        }
        self.char_data.contains_key(&cp)
            || self.is_cjk_unified_ideograph(cp)
            || self.is_tangut_ideograph(cp)
            || self.is_khitan_character(cp)
            || (0xAC00..=0xD7A3).contains(&cp)
            || (0xE000..=0xF8FF).contains(&cp)
            || (0xF0000..=0xFFFFD).contains(&cp)
            || (0x100000..=0x10FFFD).contains(&cp)
    }
}

fn load_tables(version: UnicodeVersion) -> UnicodeDataTables {
    // 1. Parse Blocks.txt
    let mut blocks = Vec::new();
    if let Some(content) = get_ucd_file_for_version(version, "Blocks.txt") {
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
    }

    // 2. Parse NameAliases.txt
    let mut name_aliases: HashMap<u32, NameAliasEntry> = HashMap::new();
    if let Some(content) = get_ucd_file_for_version(version, "NameAliases.txt")
    {
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
            let entry = name_aliases.entry(cp).or_default();
            match kind.trim() {
                "correction" => {
                    entry.correction = Some(alias.trim().to_string());
                }
                "control" => entry.control = Some(alias.trim().to_string()),
                "abbreviation" => {
                    entry.abbreviation = Some(alias.trim().to_string());
                }
                "alternate" => entry.alternate = Some(alias.trim().to_string()),
                "figment" => entry.figment = Some(alias.trim().to_string()),
                _ => {}
            }
        }
    }

    // 3. Parse UnicodeData.txt and NamesList.txt
    let mut char_data: HashMap<u32, UnicodeCharInfo> = HashMap::new();
    if let Some(content) = get_ucd_file_for_version(version, "UnicodeData.txt")
    {
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
            char_data.insert(
                cp,
                UnicodeCharInfo {
                    name: name.trim().to_string(),
                    informative_aliases: Vec::new(),
                    is_control,
                    nameslist_control_name: None,
                    nameslist_control_abbr: None,
                },
            );
        }
    }

    if let Some(content) = get_ucd_file_for_version(version, "NamesList.txt") {
        let mut current_cp: Option<u32> = None;
        for line in content.lines() {
            if line.starts_with('\t') {
                if let Some(cp) = current_cp {
                    let rest = line.trim_start_matches('\t');
                    if let Some(alias) = rest.strip_prefix("= ") {
                        let trimmed_alias = alias.trim();
                        let entry = char_data.entry(cp).or_default();
                        if entry.is_control {
                            if entry.nameslist_control_name.is_none() {
                                if let Some((name_part, abbr_part)) =
                                    trimmed_alias
                                        .strip_suffix(')')
                                        .and_then(|s| s.rsplit_once('('))
                                {
                                    entry.nameslist_control_name =
                                        Some(name_part.trim().to_string());
                                    entry.nameslist_control_abbr =
                                        Some(abbr_part.trim().to_string());
                                } else {
                                    entry.nameslist_control_name =
                                        Some(trimmed_alias.to_string());
                                }
                            } else if !trimmed_alias.ends_with("(1.0)") {
                                entry
                                    .informative_aliases
                                    .push(trimmed_alias.to_string());
                            }
                        } else if !trimmed_alias.ends_with("(1.0)") {
                            entry
                                .informative_aliases
                                .push(trimmed_alias.to_string());
                        }
                    }
                }
            } else if !line.starts_with('@') && !line.is_empty() {
                let trimmed = line.trim();
                if let Some((hex_str, name)) = trimmed.split_once('\t') {
                    if let Ok(cp) = u32::from_str_radix(hex_str.trim(), 16) {
                        current_cp = Some(cp);
                        let entry = char_data.entry(cp).or_default();
                        if entry.name.is_empty() || entry.name.starts_with('<')
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

    // 4. Parse StandardizedVariants.txt
    let mut standardized_variants = HashMap::new();
    if let Some(content) =
        get_ucd_file_for_version(version, "StandardizedVariants.txt")
    {
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
            if let Some(comp_name) =
                desc.strip_prefix("CJK COMPATIBILITY IDEOGRAPH-")
            {
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
                    standardized_variants.insert(comp_cp, formatted);
                }
            }
        }
    }

    // 5. Parse Unikemet.txt
    let mut unikemet_data: HashMap<u32, UnikemetEntry> = HashMap::new();
    if let Some(content) = get_ucd_file_for_version(version, "Unikemet.txt") {
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
            let entry = unikemet_data.entry(cp).or_default();
            match key.trim() {
                "kEH_UniK" => entry.unik_code = Some(val.trim().to_string()),
                "kEH_Desc" => {
                    let desc = if version == UnicodeVersion::V16_0 {
                        val.trim_end_matches('.').to_string()
                    } else {
                        val.trim().trim_end_matches('.').to_string()
                    };
                    entry.description = Some(desc);
                }
                _ => {}
            }
        }
    }

    // 6. Parse Tangut data
    let mut tangut_data: HashMap<u32, TangutEntry> = HashMap::new();
    if let Some(content) =
        get_ucd_file_for_version(version, "TangutSources.txt")
    {
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
                let entry = tangut_data.entry(cp).or_default();
                entry.source_ref = Some(val.trim().to_string());
            }
        }
    }

    if let Some(content) = get_character_descriptions_file("TangutMeanings.csv")
    {
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
                let entry = tangut_data.entry(cp).or_default();
                entry.meaning = Some(def.trim().to_string());
            }
        }
    }

    if let Some(content) =
        get_character_descriptions_file("TangutSupplementMeanings.csv")
    {
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
            let entry = tangut_data.entry(cp).or_default();
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

    // 7. Parse DerivedAge.txt to map age version string -> set of code points
    let mut derived_age: HashMap<String, HashSet<u32>> = HashMap::new();
    if let Some(content) =
        get_ucd_file_for_version(UnicodeVersion::V17_0, "DerivedAge.txt")
    {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((range_part, rest)) = trimmed.split_once(';') else {
                continue;
            };
            // Reason for fallback: line with no comment delimiter uses entire rest segment as age string.
            let age_val = rest.split('#').next().unwrap_or("").trim();
            if !age_val.is_empty() {
                let set = derived_age.entry(age_val.to_string()).or_default();
                if let Some((start_hex, end_hex)) =
                    range_part.trim().split_once("..")
                {
                    if let (Ok(start), Ok(end)) = (
                        u32::from_str_radix(start_hex.trim(), 16),
                        u32::from_str_radix(end_hex.trim(), 16),
                    ) {
                        for cp in start..=end {
                            set.insert(cp);
                        }
                    }
                } else if let Ok(cp) =
                    u32::from_str_radix(range_part.trim(), 16)
                {
                    set.insert(cp);
                }
            }
        }
    }

    // 8. Parse PropList.txt for Unified_Ideograph
    let mut unified_ideographs: HashSet<u32> = HashSet::new();
    if let Some(content) = get_ucd_file_for_version(version, "PropList.txt") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((range_part, rest)) = trimmed.split_once(';') else {
                continue;
            };
            // Reason for fallback: line with no comment delimiter uses entire rest segment as property name.
            let prop = rest.split('#').next().unwrap_or("").trim();
            if prop == "Unified_Ideograph" {
                if let Some((start_hex, end_hex)) =
                    range_part.trim().split_once("..")
                {
                    if let (Ok(start), Ok(end)) = (
                        u32::from_str_radix(start_hex.trim(), 16),
                        u32::from_str_radix(end_hex.trim(), 16),
                    ) {
                        for cp in start..=end {
                            unified_ideographs.insert(cp);
                        }
                    }
                } else if let Ok(cp) =
                    u32::from_str_radix(range_part.trim(), 16)
                {
                    unified_ideographs.insert(cp);
                }
            }
        }
    }

    // 9. Parse Scripts.txt for Tangut and Khitan_Small_Script
    let mut tangut_ideographs: HashSet<u32> = HashSet::new();
    let mut khitan_characters: HashSet<u32> = HashSet::new();
    if let Some(content) = get_ucd_file_for_version(version, "Scripts.txt") {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((range_part, rest)) = trimmed.split_once(';') else {
                continue;
            };
            // Reason for fallback: line with no comment delimiter has no comment part, using entire rest as script.
            let (script_part, comment) =
                rest.split_once('#').unwrap_or((rest, ""));
            let script = script_part.trim();
            let comment_trimmed = comment.trim();

            if script == "Tangut" {
                if comment_trimmed.contains("TANGUT IDEOGRAPH-") {
                    if let Some((start_hex, end_hex)) =
                        range_part.trim().split_once("..")
                    {
                        if let (Ok(start), Ok(end)) = (
                            u32::from_str_radix(start_hex.trim(), 16),
                            u32::from_str_radix(end_hex.trim(), 16),
                        ) {
                            for cp in start..=end {
                                tangut_ideographs.insert(cp);
                            }
                        }
                    } else if let Ok(cp) =
                        u32::from_str_radix(range_part.trim(), 16)
                    {
                        tangut_ideographs.insert(cp);
                    }
                }
            } else if script == "Khitan_Small_Script"
                && comment_trimmed.contains("KHITAN SMALL SCRIPT CHARACTER-")
            {
                if let Some((start_hex, end_hex)) =
                    range_part.trim().split_once("..")
                {
                    if let (Ok(start), Ok(end)) = (
                        u32::from_str_radix(start_hex.trim(), 16),
                        u32::from_str_radix(end_hex.trim(), 16),
                    ) {
                        for cp in start..=end {
                            khitan_characters.insert(cp);
                        }
                    }
                } else if let Ok(cp) =
                    u32::from_str_radix(range_part.trim(), 16)
                {
                    khitan_characters.insert(cp);
                }
            }
        }
    }

    UnicodeDataTables {
        blocks,
        name_aliases,
        char_data,
        standardized_variants,
        unikemet_data,
        tangut_data,
        derived_age,
        unified_ideographs,
        tangut_ideographs,
        khitan_characters,
    }
}

pub static TABLES_V17: LazyLock<UnicodeDataTables> =
    LazyLock::new(|| load_tables(UnicodeVersion::V17_0));
pub static TABLES_V16: LazyLock<UnicodeDataTables> =
    LazyLock::new(|| load_tables(UnicodeVersion::V16_0));
pub static TABLES_V15_1: LazyLock<UnicodeDataTables> =
    LazyLock::new(|| load_tables(UnicodeVersion::V15_1));
pub static TABLES_V15_0: LazyLock<UnicodeDataTables> =
    LazyLock::new(|| load_tables(UnicodeVersion::V15_0));

pub fn get_tables(version: UnicodeVersion) -> &'static UnicodeDataTables {
    match version {
        UnicodeVersion::V17_0 => &TABLES_V17,
        UnicodeVersion::V16_0 => &TABLES_V16,
        UnicodeVersion::V15_1 => &TABLES_V15_1,
        UnicodeVersion::V15_0 => &TABLES_V15_0,
    }
}

/// Table of Khitan Small Script character meanings from KhitanMeanings.csv.
pub static KHITAN_DATA: LazyLock<HashMap<u32, String>> = LazyLock::new(|| {
    let Some(content) = get_character_descriptions_file("KhitanMeanings.csv")
    else {
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

/// Unihan reading and definition data from `Unihan_Readings.txt`.
#[derive(Debug, Default, Clone)]
pub struct UnihanReadingEntry {
    pub definition: Option<String>,
    pub mandarin: Option<String>,
    pub cantonese: Option<String>,
    pub japanese: Option<String>,
    pub korean: Option<String>,
    pub vietnamese: Option<String>,
}

/// Table of Unihan readings and definitions mapped by codepoint.
pub static UNIHAN_DATA: LazyLock<HashMap<u32, UnihanReadingEntry>> =
    LazyLock::new(|| {
        let Some(content) = ctb_storage_asset_bundle::get_asset_utf8(
            "data/Unicode/Unicode-17.0.0/Unihan/Unihan/Unihan_Readings.txt",
        )
        .ok() else {
            return HashMap::new();
        };
        let mut map: HashMap<u32, UnihanReadingEntry> = HashMap::new();
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
                "kDefinition" => {
                    entry.definition = Some(val.trim().to_string());
                }
                "kMandarin" => entry.mandarin = Some(val.trim().to_string()),
                "kCantonese" => entry.cantonese = Some(val.trim().to_string()),
                "kJapanese" | "kJapaneseKun" | "kJapaneseOn" => {
                    if entry.japanese.is_none() {
                        entry.japanese = Some(val.trim().to_string());
                    }
                }
                "kKorean" | "kHangul" => {
                    if entry.korean.is_none() {
                        entry.korean = Some(val.trim().to_string());
                    }
                }
                "kVietnamese" => {
                    entry.vietnamese = Some(val.trim().to_string());
                }
                _ => {}
            }
        }
        map
    });

/// Checks if a codepoint is a Noncharacter according to Unicode PropList.txt.
pub fn is_noncharacter(cp: u32) -> bool {
    if (0xFDD0..=0xFDEF).contains(&cp) {
        return true;
    }
    let low16 = cp & 0xFFFF;
    (low16 == 0xFFFE || low16 == 0xFFFF) && cp <= 0x10FFFF
}

/// Finds the block name for a codepoint in the given version if it belongs to one.
pub fn find_block_with_version(
    version: UnicodeVersion,
    cp: u32,
) -> Option<&'static str> {
    let tables = get_tables(version);
    for b in &tables.blocks {
        if cp >= b.start && cp <= b.end {
            return Some(&b.name);
        }
    }
    None
}

/// Finds the block name for a codepoint in Unicode 17.0.
pub fn find_block(cp: u32) -> Option<&'static str> {
    find_block_with_version(UnicodeVersion::V17_0, cp)
}

/// Checks if a codepoint is an assigned character in Unicode 17.0.
pub fn is_assigned_unicode(cp: u32) -> bool {
    get_tables(UnicodeVersion::V17_0).is_assigned(cp)
}
