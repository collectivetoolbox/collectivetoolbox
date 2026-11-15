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

//! StageL 3 `: @dc l<id>` ledger auto-allocator, System-graph Token-Dc compiler,
//! and Gate 4 `.WASM` boundary validator for `stage_1_impls/` and `impls/`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use ctb_storage_minimal::global_graph_layout::{
    SYSTEM_REGION_END, SYSTEM_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;

use crate::dc::{
    DC_BEGIN_WASM_PROGRAM, DC_CANONICAL_DECOMPOSITION,
    DC_DATA_FORMAT_DESCRIPTION, DC_DECOMPOSITION,
    DC_IN_CONTEXT_OF, DC_PROGRAMMATIC_IDENTIFIER, DC_SEMANTIC_DECOMPOSITION,
    DC_TYPE_NOMINAL, DC_TYPE_ROUTINE, DC_TYPE_STRUCT, DcChar,
};
use crate::format_spec::FormatExpr;
use crate::graph_ledger::{
    DesiredDatasetState, LedgerNode, LedgerUpdateStats, ValueSpec,
    allocate_stagel_entities_in_memory, allocate_stagel_entities_in_repo,
    load_ledger_nodes_from_dir, make_string_value_spec,
    single_valued_property_set,
};
use crate::report::ValidationReport;
use crate::seed_graph::SeedGraphIndex;
use crate::stagel3::ast::{
    BlockItem, BrandTypeDecl, EnumDecl, EnumVariantDecl, Expr, MatchArm,
    NamedFieldDecl, NamedStructDecl, Param, PositionalStructDecl,
    RoutineDecl, StageDialect, StageLType, StructTypeRef, SymbolTarget,
    TestBlock, TestCase, TestExpected, TopDecl, TopValueDecl,
    format_dc_shorthand,
};
use crate::stagel3::dc_tokens::{
    decode_module_from_dc_tokens, decode_top_decl_from_dc_tokens,
    encode_filepath_dc_tokens, encode_stagel_type_dc_tokens,
    encode_top_decl_to_dc_tokens, format_module_to_sl3, format_module_to_sr3,
    format_top_decl_to_sl3, format_top_decl_to_sr3,
};
use crate::stagel3::eval::run_validated_workspace_inline_tests;
use crate::stagel3::validator::{
    ModuleInput, RoutineKey, ValidatedModule, ValidatedWorkspace, ValueKey,
    parse_and_validate_workspace, top_decl_uses_stager,
};
pub use crate::wat::ImplStage;

#[must_use]
fn sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Statistics returned by [`assign_and_update_stagel_dc_ids`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct StagelAllocStats {
    /// Number of new System-block `l<id>` entity nodes allocated and written as `: @dc l<id>`.
    pub entities_allocated: usize,
    /// Number of `.sl3` files updated on disk.
    pub files_updated: usize,
}

/// One discovered `.sl3` PSR-4 namespace file under `stage_1_impls/` or `impls/`.
#[derive(Debug, Clone)]
pub struct DiscoveredStagelFile {
    /// Stage (`Stage1` or `Stage2`).
    pub stage: ImplStage,
    /// Absolute filesystem path to the `.sl3` file.
    pub disk_path: PathBuf,
    /// Repository-relative path (`src/formats/dcdata/data-src/<stage>/<subpath>.sl3`).
    pub repo_rel_path: String,
    /// Stage-relative PSR-4 path (`<subpath>.sl3`, e.g. `std/math.sl3`).
    pub psr4_rel_path: String,
    /// Raw file bytes.
    pub raw_bytes: Vec<u8>,
    /// UTF-8 file contents.
    pub source: String,
}

fn collect_subdir_files_recursive(
    stage: ImplStage,
    stage_root: &Path,
    cur_dir: &Path,
    out: &mut Vec<DiscoveredStagelFile>,
) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(cur_dir)
        .with_context(|| {
            format!("Failed to read directory {}", cur_dir.display())
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_subdir_files_recursive(stage, stage_root, &path, out)?;
            continue;
        }
        if name.ends_with(".md") {
            continue;
        }
        // Reason for fallback: extensionless files map to "" and fail the stage extension check below.
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        ensure!(
            ext != "wat",
            "{} file '{}' cannot be a .wat file (.wat is restricted to stage_0_impls/)",
            stage.dir_name(),
            path.display()
        );
        match stage {
            ImplStage::Stage0 => {
                bail!("Unexpected Stage0 subdirectory scan in {}", path.display())
            }
            ImplStage::Stage1 => {
                ensure!(
                    ext == "sl3",
                    "Unsupported file extension '.{ext}' in {}: expected .sl3 in stage_1_impls/ namespace directories",
                    path.display()
                );
            }
            ImplStage::Stage2 => {
                ensure!(
                    ext == "sl3" || ext == "sr3",
                    "Unsupported file extension '.{ext}' in {}: expected .sl3 or .sr3 namespace modules in impls/",
                    path.display()
                );
            }
        }

        let rel_to_stage = path
            .strip_prefix(stage_root)
            .with_context(|| {
                format!(
                    "Path {} is not inside {}",
                    path.display(),
                    stage_root.display()
                )
            })?
            .to_str()
            .with_context(|| {
                format!("Non-UTF-8 path {}", path.display())
            })?
            .replace('\\', "/");
        let repo_rel_path = format!(
            "src/formats/dcdata/data-src/{}/{rel_to_stage}",
            stage.dir_name()
        );
        let raw_bytes = std::fs::read(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let source = String::from_utf8(raw_bytes.clone()).with_context(|| {
            format!("StageL file {repo_rel_path} is not valid UTF-8")
        })?;
        out.push(DiscoveredStagelFile {
            stage,
            disk_path: path,
            repo_rel_path,
            psr4_rel_path: rel_to_stage,
            raw_bytes,
            source,
        });
    }
    Ok(())
}

/// Discovers all PSR-4 StageL 3 namespace files under subdirectories of
/// `data_src_dir/stage_1_impls/` and `data_src_dir/impls/`.
pub fn discover_stagel_namespace_files(
    data_src_dir: &Path,
) -> Result<Vec<DiscoveredStagelFile>> {
    let mut files = Vec::new();
    for stage in [ImplStage::Stage1, ImplStage::Stage2] {
        let stage_root = data_src_dir.join(stage.dir_name());
        if !stage_root.exists() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&stage_root)
            .with_context(|| {
                format!("Failed to read {}", stage_root.display())
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.starts_with('.') && path.is_dir() {
                collect_subdir_files_recursive(
                    stage,
                    &stage_root,
                    &path,
                    &mut files,
                )?;
            }
        }
    }
    Ok(files)
}

fn parse_validate_and_test_stagel_files(
    files: &[DiscoveredStagelFile],
) -> Result<(ValidatedWorkspace, HashMap<String, usize>)> {
    if files.is_empty() {
        return Ok((
            ValidatedWorkspace {
                modules: Vec::new(),
            },
            HashMap::new(),
        ));
    }

    for f in files {
        if f.stage == ImplStage::Stage1 {
            ensure!(
                f.psr4_rel_path.ends_with(".sl3"),
                "Stage 1 module '{}' must be a StageL 3 .sl3 file",
                f.repo_rel_path
            );
        }
    }

    let inputs: Vec<ModuleInput<'_>> = files
        .iter()
        .map(|f| ModuleInput {
            source: f.source.as_str(),
            source_path: Some(f.repo_rel_path.as_str()),
        })
        .collect();
    let workspace = parse_and_validate_workspace(&inputs)?;

    let mut ns_to_file_idx = HashMap::with_capacity(files.len());
    for (file_idx, f) in files.iter().enumerate() {
        let expected_ns = f
            .psr4_rel_path
            .strip_suffix(".sl3")
            .or_else(|| f.psr4_rel_path.strip_suffix(".sr3"))
            .with_context(|| {
                format!(
                    "Expected .sl3 or .sr3 suffix on '{}'",
                    f.repo_rel_path
                )
            })?;
        ns_to_file_idx.insert(expected_ns.to_owned(), file_idx);
    }

    for vm in &workspace.modules {
        let Some(ns_key) = &vm.namespace_key else {
            bail!(
                "StageL file in namespace subdirectory must declare 'ns <path>.'"
            );
        };
        let &file_idx = ns_to_file_idx.get(ns_key).with_context(|| {
            format!("Internal namespace lookup failed for '{ns_key}'")
        })?;
        let file_meta = files.get(file_idx).context("Invalid file index")?;
        if file_meta.stage == ImplStage::Stage1 {
            ensure!(
                vm.module.dialect == StageDialect::StageL3,
                "Stage 1 module '{}' cannot use StageR 3 dialect",
                file_meta.repo_rel_path
            );
            for u in &vm.module.uses {
                let target_ns = u.path.join("/");
                if let Some(&dep_idx) = ns_to_file_idx.get(&target_ns)
                    && let Some(dep_file) = files.get(dep_idx)
                {
                    ensure!(
                        dep_file.stage == ImplStage::Stage1,
                        "Stage 1 module '{}' cannot import Stage 2 module '{}'",
                        file_meta.repo_rel_path,
                        dep_file.repo_rel_path
                    );
                }
            }
        }
    }

    run_validated_workspace_inline_tests(&workspace).context(
        "Inline contract test failed in staged StageL 3 namespace library",
    )?;
    Ok((workspace, ns_to_file_idx))
}

fn build_workspace_type_dcs(
    workspace: &ValidatedWorkspace,
) -> (
    HashMap<(Option<String>, String), DcChar>,
    HashMap<(Option<String>, String), DcChar>,
) {
    let mut struct_dcs = HashMap::new();
    let mut nominal_dcs = HashMap::new();
    for vm in &workspace.modules {
        for item in &vm.module.items {
            match item {
                TopDecl::Struct(s) => {
                    if let Some(dc) = s.dc_id {
                        struct_dcs
                            .insert((vm.namespace_key.clone(), s.name.clone()), dc);
                    }
                }
                TopDecl::NamedStruct(s) => {
                    if let Some(dc) = s.dc_id {
                        struct_dcs
                            .insert((vm.namespace_key.clone(), s.name.clone()), dc);
                    }
                }
                TopDecl::BrandType(b) => {
                    if let Some(dc) = b.dc_id {
                        nominal_dcs
                            .insert((vm.namespace_key.clone(), b.name.clone()), dc);
                    }
                }
                TopDecl::Enum(e) => {
                    if let Some(dc) = e.dc_id {
                        nominal_dcs
                            .insert((vm.namespace_key.clone(), e.name.clone()), dc);
                    }
                }
                TopDecl::Value(_) | TopDecl::Routine(_) | TopDecl::Comment(_) => {}
            }
        }
    }
    (struct_dcs, nominal_dcs)
}

fn rewrite_type_to_struct_dcs(
    ty: &StageLType,
    vm: &ValidatedModule,
    struct_dcs: &HashMap<(Option<String>, String), DcChar>,
    nominal_dcs: &HashMap<(Option<String>, String), DcChar>,
) -> StageLType {
    fn map_canon(
        ty: &StageLType,
        struct_dcs: &HashMap<(Option<String>, String), DcChar>,
        nominal_dcs: &HashMap<(Option<String>, String), DcChar>,
    ) -> StageLType {
        match ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::Void
            | StageLType::Never
            | StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat
            | StageLType::Dc
            | StageLType::Dcs
            | StageLType::S
            | StageLType::Error
            | StageLType::View
            | StageLType::Cursor => ty.clone(),
            StageLType::List(inner) => StageLType::List(Box::new(map_canon(
                inner,
                struct_dcs,
                nominal_dcs,
            ))),
            StageLType::Buf(inner) => StageLType::Buf(Box::new(map_canon(
                inner,
                struct_dcs,
                nominal_dcs,
            ))),
            StageLType::Tuple(elems) => StageLType::Tuple(
                elems
                    .iter()
                    .map(|e| map_canon(e, struct_dcs, nominal_dcs))
                    .collect(),
            ),
            StageLType::Struct(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if let Some(&dc) = struct_dcs.get(&key) {
                    StageLType::Struct(StructTypeRef::Dc(dc))
                } else {
                    StageLType::Struct(StructTypeRef::Ident(name.clone()))
                }
            }
            StageLType::Struct(other) => StageLType::Struct(other.clone()),
            StageLType::Option(inner) => StageLType::Option(Box::new(map_canon(
                inner,
                struct_dcs,
                nominal_dcs,
            ))),
            StageLType::Result(inner) => StageLType::Result(Box::new(map_canon(
                inner,
                struct_dcs,
                nominal_dcs,
            ))),
            StageLType::Nominal(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if let Some(&dc) = nominal_dcs.get(&key) {
                    StageLType::Nominal(StructTypeRef::Dc(dc))
                } else {
                    StageLType::Nominal(StructTypeRef::Ident(name.clone()))
                }
            }
            StageLType::Nominal(other) => StageLType::Nominal(other.clone()),
        }
    }
    let canon = vm.canonicalize_type(ty);
    map_canon(&canon, struct_dcs, nominal_dcs)
}

