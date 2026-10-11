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

//! Codegen for operational format support enums (`SupportedCompressionFormat`,
//! `SupportedDecompressionFormat`, `SupportedHashFormat`) and conversion implementations
//! from format category CSV data tables.

use anyhow::{Context, Result, ensure};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct SupportLevel {
    general: i32,
    dc: i32,
}

fn parse_support_level(s: &str) -> SupportLevel {
    let mut parts = s.split_whitespace();
    let general = parts.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
    let dc = parts.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
    SupportLevel { general, dc }
}

/// A parsed format support record from category CSV tables.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FormatSupportRecord {
    ident: String,
    label: String,
    extensions: Vec<String>,
    nicknames: Vec<String>,
    import_support: SupportLevel,
    export_support: SupportLevel,
}

fn read_support_records(csv_path: &Path) -> Result<Vec<FormatSupportRecord>> {
    let content = fs::read_to_string(csv_path)
        .with_context(|| format!("Reading {}", csv_path.display()))?;
    let mut records = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    for result in rdr.records() {
        let record = result?;
        let get = |idx: usize| -> String {
            record.get(idx).unwrap_or("").trim().to_string()
        };

        let ident = get(2);
        let label = get(3);
        let extensions: Vec<String> = get(6)
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let nicknames: Vec<String> = get(10)
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let import_support = parse_support_level(&get(11));
        let export_support = parse_support_level(&get(12));

        if !ident.is_empty() {
            records.push(FormatSupportRecord {
                ident,
                label,
                extensions,
                nicknames,
                import_support,
                export_support,
            });
        }
    }
    Ok(records)
}

