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

//! Build-time code generator for `src/formats/dcdata/dc.generated.rs` and
//! `src/formats/dcdata/syntax/role.generated.rs` from the system graph in
//! `assets/data/graph/`.

use crate::ctbnbdl::CtbnbdlFramingDcs;
use crate::graph_index::{
    BuildGraphIndex, BuildSeedEntity,
    resolve_ctbnbdl_framing_dcs_from_graph_dir,
};
use crate::license_consts::DEFAULT_AGPL_HEADER;
use crate::shorthand::DcShorthand;
use anyhow::{Context, Result, bail, ensure};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct DcConstRecord {
    short_id: u32,
    ident: String,
    name: String,
    description: String,
}

/// Validates that an identifier is a valid non-empty Rust identifier.
fn is_valid_rust_ident(ident: &str) -> bool {
    if ident.is_empty() {
        return false;
    }
    let mut chars = ident.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Converts a `PascalCase` or `camelCase` identifier into `SCREAMING_SNAKE_CASE`.
#[must_use]
pub fn to_screaming_snake_case(ident: &str) -> String {
    if ident.chars().all(|c| !c.is_ascii_lowercase()) {
        return ident.to_string();
    }

    let chars: Vec<char> = ident.chars().collect();
    let mut out = String::with_capacity(ident.len().saturating_add(4));

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' {
            if !out.ends_with('_') {
                out.push('_');
            }
            continue;
        }
        if i > 0 && ch.is_ascii_uppercase() {
            let prev = chars.get(i.saturating_sub(1)).copied();
            let next = chars.get(i.saturating_add(1)).copied();
            let prev_is_lower_or_digit = prev.is_some_and(|p| {
                p.is_ascii_lowercase() || p.is_ascii_digit()
            });
            let prev_is_upper = prev.is_some_and(|p| p.is_ascii_uppercase());
            let next_is_lower = next.is_some_and(|n| n.is_ascii_lowercase());

            if (prev_is_lower_or_digit || (prev_is_upper && next_is_lower))
                && !out.ends_with('_')
            {
                out.push('_');
            }
        }
        out.push(ch.to_ascii_uppercase());
    }

    out
}

/// Collects all Short Dc records that declare a `ProgrammaticIdentifier` in `index`.
fn collect_dc_const_records(
    index: &BuildGraphIndex,
) -> Result<Vec<DcConstRecord>> {
    let sh_name = index.require_ident("CanonicalName")?;
    let sh_desc = index.require_ident("Description")?;

    let mut records = Vec::new();
    let mut seen_idents = HashSet::new();
    let mut seen_shorts = HashSet::new();

    for short_id in index.all_short_dc_ids()? {
        let subj = DcShorthand::Short(short_id);
        let Some(ident_raw) = index.ident_for_shorthand(subj) else {
            continue;
        };
        let ident = ident_raw.trim().to_string();
        ensure!(
            is_valid_rust_ident(&ident),
            "Invalid @ident(\"{ident}\") on Dc {short_id}"
        );
        let const_name = format!("DC_{}", to_screaming_snake_case(&ident));
        if !seen_idents.insert(const_name.clone()) {
            bail!(
                "Duplicate @ident(\"{ident}\") (constant {const_name}) on Dc {short_id}"
            );
        }
        if !seen_shorts.insert(short_id) {
            bail!("Duplicate short ID {short_id} with @ident(\"{ident}\")");
        }
        // Reason for fallback: subjects returned by all_short_dc_ids have a CanonicalName.
        let name = index
            .single_property_string(subj, sh_name)
            .unwrap_or_default()
            .trim()
            .to_string();
        // Reason for fallback: Dcs without an explicit description default to empty string.
        let description = index
            .single_property_string(subj, sh_desc)
            .unwrap_or_default()
            .trim()
            .to_string();

        records.push(DcConstRecord {
            short_id,
            ident,
            name,
            description,
        });
    }

    records.sort_by_key(|r| r.short_id);
    Ok(records)
}

