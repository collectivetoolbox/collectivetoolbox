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

//! Build-time code generator for `src/formats/utilities/encoding.generated.rs`
//! from the system graph in `assets/data/graph/`.

use crate::graph_index::BuildGraphIndex;
use crate::license_consts::DEFAULT_AGPL_HEADER;
use crate::shorthand::DcShorthand;
use anyhow::{Context, Result, bail, ensure};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct LineEndingRecord {
    ident: String,
    kind_variant: String,
    as_str_expr: &'static str,
    doc_summary: &'static str,
}

#[derive(Debug, Clone)]
struct EncodingRecord {
    ident: String,
    label: String,
    default_line_ending: Option<String>,
}

fn line_ending_kind_metadata(
    kind_variant: &str,
) -> Result<(&'static str, &'static str)> {
    match kind_variant {
        "Lf" => Ok(("\"\\n\"", "POSIX / Unix newline (`\\n`, LF, 0x0A).")),
        "Cr" => Ok(("\"\\r\"", "Classic Macintosh newline (`\\r`, CR, 0x0D).")),
        "CrLf" => Ok((
            "\"\\r\\n\"",
            "Windows / DOS newline (`\\r\\n`, CRLF, 0x0D 0x0A).",
        )),
        "LfCr" => Ok((
            "\"\\n\\r\"",
            "Acorn / RISC OS newline (`\\n\\r`, LFCR, 0x0A 0x0D).",
        )),
        "Rs" => Ok((
            "\"\\x1E\"",
            "QNX traditional Record Separator (`\\x1E`, RS, 0x1E).",
        )),
        "Nl" => Ok((
            "\"\\u{0085}\"",
            "IBM / EBCDIC Next Line (`\\u{0085}`, NEL).",
        )),
        other => bail!(
            "Unsupported line ending kind variant '{other}' in v:lineEndings"
        ),
    }
}

fn read_line_endings_from_index(
    index: &BuildGraphIndex,
) -> Result<(Vec<LineEndingRecord>, Vec<LineEndingRecord>)> {
    let mut terminated = Vec::new();
    let mut separated = Vec::new();

    for short_id in index.all_format_short_ids()? {
        let fid_usize = usize::try_from(short_id).with_context(|| {
            format!("Format short ID {short_id} exceeds usize::MAX")
        })?;
        let subj = DcShorthand::Format(fid_usize);
        if index.subject_script(subj) != Some(".Formats:v:lineEndings") {
            continue;
        }
        let Some(ident_raw) = index.ident_for_shorthand(subj) else {
            continue;
        };
        let ident = ident_raw.trim();

        if let Some(kind_variant) = ident.strip_prefix("LineEnding") {
            let (as_str_expr, doc_summary) =
                line_ending_kind_metadata(kind_variant)?;
            terminated.push(LineEndingRecord {
                ident: ident.to_string(),
                kind_variant: kind_variant.to_string(),
                as_str_expr,
                doc_summary,
            });
        } else if let Some(kind_variant) = ident.strip_prefix("LineSeparator") {
            let (as_str_expr, doc_summary) =
                line_ending_kind_metadata(kind_variant)?;
            separated.push(LineEndingRecord {
                ident: ident.to_string(),
                kind_variant: kind_variant.to_string(),
                as_str_expr,
                doc_summary,
            });
        }
    }

    ensure!(
        !terminated.is_empty(),
        "No LineEnding* records found in system graph (.Formats:v:lineEndings)"
    );
    ensure!(
        terminated.len() == separated.len(),
        "Mismatched LineEnding* ({}) and LineSeparator* ({}) counts in system graph",
        terminated.len(),
        separated.len()
    );

    Ok((terminated, separated))
}