fn generate_enum_code(
    enum_name: &str,
    capability_doc: &str,
    records: &[&FormatSupportRecord],
    extra_family_mappings: &[(&str, &str)],
    error_name: &str,
    is_value_enum: bool,
    default_variant: Option<&str>,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("/// {capability_doc}\n"));
    out.push_str("#[expect(\n");
    out.push_str("    non_camel_case_types,\n");
    out.push_str("    reason = \"Variant names align with canonical FormatId identifier naming\"\n");
    out.push_str(")]\n");
    if is_value_enum {
        if default_variant.is_some() {
            out.push_str("#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]\n");
        } else {
            out.push_str("#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n");
        }
    } else {
        out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n");
    }
    out.push_str(&format!("pub enum {enum_name} {{\n"));
    for r in records {
        let escaped_label = r.label.replace('`', "");
        out.push_str(&format!("    /// {escaped_label}\n"));
        if is_value_enum && default_variant == Some(r.ident.as_str()) {
            out.push_str("    #[default]\n");
        }
        if is_value_enum {
            let mut names: Vec<String> = Vec::new();
            for nick in &r.nicknames {
                let n = nick.to_ascii_lowercase();
                if !names.contains(&n) {
                    names.push(n);
                }
            }
            let lower_ident = r.ident.to_ascii_lowercase();
            if !names.contains(&lower_ident) {
                names.push(lower_ident);
            }
            for ext in &r.extensions {
                let clean = ext.trim_start_matches('.').to_ascii_lowercase();
                if !clean.is_empty() && !names.contains(&clean) {
                    names.push(clean);
                }
            }
            if let Some(primary) = names.first() {
                out.push_str(&format!("    #[value(name = \"{primary}\""));
                for alias in names.iter().skip(1) {
                    out.push_str(&format!(", alias = \"{alias}\""));
                }
                out.push_str(")]\n");
            }
        }
        out.push_str(&format!("    {},\n", r.ident));
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl {enum_name} {{\n"));
    out.push_str(&format!("    /// Ordered list of all formats supported by this capability.\n"));
    out.push_str(&format!("    pub const ALL: &'static [{enum_name}] = &[\n"));
    for r in records {
        out.push_str(&format!("        Self::{},\n", r.ident));
    }
    out.push_str("    ];\n\n");

    out.push_str("    /// Ordered list of all canonical `FormatId`s supported by this capability.\n");
    out.push_str("    pub const ALL_FORMAT_IDS: &'static [FormatId] = &[\n");
    for r in records {
        out.push_str(&format!("        FormatId::{},\n", r.ident));
    }
    out.push_str("    ];\n\n");
    if enum_name == "SupportedDcInputFormat" {
        out.push_str("    /// Backward-compatible alias for [`Self::DcIntegerList`].\n");
        out.push_str("    pub const Dcil: Self = Self::DcIntegerList;\n\n");
    }

    out.push_str("    /// Maps this variant to the canonical global `FormatId`.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn to_format_id(&self) -> FormatId {\n");
    out.push_str("        match self {\n");
    for r in records {
        out.push_str(&format!("            Self::{} => FormatId::{},\n", r.ident, r.ident));
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(&format!("    /// Converts a global `FormatId` to `{enum_name}` if supported.\n"));
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn from_format_id(id: FormatId) -> Option<Self> {\n");
    out.push_str("        match id {\n");
    for r in records {
        out.push_str(&format!("            FormatId::{} => Some(Self::{}),\n", r.ident, r.ident));
    }
    for (src, dst) in extra_family_mappings {
        out.push_str(&format!("            FormatId::{src} => Some(Self::{dst}),\n"));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns true if the given `FormatId` is supported by this capability.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn is_supported(id: FormatId) -> bool {\n");
    out.push_str("        Self::from_format_id(id).is_some()\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str(&format!("impl From<{enum_name}> for FormatId {{\n"));
    out.push_str(&format!("    fn from(fmt: {enum_name}) -> Self {{\n"));
    out.push_str("        fmt.to_format_id()\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str(&format!("impl TryFrom<FormatId> for {enum_name} {{\n"));
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str(&format!("    fn try_from(id: FormatId) -> Result<Self, Self::Error> {{\n"));
    out.push_str(&format!(
        "        Self::from_format_id(id).ok_or_else(|| anyhow::anyhow!(\"Format {{id:?}} is not a supported {error_name}\"))\n"
    ));
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str(&format!("impl TryFrom<&str> for {enum_name} {{\n"));
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str(&format!("    fn try_from(s: &str) -> Result<Self, Self::Error> {{\n"));
    if is_value_enum {
        out.push_str("        if let Ok(val) = <Self as clap::ValueEnum>::from_str(s, true) {\n");
        out.push_str("            return Ok(val);\n");
        out.push_str("        }\n");
    }
    out.push_str("        let clean = s.trim().to_ascii_lowercase();\n");
    out.push_str("        if let Some(id) = FormatId::from_ident(&clean) {\n");
    out.push_str("            if let Some(fmt) = Self::from_format_id(id) {\n");
    out.push_str("                return Ok(fmt);\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str(&format!("        anyhow::bail!(\"Unknown {error_name}: '{{s}}'\")\n"));
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str(&format!("impl TryFrom<String> for {enum_name} {{\n"));
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str(&format!("    fn try_from(s: String) -> Result<Self, Self::Error> {{\n"));
    out.push_str("        Self::try_from(s.as_str())\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out.push_str(&format!("impl std::str::FromStr for {enum_name} {{\n"));
    out.push_str("    type Err = anyhow::Error;\n\n");
    out.push_str(&format!("    fn from_str(s: &str) -> Result<Self, Self::Err> {{\n"));
    out.push_str("        Self::try_from(s)\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    out
}

/// Generates code for format support enums and `FormatId` conversion methods.
///
/// # Errors
/// Returns an error if reading or parsing category CSV files fails.
pub fn generate_format_support_code(
    compression_csv: &Path,
    hash_csv: &Path,
    document_csv: &Path,
    encoding_csv: &Path,
    database_csv: &Path,
) -> Result<String> {
    ensure!(
        compression_csv.is_file(),
        "Could not locate compression.csv at {}",
        compression_csv.display()
    );
    ensure!(
        hash_csv.is_file(),
        "Could not locate hash.csv at {}",
        hash_csv.display()
    );
    ensure!(
        document_csv.is_file(),
        "Could not locate document.csv at {}",
        document_csv.display()
    );
    ensure!(
        encoding_csv.is_file(),
        "Could not locate encoding.csv at {}",
        encoding_csv.display()
    );
    ensure!(
        database_csv.is_file(),
        "Could not locate database.csv at {}",
        database_csv.display()
    );

    let compression_records = read_support_records(compression_csv)?;
    let hash_records = read_support_records(hash_csv)?;
    let document_records = read_support_records(document_csv)?;
    let encoding_records = read_support_records(encoding_csv)?;
    let database_records = read_support_records(database_csv)?;

    let decompress_records: Vec<&FormatSupportRecord> = compression_records
        .iter()
        .filter(|r| r.import_support.general >= 2)
        .collect();

    let compress_records: Vec<&FormatSupportRecord> = compression_records
        .iter()
        .filter(|r| r.export_support.general >= 2)
        .collect();

    let hash_supported_records: Vec<&FormatSupportRecord> = hash_records
        .iter()
        .filter(|r| r.export_support.general >= 2 || r.import_support.general >= 2)
        .collect();

    let dc_input_family_mappings = [
        ("CscJournal1Meta1", "CscJournal"),
        ("CscJournal1Meta2", "CscJournal"),
        ("CscJournal1Meta3", "CscJournal"),
        ("CscJournal4", "CscJournal"),
    ];

    let dc_input_records: Vec<&FormatSupportRecord> = document_records
        .iter()
        .chain(encoding_records.iter())
        .chain(database_records.iter())
        .filter(|r| {
            r.import_support.dc >= 1
                && !dc_input_family_mappings
                    .iter()
                    .any(|(src, _)| *src == r.ident.as_str())
        })
        .collect();

    let dc_output_records: Vec<&FormatSupportRecord> = document_records
        .iter()
        .chain(encoding_records.iter())
        .filter(|r| r.export_support.dc >= 1)
        .collect();

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n//! @generated by ctb-build-support::format_support_codegen from format category data tables.\n");
    out.push_str("//! Operational format support enums and conversion implementations.\n\n");
    out.push_str("use crate::FormatId;\n");
    out.push_str("use anyhow::Result;\n\n");

    out.push_str(&generate_enum_code(
        "SupportedDecompressionFormat",
        "Single-stream compression formats supported for decompression / extraction by the workspace.",
        &decompress_records,
        &[],
        "decompression format",
        false,
        None,
    ));

    out.push_str(&generate_enum_code(
        "SupportedCompressionFormat",
        "Single-stream compression formats supported for compression / encoding by the workspace.",
        &compress_records,
        &[],
        "compression format",
        false,
        None,
    ));

    let hash_family_mappings = [
        ("Fnv0", "Fnv0_64"),
        ("Fnv1", "Fnv1_64"),
        ("Fnv1a", "Fnv1a_64"),
        ("FowlerNollVo", "Fnv1a_64"),
    ];

    out.push_str(&generate_enum_code(
        "SupportedHashFormat",
        "Cryptographic and non-cryptographic hash algorithms supported by the workspace.",
        &hash_supported_records,
        &hash_family_mappings,
        "hash algorithm",
        false,
        None,
    ));

    out.push_str(&generate_enum_code(
        "SupportedDcInputFormat",
        "Document and text formats supported for conversion to Document Character String (DcMstring / DcString).",
        &dc_input_records,
        &dc_input_family_mappings,
        "Dc input format",
        true,
        Some("Utf8"),
    ));

    out.push_str(&generate_enum_code(
        "SupportedDcOutputFormat",
        "Document and text formats supported for conversion from Document Character String (DcMstring / DcString).",
        &dc_output_records,
        &[],
        "Dc output format",
        true,
        Some("Utf8"),
    ));

    out.push_str("impl FormatId {\n");
    out.push_str("    /// Converts this `FormatId` to [`SupportedDecompressionFormat`] if supported.\n");
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str("    /// Returns an error if this format is not supported for decompression.\n");
    out.push_str("    pub fn to_supported_decompression_format(self) -> Result<SupportedDecompressionFormat> {\n");
    out.push_str("        SupportedDecompressionFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Converts this `FormatId` to [`SupportedCompressionFormat`] if supported.\n");
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str("    /// Returns an error if this format is not supported for compression.\n");
    out.push_str("    pub fn to_supported_compression_format(self) -> Result<SupportedCompressionFormat> {\n");
    out.push_str("        SupportedCompressionFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Converts this `FormatId` to [`SupportedHashFormat`] if supported.\n");
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str("    /// Returns an error if this format is not a supported hash algorithm.\n");
    out.push_str("    pub fn to_supported_hash_format(self) -> Result<SupportedHashFormat> {\n");
    out.push_str("        SupportedHashFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Converts this `FormatId` to [`SupportedDcInputFormat`] if supported.\n");
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str("    /// Returns an error if this format is not a supported Dc input format.\n");
    out.push_str("    pub fn to_supported_dc_input_format(self) -> Result<SupportedDcInputFormat> {\n");
    out.push_str("        SupportedDcInputFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Converts this `FormatId` to [`SupportedDcOutputFormat`] if supported.\n");
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str("    /// Returns an error if this format is not a supported Dc output format.\n");
    out.push_str("    pub fn to_supported_dc_output_format(self) -> Result<SupportedDcOutputFormat> {\n");
    out.push_str("        SupportedDcOutputFormat::try_from(self)\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generates `format_support.generated.rs` in `src/utilities/`.
///
/// # Errors
/// Returns an error if repository root lookup or writing fails.
pub fn generate_format_support_file(base_dir: &Path) -> Result<bool> {
    let repo_root = find_repository_root_from(base_dir)?;
    let compression_csv = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats")
        .join("compression.csv");
    let hash_csv = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats")
        .join("hash.csv");
    let document_csv = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats")
        .join("document.csv");
    let encoding_csv = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats")
        .join("encoding.csv");
    let database_csv = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("data")
        .join("categories")
        .join("formats")
        .join("database.csv");

    let target_file = repo_root
        .join("src")
        .join("utilities")
        .join("format_support.generated.rs");

    let code = generate_format_support_code(
        &compression_csv,
        &hash_csv,
        &document_csv,
        &encoding_csv,
        &database_csv,
    )?;
    write_if_changed(&target_file, &code)
}
