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

//! Unified Document Character Definition (`DcDefn`) and format metadata container.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::format_spec::FormatExpr;
use crate::syntax::DcSyntaxRule;
use serde::{Deserialize, Serialize};

/// Standard Unicode General Category codes and Collective non-Unicode extension (`!Cx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeneralCategory {
    Lu,
    Ll,
    Lt,
    Lm,
    Lo,
    Mn,
    Mc,
    Me,
    Nd,
    Nl,
    No,
    Pc,
    Pd,
    Ps,
    Pe,
    Pi,
    Pf,
    Po,
    Sm,
    Sc,
    Sk,
    So,
    Zs,
    Zl,
    Zp,
    Cc,
    Cf,
    Cs,
    Co,
    Cn,
    #[serde(rename = "!Cx")]
    NonUnicodeControl,
}

impl GeneralCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lu => "Lu",
            Self::Ll => "Ll",
            Self::Lt => "Lt",
            Self::Lm => "Lm",
            Self::Lo => "Lo",
            Self::Mn => "Mn",
            Self::Mc => "Mc",
            Self::Me => "Me",
            Self::Nd => "Nd",
            Self::Nl => "Nl",
            Self::No => "No",
            Self::Pc => "Pc",
            Self::Pd => "Pd",
            Self::Ps => "Ps",
            Self::Pe => "Pe",
            Self::Pi => "Pi",
            Self::Pf => "Pf",
            Self::Po => "Po",
            Self::Sm => "Sm",
            Self::Sc => "Sc",
            Self::Sk => "Sk",
            Self::So => "So",
            Self::Zs => "Zs",
            Self::Zl => "Zl",
            Self::Zp => "Zp",
            Self::Cc => "Cc",
            Self::Cf => "Cf",
            Self::Cs => "Cs",
            Self::Co => "Co",
            Self::Cn => "Cn",
            Self::NonUnicodeControl => "!Cx",
        }
    }
}

impl std::fmt::Display for GeneralCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for GeneralCategory {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "Lu" => Ok(Self::Lu),
            "Ll" => Ok(Self::Ll),
            "Lt" => Ok(Self::Lt),
            "Lm" => Ok(Self::Lm),
            "Lo" => Ok(Self::Lo),
            "Mn" => Ok(Self::Mn),
            "Mc" => Ok(Self::Mc),
            "Me" => Ok(Self::Me),
            "Nd" => Ok(Self::Nd),
            "Nl" => Ok(Self::Nl),
            "No" => Ok(Self::No),
            "Pc" => Ok(Self::Pc),
            "Pd" => Ok(Self::Pd),
            "Ps" => Ok(Self::Ps),
            "Pe" => Ok(Self::Pe),
            "Pi" => Ok(Self::Pi),
            "Pf" => Ok(Self::Pf),
            "Po" => Ok(Self::Po),
            "Sm" => Ok(Self::Sm),
            "Sc" => Ok(Self::Sc),
            "Sk" => Ok(Self::Sk),
            "So" => Ok(Self::So),
            "Zs" => Ok(Self::Zs),
            "Zl" => Ok(Self::Zl),
            "Zp" => Ok(Self::Zp),
            "Cc" => Ok(Self::Cc),
            "Cf" => Ok(Self::Cf),
            "Cs" => Ok(Self::Cs),
            "Co" => Ok(Self::Co),
            "Cn" => Ok(Self::Cn),
            "!Cx" => Ok(Self::NonUnicodeControl),
            other => bail!(
                "Invalid General Category '{other}' (expected standard Unicode category or '!Cx')"
            ),
        }
    }
}

/// Standard Unicode Bidirectional Class codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BidiClass {
    L,
    R,
    AL,
    EN,
    ES,
    ET,
    AN,
    CS,
    NSM,
    BN,
    B,
    S,
    WS,
    ON,
    LRE,
    LRO,
    RLE,
    RLO,
    PDF,
    LRI,
    RLI,
    FSI,
    PDI,
}

impl BidiClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L => "L",
            Self::R => "R",
            Self::AL => "AL",
            Self::EN => "EN",
            Self::ES => "ES",
            Self::ET => "ET",
            Self::AN => "AN",
            Self::CS => "CS",
            Self::NSM => "NSM",
            Self::BN => "BN",
            Self::B => "B",
            Self::S => "S",
            Self::WS => "WS",
            Self::ON => "ON",
            Self::LRE => "LRE",
            Self::LRO => "LRO",
            Self::RLE => "RLE",
            Self::RLO => "RLO",
            Self::PDF => "PDF",
            Self::LRI => "LRI",
            Self::RLI => "RLI",
            Self::FSI => "FSI",
            Self::PDI => "PDI",
        }
    }
}