fn read_encodings_from_index(
    index: &BuildGraphIndex,
) -> Result<Vec<EncodingRecord>> {
    let sh_name = index.require_ident("CanonicalName")?;
    let sh_default_var = index.require_ident("DefaultFormatVariant")?;

    let mut records = Vec::new();
    for short_id in index.all_format_short_ids()? {
        let fid_usize = usize::try_from(short_id).with_context(|| {
            format!("Format short ID {short_id} exceeds usize::MAX")
        })?;
        let subj = DcShorthand::Format(fid_usize);
        if index.subject_script(subj) != Some(".Formats:encoding") {
            continue;
        }
        let Some(ident_raw) = index.ident_for_shorthand(subj) else {
            continue;
        };
        let ident = ident_raw.trim().to_string();
        // Reason for fallback: subjects returned by all_format_short_ids have a CanonicalName.
        let label = index
            .single_property_string(subj, sh_name)
            .unwrap_or_default()
            .trim()
            .to_string();

        let default_line_ending = index
            .single_property(subj, sh_default_var)
            .and_then(|var_sh| index.ident_for_shorthand(var_sh))
            .and_then(|var_ident| {
                var_ident
                    .strip_prefix("LineEnding")
                    .or_else(|| var_ident.strip_prefix("LineSeparator"))
            })
            .map(str::to_owned);

        records.push(EncodingRecord {
            ident,
            label,
            default_line_ending,
        });
    }

    Ok(records)
}

/// Converts a `FormatId` ident for a simple single-byte encoding into its
/// `(CharEncoding variant, constructor fn, optional alias constructor fn)`.
fn simple_encoding_mapping(
    format_ident: &str,
) -> Option<(&'static str, &'static str, Option<&'static str>)> {
    match format_ident {
        "Ascii" => Some(("Ascii", "ascii", None)),
        "MacRoman" => Some(("MacRoman", "mac_roman", None)),
        "Win1252" => Some(("Windows1252", "windows_1252", Some("win1252"))),
        "Iso88591" => Some(("Iso88591", "iso_88591", Some("iso_8859_1"))),
        _ => None,
    }
}

