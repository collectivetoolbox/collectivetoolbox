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

//! Codegen for character encoding definitions, line ending mappings, and
//! `encoding.generated.rs` from format category CSV data tables.

use anyhow::{Context, Result, ensure};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::find_repository_root_from;
use crate::license_consts::DEFAULT_AGPL_HEADER;

fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.is_file() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(false);
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

/// Converts a PascalCase or camelCase identifier string to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    let mut prev_is_upper = false;
    let mut prev_is_digit = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
            prev_is_upper = true;
            prev_is_digit = false;
        } else if c.is_ascii_digit() {
            if i > 0 && !prev_is_digit {
                res.push('_');
            }
            res.push(c);
            prev_is_upper = false;
            prev_is_digit = true;
        } else {
            res.push(c);
            prev_is_upper = false;
            prev_is_digit = false;
        }
    }
    res
}

/// Extracts `@default_line_ending(...)` annotation from a string.
fn extract_default_line_ending_annotation(s: &str) -> Option<String> {
    let prefix = "@default_line_ending(";
    if let Some(pos) = s.find(prefix) {
        let rest = s.get(pos.saturating_add(prefix.len())..)?;
        if let Some(end) = rest.find(')') {
            let inner = rest.get(..end).unwrap_or("").trim().trim_matches('"');
            if !inner.is_empty() {
                return Some(inner.to_ascii_lowercase());
            }
        }
    }
    None
}

/// Parses a line ending string/identifier into the corresponding Rust `LineEndingKind` path.
fn parse_line_ending_kind_path(s: &str) -> Option<&'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "lf" | "line_ending_lf" | "lineendinglf" => Some("LineEndingKind::Lf"),
        "cr" | "line_ending_cr" | "lineendingcr" => Some("LineEndingKind::Cr"),
        "crlf" | "line_ending_crlf" | "lineendingcrlf" => Some("LineEndingKind::CrLf"),
        "lfcr" | "line_ending_lfcr" | "lineendinglfcr" => Some("LineEndingKind::LfCr"),
        "rs" | "line_ending_rs" | "lineendingrs" => Some("LineEndingKind::Rs"),
        "nl" | "line_ending_nl" | "lineendingnl" => Some("LineEndingKind::Nl"),
        _ => None,
    }
}

/// Delimiter literal and documentation for a line ending kind.
fn delimiter_for_kind(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "Lf" => ("\\n", "POSIX / Unix newline (`\\n`, LF, 0x0A)"),
        "Cr" => ("\\r", "Classic Macintosh newline (`\\r`, CR, 0x0D)"),
        "CrLf" => ("\\r\\n", "Windows / DOS newline (`\\r\\n`, CRLF, 0x0D 0x0A)"),
        "LfCr" => ("\\n\\r", "Acorn / RISC OS newline (`\\n\\r`, LFCR, 0x0A 0x0D)"),
        "Rs" => ("\\x1E", "QNX traditional Record Separator (`\\x1E`, RS, 0x1E)"),
        "Nl" => ("\\u{0085}", "IBM / EBCDIC Next Line (`\\u{0085}`, NEL)"),
        _ => ("\\n", "Newline delimiter"),
    }
}

/// A parsed record from `v.lineEndings.csv`.
#[derive(Debug, Clone)]
struct LineEndingRecord {
    ident: String,
    mode: &'static str,
    kind: String,
}

/// Simple single-byte format entry parsed from `encoding.csv`.
#[derive(Debug, Clone)]
struct SimpleEncodingRecord {
    ident: String,
    label: String,
    default_line_ending: &'static str,
}