impl std::fmt::Display for BidiClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BidiClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "L" => Ok(Self::L),
            "R" => Ok(Self::R),
            "AL" => Ok(Self::AL),
            "EN" => Ok(Self::EN),
            "ES" => Ok(Self::ES),
            "ET" => Ok(Self::ET),
            "AN" => Ok(Self::AN),
            "CS" => Ok(Self::CS),
            "NSM" => Ok(Self::NSM),
            "BN" => Ok(Self::BN),
            "B" => Ok(Self::B),
            "S" => Ok(Self::S),
            "WS" => Ok(Self::WS),
            "ON" => Ok(Self::ON),
            "LRE" => Ok(Self::LRE),
            "LRO" => Ok(Self::LRO),
            "RLE" => Ok(Self::RLE),
            "RLO" => Ok(Self::RLO),
            "PDF" => Ok(Self::PDF),
            "LRI" => Ok(Self::LRI),
            "RLI" => Ok(Self::RLI),
            "FSI" => Ok(Self::FSI),
            "PDI" => Ok(Self::PDI),
            other => bail!(
                "Invalid Bidi Class '{other}' (expected standard Unicode bidi class)"
            ),
        }
    }
}

/// Parsed two-level format support specification (`<general> <dc>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SupportLevel {
    /// General format support (e.g. streaming, compression, execution, parsing).
    pub general: i32,
    /// Dc document conversion support (converting to/from Dc syntax structures).
    pub dc: i32,
}

impl SupportLevel {
    /// Parses a support level specification.
    /// Empty string returns `SupportLevel { general: 0, dc: 0 }`.
    /// Non-empty input MUST contain exactly two space-separated integers in range `-1..=5`.
    /// Single values are rejected.
    pub fn parse(val: &str) -> Result<Self> {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            return Ok(Self { general: 0, dc: 0 });
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let (first, second) = match parts.as_slice() {
            [first, second] => (*first, *second),
            _ => bail!(
                "Invalid support level '{val}' (expected exactly two space-separated levels '<general> <dc>' in range -1..=5, or blank)"
            ),
        };
        let parse_part = |part: &str| -> Result<i32> {
            match part {
                "-1" | "0" | "1" | "2" | "3" | "4" | "5" => {
                    part.parse::<i32>().context("Parsing support level integer")
                }
                _ => bail!(
                    "Invalid support level component '{part}' (expected -1..=5)"
                ),
            }
        };
        let general = parse_part(first)?;
        let dc = parse_part(second)?;
        Ok(Self { general, dc })
    }

    /// Formats the support level as `"<general> <dc>"`, or empty string if both are 0.
    #[must_use]
    pub fn format_csv(&self) -> String {
        if self.general == 0 && self.dc == 0 {
            String::new()
        } else {
            format!("{} {}", self.general, self.dc)
        }
    }
}

/// Detailed format-specific properties for entries representing file/data formats.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FormatDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implies: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub based_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_spec: Option<FormatExpr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uti: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nicknames: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_support: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_support: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_types: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_line_ending: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vary_as: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_shorthands",
        deserialize_with = "deserialize_shorthands"
    )]
    pub varies: Vec<ctb_storage_minimal::shorthand::DcShorthand>,
}

fn serialize_shorthands<S>(
    shorthands: &[ctb_storage_minimal::shorthand::DcShorthand],
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(shorthands.len()))?;
    for sh in shorthands {
        seq.serialize_element(&sh.to_string())?;
    }
    seq.end()
}

fn deserialize_shorthands<'de, D>(
    deserializer: D,
) -> std::result::Result<
    Vec<ctb_storage_minimal::shorthand::DcShorthand>,
    D::Error,
>
where
    D: serde::Deserializer<'de>,
{
    let strings = Vec::<String>::deserialize(deserializer)?;
    let mut res = Vec::with_capacity(strings.len());
    for s in strings {
        let sh = ctb_storage_minimal::shorthand::DcShorthand::parse(&s)
            .map_err(serde::de::Error::custom)?;
        res.push(sh);
    }
    Ok(res)
}

fn serialize_opt_shorthand<S>(
    shorthand: &Option<ctb_storage_minimal::shorthand::DcShorthand>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match shorthand {
        Some(sh) => serializer.serialize_some(&sh.to_string()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_opt_shorthand<'de, D>(
    deserializer: D,
) -> std::result::Result<
    Option<ctb_storage_minimal::shorthand::DcShorthand>,
    D::Error,
>
where
    D: serde::Deserializer<'de>,
{
    let opt_str = Option::<String>::deserialize(deserializer)?;
    match opt_str {
        Some(s) => {
            let sh = ctb_storage_minimal::shorthand::DcShorthand::parse(&s)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(sh))
        }
        None => Ok(None),
    }
}

/// Unified Document Character definition for characters and formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcDefn {
    // Identity & Core Display
    pub dc_id: u128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stable_ident: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub name: String,
    pub category: String,

    // Character Metadata (flattened across all Dcs)
    pub combining_class: u8,
    pub bidi_class: BidiClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casing_partner: Option<u32>,
    pub general_category: GeneralCategory,
    pub script: String,
    pub is_deprecated: bool,
    pub decompositions: Vec<String>,

    // Cross-references & Aliases
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formal_aliases: Vec<(String, String)>,
    pub cross_references: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub based_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub os_associations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub magic_names: Vec<String>,

    // Syntax Rule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<DcSyntaxRule>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_shorthand",
        deserialize_with = "deserialize_opt_shorthand"
    )]
    pub closes: Option<ctb_storage_minimal::shorthand::DcShorthand>,

    // Documentation
    pub description: String,

    // Format-Specific Attributes container (None for character Dcs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatDetails>,

    // Source Origin
    pub source_file: String,
    pub line_number: usize,
}
