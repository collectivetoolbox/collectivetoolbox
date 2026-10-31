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
//! `SupportedDecompressionFormat`, `SupportedHashFormat`, `SupportedDcInputFormat`,
//! `SupportedDcOutputFormat`) and conversion implementations from the system graph
//! in `assets/data/graph/`.

use crate::graph_index::BuildGraphIndex;
use crate::license_consts::DEFAULT_AGPL_HEADER;
use crate::shorthand::DcShorthand;
use anyhow::{Context, Result, ensure};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default)]
struct SupportCell {
    general: i8,
    dc: i8,
}

#[derive(Debug, Clone)]
struct FormatSupportRow {
    short_id: u32,
    ident: String,
    label: String,
    extensions: Vec<String>,
    nicknames: Vec<String>,
    import_support: SupportCell,
    export_support: SupportCell,
}

impl FormatSupportRow {
    fn primary_cli_name(&self) -> String {
        for nick in &self.nicknames {
            let n = nick.trim().to_ascii_lowercase();
            if !n.is_empty() {
                return n;
            }
        }
        self.ident.to_ascii_lowercase()
    }
}

fn clean_extension_token(part: &str) -> Option<String> {
    let p = part.trim();
    if p.is_empty() {
        return None;
    }
    let clean = p.strip_prefix("case:").unwrap_or(p).trim();
    let no_dot = clean.strip_prefix('.').unwrap_or(clean).trim();
    if no_dot.is_empty() {
        None
    } else {
        Some(no_dot.to_string())
    }
}