pub(crate) fn class_value_spec_for_top_decl(decl: &TopDecl) -> Result<ValueSpec> {
    match decl {
        TopDecl::Routine(_) => Ok(ValueSpec::Direct(sh(DC_TYPE_ROUTINE))),
        TopDecl::Struct(_) | TopDecl::NamedStruct(_) => {
            Ok(ValueSpec::Direct(sh(DC_TYPE_STRUCT)))
        }
        TopDecl::BrandType(_) | TopDecl::Enum(_) => {
            Ok(ValueSpec::Direct(sh(DC_TYPE_NOMINAL)))
        }
        TopDecl::Value(val) => {
            let type_chars = encode_stagel_type_dc_tokens(&val.ty)?;
            let shorthands: Vec<DcShorthand> = type_chars
                .iter()
                .map(|c| DcShorthand::from_global_id(c.to_u128()))
                .collect();
            if shorthands.len() == 1
                && let Some(&only) = shorthands.first()
            {
                Ok(ValueSpec::Direct(only))
            } else {
                Ok(ValueSpec::ValueNode {
                    tokens: shorthands,
                    comment: format!("StageL type {}", val.ty),
                })
            }
        }
        TopDecl::Comment(_) => {
            bail!("Standalone comment does not have a graph entity class")
        }
    }
}

#[derive(Debug)]
struct UnassignedDeclSite {
    file_idx: usize,
    one_based_line: usize,
    class_spec: ValueSpec,
    label: String,
}

fn collect_unassigned_and_validate_assigned_dcs(
    workspace: &ValidatedWorkspace,
    files: &[DiscoveredStagelFile],
    ns_to_file_idx: &HashMap<String, usize>,
    only_structs: bool,
) -> Result<Vec<UnassignedDeclSite>> {
    // Process files in deterministic `files` order so allocated IDs follow
    // lexicographical file order and top-to-bottom declaration order.
    let mut vm_by_file_idx: BTreeMap<usize, &ValidatedModule> = BTreeMap::new();
    for vm in &workspace.modules {
        if let Some(ns_key) = &vm.namespace_key
            && let Some(&fidx) = ns_to_file_idx.get(ns_key)
        {
            vm_by_file_idx.insert(fidx, vm);
        }
    }

    let (struct_dcs, nominal_dcs) = build_workspace_type_dcs(workspace);
    let mut seen_dc_ids: HashMap<DcChar, String> = HashMap::new();
    let mut unassigned = Vec::new();

    for (&fidx, &vm) in &vm_by_file_idx {
        let fmeta = files.get(fidx).context("Invalid file index")?;
        // Reason for fallback: validated namespace modules always set namespace_key above.
        let ns_key = vm.namespace_key.as_deref().unwrap_or("");

        for item in &vm.module.items {
            let (name, span, dc_opt, pub_dc_syntax) = match item {
                TopDecl::Routine(r) => {
                    (r.name.as_str(), r.span, r.dc_id, r.pub_dc_syntax)
                }
                TopDecl::Value(v) => {
                    (v.name.as_str(), v.span, v.dc_id, v.pub_dc_syntax)
                }
                TopDecl::Struct(s) => {
                    (s.name.as_str(), s.span, s.dc_id, s.pub_dc_syntax)
                }
                TopDecl::BrandType(b) => {
                    (b.name.as_str(), b.span, b.dc_id, b.pub_dc_syntax)
                }
                TopDecl::NamedStruct(s) => {
                    (s.name.as_str(), s.span, s.dc_id, s.pub_dc_syntax)
                }
                TopDecl::Enum(e) => {
                    (e.name.as_str(), e.span, e.dc_id, e.pub_dc_syntax)
                }
                TopDecl::Comment(_) => continue,
            };
            let loc_desc = format!("{} ({ns_key}::{name})", fmeta.repo_rel_path);

            if let Some(dc) = dc_opt {
                if let Some(prev) = seen_dc_ids.insert(dc, loc_desc.clone()) {
                    bail!(
                        "Duplicate StageL Dc identity '{}' shared by {prev} and {loc_desc}",
                        format_dc_shorthand(dc)
                    );
                }
                let dc_sh = DcShorthand::from_global_id(dc.to_u128());
                if pub_dc_syntax {
                    ensure!(
                        matches!(
                            dc_sh,
                            DcShorthand::Short(_) | DcShorthand::Format(_)
                        ),
                        "'pub dc/{dc_sh}' in {loc_desc} must target a Short or Format Dc"
                    );
                } else {
                    let DcShorthand::Long(nid) = dc_sh else {
                        bail!(
                            "': @dc {dc_sh}' on {loc_desc} must be a System-block 'l<id>' node ID (use 'pub dc/{dc_sh}' for Short/Format Dcs)"
                        );
                    };
                    ensure!(
                        (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&nid),
                        "': @dc l{nid}' on {loc_desc} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
                    );
                }
            } else {
                ensure!(
                    !pub_dc_syntax,
                    "Missing target Dc on 'pub dc/...' declaration in {loc_desc}"
                );
                if only_structs
                    && !matches!(
                        item,
                        TopDecl::Struct(_)
                            | TopDecl::NamedStruct(_)
                            | TopDecl::BrandType(_)
                            | TopDecl::Enum(_)
                    )
                {
                    continue;
                }
                let class_spec = match item {
                    TopDecl::Value(v) => {
                        let rewritten_val = TopDecl::Value(TopValueDecl {
                            ty: rewrite_type_to_struct_dcs(
                                &v.ty,
                                vm,
                                &struct_dcs,
                                &nominal_dcs,
                            ),
                            ..v.clone()
                        });
                        class_value_spec_for_top_decl(&rewritten_val)?
                    }
                    other => class_value_spec_for_top_decl(other)?,
                };
                unassigned.push(UnassignedDeclSite {
                    file_idx: fidx,
                    one_based_line: span.line,
                    class_spec,
                    label: format!("{ns_key}::{name}"),
                });
            }
        }
    }

    Ok(unassigned)
}

fn insert_dc_directives_into_source(
    original: &str,
    insertions_one_based_line_to_id: &BTreeMap<usize, u128>,
) -> Result<String> {
    if insertions_one_based_line_to_id.is_empty() {
        return Ok(original.to_owned());
    }
    let lines: Vec<&str> = original.split('\n').collect();
    for &line_no in insertions_one_based_line_to_id.keys() {
        ensure!(
            line_no >= 1 && line_no <= lines.len(),
            "Invalid declaration line {line_no} (file has {} lines)",
            lines.len()
        );
    }

    let mut out = String::with_capacity(
        original
            .len()
            .saturating_add(insertions_one_based_line_to_id.len().saturating_mul(24)),
    );
    for (idx, &raw_line) in lines.iter().enumerate() {
        let one_based = idx.saturating_add(1);
        if let Some(&entity_id) = insertions_one_based_line_to_id.get(&one_based)
        {
            out.push_str(": @dc l");
            out.push_str(&entity_id.to_string());
            if raw_line.ends_with('\r') {
                out.push_str("\r\n");
            } else {
                out.push('\n');
            }
        }
        out.push_str(raw_line);
        if idx.saturating_add(1) < lines.len() {
            out.push('\n');
        }
    }
    Ok(out)
}

#[derive(Debug, Clone)]
struct DiscoveredStandaloneConverterFile {
    disk_path: PathBuf,
    repo_rel_path: String,
    ext: String,
    source: String,
    converts_line: usize,
    src_sh: DcShorthand,
    dst_sh: DcShorthand,
    dc_entity: Option<u128>,
}

fn discover_standalone_converter_files(
    data_src_dir: &Path,
) -> Result<Vec<DiscoveredStandaloneConverterFile>> {
    let mut out = Vec::new();
    for stage in [ImplStage::Stage0, ImplStage::Stage1, ImplStage::Stage2] {
        let dir_path = data_src_dir.join(stage.dir_name());
        if !dir_path.exists() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&dir_path)
            .with_context(|| {
                format!("Failed to read directory {}", dir_path.display())
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                continue;
            };
            if file_name.starts_with('.') || file_name.ends_with(".md") {
                continue;
            }
            // Reason for fallback: extensionless files map to "".
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_owned();
            let rel_path = format!(
                "src/formats/dcdata/data-src/{}/{file_name}",
                stage.dir_name()
            );
            let raw_bytes = std::fs::read(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            let source = String::from_utf8(raw_bytes).with_context(|| {
                format!("Implementation file {rel_path} is not valid UTF-8")
            })?;
            let directives =
                crate::wat::parse_staged_header_directives(&source, &rel_path)?;
            if let Some((conv_line, src_sh, dst_sh)) = directives.converts {
                out.push(DiscoveredStandaloneConverterFile {
                    disk_path: path,
                    repo_rel_path: rel_path,
                    ext,
                    source,
                    converts_line: conv_line,
                    src_sh,
                    dst_sh,
                    dc_entity: directives.dc_entity.map(|(_, nid)| nid),
                });
            }
        }
    }
    Ok(out)
}

fn insert_staged_dc_directive_at_line(
    original: &str,
    ext: &str,
    one_based_line: usize,
    entity_id: u128,
) -> Result<String> {
    let prefix = match ext {
        "wat" => ";; @dc l",
        "dcsh" => "# @dc l",
        _ => ": @dc l",
    };
    let lines: Vec<&str> = original.split('\n').collect();
    ensure!(
        one_based_line >= 1 && one_based_line <= lines.len(),
        "Invalid directive insertion line {one_based_line} (file has {} lines)",
        lines.len()
    );
    let mut out = String::with_capacity(original.len().saturating_add(32));
    for (idx, &raw_line) in lines.iter().enumerate() {
        let cur = idx.saturating_add(1);
        if cur == one_based_line {
            out.push_str(prefix);
            out.push_str(&entity_id.to_string());
            if raw_line.ends_with('\r') {
                out.push_str("\r\n");
            } else {
                out.push('\n');
            }
        }
        out.push_str(raw_line);
        if idx.saturating_add(1) < lines.len() {
            out.push('\n');
        }
    }
    Ok(out)
}

fn assign_and_update_standalone_converter_dc_ids(
    repo_root: &Path,
) -> Result<StagelAllocStats> {
    let data_dir = repo_root.join("src/formats/dcdata/data-src");
    let conv_files = discover_standalone_converter_files(&data_dir)?;
    let unassigned: Vec<&DiscoveredStandaloneConverterFile> = conv_files
        .iter()
        .filter(|f| f.dc_entity.is_none())
        .collect();
    if unassigned.is_empty() {
        return Ok(StagelAllocStats::default());
    }

    // Dry-run compile each unassigned standalone converter BEFORE allocating permanent ledger nodes.
    for (idx, cf) in unassigned.iter().enumerate() {
        let placeholder_id =
            SYSTEM_REGION_END.saturating_sub(u128::try_from(idx)?);
        let preview = insert_staged_dc_directive_at_line(
            &cf.source,
            &cf.ext,
            cf.converts_line,
            placeholder_id,
        )?;
        let _ = crate::wat::parse_staged_header_directives(
            &preview,
            &cf.repo_rel_path,
        )?;
        if cf.ext == "wat" {
            let _ = crate::wat::wat_to_staged_dc_entries(&preview)
                .with_context(|| {
                    format!("Failed to compile {}", cf.repo_rel_path)
                })?;
        }
    }

    let specs: Vec<(ValueSpec, String)> = unassigned
        .iter()
        .map(|cf| {
            (
                ValueSpec::Direct(sh(DC_TYPE_ROUTINE)),
                format!(
                    "{} (@converts {} -> {})",
                    cf.repo_rel_path, cf.src_sh, cf.dst_sh
                ),
            )
        })
        .collect();
    let allocated_ids = allocate_stagel_entities_in_repo(repo_root, &specs)?;
    ensure!(
        allocated_ids.len() == unassigned.len(),
        "Allocated entity count {} did not match unassigned converter count {}",
        allocated_ids.len(),
        unassigned.len()
    );

    for (cf, &entity_id) in unassigned.iter().zip(allocated_ids.iter()) {
        let updated = insert_staged_dc_directive_at_line(
            &cf.source,
            &cf.ext,
            cf.converts_line,
            entity_id,
        )?;
        std::fs::write(&cf.disk_path, updated.as_bytes()).with_context(|| {
            format!(
                "Failed to write updated converter file {}",
                cf.repo_rel_path
            )
        })?;
    }

    Ok(StagelAllocStats {
        entities_allocated: allocated_ids.len(),
        files_updated: unassigned.len(),
    })
}