/// Generates the contents of `encoding.generated.rs` from a [`BuildGraphIndex`].
///
/// # Errors
/// Returns an error if required ontology entities or encoding definitions are missing.
pub fn generate_encoding_code_from_index(
    index: &BuildGraphIndex,
) -> Result<String> {
    let (terminated, separated) = read_line_endings_from_index(index)?;
    let encodings = read_encodings_from_index(index)?;

    let mut simple_encodings = Vec::new();
    let mut cp437_default_le = None;
    let mut neo_default_le = None;

    for rec in &encodings {
        if rec.ident == "Cp437" {
            cp437_default_le.clone_from(&rec.default_line_ending);
        } else if rec.ident == "AlphaSmartNeo" {
            neo_default_le.clone_from(&rec.default_line_ending);
        } else if let Some((enum_var, ctor, alias_ctor)) =
            simple_encoding_mapping(&rec.ident)
        {
            // Reason for fallback: encodings without an explicit DefaultFormatVariant use POSIX LF (`LineEndingKind::Lf`).
            let le = rec
                .default_line_ending
                .clone()
                .unwrap_or_else(|| "Lf".to_string());
            simple_encodings.push((
                rec.ident.clone(),
                enum_var,
                ctor,
                alias_ctor,
                rec.label.clone(),
                le,
            ));
        }
    }

    let cp437_le = cp437_default_le.context(
        "Cp437 in system graph (.Formats:encoding) is missing DefaultFormatVariant",
    )?;
    let neo_le = neo_default_le.context(
        "AlphaSmartNeo in system graph (.Formats:encoding) is missing DefaultFormatVariant",
    )?;

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str(
        "\n\n//! Character encoding definitions and settings for table-driven single-byte encodings.\n\
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

    for (idx, le) in terminated.iter().enumerate() {
        let _ = writeln!(out, "    /// {}", le.doc_summary);
        if idx == 0 {
            out.push_str("    #[default]\n");
        }
        let _ = writeln!(out, "    {},", le.kind_variant);
    }
    out.push_str("}\n\n");

    out.push_str(
        "impl LineEndingKind {\n\
/// Returns the string representation of this line ending delimiter.\n\
#[must_use]\n\
pub const fn as_str(self) -> &'static str {\n\
match self {\n",
    );
    for le in &terminated {
        let _ = writeln!(
            out,
            "                    Self::{} => {},",
            le.kind_variant, le.as_str_expr
        );
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

    for le in &terminated {
        let _ = writeln!(
            out,
            "                    (TerminationMode::Terminated, LineEndingKind::{}) => FormatId::{},",
            le.kind_variant, le.ident
        );
    }
    for le in &separated {
        let _ = writeln!(
            out,
            "                    (TerminationMode::Separated, LineEndingKind::{}) => FormatId::{},",
            le.kind_variant, le.ident
        );
    }
    out.push_str(
        "                }\n\
}\n\n\
/// Converts a `FormatId` to the corresponding `LineEndingFormat` if it represents a line ending.\n\
#[must_use]\n\
pub const fn from_format_id(id: FormatId) -> Option<Self> {\n\
match id {\n",
    );

    for le in &terminated {
        let _ = writeln!(
            out,
            "                    FormatId::{} => Some(Self::terminated(LineEndingKind::{})),",
            le.ident, le.kind_variant
        );
    }
    for le in &separated {
        let _ = writeln!(
            out,
            "                    FormatId::{} => Some(Self::separated(LineEndingKind::{})),",
            le.ident, le.kind_variant
        );
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

    for (_fmt_ident, enum_var, _ctor, _alias_ctor, label, _le) in
        &simple_encodings
    {
        let _ = writeln!(out, "            /// {label}.");
        let _ = writeln!(out, "            {enum_var},");
    }
    out.push_str("        }\n\n");

    out.push_str(
        "impl CharEncoding {\n\
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

    for (_fmt_ident, enum_var, ctor, alias_ctor, label, _le) in
        &simple_encodings
    {
        let _ = writeln!(out, "            /// {label}.");
        let _ = writeln!(
            out,
            "#[must_use]\npub const fn {ctor}() -> Self {{\nSelf::{enum_var}\n}}\n"
        );
        if let Some(alias) = alias_ctor {
            let _ = writeln!(out, "            /// {label} encoding alias.");
            let _ = writeln!(
                out,
                "#[must_use]\npub const fn {alias}() -> Self {{\nSelf::{enum_var}\n}}\n"
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
    for (fmt_ident, enum_var, _ctor, _alias_ctor, _label, _le) in
        &simple_encodings
    {
        let _ = writeln!(
            out,
            "                    Self::{enum_var} => FormatId::{fmt_ident},"
        );
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
    for (fmt_ident, enum_var, _ctor, _alias_ctor, _label, _le) in
        &simple_encodings
    {
        let _ = writeln!(
            out,
            "                    FormatId::{fmt_ident} => Some(Self::{enum_var}),"
        );
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
    let _ = writeln!(
        out,
        "                    Self::Cp437 {{ .. }} => LineEndingKind::{cp437_le},"
    );
    let _ = writeln!(
        out,
        "                    Self::Neo {{ .. }} => LineEndingKind::{neo_le},"
    );
    for (_fmt_ident, enum_var, _ctor, _alias_ctor, _label, le) in
        &simple_encodings
    {
        let _ = writeln!(
            out,
            "                    Self::{enum_var} => LineEndingKind::{le},"
        );
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

    let simple_pat: Vec<&str> = simple_encodings
        .iter()
        .map(|(_, enum_var, _, _, _, _)| *enum_var)
        .collect();
    let joined_simple = simple_pat
        .iter()
        .map(|v| format!("Self::{v}"))
        .collect::<Vec<_>>()
        .join(" | ");
    let _ = writeln!(out, "                        {joined_simple} => true,");
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

/// Generates the contents of `encoding.generated.rs` from the system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or contains invalid data.
pub fn generate_encoding_code(graph_dir: &Path) -> Result<String> {
    let index = BuildGraphIndex::from_graph_dir(graph_dir)?;
    generate_encoding_code_from_index(&index)
}

/// Generates `src/formats/utilities/encoding.generated.rs` if its contents have changed.
///
/// # Errors
/// Returns an error if reading the system graph or writing the generated file fails.
pub fn generate_encoding_file(base_dir: &Path) -> Result<bool> {
    let repo_root = crate::find_repository_root_from(base_dir)?;
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let out_file = repo_root
        .join("src")
        .join("formats")
        .join("utilities")
        .join("encoding.generated.rs");

    ensure!(
        graph_dir.is_dir(),
        "Could not locate system graph directory at {}",
        graph_dir.display()
    );

    let code = generate_encoding_code(&graph_dir)?;

    if let Ok(existing) = fs::read_to_string(&out_file) {
        if existing == code {
            return Ok(false);
        }
    }

    fs::write(&out_file, code)
        .with_context(|| format!("Failed to write {}", out_file.display()))?;
    Ok(true)
}