fn read_support_rows_for_script(
    index: &BuildGraphIndex,
    script_name: &str,
    import_map: &BTreeMap<DcShorthand, (i8, i8)>,
    export_map: &BTreeMap<DcShorthand, (i8, i8)>,
) -> Result<Vec<FormatSupportRow>> {
    let sh_name = index.require_ident("CanonicalName")?;
    let sh_nick = index.require_ident("Nickname")?;
    let sh_file_ext = index.require_ident("FileExtension")?;

    let mut rows = Vec::new();
    for short_id in index.all_format_short_ids()? {
        let fid_usize = usize::try_from(short_id).with_context(|| {
            format!("Format short ID {short_id} exceeds usize::MAX")
        })?;
        let subj = DcShorthand::Format(fid_usize);
        if index.subject_script(subj) != Some(script_name) {
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

        let extensions: Vec<String> = index
            .multi_property_strings_preferred_first(subj, sh_file_ext)
            .into_iter()
            .filter_map(|s| clean_extension_token(&s))
            .collect();

        let nicknames: Vec<String> = index
            .multi_property_strings_preferred_first(subj, sh_nick)
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Reason for fallback: formats without an explicit support statement on CollectiveToolbox have level (0, 0).
        let (in_gen, in_dc) = import_map.get(&subj).copied().unwrap_or((0, 0));
        // Reason for fallback: formats without an explicit support statement on CollectiveToolbox have level (0, 0).
        let (out_gen, out_dc) =
            export_map.get(&subj).copied().unwrap_or((0, 0));

        rows.push(FormatSupportRow {
            short_id,
            ident,
            label,
            extensions,
            nicknames,
            import_support: SupportCell {
                general: in_gen,
                dc: in_dc,
            },
            export_support: SupportCell {
                general: out_gen,
                dc: out_dc,
            },
        });
    }

    Ok(rows)
}

fn emit_supported_enum(
    out: &mut String,
    enum_name: &str,
    doc_comment: &str,
    domain_desc: &str,
    rows: &[FormatSupportRow],
    use_clap: bool,
    default_ident: Option<&str>,
    backward_compat_consts: &[(&str, &str)],
    extra_from_format_id_arms: &[(&str, &str)],
) {
    let _ = writeln!(out, "/// {doc_comment}");
    out.push_str(
        "#[expect(\n    non_camel_case_types,\n    reason = \"Variant names align with canonical FormatId identifier naming\"\n)]\n",
    );
    if use_clap {
        if default_ident.is_some() {
            out.push_str(
                "#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]\n",
            );
        } else {
            out.push_str(
                "#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n",
            );
        }
    } else if default_ident.is_some() {
        out.push_str(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]\n",
        );
    } else {
        out.push_str(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\n",
        );
    }
    let mut ordered_rows: Vec<FormatSupportRow> = rows.to_vec();
    if use_clap {
        ordered_rows.sort_by(|a, b| {
            a.primary_cli_name()
                .cmp(&b.primary_cli_name())
                .then_with(|| a.ident.cmp(&b.ident))
        });
    }
    let rows = ordered_rows.as_slice();

    let _ = writeln!(out, "pub enum {enum_name} {{");
    let mut used_value_names: HashSet<String> = HashSet::new();
    let mut row_value_names: Vec<Vec<String>> =
        Vec::with_capacity(rows.len());

    if use_clap {
        for row in rows {
            let mut names: Vec<String> = Vec::new();
            for nick in &row.nicknames {
                let n = nick.to_ascii_lowercase();
                if !n.is_empty()
                    && !names.contains(&n)
                    && used_value_names.insert(n.clone())
                {
                    names.push(n);
                }
            }
            let lower_ident = row.ident.to_ascii_lowercase();
            if !names.contains(&lower_ident)
                && used_value_names.insert(lower_ident.clone())
            {
                names.push(lower_ident);
            }
            if !names.contains(&row.ident)
                && used_value_names.insert(row.ident.clone())
            {
                names.push(row.ident.clone());
            }
            let shorthand = format!("f{}", row.short_id);
            if !names.contains(&shorthand)
                && used_value_names.insert(shorthand.clone())
            {
                names.push(shorthand);
            }
            for &(extra_fmt, target_var) in extra_from_format_id_arms {
                if target_var == row.ident {
                    let lower_extra = extra_fmt.to_ascii_lowercase();
                    if !names.contains(&lower_extra)
                        && used_value_names.insert(lower_extra.clone())
                    {
                        names.push(lower_extra);
                    }
                    let exact_extra = extra_fmt.to_string();
                    if !names.contains(&exact_extra)
                        && used_value_names.insert(exact_extra.clone())
                    {
                        names.push(exact_extra);
                    }
                }
            }
            row_value_names.push(names);
        }

        for (row, names) in rows.iter().zip(row_value_names.iter_mut()) {
            for ext in &row.extensions {
                let clean = ext.trim_start_matches('.').to_ascii_lowercase();
                if !clean.is_empty()
                    && !clean.starts_with('~')
                    && !names.contains(&clean)
                    && used_value_names.insert(clean.clone())
                {
                    names.push(clean);
                }
            }
        }
    }

    for (idx, row) in rows.iter().enumerate() {
        let escaped_label = row.label.replace('`', "");
        let _ = writeln!(out, "    /// {escaped_label}");
        if use_clap && default_ident == Some(row.ident.as_str()) {
            out.push_str("    #[default]\n");
        }
        if use_clap {
            if let Some(names) = row_value_names.get(idx) {
                if let Some(primary) = names.first() {
                    out.push_str(&format!("    #[value(name = \"{primary}\""));
                    for alias in names.iter().skip(1) {
                        out.push_str(&format!(", alias = \"{alias}\""));
                    }
                    out.push_str(")]\n");
                }
            }
        }
        let _ = writeln!(out, "    {},", row.ident);
    }
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl {enum_name} {{");
    out.push_str(
        "    /// Ordered list of all formats supported by this capability.\n",
    );
    let _ = writeln!(out, "    pub const ALL: &'static [{enum_name}] = &[");
    for row in rows {
        let _ = writeln!(out, "        Self::{},", row.ident);
    }
    out.push_str("    ];\n\n");

    out.push_str(
        "    /// Ordered list of all canonical `FormatId`s supported by this capability.\n",
    );
    out.push_str("    pub const ALL_FORMAT_IDS: &'static [FormatId] = &[\n");
    for row in rows {
        let _ = writeln!(out, "        FormatId::{},", row.ident);
    }
    out.push_str("    ];\n\n");

    for (const_name, target_variant) in backward_compat_consts {
        let _ = writeln!(
            out,
            "    /// Backward-compatible alias for [`Self::{target_variant}`]."
        );
        let _ = writeln!(
            out,
            "    pub const {const_name}: Self = Self::{target_variant};\n"
        );
    }

    out.push_str(
        "    /// Returns the primary CLI identifier for this format.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn primary_cli_name(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for row in rows {
        let primary = row.primary_cli_name();
        let _ = writeln!(
            out,
            "            Self::{} => \"{primary}\",",
            row.ident
        );
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Maps this variant to the canonical global `FormatId`.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn to_format_id(&self) -> FormatId {\n");
    out.push_str("        match self {\n");
    for row in rows {
        let _ = writeln!(
            out,
            "            Self::{} => FormatId::{},",
            row.ident, row.ident
        );
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    let _ = writeln!(
        out,
        "    /// Converts a global `FormatId` to `{enum_name}` if supported."
    );
    out.push_str("    #[must_use]\n");
    out.push_str(
        "    pub const fn from_format_id(id: FormatId) -> Option<Self> {\n",
    );
    out.push_str("        match id {\n");
    for row in rows {
        let _ = writeln!(
            out,
            "            FormatId::{} => Some(Self::{}),",
            row.ident, row.ident
        );
    }
    for (extra_fmt, target_var) in extra_from_format_id_arms {
        if rows.iter().any(|r| r.ident == *target_var) {
            let _ = writeln!(
                out,
                "            FormatId::{extra_fmt} => Some(Self::{target_var}),"
            );
        }
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Returns true if the given `FormatId` is supported by this capability.\n",
    );
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn is_supported(id: FormatId) -> bool {\n");
    out.push_str("        Self::from_format_id(id).is_some()\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl From<{enum_name}> for FormatId {{");
    let _ = writeln!(out, "    fn from(fmt: {enum_name}) -> Self {{");
    out.push_str("        fmt.to_format_id()\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl TryFrom<FormatId> for {enum_name} {{");
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str(
        "    fn try_from(id: FormatId) -> Result<Self, Self::Error> {\n",
    );
    let _ = writeln!(
        out,
        "        Self::from_format_id(id).ok_or_else(|| anyhow::anyhow!(\"Format {{id:?}} is not a supported {domain_desc}\"))"
    );
    out.push_str("    }\n");
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl TryFrom<&str> for {enum_name} {{");
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str("    fn try_from(s: &str) -> Result<Self, Self::Error> {\n");
    if use_clap {
        out.push_str(
            "        if let Ok(val) = <Self as clap::ValueEnum>::from_str(s, true) {\n",
        );
        out.push_str("            return Ok(val);\n");
        out.push_str("        }\n");
    }
    out.push_str("        let clean = s.trim().to_ascii_lowercase();\n");
    out.push_str("        if let Some(id) = FormatId::from_ident(&clean) {\n");
    out.push_str("            if let Some(fmt) = Self::from_format_id(id) {\n");
    out.push_str("                return Ok(fmt);\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    let _ = writeln!(
        out,
        "        anyhow::bail!(\"Unknown {domain_desc}: '{{s}}'\")"
    );
    out.push_str("    }\n");
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl TryFrom<String> for {enum_name} {{");
    out.push_str("    type Error = anyhow::Error;\n\n");
    out.push_str("    fn try_from(s: String) -> Result<Self, Self::Error> {\n");
    out.push_str("        Self::try_from(s.as_str())\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");

    let _ = writeln!(out, "impl std::str::FromStr for {enum_name} {{");
    out.push_str("    type Err = anyhow::Error;\n\n");
    out.push_str("    fn from_str(s: &str) -> Result<Self, Self::Err> {\n");
    out.push_str("        Self::try_from(s)\n");
    out.push_str("    }\n");
    out.push_str("}\n\n");
}

/// Generates the contents of `format_support.generated.rs` from a [`BuildGraphIndex`].
///
/// # Errors
/// Returns an error if required ontology entities or support declarations are missing.
pub fn generate_format_support_code_from_index(
    index: &BuildGraphIndex,
) -> Result<String> {
    let (import_map, export_map) = index.collective_toolbox_format_support()?;

    let comp_rows = read_support_rows_for_script(
        index,
        ".Formats:compression",
        &import_map,
        &export_map,
    )?;
    let hash_rows = read_support_rows_for_script(
        index,
        ".Formats:hash",
        &import_map,
        &export_map,
    )?;
    let doc_rows = read_support_rows_for_script(
        index,
        ".Formats:document",
        &import_map,
        &export_map,
    )?;
    let enc_rows = read_support_rows_for_script(
        index,
        ".Formats:encoding",
        &import_map,
        &export_map,
    )?;
    let db_rows = read_support_rows_for_script(
        index,
        ".Formats:database",
        &import_map,
        &export_map,
    )?;

    let decomp_supported: Vec<FormatSupportRow> = comp_rows
        .iter()
        .filter(|r| r.import_support.general >= 2)
        .cloned()
        .collect();
    let comp_supported: Vec<FormatSupportRow> = comp_rows
        .iter()
        .filter(|r| r.export_support.general >= 2)
        .cloned()
        .collect();

    let hash_supported: Vec<FormatSupportRow> = hash_rows
        .iter()
        .filter(|r| {
            r.export_support.general >= 2 || r.import_support.general >= 2
        })
        .cloned()
        .collect();

    let dc_input_family_mappings = [
        ("CscJournal1Meta1", "CscJournal"),
        ("CscJournal1Meta2", "CscJournal"),
        ("CscJournal1Meta3", "CscJournal"),
        ("CscJournal4", "CscJournal"),
        ("CscJournal5", "CscJournal"),
    ];

    let dc_in_primary: Vec<FormatSupportRow> = doc_rows
        .iter()
        .chain(enc_rows.iter())
        .chain(db_rows.iter())
        .filter(|r| {
            r.import_support.dc >= 1
                && !dc_input_family_mappings
                    .iter()
                    .any(|(src, _)| *src == r.ident.as_str())
        })
        .cloned()
        .collect();

    let dc_out_rows: Vec<FormatSupportRow> = doc_rows
        .iter()
        .chain(enc_rows.iter())
        .chain(db_rows.iter())
        .filter(|r| {
            r.export_support.dc >= 1
                && !dc_input_family_mappings
                    .iter()
                    .any(|(src, _)| *src == r.ident.as_str())
        })
        .cloned()
        .collect();

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str(
        "\n//! @generated by ctb-build-support::format_support_codegen from format category data tables.\n",
    );
    out.push_str(
        "//! Operational format support enums and conversion implementations.\n\n",
    );
    out.push_str("use crate::FormatId;\n");
    out.push_str("use anyhow::Result;\n\n");

    emit_supported_enum(
        &mut out,
        "SupportedDecompressionFormat",
        "Single-stream compression formats supported for decompression / extraction by the workspace.",
        "decompression format",
        &decomp_supported,
        false,
        None,
        &[],
        &[],
    );

    emit_supported_enum(
        &mut out,
        "SupportedCompressionFormat",
        "Single-stream compression formats supported for compression / encoding by the workspace.",
        "compression format",
        &comp_supported,
        false,
        None,
        &[],
        &[],
    );

    let hash_family_mappings = [
        ("Fnv0", "Fnv0_64"),
        ("Fnv1", "Fnv1_64"),
        ("Fnv1a", "Fnv1a_64"),
        ("FowlerNollVo", "Fnv1a_64"),
    ];

    emit_supported_enum(
        &mut out,
        "SupportedHashFormat",
        "Cryptographic and non-cryptographic hash algorithms supported by the workspace.",
        "hash algorithm",
        &hash_supported,
        false,
        None,
        &[],
        &hash_family_mappings,
    );

    emit_supported_enum(
        &mut out,
        "SupportedDcInputFormat",
        "Document and text formats supported for conversion to Document Character String (DcMstring / DcString).",
        "Dc input format",
        &dc_in_primary,
        true,
        Some("Utf8"),
        &[("Dcil", "DcIntegerList")],
        &dc_input_family_mappings,
    );

    emit_supported_enum(
        &mut out,
        "SupportedDcOutputFormat",
        "Document and text formats supported for conversion from Document Character String (DcMstring / DcString).",
        "Dc output format",
        &dc_out_rows,
        true,
        Some("Utf8"),
        &[],
        &dc_input_family_mappings,
    );

    out.push_str("impl FormatId {\n");
    out.push_str(
        "    /// Converts this `FormatId` to [`SupportedDecompressionFormat`] if supported.\n",
    );
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str(
        "    /// Returns an error if this format is not supported for decompression.\n",
    );
    out.push_str(
        "    pub fn to_supported_decompression_format(self) -> Result<SupportedDecompressionFormat> {\n",
    );
    out.push_str("        SupportedDecompressionFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Converts this `FormatId` to [`SupportedCompressionFormat`] if supported.\n",
    );
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str(
        "    /// Returns an error if this format is not supported for compression.\n",
    );
    out.push_str(
        "    pub fn to_supported_compression_format(self) -> Result<SupportedCompressionFormat> {\n",
    );
    out.push_str("        SupportedCompressionFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Converts this `FormatId` to [`SupportedHashFormat`] if supported.\n",
    );
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str(
        "    /// Returns an error if this format is not a supported hash algorithm.\n",
    );
    out.push_str(
        "    pub fn to_supported_hash_format(self) -> Result<SupportedHashFormat> {\n",
    );
    out.push_str("        SupportedHashFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Converts this `FormatId` to [`SupportedDcInputFormat`] if supported.\n",
    );
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str(
        "    /// Returns an error if this format is not a supported Dc input format.\n",
    );
    out.push_str(
        "    pub fn to_supported_dc_input_format(self) -> Result<SupportedDcInputFormat> {\n",
    );
    out.push_str("        SupportedDcInputFormat::try_from(self)\n");
    out.push_str("    }\n\n");

    out.push_str(
        "    /// Converts this `FormatId` to [`SupportedDcOutputFormat`] if supported.\n",
    );
    out.push_str("    ///\n");
    out.push_str("    /// # Errors\n");
    out.push_str(
        "    /// Returns an error if this format is not a supported Dc output format.\n",
    );
    out.push_str(
        "    pub fn to_supported_dc_output_format(self) -> Result<SupportedDcOutputFormat> {\n",
    );
    out.push_str("        SupportedDcOutputFormat::try_from(self)\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generates the contents of `format_support.generated.rs` from the system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or contains invalid data.
pub fn generate_format_support_code(graph_dir: &Path) -> Result<String> {
    let index = BuildGraphIndex::from_graph_dir(graph_dir)?;
    generate_format_support_code_from_index(&index)
}

/// Generates `src/utilities/format_support.generated.rs` if its contents have changed.
///
/// # Errors
/// Returns an error if reading the system graph or writing the generated file fails.
pub fn generate_format_support_file(base_dir: &Path) -> Result<bool> {
    let repo_root = crate::find_repository_root_from(base_dir)?;
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let out_file = repo_root
        .join("src")
        .join("utilities")
        .join("format_support.generated.rs");

    ensure!(
        graph_dir.is_dir(),
        "Could not locate system graph directory at {}",
        graph_dir.display()
    );

    let code = generate_format_support_code(&graph_dir)?;

    if let Ok(existing) = fs::read_to_string(&out_file) {
        if existing == code {
            return Ok(false);
        }
    }

    fs::write(&out_file, code).with_context(|| {
        format!("Failed to write {}", out_file.display())
    })?;
    Ok(true)
}