/// Verifies that no `.sl3` namespace files or standalone `@converts(...)` files in `repo_root`
/// have missing `@dc l<id>` directives.
pub fn check_stagel_files_need_no_dc_updates(repo_root: &Path) -> Result<()> {
    let data_dir = repo_root.join("src/formats/dcdata/data-src");
    let conv_files = discover_standalone_converter_files(&data_dir)?;
    for cf in &conv_files {
        if cf.dc_entity.is_none() {
            bail!(
                "Standalone converter '@converts({} {})' in {} (line {}) is missing '@dc l<id>'; run validate-data-tables --write to allocate IDs",
                cf.src_sh,
                cf.dst_sh,
                cf.repo_rel_path,
                cf.converts_line
            );
        }
    }
    let files = discover_stagel_namespace_files(&data_dir)?;
    let (workspace, ns_to_file_idx) =
        parse_validate_and_test_stagel_files(&files)?;
    let unassigned = collect_unassigned_and_validate_assigned_dcs(
        &workspace,
        &files,
        &ns_to_file_idx,
        false,
    )?;
    if let Some(first) = unassigned.first() {
        let fmeta = files.get(first.file_idx).context("Invalid file index")?;
        bail!(
            "StageL declaration '{}' in {} (line {}) is missing ': @dc l<id>'; run validate-data-tables --write to allocate IDs",
            first.label,
            fmeta.repo_rel_path,
            first.one_based_line
        );
    }
    Ok(())
}

/// Allocates permanent System-block `l<id>` entity nodes (`586 615 <class>`) for any
/// unassigned standalone `@converts(...)` files and top-level declarations in
/// `stage_1_impls/**/*.sl3` and `impls/**/*.sl3`, writing `@dc l<id>` directives back to disk.
pub fn assign_and_update_stagel_dc_ids(
    repo_root: &Path,
) -> Result<StagelAllocStats> {
    let conv_stats = assign_and_update_standalone_converter_dc_ids(repo_root)?;
    let data_dir = repo_root.join("src/formats/dcdata/data-src");
    let mut files = discover_stagel_namespace_files(&data_dir)?;
    if files.is_empty() {
        return Ok(conv_stats);
    }

    let mut total_entities_allocated = 0usize;
    let mut touched_file_indices = BTreeSet::new();

    // Allocate positional `struct` entities first so any unassigned `TopDecl::Value`
    // whose type references a new struct encodes `[303 l<struct_id>]` in its entity class.
    for only_structs in [true, false] {
        let (workspace, ns_to_file_idx) =
            parse_validate_and_test_stagel_files(&files)?;
        let unassigned = collect_unassigned_and_validate_assigned_dcs(
            &workspace,
            &files,
            &ns_to_file_idx,
            only_structs,
        )?;
        if unassigned.is_empty() {
            continue;
        }

        // Dry-run inserting placeholder System-region `: @dc l<id>` directives and
        // re-validating every `.sl3` file in memory BEFORE committing permanent entity
        // nodes into `.ctbnodes` so syntax/directive errors never leave unreferenced nodes.
        {
            let mut dry_insertions: BTreeMap<usize, BTreeMap<usize, u128>> =
                BTreeMap::new();
            for (idx, site) in unassigned.iter().enumerate() {
                let placeholder_id =
                    SYSTEM_REGION_END.saturating_sub(u128::try_from(idx)?);
                dry_insertions
                    .entry(site.file_idx)
                    .or_default()
                    .insert(site.one_based_line, placeholder_id);
            }
            let mut dry_files = files.clone();
            for (&fidx, line_map) in &dry_insertions {
                let fmeta =
                    dry_files.get_mut(fidx).context("Invalid file index")?;
                let preview =
                    insert_dc_directives_into_source(&fmeta.source, line_map)?;
                fmeta.source = preview;
                fmeta.raw_bytes = fmeta.source.as_bytes().to_vec();
            }
            let (dry_ws, dry_map) =
                parse_validate_and_test_stagel_files(&dry_files)?;
            let dry_rem = collect_unassigned_and_validate_assigned_dcs(
                &dry_ws,
                &dry_files,
                &dry_map,
                only_structs,
            )?;
            ensure!(
                dry_rem.is_empty(),
                "Internal error: {} StageL items remained unassigned in dry-run ': @dc' insertion",
                dry_rem.len()
            );
        }

        let specs: Vec<(ValueSpec, String)> = unassigned
            .iter()
            .map(|u| (u.class_spec.clone(), u.label.clone()))
            .collect();
        let allocated_ids = allocate_stagel_entities_in_repo(repo_root, &specs)?;
        ensure!(
            allocated_ids.len() == unassigned.len(),
            "Allocated entity count {} did not match unassigned StageL item count {}",
            allocated_ids.len(),
            unassigned.len()
        );

        let mut per_file_insertions: BTreeMap<usize, BTreeMap<usize, u128>> =
            BTreeMap::new();
        for (site, &entity_id) in unassigned.iter().zip(allocated_ids.iter()) {
            per_file_insertions
                .entry(site.file_idx)
                .or_default()
                .insert(site.one_based_line, entity_id);
        }

        let mut updated_sources: BTreeMap<usize, String> = BTreeMap::new();
        for (&fidx, line_map) in &per_file_insertions {
            let fmeta = files.get(fidx).context("Invalid file index")?;
            let new_src =
                insert_dc_directives_into_source(&fmeta.source, line_map)?;
            updated_sources.insert(fidx, new_src);
        }

        // Re-parse and re-validate all updated `.sl3` sources in memory BEFORE writing to disk.
        for (&fidx, new_src) in &updated_sources {
            if let Some(fmeta) = files.get_mut(fidx) {
                fmeta.source.clone_from(new_src);
                fmeta.raw_bytes = new_src.as_bytes().to_vec();
            }
            touched_file_indices.insert(fidx);
        }
        let (revalidated_ws, revalidated_map) =
            parse_validate_and_test_stagel_files(&files)?;
        let remaining = collect_unassigned_and_validate_assigned_dcs(
            &revalidated_ws,
            &files,
            &revalidated_map,
            only_structs,
        )?;
        ensure!(
            remaining.is_empty(),
            "Internal error: {} StageL items remained unassigned after ': @dc' insertion",
            remaining.len()
        );

        for &fidx in updated_sources.keys() {
            let fmeta = files.get(fidx).context("Invalid file index")?;
            std::fs::write(&fmeta.disk_path, &fmeta.raw_bytes).with_context(
                || {
                    format!(
                        "Failed to write updated StageL file {}",
                        fmeta.repo_rel_path
                    )
                },
            )?;
        }
        total_entities_allocated =
            total_entities_allocated.saturating_add(allocated_ids.len());
    }

    Ok(StagelAllocStats {
        entities_allocated: conv_stats
            .entities_allocated
            .saturating_add(total_entities_allocated),
        files_updated: conv_stats
            .files_updated
            .saturating_add(touched_file_indices.len()),
    })
}

/// In-memory equivalent of [`assign_and_update_stagel_dc_ids`] for unit tests.
pub fn assign_stagel_dc_ids_in_memory(
    mut existing_nodes: Vec<LedgerNode>,
    files: &mut [DiscoveredStagelFile],
) -> Result<(Vec<LedgerNode>, StagelAllocStats)> {
    let mut total_entities_allocated = 0usize;
    let mut touched_file_indices = BTreeSet::new();

    for only_structs in [true, false] {
        let (workspace, ns_to_file_idx) =
            parse_validate_and_test_stagel_files(files)?;
        let unassigned = collect_unassigned_and_validate_assigned_dcs(
            &workspace,
            files,
            &ns_to_file_idx,
            only_structs,
        )?;
        if unassigned.is_empty() {
            continue;
        }

        let specs: Vec<(ValueSpec, String)> = unassigned
            .iter()
            .map(|u| (u.class_spec.clone(), u.label.clone()))
            .collect();
        let (updated_nodes, allocated_ids) =
            allocate_stagel_entities_in_memory(existing_nodes, &specs)?;
        existing_nodes = updated_nodes;

        let mut per_file_insertions: BTreeMap<usize, BTreeMap<usize, u128>> =
            BTreeMap::new();
        for (site, &entity_id) in unassigned.iter().zip(allocated_ids.iter()) {
            per_file_insertions
                .entry(site.file_idx)
                .or_default()
                .insert(site.one_based_line, entity_id);
        }

        for (fidx, line_map) in per_file_insertions {
            let fmeta = files.get_mut(fidx).context("Invalid file index")?;
            let new_src =
                insert_dc_directives_into_source(&fmeta.source, &line_map)?;
            fmeta.source = new_src;
            fmeta.raw_bytes = fmeta.source.as_bytes().to_vec();
            touched_file_indices.insert(fidx);
        }
        total_entities_allocated =
            total_entities_allocated.saturating_add(allocated_ids.len());
    }

    Ok((
        existing_nodes,
        StagelAllocStats {
            entities_allocated: total_entities_allocated,
            files_updated: touched_file_indices.len(),
        },
    ))
}

struct DeclRewriteMaps<'a> {
    vm: &'a ValidatedModule,
    all_struct_dcs: &'a HashMap<(Option<String>, String), DcChar>,
    all_nominal_dcs: &'a HashMap<(Option<String>, String), DcChar>,
    value_dcs_by_span: HashMap<(usize, usize), DcChar>,
    routine_dcs_by_span: HashMap<(usize, usize), DcChar>,
    struct_dcs_by_span: HashMap<(usize, usize), DcChar>,
    named_struct_dcs_by_span: HashMap<(usize, usize), DcChar>,
    enum_dcs_by_span: HashMap<(usize, usize), DcChar>,
}

impl DeclRewriteMaps<'_> {
    fn rewrite_type(&self, ty: &StageLType) -> StageLType {
        rewrite_type_to_struct_dcs(
            ty,
            self.vm,
            self.all_struct_dcs,
            self.all_nominal_dcs,
        )
    }
}

#[derive(Default)]
struct LexicalShadowScope {
    values: HashSet<String>,
    routines: HashSet<String>,
}

struct LexicalShadows {
    layers: Vec<LexicalShadowScope>,
}

impl LexicalShadows {
    fn new() -> Self {
        Self { layers: Vec::new() }
    }

    fn has_value(&self, name: &str) -> bool {
        self.layers.iter().any(|l| l.values.contains(name))
    }

    fn has_routine(&self, name: &str) -> bool {
        self.layers.iter().any(|l| l.routines.contains(name))
    }
}