/// Generates the contents of `dc.generated.rs` from a pre-built [`BuildGraphIndex`].
///
/// # Errors
/// Returns an error if the graph is missing required ontology Dcs or contains invalid identifiers.
pub fn generate_dc_code_from_index(
    index: &BuildGraphIndex,
) -> Result<String> {
    let records = collect_dc_const_records(index)?;

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str(
        "//! Constants and definitions for Document Character (Dc) data.\n",
    );
    out.push_str(
        "//! @generated by ctb-build-support::dc_codegen from category data tables.\n",
    );
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str(
        "#[expect(\n    unused_imports,\n    clippy::wildcard_imports,\n    reason = \"Standard workspace module prelude\"\n)]\nuse crate::utilities::*;\n\n",
    );
    out.push_str(
        "pub use ctb_storage_minimal::global_graph_layout::{\n    SHORT_DC_REGION_END, SHORT_DC_REGION_START,\n};\n\n",
    );
    out.push_str("pub use crate::dc_char::DcChar;\n\n");

    for rec in &records {
        let const_name = to_screaming_snake_case(&rec.ident);
        let doc = if !rec.description.is_empty() {
            &rec.description
        } else {
            &rec.name
        };
        let doc_clean = doc.replace('\n', " ").replace('\r', "");
        out.push_str(&format!(
            "/// {} (Dc {}).\npub const DC_{}: DcChar = DcChar::from_short({});\n",
            doc_clean, rec.short_id, const_name, rec.short_id
        ));
    }

    out.push_str(
        "\n/// Converts a short Document Character (Dc) ID to its long (Global Graph) ID.\n",
    );
    out.push_str("#[must_use]\n");
    out.push_str("pub fn short_to_long_dc(short_id: u32) -> u128 {\n");
    out.push_str(
        "    SHORT_DC_REGION_START.saturating_add(u128::from(short_id))\n",
    );
    out.push_str("}\n\n");

    out.push_str(
        "/// Converts a long (Global Graph) Document Character ID to its short Dc ID, if within short range.\n",
    );
    out.push_str("#[must_use]\n");
    out.push_str("pub fn long_to_short_dc(dc_id: u128) -> Option<u32> {\n");
    out.push_str(
        "    if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&dc_id) {\n",
    );
    out.push_str(
        "        u32::try_from(dc_id.saturating_sub(SHORT_DC_REGION_START)).ok()\n",
    );
    out.push_str("    } else {\n");
    out.push_str("        None\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generates the contents of `dc.generated.rs` from the system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or contains invalid data.
pub fn generate_dc_code(graph_dir: &Path) -> Result<String> {
    let index = BuildGraphIndex::from_graph_dir(graph_dir)?;
    generate_dc_code_from_index(&index)
}

/// Generates `src/formats/dcdata/dc.generated.rs` if its contents have changed.
///
/// # Errors
/// Returns an error if reading the system graph or writing the generated file fails.
pub fn generate_dc_file(base_dir: &Path) -> Result<bool> {
    let repo_root = crate::find_repository_root_from(base_dir)?;
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let out_file = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("dc.generated.rs");

    ensure!(
        graph_dir.is_dir(),
        "Could not locate system graph directory at {}",
        graph_dir.display()
    );

    let code = generate_dc_code(&graph_dir)?;

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

/// Formats a sorted set of `u32` short IDs into a compact Rust pattern (e.g. `127..=190 | 195`).
fn format_short_id_pattern(ids: &BTreeSet<u32>) -> String {
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for &id in ids {
        if let Some(last) = ranges.last_mut() {
            if id == last.1.saturating_add(1) {
                last.1 = id;
                continue;
            }
        }
        ranges.push((id, id));
    }

    let parts: Vec<String> = ranges
        .into_iter()
        .map(|(start, end)| {
            if start == end {
                format!("{start}")
            } else if end == start.saturating_add(1) {
                format!("{start} | {end}")
            } else {
                format!("{start}..={end}")
            }
        })
        .collect();

    if parts.is_empty() {
        "u32::MAX".to_string()
    } else {
        parts.join(" | ")
    }
}

/// Generates the contents of `src/formats/dcdata/syntax/role.generated.rs` from a [`BuildGraphIndex`].
///
/// # Errors
/// Returns an error if required ontology entities are missing from `index`.
pub fn generate_role_code_from_index(
    index: &BuildGraphIndex,
) -> Result<String> {
    let records = collect_dc_const_records(index)?;

    let require_short = |ident: &str| -> Result<u32> {
        let sh = index.require_ident(ident)?;
        let DcShorthand::Short(sid) = sh else {
            bail!("Expected Short Dc for @ident(\"{ident}\"), found {sh}");
        };
        Ok(sid)
    };

    let base64_start = require_short("Base64Start")?;
    let base64_end = require_short("Base64End")?;
    let base64_padding = require_short("Base64Padding")?;
    ensure!(
        base64_start <= base64_end,
        "Base64Start ({base64_start}) must be <= Base64End ({base64_end})"
    );
    let base64_digits: BTreeSet<u32> = (base64_start..=base64_end).collect();

    // 1. TypeToken: all Short Dcs in `.EL Types`, plus TypeUnion, FilePathDelimiter,
    // TypeIntersection, FormatParamApplication, EntityType, RelationshipType, Relationship, Entity.
    let mut type_tokens: BTreeSet<u32> = BTreeSet::new();
    for short_id in index.all_short_dc_ids()? {
        let subj = DcShorthand::Short(short_id);
        if index.subject_script(subj) == Some(".EL Types") {
            type_tokens.insert(short_id);
        }
    }
    for ident in [
        "TypeUnion",
        "FilePathDelimiter",
        "TypeIntersection",
        "FormatParamApplication",
        "EntityType",
        "RelationshipType",
        "Relationship",
        "Entity",
    ] {
        type_tokens.insert(require_short(ident)?);
    }

    // 2. PrimitiveValueAtom: OptionalAbsent + Short Dc members of NamedConstruct("boolean").
    let sh_syntax_rule = index.require_ident("SyntaxRule")?;
    let sh_pat_alt = index.require_ident("PatternAlternation")?;
    let sh_subpat = index.require_ident("SubpatternMatch")?;
    let sh_begin_group = index.require_ident("BeginAssociativityGroup")?;
    let sh_end_group = index.require_ident("EndAssociativityGroup")?;

    let mut primitive_atoms: BTreeSet<u32> = BTreeSet::new();
    primitive_atoms.insert(require_short("OptionalAbsent")?);
    if let Some(bool_seed_id) = index.seed_node_id(
        &BuildSeedEntity::NamedConstruct("boolean".to_string()),
    ) {
        if let Some(DcShorthand::Long(rule_node_id)) = index.single_property(
            DcShorthand::Long(bool_seed_id),
            sh_syntax_rule,
        ) {
            if let Some(tokens) = index.node_tokens(rule_node_id) {
                for &tok in tokens {
                    if tok != sh_pat_alt
                        && tok != sh_begin_group
                        && tok != sh_end_group
                    {
                        if let DcShorthand::Short(sid) = tok {
                            primitive_atoms.insert(sid);
                        }
                    }
                }
            }
        }
    }

    // 3. DomainValueAtom: PrimitiveValueAtom + FilePathDelimiter + `.Semantic` Short Dcs
    // that have no SyntaxRule and no IsClosingDelimiterFor of their own and represent
    // stream kinds or Finder label values.
    let mut domain_atoms: BTreeSet<u32> = primitive_atoms.clone();
    domain_atoms.insert(require_short("FilePathDelimiter")?);

    let sh_name = index.require_ident("CanonicalName")?;
    let sh_closes = index.framing().is_closing_delimiter_for;
    for short_id in index.all_short_dc_ids()? {
        let subj = DcShorthand::Short(short_id);
        if index.subject_script(subj) == Some(".Semantic") {
            let has_rule =
                index.single_property(subj, sh_syntax_rule).is_some();
            let has_closes =
                !index.direct_objects(sh_closes, subj).is_empty();
            if !has_rule && !has_closes {
                if let Some(name) = index.single_property_string(subj, sh_name)
                {
                    if name.starts_with("Stream kind:")
                        || name.starts_with("Mac OS Finder label")
                    {
                        domain_atoms.insert(short_id);
                    }
                }
            }
        }
    }

    // 4. EvaluableExpressionStarter: all Short Dcs in `.EL Routines`, plus Short Dc
    // subpattern targets (`605 <sid>`) in NamedConstruct("value"), plus
    // EquivalentRepresentations and PartiallyEquivalentRepresentations.
    let mut evaluable_starters: BTreeSet<u32> = BTreeSet::new();
    for short_id in index.all_short_dc_ids()? {
        let subj = DcShorthand::Short(short_id);
        if index.subject_script(subj) == Some(".EL Routines") {
            evaluable_starters.insert(short_id);
        }
    }
    if let Some(val_seed_id) = index.seed_node_id(
        &BuildSeedEntity::NamedConstruct("value".to_string()),
    ) {
        if let Some(DcShorthand::Long(rule_node_id)) = index.single_property(
            DcShorthand::Long(val_seed_id),
            sh_syntax_rule,
        ) {
            if let Some(tokens) = index.node_tokens(rule_node_id) {
                for window in tokens.windows(2) {
                    if let &[op, DcShorthand::Short(sid)] = window {
                        if op == sh_subpat {
                            evaluable_starters.insert(sid);
                        }
                    }
                }
            }
        }
    }
    for ident in [
        "EquivalentRepresentations",
        "PartiallyEquivalentRepresentations",
    ] {
        evaluable_starters.insert(require_short(ident)?);
    }

    // 5. StructuralSeparator: ListElementSeparator + `.Punctuation` Short Dcs that have
    // a CanonicalDecomposition and GeneralCategory `Zl`/`Zp` or BidiClass `B`.
    let mut structural_seps: BTreeSet<u32> = BTreeSet::new();
    structural_seps.insert(require_short("ListElementSeparator")?);
    let sh_canon_decomp = index.require_ident("CanonicalDecomposition")?;
    let sh_bidi_prop = index.require_ident("BidirectionalClass")?;
    let bidi_b_seed =
        index.seed_node_id(&BuildSeedEntity::BidiClass("B".to_string()));

    for short_id in index.all_short_dc_ids()? {
        let subj = DcShorthand::Short(short_id);
        if index.subject_script(subj) == Some(".Punctuation")
            && !index.property_entries(subj, sh_canon_decomp).is_empty()
        {
            let gc_is_line_or_para = matches!(
                index.subject_general_category(subj),
                Some("Zl" | "Zp")
            );
            let bidi_is_b = bidi_b_seed.is_some_and(|b_id| {
                index.single_property(subj, sh_bidi_prop)
                    == Some(DcShorthand::Long(b_id))
            });
            if gc_is_line_or_para || bidi_is_b {
                structural_seps.insert(short_id);
            }
        }
    }

    let base64_digits_pat = format_short_id_pattern(&base64_digits);
    let type_tokens_pat = format_short_id_pattern(&type_tokens);
    let primitive_atoms_pat = format_short_id_pattern(&primitive_atoms);
    let domain_atoms_pat = format_short_id_pattern(&domain_atoms);
    let evaluable_starters_pat =
        format_short_id_pattern(&evaluable_starters);
    let structural_seps_pat = format_short_id_pattern(&structural_seps);

    let mut out = String::new();
    out.push_str(DEFAULT_AGPL_HEADER);
    out.push_str("\n\n");
    out.push_str(
        "//! Document Character (Dc) syntactic and semantic role classifications.\n",
    );
    out.push_str("//!\n");
    out.push_str(
        "//! @generated by ctb-build-support::dc_codegen from category data tables.\n",
    );
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str(
        "#[allow(\n    unused_imports,\n    clippy::wildcard_imports,\n    reason = \"Standard workspace module prelude\"\n)]\nuse crate::utilities::*;\n\n",
    );
    out.push_str(
        "use ctb_utilities::dc_char::{FORMAT_REGION_END, FORMAT_REGION_START};\n",
    );
    out.push_str("use serde::{Deserialize, Serialize};\n\n");

    out.push_str(
        "/// Classification of syntactic framing and semantic roles for Document\n",
    );
    out.push_str("/// Characters.\n");
    out.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]\n",
    );
    out.push_str("pub enum DcRole {\n");

    for rec in &records {
        let doc = if !rec.description.is_empty() {
            &rec.description
        } else {
            &rec.name
        };
        let doc_clean = doc.replace('\n', " ").replace('\r', "");
        out.push_str(&format!(
            "    /// {} (`{}`).\n    {},\n",
            doc_clean, rec.short_id, rec.ident
        ));
    }

    out.push_str(
        "    /// Format indicator for an integer in a number header (`FormatId::Integer`).\n\
    IntegerFormatIndicator,\n\
    /// Base64 encapsulation digit character (0..=63).\n\
    Base64Digit,\n\
    /// Built-in type token, format token, or type operator.\n\
    TypeToken,\n\
    /// Self-delimiting primitive value atom (`OptionalAbsent`, `BooleanTrue`, `BooleanFalse`).\n\
    PrimitiveValueAtom,\n\
    /// Self-delimiting domain or primitive value atom (including stream kinds and Finder labels).\n\
    DomainValueAtom,\n\
    /// Expression starter token that can be evaluated (`.EL Routines`, references, invocations, representation sets).\n\
    EvaluableExpressionStarter,\n\
    /// Structural separator (newlines, paragraph separators, list element separators).\n\
    StructuralSeparator,\n\
}\n\n"
    );

    out.push_str("impl DcRole {\n");
    out.push_str(
        "    /// Returns the defining short Dc ID for this role, if uniquely defined by a single Document Character.\n\
#[must_use]\n\
pub fn defining_short(self) -> Option<u32> {\n\
match self {\n"
    );
    for rec in &records {
        out.push_str(&format!(
            "            Self::{} => Some({}),\n",
            rec.ident, rec.short_id
        ));
    }
    out.push_str(
        "            _ => None,\n\
}\n\
}\n\n\
    /// Returns the defining `DcChar` for this role, if uniquely defined by a single Document Character.\n\
#[must_use]\n\
pub fn defining_dc(self) -> Option<DcChar> {\n\
self.defining_short().map(DcChar::from_short)\n\
}\n\n\
    /// Returns true if the given canonical token (in global graph `u128` layout)\n\
/// directly matches this role.\n\
#[must_use]\n\
pub fn matches_canonical(self, token: u128) -> bool {\n\
let short = DcChar::from_u128(token).to_short().ok();\n\
match self {\n"
    );
    for rec in &records {
        out.push_str(&format!(
            "            Self::{} => short == Some({}),\n",
            rec.ident, rec.short_id
        ));
    }
    out.push_str(&format!(
        "            Self::IntegerFormatIndicator => {{\n\
(FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)\n\
&& ctb_utilities::format_id::FormatId::from_dc_id(token)\n\
== Some(ctb_utilities::format_id::FormatId::Integer)\n\
}}\n\
Self::Base64Digit => matches!(short, Some({base64_digits_pat})),\n\
Self::TypeToken => {{\n\
(FORMAT_REGION_START..=FORMAT_REGION_END).contains(&token)\n\
|| matches!(short, Some({type_tokens_pat}))\n\
}}\n\
Self::PrimitiveValueAtom => matches!(short, Some({primitive_atoms_pat})),\n\
Self::DomainValueAtom => matches!(short, Some({domain_atoms_pat})),\n\
Self::EvaluableExpressionStarter => {{\n\
matches!(short, Some({evaluable_starters_pat}))\n\
}}\n\
Self::StructuralSeparator => {{\n\
matches!(short, Some({structural_seps_pat}))\n\
}}\n\
}}\n\
}}\n\n\
    /// Returns the Base64 numeral digit value (`0..=63`) or padding sentinel\n\
/// (`64`) if `token` represents a Base64 encapsulation character.\n\
#[must_use]\n\
pub fn base64_digit_value(token: u128) -> Option<u8> {{\n\
let short = DcChar::from_u128(token).to_short().ok()?;\n\
if short == {base64_padding} {{\n\
Some(64)\n\
}} else if ({base64_start}..={base64_end}).contains(&short) {{\n\
u8::try_from(short.saturating_sub({base64_start})).ok()\n\
}} else {{\n\
None\n\
}}\n\
}}\n\
}}\n"
    ));

    Ok(out)
}

/// Generates the contents of `src/formats/dcdata/syntax/role.generated.rs` from the system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or contains invalid data.
pub fn generate_role_code(graph_dir: &Path) -> Result<String> {
    let index = BuildGraphIndex::from_graph_dir(graph_dir)?;
    generate_role_code_from_index(&index)
}

/// Generates `src/formats/dcdata/syntax/role.generated.rs` if its contents have changed.
///
/// # Errors
/// Returns an error if reading the system graph or writing the generated file fails.
pub fn generate_role_file(base_dir: &Path) -> Result<bool> {
    let repo_root = crate::find_repository_root_from(base_dir)?;
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let out_file = repo_root
        .join("src")
        .join("formats")
        .join("dcdata")
        .join("syntax")
        .join("role.generated.rs");

    ensure!(
        graph_dir.is_dir(),
        "Could not locate system graph directory at {}",
        graph_dir.display()
    );

    let code = generate_role_code(&graph_dir)?;

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

/// Resolves all 16 [`CtbnbdlFramingDcs`] short IDs from the `.ctbnodes` system graph in `graph_dir`.
///
/// # Errors
/// Returns an error if `graph_dir` cannot be read or any required framing Dc is missing.
pub fn resolve_ctbnbdl_framing_dcs(
    graph_dir: &Path,
) -> Result<CtbnbdlFramingDcs> {
    resolve_ctbnbdl_framing_dcs_from_graph_dir(graph_dir)
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

    #[test]
    fn test_to_screaming_snake_case() {
        assert_eq!(to_screaming_snake_case("BeginNumber"), "BEGIN_NUMBER");
        assert_eq!(
            to_screaming_snake_case("StartEncapsulationUtf8"),
            "START_ENCAPSULATION_UTF8"
        );
        assert_eq!(to_screaming_snake_case("LangSay"), "LANG_SAY");
        assert_eq!(to_screaming_snake_case("LongDc"), "LONG_DC");
        assert_eq!(to_screaming_snake_case("ALREADY_UPPER"), "ALREADY_UPPER");
    }

    #[test]
    fn test_generate_dc_code_succeeds() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate::find_repository_root_from(manifest).unwrap();
        let graph_dir = repo_root.join("assets").join("data").join("graph");

        let code = generate_dc_code(&graph_dir).unwrap();
        assert!(
            code.contains("pub const DC_RANGE: DcChar = DcChar::from_short(4);")
        );
        assert!(
            code.contains("pub const DC_LONG_DC: DcChar = DcChar::from_short(308);")
        );
        assert!(code.contains("pub fn short_to_long_dc("));
        assert!(code.contains("pub fn long_to_short_dc("));
    }

    #[test]
    fn test_generate_role_code_succeeds() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate::find_repository_root_from(manifest).unwrap();
        let graph_dir = repo_root.join("assets").join("data").join("graph");

        let code = generate_role_code(&graph_dir).unwrap();
        assert!(code.contains("pub enum DcRole {"));
        assert!(code.contains("BeginNumber,"));
        assert!(code.contains("BeginLiteral,"));
        assert!(code.contains("TypeToken,"));
        assert!(code.contains("EvaluableExpressionStarter,"));
        assert!(
            code.contains("pub fn matches_canonical(self, token: u128) -> bool")
        );
        assert!(
            code.contains("pub fn base64_digit_value(token: u128) -> Option<u8>")
        );
    }

    #[test]
    fn test_resolve_ctbnbdl_framing_dcs_succeeds() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = crate::find_repository_root_from(manifest).unwrap();
        let graph_dir = repo_root.join("assets").join("data").join("graph");

        let framing = resolve_ctbnbdl_framing_dcs(&graph_dir).unwrap();
        assert_eq!(framing.begin_node, DcShorthand::Short(617));
        assert_eq!(framing.end_node, DcShorthand::Short(618));
        assert_eq!(framing.escape, DcShorthand::Short(255));
        assert_eq!(framing.evaluate_as_atom, DcShorthand::Short(616));
        assert_eq!(framing.has_a, DcShorthand::Short(584));
        assert_eq!(framing.has_value, DcShorthand::Short(610));
        assert_eq!(framing.preferred, DcShorthand::Short(570));
        assert_eq!(framing.supersedes, DcShorthand::Short(591));
        assert_eq!(framing.retracts, DcShorthand::Short(592));
        assert_eq!(framing.enclosing_document, DcShorthand::Short(615));
        assert_eq!(framing.is_instance_of, DcShorthand::Short(586));
        assert_eq!(framing.is_subtype_of, DcShorthand::Short(587));
        assert_eq!(framing.is_based_on, DcShorthand::Short(588));
        assert_eq!(framing.is_deprecated, DcShorthand::Short(589));
        assert_eq!(framing.is_closing_delimiter_for, DcShorthand::Short(590));
        assert_eq!(framing.is_associated_with, DcShorthand::Short(598));
    }
}