/// Generates the contents of `encoding.generated.rs` from format category CSV tables.
///
/// # Errors
/// Returns an error if reading CSV tables fails.
pub fn generate_encoding_code(formats_dir: &Path) -> Result<String> {
    let line_endings_csv = formats_dir.join("v.lineEndings.csv");
    ensure!(
        line_endings_csv.is_file(),
        "Could not locate v.lineEndings.csv at {}",
        line_endings_csv.display()
    );

    let encoding_csv = formats_dir.join("encoding.csv");
    ensure!(
        encoding_csv.is_file(),
        "Could not locate encoding.csv at {}",
        encoding_csv.display()
    );

    // 1. Parse line ending records from v.lineEndings.csv
    let mut line_ending_records = Vec::new();
    let mut line_ending_kinds = Vec::new();
    let mut le_rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&line_endings_csv)
        .with_context(|| {
            format!("Failed to open CSV file {}", line_endings_csv.display())
        })?;

    for result in le_rdr.records() {
        let record = result.with_context(|| {
            format!("Failed to read record in {}", line_endings_csv.display())
        })?;
        let ident = record.get(2).unwrap_or("").trim().to_string();
        if ident.is_empty() {
            continue;
        }

        let (mode, kind) = if let Some(k) = ident.strip_prefix("LineEnding") {
            ("Terminated", k.to_string())
        } else if let Some(k) = ident.strip_prefix("LineSeparator") {
            ("Separated", k.to_string())
        } else {
            continue;
        };

        if !line_ending_kinds.contains(&kind) {
            line_ending_kinds.push(kind.clone());
        }

        line_ending_records.push(LineEndingRecord {
            ident,
            mode,
            kind,
        });
    }

    // 2. Parse encoding records from encoding.csv
    let mut default_endings: HashMap<String, &'static str> = HashMap::new();
    let mut simple_encodings = Vec::new();

    let mut encoding_rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(&encoding_csv)
        .with_context(|| {
            format!("Failed to open CSV file {}", encoding_csv.display())
        })?;

    for result in encoding_rdr.records() {
        let record = result.with_context(|| {
            format!("Failed to read record in {}", encoding_csv.display())
        })?;
        let ident = record.get(2).unwrap_or("").trim().to_string();
        if ident.is_empty() {
            continue;
        }
        let category = record.get(4).unwrap_or("").trim();
        if category != "encoding" {
            continue;
        }

        let label = record.get(3).unwrap_or("").trim().to_string();
        let col5 = record.get(5).unwrap_or("");
        let comments = record.get(15).unwrap_or("");

        let ending_path = extract_default_line_ending_annotation(col5)
            .or_else(|| extract_default_line_ending_annotation(comments))
            .and_then(|raw| parse_line_ending_kind_path(&raw))
            .unwrap_or("LineEndingKind::Lf");

        default_endings.insert(ident.clone(), ending_path);

        // Simple single-byte / 7-bit encodings (excluding multi-byte and configurable families)
        if matches!(
            ident.as_str(),
            "MacRoman" | "Win1252" | "Iso88591" | "Ascii"
        ) {
            simple_encodings.push(SimpleEncodingRecord {
                ident,
                label,
                default_line_ending: ending_path,
            });
        }
    }

    let cp437_ending = default_endings.get("Cp437").copied().unwrap_or("LineEndingKind::CrLf");
    let neo_ending = default_endings.get("AlphaSmartNeo").copied().unwrap_or("LineEndingKind::Cr");

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str(
        "//! Character encoding definitions and settings for table-driven single-byte encodings.\n\
        //! @generated by ctb-build-support::encoding_codegen from format category data tables.\n\
        //! Do not edit by hand.\n\n\
        #[expect(\n\
            unused_imports,\n\
            clippy::wildcard_imports,\n\
            reason = \"Standard workspace module prelude\"\n\
        )]\n\
        use crate::utilities::*;\n\
        use ctb_utilities::FormatId;\n\
        use ctb_utilities::dc_char::DcChar;\n\n\
        /// Mode for handling low character codes (0x00..=0x1F) in single-byte encodings.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]\n\
        pub enum LowArea {\n\
            /// Graphical symbols (e.g. Neo graphical symbols, CP437 dingbats).\n\
            #[default]\n\
            Graphical,\n\
            /// Control characters (standard C0 control codes).\n\
            Control,\n\
        }\n\n\
        /// Regional character layout for Neo encodings.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]\n\
        pub enum NeoRegion {\n\
            /// United States layout.\n\
            #[default]\n\
            Us,\n\
            /// Ukrainian Macintosh layout.\n\
            UaMac,\n\
            /// Ukrainian PC layout.\n\
            UaPc,\n\
        }\n\n\
        impl NeoRegion {\n\
            /// Returns the dedicated `FormatId` for this Neo regional variant and low-area mode.\n\
            #[must_use]\n\
            pub const fn to_format_id(self, low_area: LowArea) -> FormatId {\n\
                match (self, low_area) {\n\
                    (Self::Us, LowArea::Graphical) => FormatId::AlphaSmartNeoLowGrUs,\n\
                    (Self::UaMac, LowArea::Graphical) => FormatId::AlphaSmartNeoLowGrUaMac,\n\
                    (Self::UaPc, LowArea::Graphical) => FormatId::AlphaSmartNeoLowGrUaPc,\n\
                    (Self::Us, LowArea::Control) => FormatId::AlphaSmartNeoLowCtlUs,\n\
                    (Self::UaMac, LowArea::Control) => FormatId::AlphaSmartNeoLowCtlUaMac,\n\
                    (Self::UaPc, LowArea::Control) => FormatId::AlphaSmartNeoLowCtlUaPc,\n\
                }\n\
            }\n\n\
            /// Returns the dedicated `DcChar` constant for this Neo variant if known.\n\
            #[must_use]\n\
            pub const fn dc_char(self, low_area: LowArea) -> Option<DcChar> {\n\
                self.to_format_id(low_area).dc_char()\n\
            }\n\
        }\n\n\
        /// Line ending delimiter pattern.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]\n\
        pub enum LineEndingKind {\n",
    );

    for kind in &line_ending_kinds {
        let (_, doc) = delimiter_for_kind(kind);
        if kind == "Lf" {
            out.push_str("    /// ");
            out.push_str(doc);
            out.push_str(".\n    #[default]\n    Lf,\n");
        } else {
            out.push_str(&format!("    /// {doc}.\n    {kind},\n"));
        }
    }

    out.push_str(
        "}\n\n\
        impl LineEndingKind {\n\
            /// Returns the string representation of this line ending delimiter.\n\
            #[must_use]\n\
            pub const fn as_str(self) -> &'static str {\n\
                match self {\n",
    );

    for kind in &line_ending_kinds {
        let (del, _) = delimiter_for_kind(kind);
        out.push_str(&format!("                    Self::{kind} => \"{del}\",\n"));
    }

    out.push_str(
        "                }\n\
            }\n\n\
            /// Returns the byte sequence for this line ending in UTF-8.\n\
            #[must_use]\n\
            pub const fn as_bytes(self) -> &'static [u8] {\n\
                self.as_str().as_bytes()\n\
            }\n\n\
            /// Creates a `LineEndingFormat` with terminated mode for this delimiter kind.\n\
            #[must_use]\n\
            pub const fn terminated(self) -> LineEndingFormat {\n\
                LineEndingFormat::terminated(self)\n\
            }\n\n\
            /// Creates a `LineEndingFormat` with separated mode for this delimiter kind.\n\
            #[must_use]\n\
            pub const fn separated(self) -> LineEndingFormat {\n\
                LineEndingFormat::separated(self)\n\
            }\n\n\
            /// Maps this terminated line ending kind to its corresponding `FormatId`.\n\
            #[must_use]\n\
            pub const fn to_format_id(self) -> FormatId {\n\
                self.terminated().to_format_id()\n\
            }\n\n\
            /// Returns the dedicated `DcChar` for this line ending in terminated mode.\n\
            #[must_use]\n\
            pub const fn dc_char(self) -> Option<DcChar> {\n\
                self.to_format_id().dc_char()\n\
            }\n\
        }\n\n\
        /// Mode defining whether newlines terminate every line or only separate lines.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]\n\
        pub enum TerminationMode {\n\
            /// Every line including the last line is terminated by the newline sequence.\n\
            #[default]\n\
            Terminated,\n\
            /// Newline sequences only appear between lines; no trailing terminator on final line.\n\
            Separated,\n\
        }\n\n\
        /// Full specification of line ending style and termination mode.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]\n\
        pub struct LineEndingFormat {\n\
            /// The line ending delimiter pattern.\n\
            pub kind: LineEndingKind,\n\
            /// Whether the delimiter terminates all lines or only separates them.\n\
            pub mode: TerminationMode,\n\
        }\n\n\
        impl LineEndingFormat {\n\
            /// Creates a new `LineEndingFormat` with terminated mode.\n\
            #[must_use]\n\
            pub const fn terminated(kind: LineEndingKind) -> Self {\n\
                Self {\n\
                    kind,\n\
                    mode: TerminationMode::Terminated,\n\
                }\n\
            }\n\n\
            /// Creates a new `LineEndingFormat` with separated mode.\n\
            #[must_use]\n\
            pub const fn separated(kind: LineEndingKind) -> Self {\n\
                Self {\n\
                    kind,\n\
                    mode: TerminationMode::Separated,\n\
                }\n\
            }\n\n\
            /// Maps this line ending format to its canonical `FormatId`.\n\
            #[must_use]\n\
            pub const fn to_format_id(self) -> FormatId {\n\
                match (self.mode, self.kind) {\n",
    );

    for r in &line_ending_records {
        out.push_str(&format!(
            "                    (TerminationMode::{}, LineEndingKind::{}) => FormatId::{},\n",
            r.mode, r.kind, r.ident
        ));
    }

    out.push_str(
        "                }\n\
            }\n\n\
            /// Converts a `FormatId` to the corresponding `LineEndingFormat` if it represents a line ending.\n\
            #[must_use]\n\
            pub const fn from_format_id(id: FormatId) -> Option<Self> {\n\
                match id {\n",
    );

    for r in &line_ending_records {
        let ctor = if r.mode == "Terminated" { "terminated" } else { "separated" };
        out.push_str(&format!(
            "                    FormatId::{} => Some(Self::{}(LineEndingKind::{})),\n",
            r.ident, ctor, r.kind
        ));
    }

    out.push_str(
        "                    _ => None,\n\
                }\n\
            }\n\n\
            /// Returns the dedicated `DcChar` for this line ending format if known.\n\
            #[must_use]\n\
            pub const fn dc_char(self) -> Option<DcChar> {\n\
                self.to_format_id().dc_char()\n\
            }\n\
        }\n\n\
        /// Option controlling line ending conversion during encoding, decoding, and transcoding.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]\n\
        pub enum LineEndingOption {\n\
            /// Keep line endings as they are (no conversion / pure character conversion).\n\
            #[default]\n\
            Preserve,\n\
            /// Convert to the idiomatic line ending for the target character encoding.\n\
            EncodingDefault,\n\
            /// Convert to a specific line ending format.\n\
            Specific(LineEndingFormat),\n\
        }\n\n\
        /// Structured single-byte character encoding settings.\n\
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n\
        pub enum CharEncoding {\n\
            /// Code Page 437 (DOS Latin US).\n\
            Cp437 {\n\
                /// Low-area mode (graphical dingbats vs control codes).\n\
                low_area: LowArea,\n\
                /// Whether alternative Unicode variants are included in reverse encoding.\n\
                include_variants: bool,\n\
            },\n\
            /// Neo character encoding.\n\
            Neo {\n\
                /// Regional layout variant.\n\
                region: NeoRegion,\n\
                /// Low-area mode.\n\
                low_area: LowArea,\n\
            },\n",
    );

    // Simple single-byte variants
    for s in &simple_encodings {
        let variant_name = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
        out.push_str(&format!("            /// {}.\n            {variant_name},\n", s.label));
    }

    out.push_str(
        "        }\n\n\
        impl CharEncoding {\n\
            /// Default CP437 encoding with graphical dingbats and variant aliases.\n\
            #[must_use]\n\
            pub const fn cp437() -> Self {\n\
                Self::Cp437 {\n\
                    low_area: LowArea::Graphical,\n\
                    include_variants: true,\n\
                }\n\
            }\n\n\
            /// CP437 encoding with control characters in low area.\n\
            #[must_use]\n\
            pub const fn cp437_control() -> Self {\n\
                Self::Cp437 {\n\
                    low_area: LowArea::Control,\n\
                    include_variants: true,\n\
                }\n\
            }\n\n\
            /// Standard Neo US layout with graphical low area.\n\
            #[must_use]\n\
            pub const fn neo_us() -> Self {\n\
                Self::Neo {\n\
                    region: NeoRegion::Us,\n\
                    low_area: LowArea::Graphical,\n\
                }\n\
            }\n\n\
            /// Neo encoding with custom region and low area.\n\
            #[must_use]\n\
            pub const fn neo(region: NeoRegion, low_area: LowArea) -> Self {\n\
                Self::Neo { region, low_area }\n\
            }\n\n",
    );

    // Generate constructor methods dynamically for simple encodings
    for s in &simple_encodings {
        let variant_name = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
        let fn_name = to_snake_case(variant_name);
        out.push_str(&format!(
            "            /// {}.\n\
            #[must_use]\n\
            pub const fn {fn_name}() -> Self {{\n\
                Self::{variant_name}\n\
            }}\n\n",
            s.label
        ));
        if s.ident == "Win1252" {
            out.push_str(
                "            /// Windows-1252 (\"ANSI\") encoding alias.\n\
            #[must_use]\n\
            pub const fn win1252() -> Self {\n\
                Self::Windows1252\n\
            }\n\n",
            );
        } else if s.ident == "Iso88591" {
            out.push_str(
                "            /// ISO 8859-1 encoding alias.\n\
            #[must_use]\n\
            pub const fn iso_8859_1() -> Self {\n\
                Self::Iso88591\n\
            }\n\n",
            );
        }
    }

    out.push_str(
        "            /// Maps this character encoding to its canonical `FormatId`.\n\
            #[must_use]\n\
            pub const fn to_format_id(self) -> FormatId {\n\
                match self {\n\
                    Self::Cp437 { .. } => FormatId::Cp437,\n\
                    Self::Neo { region, low_area } => region.to_format_id(low_area),\n",
    );

    for s in &simple_encodings {
        let variant_name = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
        out.push_str(&format!("                    Self::{variant_name} => FormatId::{},\n", s.ident));
    }

    out.push_str(
        "                }\n\
            }\n\n\
            /// Converts a `FormatId` to the corresponding `CharEncoding` if it represents an encoding.\n\
            #[must_use]\n\
            pub const fn from_format_id(id: FormatId) -> Option<Self> {\n\
                match id {\n\
                    FormatId::Cp437 => Some(Self::cp437()),\n\
                    FormatId::AlphaSmartNeo => Some(Self::neo_us()),\n\
                    FormatId::AlphaSmartNeoLowGrUs => Some(Self::neo(NeoRegion::Us, LowArea::Graphical)),\n\
                    FormatId::AlphaSmartNeoLowGrUaMac => Some(Self::neo(NeoRegion::UaMac, LowArea::Graphical)),\n\
                    FormatId::AlphaSmartNeoLowGrUaPc => Some(Self::neo(NeoRegion::UaPc, LowArea::Graphical)),\n\
                    FormatId::AlphaSmartNeoLowCtlUs => Some(Self::neo(NeoRegion::Us, LowArea::Control)),\n\
                    FormatId::AlphaSmartNeoLowCtlUaMac => Some(Self::neo(NeoRegion::UaMac, LowArea::Control)),\n\
                    FormatId::AlphaSmartNeoLowCtlUaPc => Some(Self::neo(NeoRegion::UaPc, LowArea::Control)),\n",
    );

    for s in &simple_encodings {
        let variant_name = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
        out.push_str(&format!("                    FormatId::{} => Some(Self::{variant_name}),\n", s.ident));
    }

    out.push_str(
        "                    _ => None,\n\
                }\n\
            }\n\n\
            /// Returns the dedicated `DcChar` constant for this character encoding if known.\n\
            #[must_use]\n\
            pub const fn dc_char(self) -> Option<DcChar> {\n\
                self.to_format_id().dc_char()\n\
            }\n\n\
            /// Returns the idiomatic / natural default line ending for this character encoding.\n\
            #[must_use]\n\
            pub const fn default_line_ending(self) -> LineEndingKind {\n\
                match self {\n",
    );

    out.push_str(&format!("                    Self::Cp437 {{ .. }} => {cp437_ending},\n"));
    out.push_str(&format!("                    Self::Neo {{ .. }} => {neo_ending},\n"));
    for s in &simple_encodings {
        let variant_name = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
        out.push_str(&format!("                    Self::{variant_name} => {},\n", s.default_line_ending));
    }

    out.push_str(
        "                }\n\
            }\n\n\
            /// Checks whether the specified line ending can be encoded in this character encoding.\n\
            #[must_use]\n\
            pub const fn supports_line_ending(self, ending: LineEndingKind) -> bool {\n\
                match ending {\n\
                    LineEndingKind::Lf\n\
                    | LineEndingKind::Cr\n\
                    | LineEndingKind::CrLf\n\
                    | LineEndingKind::LfCr => true,\n\
                    LineEndingKind::Rs => match self {\n",
    );

    let simple_variant_patterns = simple_encodings
        .iter()
        .map(|s| {
            let var = if s.ident == "Win1252" { "Windows1252" } else { &s.ident };
            format!("Self::{var}")
        })
        .collect::<Vec<_>>()
        .join(" | ");

    out.push_str(&format!("                        {simple_variant_patterns} => true,\n"));
    out.push_str(
        "                        Self::Cp437 { low_area, .. } | Self::Neo { low_area, .. } => {\n\
                            matches!(low_area, LowArea::Control)\n\
                        }\n\
                    },\n\
                    LineEndingKind::Nl => false,\n\
                }\n\
            }\n\
        }\n\n\
        impl Default for CharEncoding {\n\
            fn default() -> Self {\n\
                Self::cp437()\n\
            }\n\
        }\n",
    );

    Ok(out)
}

/// Generates or updates `src/formats/utilities/encoding.generated.rs` if contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_encoding_file(base_dir: &Path) -> Result<bool> {
    let repo_root = find_repository_root_from(base_dir)?;
    let formats_dir = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats");
    ensure!(
        formats_dir.is_dir(),
        "Could not locate formats directory at {}",
        formats_dir.display()
    );

    let target_file = repo_root
        .join("src")
        .join("formats")
        .join("utilities")
        .join("encoding.generated.rs");

    let code = generate_encoding_code(&formats_dir)?;
    write_if_changed(&target_file, &code)
}