fn rewrite_top_decl_refs_to_dc(
    decl: &TopDecl,
    maps: &DeclRewriteMaps<'_>,
) -> TopDecl {
    match decl {
        TopDecl::Value(v) => {
            let mut shadows = LexicalShadows::new();
            TopDecl::Value(TopValueDecl {
                is_pub: v.is_pub,
                pub_dc_syntax: v.pub_dc_syntax,
                dc_id: v.dc_id,
                ty: maps.rewrite_type(&v.ty),
                name: v.name.clone(),
                value: rewrite_expr_refs(&v.value, maps, &mut shadows),
                comments_before: v.comments_before.clone(),
                span: v.span,
            })
        }
        TopDecl::Routine(r) => {
            let mut shadows = LexicalShadows::new();
            TopDecl::Routine(rewrite_routine_refs(r, maps, &mut shadows))
        }
        TopDecl::Struct(s) => TopDecl::Struct(PositionalStructDecl {
            is_pub: s.is_pub,
            pub_dc_syntax: s.pub_dc_syntax,
            dc_id: s.dc_id,
            name: s.name.clone(),
            fields: s.fields.iter().map(|f| maps.rewrite_type(f)).collect(),
            inline_body: s.inline_body,
            comments_before: s.comments_before.clone(),
            span: s.span,
        }),
        TopDecl::BrandType(b) => TopDecl::BrandType(BrandTypeDecl {
            is_pub: b.is_pub,
            pub_dc_syntax: b.pub_dc_syntax,
            is_opaque: b.is_opaque,
            dc_id: b.dc_id,
            inner_ty: maps.rewrite_type(&b.inner_ty),
            name: b.name.clone(),
            comments_before: b.comments_before.clone(),
            span: b.span,
        }),
        TopDecl::NamedStruct(s) => TopDecl::NamedStruct(NamedStructDecl {
            is_pub: s.is_pub,
            pub_dc_syntax: s.pub_dc_syntax,
            dc_id: s.dc_id,
            name: s.name.clone(),
            fields: s
                .fields
                .iter()
                .map(|f| NamedFieldDecl {
                    ty: maps.rewrite_type(&f.ty),
                    name: f.name.clone(),
                    comments_before: f.comments_before.clone(),
                    span: f.span,
                })
                .collect(),
            inline_body: s.inline_body,
            comments_before: s.comments_before.clone(),
            span: s.span,
        }),
        TopDecl::Enum(e) => TopDecl::Enum(EnumDecl {
            is_pub: e.is_pub,
            pub_dc_syntax: e.pub_dc_syntax,
            dc_id: e.dc_id,
            name: e.name.clone(),
            variants: e
                .variants
                .iter()
                .map(|v| EnumVariantDecl {
                    name: v.name.clone(),
                    payload: v.payload.as_ref().map(|p| maps.rewrite_type(p)),
                    comments_before: v.comments_before.clone(),
                    span: v.span,
                })
                .collect(),
            inline_body: e.inline_body,
            comments_before: e.comments_before.clone(),
            span: e.span,
        }),
        TopDecl::Comment(c) => TopDecl::Comment(c.clone()),
    }
}

fn rewrite_routine_refs(
    r: &RoutineDecl,
    maps: &DeclRewriteMaps<'_>,
    shadows: &mut LexicalShadows,
) -> RoutineDecl {
    let mut param_layer = LexicalShadowScope::default();
    for p in &r.params {
        param_layer.values.insert(p.name.clone());
    }
    shadows.layers.push(param_layer);

    let rewritten_tests: Vec<TestBlock> = r
        .tests
        .iter()
        .map(|tb| TestBlock {
            label: tb.label.clone(),
            cases: tb
                .cases
                .iter()
                .map(|tc| TestCase {
                    inputs: tc
                        .inputs
                        .iter()
                        .map(|inp| rewrite_expr_refs(inp, maps, shadows))
                        .collect(),
                    expected: match &tc.expected {
                        TestExpected::Expr(e) => {
                            TestExpected::Expr(rewrite_expr_refs(e, maps, shadows))
                        }
                        TestExpected::Fault(sp) => TestExpected::Fault(*sp),
                        TestExpected::Void(sp) => TestExpected::Void(*sp),
                    },
                    span: tc.span,
                })
                .collect(),
            span: tb.span,
        })
        .collect();

    let rewritten_body = rewrite_expr_refs(&r.body, maps, shadows);
    let _ = shadows.layers.pop();

    RoutineDecl {
        is_pub: r.is_pub,
        pub_dc_syntax: r.pub_dc_syntax,
        dc_id: r.dc_id,
        return_type: maps.rewrite_type(&r.return_type),
        name: r.name.clone(),
        params: r
            .params
            .iter()
            .map(|p| Param {
                ty: maps.rewrite_type(&p.ty),
                name: p.name.clone(),
                span: p.span,
            })
            .collect(),
        tests: rewritten_tests,
        body: rewritten_body,
        inline_body: r.inline_body,
        comments_before: r.comments_before.clone(),
        span: r.span,
    }
}

fn rewrite_expr_refs(
    expr: &Expr,
    maps: &DeclRewriteMaps<'_>,
    shadows: &mut LexicalShadows,
) -> Expr {
    match expr {
        Expr::Bool { .. } | Expr::Int { .. } | Expr::Bytes { .. } => expr.clone(),
        Expr::List {
            elem_ty,
            elements,
            span,
        } => Expr::List {
            elem_ty: maps.rewrite_type(elem_ty),
            elements: elements
                .iter()
                .map(|e| rewrite_expr_refs(e, maps, shadows))
                .collect(),
            span: *span,
        },
        Expr::BufNew {
            elem_ty,
            capacity,
            span,
        } => Expr::BufNew {
            elem_ty: maps.rewrite_type(elem_ty),
            capacity: Box::new(rewrite_expr_refs(capacity, maps, shadows)),
            span: *span,
        },
        Expr::ValueRef { ty, target, span } => {
            let new_target = match target {
                SymbolTarget::Ident(name) if !shadows.has_value(name) => {
                    if let Some(&dc) =
                        maps.value_dcs_by_span.get(&(span.line, span.col))
                    {
                        SymbolTarget::Dc(dc)
                    } else {
                        SymbolTarget::Ident(name.clone())
                    }
                }
                other => other.clone(),
            };
            Expr::ValueRef {
                ty: ty.as_ref().map(|t| maps.rewrite_type(t)),
                target: new_target,
                span: *span,
            }
        }
        Expr::Let {
            ty,
            name,
            init,
            span,
        } => {
            let new_init = init
                .as_ref()
                .map(|e| Box::new(rewrite_expr_refs(e, maps, shadows)));
            if let Some(top) = shadows.layers.last_mut() {
                top.values.insert(name.clone());
            }
            Expr::Let {
                ty: maps.rewrite_type(ty),
                name: name.clone(),
                init: new_init,
                span: *span,
            }
        }
        Expr::Const {
            ty,
            name,
            dc_id,
            value,
            span,
        } => {
            let new_val = Box::new(rewrite_expr_refs(value, maps, shadows));
            if let Some(top) = shadows.layers.last_mut() {
                top.values.insert(name.clone());
            }
            Expr::Const {
                ty: maps.rewrite_type(ty),
                name: name.clone(),
                dc_id: *dc_id,
                value: new_val,
                span: *span,
            }
        }
        Expr::Set {
            ty,
            name,
            value,
            span,
        } => Expr::Set {
            ty: ty.as_ref().map(|t| maps.rewrite_type(t)),
            name: name.clone(),
            value: Box::new(rewrite_expr_refs(value, maps, shadows)),
            span: *span,
        },
        Expr::RoutineDecl(nr) => {
            Expr::RoutineDecl(Box::new(rewrite_routine_refs(nr, maps, shadows)))
        }
        Expr::Call { target, args, span } => {
            let new_target = match target {
                SymbolTarget::Ident(name) if !shadows.has_routine(name) => {
                    if let Some(&dc) =
                        maps.routine_dcs_by_span.get(&(span.line, span.col))
                    {
                        SymbolTarget::Dc(dc)
                    } else {
                        SymbolTarget::Ident(name.clone())
                    }
                }
                other => other.clone(),
            };
            let new_args = args
                .iter()
                .map(|a| rewrite_expr_refs(a, maps, shadows))
                .collect();
            Expr::Call {
                target: new_target,
                args: new_args,
                span: *span,
            }
        }
        Expr::Block { stmts, span } => {
            let mut block_layer = LexicalShadowScope::default();
            for item in stmts {
                if let BlockItem::Expr(Expr::RoutineDecl(nr)) = item {
                    block_layer.routines.insert(nr.name.clone());
                }
            }
            shadows.layers.push(block_layer);
            let mut out_stmts = Vec::with_capacity(stmts.len());
            for item in stmts {
                match item {
                    BlockItem::Comment(c) => {
                        out_stmts.push(BlockItem::Comment(c.clone()));
                    }
                    BlockItem::Expr(e) => {
                        out_stmts.push(BlockItem::Expr(rewrite_expr_refs(
                            e, maps, shadows,
                        )));
                    }
                }
            }
            let _ = shadows.layers.pop();
            Expr::Block {
                stmts: out_stmts,
                span: *span,
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => Expr::If {
            cond: Box::new(rewrite_expr_refs(cond, maps, shadows)),
            then_branch: Box::new(rewrite_expr_refs(then_branch, maps, shadows)),
            else_branch: Box::new(rewrite_expr_refs(else_branch, maps, shadows)),
            span: *span,
        },
        Expr::While { cond, body, span } => Expr::While {
            cond: Box::new(rewrite_expr_refs(cond, maps, shadows)),
            body: Box::new(rewrite_expr_refs(body, maps, shadows)),
            span: *span,
        },
        Expr::ForEach {
            elem_ty,
            elem_name,
            list,
            body,
            span,
        } => {
            let new_list = Box::new(rewrite_expr_refs(list, maps, shadows));
            let mut iter_layer = LexicalShadowScope::default();
            iter_layer.values.insert(elem_name.clone());
            shadows.layers.push(iter_layer);
            let new_body = Box::new(rewrite_expr_refs(body, maps, shadows));
            let _ = shadows.layers.pop();
            Expr::ForEach {
                elem_ty: maps.rewrite_type(elem_ty),
                elem_name: elem_name.clone(),
                list: new_list,
                body: new_body,
                span: *span,
            }
        }
        Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. } => expr.clone(),
        Expr::Return { value, span } => Expr::Return {
            value: value
                .as_ref()
                .map(|v| Box::new(rewrite_expr_refs(v, maps, shadows))),
            span: *span,
        },
        Expr::Unary { op, operand, span } => Expr::Unary {
            op: *op,
            operand: Box::new(rewrite_expr_refs(operand, maps, shadows)),
            span: *span,
        },
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => Expr::Binary {
            op: *op,
            left: Box::new(rewrite_expr_refs(left, maps, shadows)),
            right: Box::new(rewrite_expr_refs(right, maps, shadows)),
            span: *span,
        },
        Expr::Ternary {
            op,
            first,
            second,
            third,
            span,
        } => Expr::Ternary {
            op: *op,
            first: Box::new(rewrite_expr_refs(first, maps, shadows)),
            second: Box::new(rewrite_expr_refs(second, maps, shadows)),
            third: Box::new(rewrite_expr_refs(third, maps, shadows)),
            span: *span,
        },
        Expr::Conv {
            total,
            expr: inner,
            target_ty,
            span,
        } => Expr::Conv {
            total: *total,
            expr: Box::new(rewrite_expr_refs(inner, maps, shadows)),
            target_ty: maps.rewrite_type(target_ty),
            span: *span,
        },
        Expr::Bitcast {
            expr: inner,
            target_ty,
            span,
        } => Expr::Bitcast {
            expr: Box::new(rewrite_expr_refs(inner, maps, shadows)),
            target_ty: maps.rewrite_type(target_ty),
            span: *span,
        },
        Expr::TupleNew { elements, span } => Expr::TupleNew {
            elements: elements
                .iter()
                .map(|e| rewrite_expr_refs(e, maps, shadows))
                .collect(),
            span: *span,
        },
        Expr::StructNew {
            target,
            fields,
            span,
        } => {
            let new_target = match target {
                SymbolTarget::Ident(_) => {
                    if let Some(&dc) =
                        maps.struct_dcs_by_span.get(&(span.line, span.col))
                    {
                        SymbolTarget::Dc(dc)
                    } else {
                        target.clone()
                    }
                }
                other => other.clone(),
            };
            Expr::StructNew {
                target: new_target,
                fields: fields
                    .iter()
                    .map(|f| rewrite_expr_refs(f, maps, shadows))
                    .collect(),
                span: *span,
            }
        }
        Expr::Field { index, base, span } => Expr::Field {
            index: *index,
            base: Box::new(rewrite_expr_refs(base, maps, shadows)),
            span: *span,
        },
        Expr::With {
            index,
            base,
            value,
            span,
        } => Expr::With {
            index: *index,
            base: Box::new(rewrite_expr_refs(base, maps, shadows)),
            value: Box::new(rewrite_expr_refs(value, maps, shadows)),
            span: *span,
        },
        Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::ErrorTag { .. } => expr.clone(),
        Expr::NamedStructNew {
            target,
            fields,
            span,
        } => {
            let new_target = match target {
                SymbolTarget::Ident(_) => {
                    if let Some(&dc) =
                        maps.named_struct_dcs_by_span.get(&(span.line, span.col))
                    {
                        SymbolTarget::Dc(dc)
                    } else {
                        target.clone()
                    }
                }
                other @ SymbolTarget::Dc(_) => other.clone(),
            };
            Expr::NamedStructNew {
                target: new_target,
                fields: fields
                    .iter()
                    .map(|(fname, fexpr, fspan)| {
                        (
                            fname.clone(),
                            rewrite_expr_refs(fexpr, maps, shadows),
                            *fspan,
                        )
                    })
                    .collect(),
                span: *span,
            }
        }
        Expr::EnumVariant {
            enum_ty,
            variant,
            payload,
            span,
        } => {
            let new_enum_ty = match enum_ty {
                StructTypeRef::Ident(_) => {
                    if let Some(&dc) =
                        maps.enum_dcs_by_span.get(&(span.line, span.col))
                    {
                        StructTypeRef::Dc(dc)
                    } else {
                        enum_ty.clone()
                    }
                }
                StructTypeRef::Canonical { namespace, name } => {
                    if let Some(&dc) = maps
                        .all_nominal_dcs
                        .get(&(namespace.clone(), name.clone()))
                    {
                        StructTypeRef::Dc(dc)
                    } else {
                        enum_ty.clone()
                    }
                }
                other @ StructTypeRef::Dc(_) => other.clone(),
            };
            Expr::EnumVariant {
                enum_ty: new_enum_ty,
                variant: variant.clone(),
                payload: payload
                    .as_ref()
                    .map(|p| Box::new(rewrite_expr_refs(p, maps, shadows))),
                span: *span,
            }
        }
        Expr::OptionNone { inner_ty, span } => Expr::OptionNone {
            inner_ty: maps.rewrite_type(inner_ty),
            span: *span,
        },
        Expr::OptionSome {
            inner_ty,
            value,
            span,
        } => Expr::OptionSome {
            inner_ty: maps.rewrite_type(inner_ty),
            value: Box::new(rewrite_expr_refs(value, maps, shadows)),
            span: *span,
        },
        Expr::ResultOk { ok_ty, value, span } => Expr::ResultOk {
            ok_ty: maps.rewrite_type(ok_ty),
            value: Box::new(rewrite_expr_refs(value, maps, shadows)),
            span: *span,
        },
        Expr::ResultErr { ok_ty, error, span } => Expr::ResultErr {
            ok_ty: maps.rewrite_type(ok_ty),
            error: Box::new(rewrite_expr_refs(error, maps, shadows)),
            span: *span,
        },
        Expr::NamedField { base, field, span } => Expr::NamedField {
            base: Box::new(rewrite_expr_refs(base, maps, shadows)),
            field: field.clone(),
            span: *span,
        },
        Expr::NamedWith {
            base,
            field,
            value,
            span,
        } => Expr::NamedWith {
            base: Box::new(rewrite_expr_refs(base, maps, shadows)),
            field: field.clone(),
            value: Box::new(rewrite_expr_refs(value, maps, shadows)),
            span: *span,
        },
        Expr::WrapNominal {
            expr: inner,
            target_ty,
            span,
        } => Expr::WrapNominal {
            expr: Box::new(rewrite_expr_refs(inner, maps, shadows)),
            target_ty: maps.rewrite_type(target_ty),
            span: *span,
        },
        Expr::UnwrapNominal { expr: inner, span } => Expr::UnwrapNominal {
            expr: Box::new(rewrite_expr_refs(inner, maps, shadows)),
            span: *span,
        },
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            span,
        } => {
            let new_scrutinee =
                Box::new(rewrite_expr_refs(scrutinee, maps, shadows));
            let mut new_arms = Vec::with_capacity(arms.len());
            for arm in arms {
                let (new_binding, new_body) =
                    if let Some((bty, bname, bsp)) = &arm.binding {
                        let mut arm_layer = LexicalShadowScope::default();
                        arm_layer.values.insert(bname.clone());
                        shadows.layers.push(arm_layer);
                        let body = rewrite_expr_refs(&arm.body, maps, shadows);
                        let _ = shadows.layers.pop();
                        (Some((maps.rewrite_type(bty), bname.clone(), *bsp)), body)
                    } else {
                        (None, rewrite_expr_refs(&arm.body, maps, shadows))
                    };
                new_arms.push(MatchArm {
                    variant: arm.variant.clone(),
                    binding: new_binding,
                    body: new_body,
                    comments_before: arm.comments_before.clone(),
                    span: arm.span,
                });
            }
            let new_fallback = fallback
                .as_ref()
                .map(|fb| Box::new(rewrite_expr_refs(fb, maps, shadows)));
            Expr::Match {
                scrutinee: new_scrutinee,
                arms: new_arms,
                fallback: new_fallback,
                span: *span,
            }
        }
        Expr::Propagate { expr: inner, span } => Expr::Propagate {
            expr: Box::new(rewrite_expr_refs(inner, maps, shadows)),
            span: *span,
        },
    }
}

/// Compiles `files` into `desired` (populating `allocated_entities`, `single_properties`,
/// `single_property_contexts`, and `multi_properties` with self-contained StageL Token Dcs).
pub(crate) fn compile_stagel_files_into_desired_state(
    files: &[DiscoveredStagelFile],
    desired: &mut DesiredDatasetState,
) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }
    let (workspace, ns_to_file_idx) =
        parse_validate_and_test_stagel_files(files)?;
    let unassigned = collect_unassigned_and_validate_assigned_dcs(
        &workspace,
        files,
        &ns_to_file_idx,
        false,
    )?;
    if let Some(first) = unassigned.first() {
        let fmeta = files.get(first.file_idx).context("Invalid file index")?;
        bail!(
            "StageL declaration '{}' in {} (line {}) is missing ': @dc l<id>' during ledger reconciliation",
            first.label,
            fmeta.repo_rel_path,
            first.one_based_line
        );
    }

    let (all_struct_dcs, all_nominal_dcs) = build_workspace_type_dcs(&workspace);

    for vm in &workspace.modules {
        let ns_segs = vm
            .module
            .namespace
            .as_ref()
            .context("Expected namespace segments on staged StageL module")?;
        let ns_key = ns_segs.join("/");
        let fp_chars = encode_filepath_dc_tokens(ns_segs)?;
        let fp_shorthands: Vec<DcShorthand> = fp_chars
            .iter()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();
        let ns_ctx_spec = ValueSpec::ValueNode {
            tokens: fp_shorthands,
            comment: format!("FilePath {ns_key:?}"),
        };

        let mut local_value_dcs: HashMap<ValueKey, DcChar> = HashMap::new();
        let mut local_routine_dcs: HashMap<RoutineKey, DcChar> = HashMap::new();
        for item in &vm.module.items {
            match item {
                TopDecl::Value(v) => {
                    let dc = v.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, v.name)
                    })?;
                    local_value_dcs.insert(vm.value_key(v), dc);
                }
                TopDecl::Routine(r) => {
                    let dc = r.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, r.name)
                    })?;
                    local_routine_dcs.insert(vm.routine_key(r), dc);
                }
                TopDecl::Struct(s) => {
                    let _ = s.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, s.name)
                    })?;
                }
                TopDecl::BrandType(b) => {
                    let _ = b.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, b.name)
                    })?;
                }
                TopDecl::NamedStruct(s) => {
                    let _ = s.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, s.name)
                    })?;
                }
                TopDecl::Enum(e) => {
                    let _ = e.dc_id.with_context(|| {
                        format!("Missing Dc on '{}::{}'", ns_key, e.name)
                    })?;
                }
                TopDecl::Comment(_) => {}
            }
        }

        let mut value_dcs_by_span: HashMap<(usize, usize), DcChar> =
            HashMap::with_capacity(vm.resolved_values.len());
        for (&span_key, rv) in &vm.resolved_values {
            let vkey = rv.value_key();
            let target_dc = if rv.namespace.is_some() {
                rv.dc_id.with_context(|| {
                    format!(
                        "Imported constant '{}/{}' in '{ns_key}' has no assigned Dc",
                        vkey.ty, vkey.name
                    )
                })?
            } else {
                rv.dc_id
                    .or_else(|| local_value_dcs.get(&vkey).copied())
                    .with_context(|| {
                        format!(
                            "Local constant '{}/{}' in '{ns_key}' has no assigned Dc",
                            vkey.ty, vkey.name
                        )
                    })?
            };
            value_dcs_by_span.insert(span_key, target_dc);
        }

        let mut routine_dcs_by_span: HashMap<(usize, usize), DcChar> =
            HashMap::with_capacity(vm.resolved_calls.len());
        for (&span_key, rc) in &vm.resolved_calls {
            let rkey = rc.routine_key();
            let target_dc = if rc.namespace.is_some() {
                rc.sig.dc_id.with_context(|| {
                    format!(
                        "Imported routine '{}' in '{ns_key}' has no assigned Dc",
                        rkey.name
                    )
                })?
            } else {
                rc.sig
                    .dc_id
                    .or_else(|| local_routine_dcs.get(&rkey).copied())
                    .with_context(|| {
                        format!(
                            "Local routine '{}' in '{ns_key}' has no assigned Dc",
                            rkey.name
                        )
                    })?
            };
            routine_dcs_by_span.insert(span_key, target_dc);
        }

        let mut struct_dcs_by_span: HashMap<(usize, usize), DcChar> =
            HashMap::with_capacity(vm.resolved_structs.len());
        for (&span_key, sdef) in &vm.resolved_structs {
            let skey = (sdef.namespace.clone(), sdef.original_name.clone());
            let target_dc = sdef
                .dc_id
                .or_else(|| all_struct_dcs.get(&skey).copied())
                .with_context(|| {
                    format!(
                        "Struct '{}' in '{ns_key}' has no assigned Dc",
                        sdef.original_name
                    )
                })?;
            struct_dcs_by_span.insert(span_key, target_dc);
        }

        let mut named_struct_dcs_by_span: HashMap<(usize, usize), DcChar> =
            HashMap::with_capacity(vm.resolved_named_structs.len());
        for (&span_key, nsdef) in &vm.resolved_named_structs {
            let skey = (nsdef.namespace.clone(), nsdef.original_name.clone());
            let target_dc = nsdef
                .dc_id
                .or_else(|| all_struct_dcs.get(&skey).copied())
                .with_context(|| {
                    format!(
                        "Named struct '{}' in '{ns_key}' has no assigned Dc",
                        nsdef.original_name
                    )
                })?;
            named_struct_dcs_by_span.insert(span_key, target_dc);
        }

        let mut enum_dcs_by_span: HashMap<(usize, usize), DcChar> =
            HashMap::with_capacity(vm.resolved_enum_variants.len());
        for (&span_key, ((enum_ns, enum_name), _)) in
            &vm.resolved_enum_variants
        {
            let ekey = (enum_ns.clone(), enum_name.clone());
            let target_dc = all_nominal_dcs
                .get(&ekey)
                .copied()
                .with_context(|| {
                    format!(
                        "Enum '{enum_name}' in '{ns_key}' has no assigned Dc"
                    )
                })?;
            enum_dcs_by_span.insert(span_key, target_dc);
        }

        let maps = DeclRewriteMaps {
            vm,
            all_struct_dcs: &all_struct_dcs,
            all_nominal_dcs: &all_nominal_dcs,
            value_dcs_by_span,
            routine_dcs_by_span,
            struct_dcs_by_span,
            named_struct_dcs_by_span,
            enum_dcs_by_span,
        };

        for item in &vm.module.items {
            let (item_name, is_pub, pub_dc_syntax, item_dc) = match item {
                TopDecl::Value(v) => (
                    v.name.as_str(),
                    v.is_pub,
                    v.pub_dc_syntax,
                    v.dc_id.context("Missing Dc on validated StageL value")?,
                ),
                TopDecl::Routine(r) => (
                    r.name.as_str(),
                    r.is_pub,
                    r.pub_dc_syntax,
                    r.dc_id.context("Missing Dc on validated StageL routine")?,
                ),
                TopDecl::Struct(s) => (
                    s.name.as_str(),
                    s.is_pub,
                    s.pub_dc_syntax,
                    s.dc_id.context("Missing Dc on validated StageL struct")?,
                ),
                TopDecl::BrandType(b) => (
                    b.name.as_str(),
                    b.is_pub,
                    b.pub_dc_syntax,
                    b.dc_id
                        .context("Missing Dc on validated StageR brand/type")?,
                ),
                TopDecl::NamedStruct(s) => (
                    s.name.as_str(),
                    s.is_pub,
                    s.pub_dc_syntax,
                    s.dc_id
                        .context("Missing Dc on validated StageR named struct")?,
                ),
                TopDecl::Enum(e) => (
                    e.name.as_str(),
                    e.is_pub,
                    e.pub_dc_syntax,
                    e.dc_id.context("Missing Dc on validated StageR enum")?,
                ),
                TopDecl::Comment(_) => continue,
            };

            let subj_sh = DcShorthand::from_global_id(item_dc.to_u128());
            let label = format!("{ns_key}::{item_name}");
            let rewritten = rewrite_top_decl_refs_to_dc(item, &maps);

            if !pub_dc_syntax {
                let DcShorthand::Long(entity_id) = subj_sh else {
                    bail!("Expected Long entity ID for {label}, found {subj_sh}");
                };
                let class_spec = class_value_spec_for_top_decl(&rewritten)?;
                desired
                    .allocated_entities
                    .insert(entity_id, (class_spec, label.clone()));

                if is_pub {
                    let prop_key =
                        (subj_sh, sh(DC_PROGRAMMATIC_IDENTIFIER));
                    desired.single_properties.insert(
                        prop_key,
                        (
                            make_string_value_spec(item_name),
                            format!("{subj_sh} StageL export {item_name:?} in {ns_key}"),
                        ),
                    );
                    desired.single_property_contexts.insert(
                        (prop_key, ns_ctx_spec.clone()),
                        format!("In context of ({ns_key})"),
                    );
                }
            }

            let dc_chars = encode_top_decl_to_dc_tokens(&rewritten)?;
            // Gate 4 lossless Token-Dc roundtrip verification before ledger emission:
            let decoded = decode_top_decl_from_dc_tokens(&dc_chars)?;
            if vm.module.dialect == StageDialect::StageR3
                || top_decl_uses_stager(&decoded)
            {
                let _formatted = format_top_decl_to_sr3(&decoded)?;
            } else {
                let _formatted = format_top_decl_to_sl3(&decoded)?;
            }

            let token_shorthands: Vec<DcShorthand> = dc_chars
                .iter()
                .map(|c| DcShorthand::from_global_id(c.to_u128()))
                .collect();
            let body_spec = ValueSpec::ValueNode {
                tokens: token_shorthands,
                comment: format!("StageL 3 Token-Dc {label}"),
            };
            desired.multi_properties.insert(
                (subj_sh, sh(DC_CANONICAL_DECOMPOSITION), body_spec),
                format!("{subj_sh} StageL 3 decomposition ({label})"),
            );
        }
    }

    Ok(())
}

/// Emits standalone `@converts(<src> <dst>)` routine entities (`Dc 280 "Built-in type identifier: Routine"`)
/// and their `Dc 595 "Data format or type description"` `[Dc 302 "Type conversion" <src> <dst>]`
/// and `.WASM` / Token-Dc decompositions into `desired`.
pub(crate) fn compile_standalone_staged_impls_into_desired_state(
    staged_impls: &[crate::wat::StagedDcImpl],
    desired: &mut DesiredDatasetState,
) -> Result<()> {
    for staged in staged_impls {
        let (DcShorthand::Long(entity_id), Some((src_sh, dst_sh))) =
            (staged.target, staged.converts)
        else {
            continue;
        };
        if let Some((existing_spec, prev_label)) =
            desired.allocated_entities.get(&entity_id)
            && *existing_spec != ValueSpec::Direct(sh(DC_TYPE_ROUTINE))
        {
            bail!(
                "Entity ID l{entity_id} in {} conflicts with '{prev_label}'",
                staged.rel_path
            );
        }
        let label =
            format!("{} (@converts {src_sh} -> {dst_sh})", staged.rel_path);
        desired.allocated_entities.insert(
            entity_id,
            (ValueSpec::Direct(sh(DC_TYPE_ROUTINE)), label.clone()),
        );

        let subj_sh = DcShorthand::Long(entity_id);
        let fmt_expr = FormatExpr::Convert(
            Box::new(FormatExpr::Dc(src_sh)),
            Box::new(FormatExpr::Dc(dst_sh)),
        );
        desired.single_properties.insert(
            (subj_sh, sh(DC_DATA_FORMAT_DESCRIPTION)),
            (
                ValueSpec::FormatExprNode {
                    expr: Box::new(fmt_expr),
                    raw_comment: format!("{subj_sh} format spec {src_sh} > {dst_sh}"),
                },
                format!("Format spec ({subj_sh}: {src_sh} > {dst_sh})"),
            ),
        );

        let decomp_dc = if staged.decomp_tag == "equiv" {
            DC_CANONICAL_DECOMPOSITION
        } else {
            DC_SEMANTIC_DECOMPOSITION
        };
        let val_spec = if staged.tokens.len() == 1
            && let Some(&only) = staged.tokens.first()
        {
            ValueSpec::Direct(only)
        } else {
            ValueSpec::ValueNode {
                tokens: staged.tokens.clone(),
                comment: format!(
                    "<{}> {} ({src_sh} > {dst_sh})",
                    staged.decomp_tag, staged.rel_path
                ),
            }
        };
        if !staged.context_tokens.is_empty() {
            let ctx_spec = if staged.context_tokens.len() == 1
                && let Some(&only_ctx) = staged.context_tokens.first()
            {
                ValueSpec::Direct(only_ctx)
            } else {
                let joined: Vec<String> = staged
                    .context_tokens
                    .iter()
                    .map(ToString::to_string)
                    .collect();
                ValueSpec::ValueNode {
                    tokens: staged.context_tokens.clone(),
                    comment: format!("context {}", joined.join(" ")),
                }
            };
            desired.multi_property_contexts.insert(
                ((subj_sh, sh(decomp_dc), val_spec.clone()), ctx_spec),
                format!("{subj_sh} <{}> context", staged.decomp_tag),
            );
        }
        desired.multi_properties.insert(
            (subj_sh, sh(decomp_dc), val_spec),
            format!("{subj_sh} <{}> ({label})", staged.decomp_tag),
        );
    }
    Ok(())
}

/// Discovers `.sl3` namespace modules and standalone `@converts(...)` converter files under
/// `data_dir`, records their SHA-256 digests in `pinned_input_sha256s`, and compiles their
/// System-graph entities and decompositions into `desired`.
pub(crate) fn compile_repo_stagel_into_desired_state(
    data_dir: &Path,
    desired: &mut DesiredDatasetState,
    pinned_input_sha256s: &mut BTreeMap<String, String>,
) -> Result<()> {
    let (staged_impls, _) = crate::wat::load_staged_dc_impls(data_dir)?;
    compile_standalone_staged_impls_into_desired_state(&staged_impls, desired)?;
    let files = discover_stagel_namespace_files(data_dir)?;
    for f in &files {
        pinned_input_sha256s.insert(
            f.repo_rel_path.clone(),
            crate::graph_ledger::sha256_bytes_hex(&f.raw_bytes),
        );
    }
    compile_stagel_files_into_desired_state(&files, desired)
}

/// Convenience helper for in-memory tests: runs StageL `: @dc` allocation + full CSV/StageL
/// ledger reconciliation in memory.
pub fn reconcile_stagel_and_csv_in_memory(
    existing_nodes: Vec<LedgerNode>,
    files: &mut [DiscoveredStagelFile],
    dc_defns: &[crate::dc_def::DcDefn],
    format_defns: &[crate::dc_def::DcDefn],
    known_scripts: &BTreeSet<String>,
    named_types_bytes: Option<&[u8]>,
) -> Result<(Vec<LedgerNode>, StagelAllocStats, LedgerUpdateStats)> {
    let base_count = existing_nodes.len();
    let (after_alloc_nodes, alloc_stats) =
        assign_stagel_dc_ids_in_memory(existing_nodes, files)?;
    let alloc_appended = after_alloc_nodes.len().saturating_sub(base_count);
    let (final_nodes, mut ledger_stats) =
        crate::graph_ledger::update_ledger_in_memory_with_extra_state(
            after_alloc_nodes,
            crate::graph_ledger::NodeProvenanceMap::default(),
            dc_defns,
            format_defns,
            known_scripts,
            named_types_bytes,
            |desired| compile_stagel_files_into_desired_state(files, desired),
        )?;
    ledger_stats.nodes_appended =
        ledger_stats.nodes_appended.saturating_add(alloc_appended);
    Ok((final_nodes, alloc_stats, ledger_stats))
}

/// Validates Gate 4 `.WASM` System-graph boundaries and lossless StageL Token-Dc roundtrips
/// across `index`, given the set of allowed Stage 0 `.wat` targets (`stage0_targets`).
pub fn validate_system_graph_wasm_and_stagel_boundaries(
    index: &SeedGraphIndex,
    stage0_targets: &HashSet<DcShorthand>,
    source_path_label: &str,
    report: &mut ValidationReport,
) {
    let sh_begin_wasm = sh(DC_BEGIN_WASM_PROGRAM);
    let sh_in_ctx = sh(DC_IN_CONTEXT_OF);

    for prop_dc in [
        DC_CANONICAL_DECOMPOSITION,
        DC_DECOMPOSITION,
        DC_SEMANTIC_DECOMPOSITION,
    ] {
        let prop_sh = sh(prop_dc);
        let mut keys: Vec<(DcShorthand, DcShorthand)> = index
            .multi_props_by_subj_prop
            .keys()
            .filter(|&&(_, p)| p == prop_sh)
            .copied()
            .collect();
        keys.sort_unstable();

        for (subj, _) in keys {
            for entry in index.get_multi_properties(subj, prop_sh) {
                let DcShorthand::Long(val_node_id) = entry.value else {
                    continue;
                };
                let Some(val_node) = index.get_active_node(val_node_id) else {
                    continue;
                };

                // 1. Gate 4 `.WASM` boundary check: `.WASM` (`Dc 1927` / `wasm_routine`)
                // may ONLY appear on Stage 0 `.wat` targets (never on `Long(_)` Stage 1/2
                // entities or non-Stage-0 Dcs).
                let is_wasm = val_node.tokens.first() == Some(&sh_begin_wasm)
                    || index.tokens_are_wasm_helper_or_definition(&val_node.tokens);
                if is_wasm && !stage0_targets.contains(&subj) {
                    let ctxs = index.get_direct_objects(
                        sh_in_ctx,
                        DcShorthand::Long(entry.has_a_id),
                    );
                    report.add_error(
                        source_path_label,
                        None,
                        Some("WASM boundary"),
                        format!(
                            "System-graph subject {subj} (contexts: {ctxs:?}) owns a .WASM decomposition (l{val_node_id}), but .WASM in the System graph is restricted to Stage 0 stage_0_impls/*.wat targets"
                        ),
                        Some("Emit Stage 1 and Stage 2 implementations as lossless StageL/StageR Token Dcs rather than compiled .WASM"),
                    );
                }

                // 2. StageL Token-Dc lossless roundtrip check: every StageL Token-Dc node
                // in the System graph must decode and format back to valid `.sl3`.
                if index.tokens_are_stagel_item_or_module(&val_node.tokens) {
                    let expanded =
                        index.expand_interned_unicode_runs(&val_node.tokens);
                    let roundtrip_res: Result<()> = (|| {
                        let mut chars = Vec::with_capacity(expanded.len());
                        for t in expanded {
                            let gid = t.to_global_id().with_context(|| {
                                format!("Invalid DcShorthand {t}")
                            })?;
                            chars.push(DcChar::from_u128(gid));
                        }
                        let stripped =
                            crate::stagel3::dc_tokens::strip_comment_frames(
                                &chars,
                            );
                        if let Some(first) = stripped.first().copied() {
                            let u = first.to_u128();
                            if index.token_fills_canonical_or_equiv_role(
                                u,
                                crate::syntax::DcRole::BeginModule,
                            ) {
                                let m = decode_module_from_dc_tokens(&chars)?;
                                let _ = format_module_to_sl3(&m)?;
                                Ok(())
                            } else if index.token_fills_canonical_or_equiv_role(
                                u,
                                crate::syntax::DcRole::BeginStagerModule,
                            ) {
                                let m = decode_module_from_dc_tokens(&chars)?;
                                let _ = format_module_to_sr3(&m)?;
                                Ok(())
                            } else {
                                let d = decode_top_decl_from_dc_tokens(&chars)?;
                                if top_decl_uses_stager(&d) {
                                    let _ = format_top_decl_to_sr3(&d)?;
                                } else {
                                    let _ = format_top_decl_to_sl3(&d)?;
                                }
                                Ok(())
                            }
                        } else {
                            bail!("Empty StageL/StageR Token-Dc decomposition node");
                        }
                    })();
                    if let Err(err) = roundtrip_res {
                        report.add_error(
                            source_path_label,
                            None,
                            Some("StageL Token-Dc"),
                            format!(
                                "StageL/StageR Token-Dc decomposition l{val_node_id} on {subj} failed lossless .sl3/.sr3 roundtrip: {err:#}"
                            ),
                            Some("Re-run validate-data-tables --write to regenerate valid StageL/StageR Token Dcs"),
                        );
                    }
                }
            }
        }
    }
}

/// Validates all StageL 3 namespace files under `data_src_dir` and checks System-graph
/// Gate 4 `.WASM` / StageL Token-Dc invariants against `repo_root/assets/data/graph`.
pub fn validate_stagel_and_system_graph_in_repo(
    repo_root: &Path,
    data_src_dir: &Path,
    known_dc_ids: &HashSet<u32>,
    known_format_ids: &HashSet<usize>,
    report: &mut ValidationReport,
) {
    let files = match discover_stagel_namespace_files(data_src_dir) {
        Ok(f) => f,
        Err(err) => {
            report.add_error(
                &data_src_dir.display().to_string(),
                None,
                Some("StageL"),
                format!("Failed to discover StageL namespace modules: {err:#}"),
                Some("Check file extensions and directory layout in stage_1_impls/ and impls/"),
            );
            return;
        }
    };

    let (workspace, ns_to_file_idx) =
        match parse_validate_and_test_stagel_files(&files) {
            Ok(pair) => pair,
            Err(err) => {
                report.add_error(
                    &data_src_dir.display().to_string(),
                    None,
                    Some("StageL"),
                    format!("StageL namespace validation or inline test failed: {err:#}"),
                    Some("Fix syntax, type, PSR-4 path, or inline test errors in .sl3 files"),
                );
                return;
            }
        };

    match collect_unassigned_and_validate_assigned_dcs(
        &workspace,
        &files,
        &ns_to_file_idx,
        false,
    ) {
        Ok(unassigned) => {
            for site in unassigned {
                // Reason for fallback: site.file_idx was constructed from files indices.
                let path_label = files
                    .get(site.file_idx)
                    .map_or("stage_1_impls", |f| f.repo_rel_path.as_str());
                report.add_error(
                    path_label,
                    Some(site.one_based_line),
                    Some("@dc"),
                    format!(
                        "StageL top-level declaration '{}' is missing ': @dc l<id>'",
                        site.label
                    ),
                    Some("Run ./scripts/validate-data-tables to allocate a permanent System-block Dc"),
                );
            }
        }
        Err(err) => {
            report.add_error(
                &data_src_dir.display().to_string(),
                None,
                Some("@dc"),
                format!("Invalid StageL ': @dc' / 'pub dc/' assignment: {err:#}"),
                Some("Ensure every ': @dc' ID is unique and in the System block"),
            );
        }
    }

    // Verify `pub dc/<short_or_format>` targets exist in CSV definitions.
    for vm in &workspace.modules {
        for item in &vm.module.items {
            let (pub_dc_syntax, dc_opt, span) = match item {
                TopDecl::Routine(r) => (r.pub_dc_syntax, r.dc_id, r.span),
                TopDecl::Value(v) => (v.pub_dc_syntax, v.dc_id, v.span),
                TopDecl::Struct(s) => (s.pub_dc_syntax, s.dc_id, s.span),
                TopDecl::BrandType(b) => (b.pub_dc_syntax, b.dc_id, b.span),
                TopDecl::NamedStruct(s) => (s.pub_dc_syntax, s.dc_id, s.span),
                TopDecl::Enum(e) => (e.pub_dc_syntax, e.dc_id, e.span),
                TopDecl::Comment(_) => continue,
            };
            if pub_dc_syntax && let Some(dc) = dc_opt {
                match DcShorthand::from_global_id(dc.to_u128()) {
                    DcShorthand::Short(sid) if !known_dc_ids.contains(&sid) => {
                        report.add_error(
                            &data_src_dir.display().to_string(),
                            Some(span.line),
                            Some("pub dc"),
                            format!("'pub dc/{sid}' targets unknown Short Dc {sid}"),
                            Some("Define the target Short Dc in categories/*.csv"),
                        );
                    }
                    DcShorthand::Format(fid)
                        if !known_format_ids.contains(&fid) =>
                    {
                        report.add_error(
                            &data_src_dir.display().to_string(),
                            Some(span.line),
                            Some("pub dc"),
                            format!("'pub dc/f{fid}' targets unknown Format Dc f{fid}"),
                            Some("Define the target Format Dc in categories/formats/*.csv"),
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    let graph_dir = repo_root.join("assets/data/graph");
    if !graph_dir.is_dir() {
        return;
    }
    let Ok(nodes) = load_ledger_nodes_from_dir(&graph_dir) else {
        return;
    };
    let single_props = single_valued_property_set();
    let index = SeedGraphIndex::from_nodes_with_single_props(nodes, &single_props);

    // Verify every assigned `: @dc l<id>` in `.sl3` exists as an active entity in `assets/data/graph`.
    for vm in &workspace.modules {
        // Reason for fallback: validated namespace modules always set namespace_key.
        let ns_key = vm.namespace_key.as_deref().unwrap_or("");
        let fmeta_opt = ns_to_file_idx
            .get(ns_key)
            .and_then(|&idx| files.get(idx));
        // Reason for fallback: defaults to data_src_dir path when file entry is absent.
        let file_label = fmeta_opt.map_or("stage_1_impls", |f| f.repo_rel_path.as_str());
        for item in &vm.module.items {
            let (name, pub_dc_syntax, dc_opt, span) = match item {
                TopDecl::Routine(r) => {
                    (r.name.as_str(), r.pub_dc_syntax, r.dc_id, r.span)
                }
                TopDecl::Value(v) => {
                    (v.name.as_str(), v.pub_dc_syntax, v.dc_id, v.span)
                }
                TopDecl::Struct(s) => {
                    (s.name.as_str(), s.pub_dc_syntax, s.dc_id, s.span)
                }
                TopDecl::BrandType(b) => {
                    (b.name.as_str(), b.pub_dc_syntax, b.dc_id, b.span)
                }
                TopDecl::NamedStruct(s) => {
                    (s.name.as_str(), s.pub_dc_syntax, s.dc_id, s.span)
                }
                TopDecl::Enum(e) => {
                    (e.name.as_str(), e.pub_dc_syntax, e.dc_id, e.span)
                }
                TopDecl::Comment(_) => continue,
            };
            if !pub_dc_syntax
                && let Some(dc) = dc_opt
                && let DcShorthand::Long(entity_id) =
                    DcShorthand::from_global_id(dc.to_u128())
                && index.get_entity_class(entity_id).is_none()
            {
                report.add_error(
                    file_label,
                    Some(span.line),
                    Some("@dc"),
                    format!(
                        "StageL entity ': @dc l{entity_id}' on '{ns_key}::{name}' does not exist as an active entity node in assets/data/graph/*.ctbnodes"
                    ),
                    Some("Run ./scripts/validate-data-tables to reconcile the System graph ledger"),
                );
            }
        }
    }

    let mut stage0_targets = HashSet::new();
    if let Ok((staged_impls, _)) = crate::wat::load_staged_dc_impls(data_src_dir)
    {
        for imp in staged_impls {
            if let DcShorthand::Long(entity_id) = imp.target
                && imp.converts.is_some()
                && index.get_entity_class(entity_id)
                    != Some(sh(DC_TYPE_ROUTINE))
            {
                report.add_error(
                    &imp.rel_path,
                    None,
                    Some("@dc"),
                    format!(
                        "Standalone converter '@dc l{entity_id}' does not exist as an active Routine (Dc 280) entity node in assets/data/graph/*.ctbnodes"
                    ),
                    Some("Run ./scripts/validate-data-tables to reconcile the System graph ledger"),
                );
            }
            if imp.stage == ImplStage::Stage0 {
                stage0_targets.insert(imp.target);
            }
        }
    }
    validate_system_graph_wasm_and_stagel_boundaries(
        &index,
        &stage0_targets,
        "assets/data/graph",
        report,
    );
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
    use crate::dc::{DC_SUPERSEDES, DC_WASM_START};
    use crate::graph_ledger::encode_statement_tokens;

    fn make_mem_file(
        stage: ImplStage,
        psr4_rel: &str,
        src: &str,
    ) -> DiscoveredStagelFile {
        DiscoveredStagelFile {
            stage,
            disk_path: PathBuf::from(psr4_rel),
            repo_rel_path: format!(
                "src/formats/dcdata/data-src/{}/{psr4_rel}",
                stage.dir_name()
            ),
            psr4_rel_path: psr4_rel.to_owned(),
            raw_bytes: src.as_bytes().to_vec(),
            source: src.to_owned(),
        }
    }

    #[crate::ctb_test]
    fn test_stagel_dc_alloc_idempotence_supersession_rename_and_retraction() -> Result<()> {
        let scripts = BTreeSet::new();
        let mut files = vec![
            make_mem_file(
                ImplStage::Stage1,
                "std/demo/base.sl3",
                concat!(
                    "ns std/demo/base.\n\n",
                    ": Helper constant\n",
                    "pub i32/OFFSET i32:10.\n\n",
                    "v/i32/double i32/x :: mul i32/x i32:2.\n\n",
                    "pub v/i32/compute i32/x\n",
                    "    test\n",
                    "        in i32:4\n",
                    "        out i32:18\n",
                    "    add double i32/x i32/OFFSET.\n",
                ),
            ),
            make_mem_file(
                ImplStage::Stage1,
                "std/demo/caller.sl3",
                concat!(
                    "ns std/demo/caller.\n",
                    "use std/demo/base: compute OFFSET ::.\n\n",
                    "pub v/i32/run i32/n\n",
                    "    test\n",
                    "        in i32:3\n",
                    "        out i32:26\n",
                    "    add compute i32/n i32/OFFSET.\n",
                ),
            ),
        ];

        // 1. Initial auto-allocation + ledger reconciliation
        let (nodes1, alloc1, stats1) = reconcile_stagel_and_csv_in_memory(
            Vec::new(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc1.entities_allocated, 4);
        assert_eq!(alloc1.files_updated, 2);
        assert!(files[0].source.contains(": @dc l"));
        assert!(files[1].source.contains(": @dc l"));
        assert!(stats1.nodes_appended > 0);
        assert_eq!(stats1.nodes_superseded, 0);
        assert_eq!(stats1.nodes_retracted, 0);

        // 2. Second run with `: @dc l<id>` already written must be a 100% no-op!
        let (nodes2, alloc2, stats2) = reconcile_stagel_and_csv_in_memory(
            nodes1.clone(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc2.entities_allocated, 0);
        assert_eq!(alloc2.files_updated, 0);
        assert_eq!(stats2.nodes_appended, 0);
        assert_eq!(stats2.nodes_superseded, 0);
        assert_eq!(stats2.nodes_retracted, 0);
        assert_eq!(nodes1, nodes2);

        // Verify SeedGraphIndex resolves exports and self-contained Token-Dc bodies
        let single_props = single_valued_property_set();
        let idx1 =
            SeedGraphIndex::from_nodes_with_single_props(nodes2.clone(), &single_props);
        let base_exports = idx1.resolve_stagel_namespace_exports(&[
            "std".to_owned(),
            "demo".to_owned(),
            "base".to_owned(),
        ])?;
        assert_eq!(base_exports.len(), 2);
        let compute_sh = *base_exports
            .get("compute")
            .and_then(|v| v.first())
            .context("missing compute")?;
        let offset_sh = *base_exports
            .get("OFFSET")
            .and_then(|v| v.first())
            .context("missing OFFSET")?;
        let (_, compute_tokens) = idx1
            .resolve_stagel_decomposition_tokens(compute_sh)?
            .unwrap();
        let decoded_compute = decode_top_decl_from_dc_tokens(&compute_tokens)?;
        let formatted_compute = format_top_decl_to_sl3(&decoded_compute)?;
        // Peer call `double` and constant `OFFSET` were rewritten to `dc/l<id>`!
        assert!(formatted_compute.contains(&format!("i32/dc/{offset_sh}")));
        assert!(formatted_compute.contains("dc/l"));

        // 3. Edit `compute`'s body (keeping its `: @dc l<id>`) -> supersedes only `Dc 610` (`Has value`) via `Dc 591`!
        let pin_before_edit = idx1.system_block_most_recent_node_id().unwrap();
        files[0].source = files[0]
            .source
            .replace("i32:18", "i32:19")
            .replace(
                "add double i32/x i32/OFFSET.",
                "add add double i32/x i32/OFFSET i32:1.",
            );
        files[0].raw_bytes = files[0].source.as_bytes().to_vec();
        files[1].source = files[1].source.replace("i32:26", "i32:27");
        files[1].raw_bytes = files[1].source.as_bytes().to_vec();

        let (nodes3, alloc3, stats3) = reconcile_stagel_and_csv_in_memory(
            nodes2,
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc3.entities_allocated, 0);
        assert!(stats3.nodes_superseded >= 1);
        assert_eq!(stats3.nodes_retracted, 0);
        assert!(nodes3.iter().any(|n| n.tokens.first() == Some(&sh(DC_SUPERSEDES))));

        // Historical snapshot pinned at `pin_before_edit` still resolves the OLD `compute` body!
        let pinned_idx =
            SeedGraphIndex::from_nodes_at_most_recent(nodes3.clone(), pin_before_edit);
        let (_, old_tokens) = pinned_idx
            .resolve_stagel_decomposition_tokens(compute_sh)?
            .unwrap();
        assert_eq!(old_tokens, compute_tokens);

        // 4. Rename `run` -> `execute` (keeping `: @dc`) and delete `caller` file's export if removed:
        files[1].source = files[1].source.replace("pub v/i32/run", "pub v/i32/execute");
        files[1].raw_bytes = files[1].source.as_bytes().to_vec();
        let (nodes4, alloc4, stats4) = reconcile_stagel_and_csv_in_memory(
            nodes3,
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc4.entities_allocated, 0);
        assert!(stats4.nodes_superseded >= 1);
        let idx4 =
            SeedGraphIndex::from_nodes_with_single_props(nodes4.clone(), &single_props);
        let caller_exports = idx4.resolve_stagel_namespace_exports(&[
            "std".to_owned(),
            "demo".to_owned(),
            "caller".to_owned(),
        ])?;
        assert!(!caller_exports.contains_key("run"));
        let execute_sh = *caller_exports
            .get("execute")
            .and_then(|v| v.first())
            .context("missing execute")?;

        // 5. Delete `caller.sl3` completely -> retracts `execute_sh` via `Dc 592`!
        files.pop();
        let (nodes5, _, stats5) = reconcile_stagel_and_csv_in_memory(
            nodes4,
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert!(stats5.nodes_retracted >= 1);
        let idx5 =
            SeedGraphIndex::from_nodes_with_single_props(nodes5, &single_props);
        assert!(idx5.resolve_stagel_decomposition_tokens(execute_sh).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate4_wasm_boundary_validator_rejects_stage1_wasm() {
        let single_props = single_valued_property_set();
        let entity_id = SYSTEM_REGION_START;
        let wasm_val_id = SYSTEM_REGION_START + 1;
        let has_a_id = SYSTEM_REGION_START + 2;
        let has_val_id = SYSTEM_REGION_START + 3;
        let nodes = vec![
            LedgerNode {
                id: entity_id,
                tokens: encode_statement_tokens(
                    sh( crate::dc::DC_IS_INSTANCE_OF ),
                    sh( crate::dc::DC_ENCLOSING_DOCUMENT ),
                    sh(DC_TYPE_ROUTINE),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: wasm_val_id,
                tokens: vec![
                    sh(DC_BEGIN_WASM_PROGRAM),
                    sh(DC_WASM_START),
                    sh(crate::dc::DC_END_WASM_PROGRAM),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: has_a_id,
                tokens: encode_statement_tokens(
                    sh(crate::dc::DC_HAS_A),
                    DcShorthand::Long(entity_id),
                    sh(DC_CANONICAL_DECOMPOSITION),
                ),
                comment: String::new(),
            },
            LedgerNode {
                id: has_val_id,
                tokens: encode_statement_tokens(
                    sh(crate::dc::DC_HAS_VALUE),
                    DcShorthand::Long(has_a_id),
                    DcShorthand::Long(wasm_val_id),
                ),
                comment: String::new(),
            },
        ];
        let index = SeedGraphIndex::from_nodes_with_single_props(nodes, &single_props);
        let mut report = ValidationReport::new();
        validate_system_graph_wasm_and_stagel_boundaries(
            &index,
            &HashSet::new(),
            "test-graph",
            &mut report,
        );
        assert!(report.has_errors());
        assert!(report.format_report().contains("WASM boundary"));
    }

    #[crate::ctb_test]
    fn test_stagel_dc_alloc_and_seed_linker_with_structs_and_tuples() -> Result<()> {
        let scripts = BTreeSet::new();
        let mut files = vec![
            make_mem_file(
                ImplStage::Stage1,
                "std/geo/types.sl3",
                concat!(
                    "ns std/geo/types.\n\n",
                    "pub struct point :: i32 i32.\n",
                    "pub struct tree :: point l/tree.\n",
                    "pub point/ORIGIN structNew point: i32:0 i32:0 ::.\n",
                ),
            ),
            make_mem_file(
                ImplStage::Stage1,
                "std/geo/ops.sl3",
                concat!(
                    "ns std/geo/ops.\n",
                    "use std/geo/types: point as pt tree ORIGIN ::.\n\n",
                    "pub v/i32/sumTree tree/node\n",
                    "    test baseCase\n",
                    "        in structNew tree: /ORIGIN l/tree: :: ::\n",
                    "        out i32:0.\n",
                    "    let i32/acc add field field /node u32:0 u32:0 field field /node u32:0 u32:1.\n",
                    "    foreach tree/ch field /node u32:1 do\n",
                    "        set /acc add /acc sumTree /ch.\n",
                    "    ::.\n",
                    "    /acc.\n\n",
                    "pub v/pt/translate pt/p tup: i32 i32 ::/delta\n",
                    "    with with /p u32:0 add field /p u32:0 field /delta u32:0 u32:1 add field /p u32:1 field /delta u32:1.\n",
                ),
            ),
        ];

        // 1. Allocate permanent Dcs (two-pass struct-first) and reconcile ledger
        let (nodes1, alloc1, _) = reconcile_stagel_and_csv_in_memory(
            Vec::new(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc1.entities_allocated, 5);
        assert_eq!(alloc1.files_updated, 2);

        // 2. Second reconciliation is 100% idempotent (entity classes match `[303 l<point_id>]`!)
        let (nodes2, alloc2, stats2) = reconcile_stagel_and_csv_in_memory(
            nodes1.clone(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc2.entities_allocated, 0);
        assert_eq!(stats2.nodes_appended, 0);
        assert_eq!(stats2.nodes_superseded, 0);
        assert_eq!(nodes1, nodes2);

        // 3. Link & evaluate an external caller module against `SeedGraphIndex`!
        let single_props = single_valued_property_set();
        let idx = SeedGraphIndex::from_nodes_with_single_props(nodes2, &single_props);
        let caller_src = r#"ns app/check.
use std/geo/types: point as vec2 tree ORIGIN ::.
use std/geo/ops: sumTree translate ::.

pub v/b/runCheck
    test linkedGraphStructs
        in.
        out true.
    const vec2/p1 translate /ORIGIN tupNew: i32:4 i32:6 ::.
    const tree/child structNew tree: /p1 l/tree: :: ::.
    const tree/root structNew tree: structNew vec2: i32:1 i32:2 :: l/tree: /child :: ::.
    eq sumTree /root i32:13.
"#;
        let report =
            crate::stagel3::seed_linker::eval_sl3_inline_tests_with_seed_graph(
                caller_src, &idx,
            )?;
        assert_eq!(report.ensure_all_passed()?, 1);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager3_dc_alloc_and_stage_gating() -> Result<()> {
        let scripts = BTreeSet::new();
        let mut files = vec![make_mem_file(
            ImplStage::Stage2,
            "impls_demo/model.sr3",
            concat!(
                "ns impls_demo/model.\n\n",
                "pub brand u64/meter.\n",
                "pub type u64/nonZero.\n",
                "pub struct rect :: meter/w meter/h.\n",
                "pub enum shape\n",
                "    empty.\n",
                "    rect/boxShape.\n\n",
                "pub rect/UNIT_SQUARE structNew rect: w wrap u64:1 meter h wrap u64:1 meter ::.\n\n",
                "pub v/result/nonZero/mkNz u64/n\n",
                "    if eq /n u64:0\n",
                "        result/nonZero:err error:parseFailure\n",
                "        result/nonZero:ok wrap /n nonZero.\n\n",
                "pub v/u64/area shape/sh\n",
                "    test computes box area\n",
                "        in shape:boxShape structNew rect: w wrap u64:3 meter h wrap u64:4 meter ::\n",
                "        out u64:12\n",
                "    match /sh\n",
                "        case empty u64:0.\n",
                "        case boxShape rect/r wrapMul unwrap field /r w unwrap field /r h.\n",
                "    ::.\n",
            ),
        )];

        // 1. Two-pass assignment puts `: @dc l<id>` on every StageR decl (`brand`, `type`, `struct`, `enum`, const, routine)
        let (nodes1, alloc1) = assign_stagel_dc_ids_in_memory(Vec::new(), &mut files)?;
        assert_eq!(alloc1.entities_allocated, 7);
        assert_eq!(alloc1.files_updated, 1);
        assert_eq!(files[0].source.matches(": @dc l").count(), 7);

        // 2. Initial full reconciliation appends statement/decomposition nodes with zero new entity IDs;
        //    re-running reconciliation on the resulting state is a 100% no-op.
        let (nodes2, alloc2, stats2) = reconcile_stagel_and_csv_in_memory(
            nodes1,
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc2.entities_allocated, 0);
        assert_eq!(alloc2.files_updated, 0);
        assert!(stats2.nodes_appended > 0);

        let (nodes3, alloc3, stats3) = reconcile_stagel_and_csv_in_memory(
            nodes2.clone(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        assert_eq!(alloc3.entities_allocated, 0);
        assert_eq!(alloc3.files_updated, 0);
        assert_eq!(stats3.nodes_appended, 0);
        assert_eq!(stats3.nodes_superseded, 0);
        assert_eq!(stats3.nodes_retracted, 0);
        assert_eq!(nodes2, nodes3);

        let mut desired = DesiredDatasetState::default();
        compile_stagel_files_into_desired_state(&files, &mut desired)?;
        assert!(
            desired.multi_properties.keys().any(|(_, prop, spec)| {
                *prop == sh(DC_CANONICAL_DECOMPOSITION)
                    && matches!(
                        spec,
                        ValueSpec::ValueNode { tokens, .. }
                            if tokens.contains(&crate::DcShorthand::Short(2224))
                    )
            }),
            "Expected compiled StageR 3 named struct Token-Dc decomposition (Dc 2224)"
        );
        let mod_tokens = crate::stagel3::dc_tokens::sr3_to_dc_tokens(&files[0].source)?;
        assert_eq!(mod_tokens.first(), Some(&crate::dc::DC_BEGIN_STAGER_MODULE));

        // 3. Reconstructed SeedGraphIndex links and executes an external `.sr3` caller
        let single_props = single_valued_property_set();
        let idx = SeedGraphIndex::from_nodes_with_single_props(nodes3, &single_props);
        let caller_sr3 = r#"ns app/use_model.
use impls_demo/model: nonZero rect shape UNIT_SQUARE area mkNz ::.

pub v/result/u64/runCheck
    test links graph StageR nominal types
        in.
        out result/u64:ok u64:2
    const nonZero/_nz mkNz? u64:5.
    const shape/sq shape:boxShape /UNIT_SQUARE.
    result/u64:ok wrapAdd area /sq u64:1.
"#;
        let report =
            crate::stagel3::seed_linker::eval_sr3_inline_tests_with_seed_graph(
                caller_sr3,
                &idx,
            )?;
        assert_eq!(report.ensure_all_passed()?, 1);

        // 4. Stage 1 (`stage_1_impls/`) must reject `.sr3` files!
        let mut stage1_bad = vec![make_mem_file(
            ImplStage::Stage1,
            "bad/stage1.sr3",
            "ns bad/stage1.\npub brand u32/m.\n",
        )];
        assert!(assign_stagel_dc_ids_in_memory(Vec::new(), &mut stage1_bad).is_err());
        Ok(())
    }
}
