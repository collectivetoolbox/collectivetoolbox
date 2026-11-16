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

//! Multi-module workspace loading, PSR-4 path enforcement, topological DAG
//! sorting, import-context pre-scanning, and StageR dialect classification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    Expr, Module, RoutineDecl, StageDialect, StageLType, StructTypeRef,
    TestExpected, TopDecl, TypedIntLit, format_dc_shorthand,
    parse_dc_shorthand,
};
use super::super::parser::{
    StageRImportContext, StageRTypeKind, parse_module_with_imports,
    parse_sr3_module_with_imports, scan_module_header_paths_with_dialect,
};
use super::imports_and_decls::validate_module_with_registry_and_dc_imports_and_options;
use super::{
    EnumVariantDef, ImportedSymbol, ModuleInput, NamespaceExports,
    ValidatedWorkspace,
};
use crate::DcChar;
use std::collections::HashMap;

/// Validates a collection of `.sl3` or `.sr3` source files (resolving PSR-4
/// `ns` paths and acyclic `use` DAG imports).
///
/// Files whose `source_path` ends with `.sr3` or `.sri3` are parsed and
/// validated as StageR 3; other paths (and `None`) default to StageL 3. Use
/// [`parse_and_validate_stager_workspace`] when validating anonymous StageR 3
/// sources without file paths.
pub fn parse_and_validate_workspace(
    inputs: &[ModuleInput<'_>],
) -> Result<ValidatedWorkspace> {
    parse_and_validate_workspace_with_dialect(inputs, StageDialect::StageL3)
}

/// Validates a collection of StageR 3 (`.sr3`) source modules, defaulting
/// modules with `source_path: None` to [`StageDialect::StageR3`].
pub fn parse_and_validate_stager_workspace(
    inputs: &[ModuleInput<'_>],
) -> Result<ValidatedWorkspace> {
    parse_and_validate_workspace_with_dialect(inputs, StageDialect::StageR3)
}

/// Returns the explicit bootstrap stage rank (`1` for `stage_1_impls/`,
/// `2` for `stage_2_impls/`, `3` for `impls/`) when `source_path` resides in a
/// staged implementation directory.
#[must_use]
pub(crate) fn stage_rank_for_source_path(source_path: Option<&str>) -> Option<u8> {
    let norm = source_path?.replace('\\', "/");
    for seg in norm.split('/') {
        match seg {
            "stage_1_impls" => return Some(1),
            "stage_2_impls" => return Some(2),
            "impls" => return Some(3),
            _ => {}
        }
    }
    None
}

pub(crate) fn can_overlay_namespace(
    prev_rank: Option<u8>,
    prev_dialect: StageDialect,
    cur_rank: Option<u8>,
    cur_dialect: StageDialect,
) -> bool {
    match (prev_rank, cur_rank) {
        (Some(pr), Some(cr)) => pr != cr,
        _ => prev_dialect != cur_dialect,
    }
}

/// Validates a collection of `.sl3` / `.sr3` source files using
/// `default_dialect` for any input without a `.sl3`/`.sli3`/`.sr3`/`.sri3`
/// file extension.
#[allow(
    clippy::too_many_lines,
    reason = "Orchestrates staged PSR-4 verification, multi-stratum DAG resolution, topological sorting, and workspace validation"
)]
pub fn parse_and_validate_workspace_with_dialect(
    inputs: &[ModuleInput<'_>],
    default_dialect: StageDialect,
) -> Result<ValidatedWorkspace> {
    let mut dialects = Vec::with_capacity(inputs.len());
    let mut stage_ranks = Vec::with_capacity(inputs.len());
    let mut headers = Vec::with_capacity(inputs.len());
    let mut ns_strata: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, inp) in inputs.iter().enumerate() {
        let dialect = dialect_for_source_path(inp.source_path, default_dialect);
        let rank = stage_rank_for_source_path(inp.source_path);
        if let Some(src_path) = inp.source_path
            && dialect == StageDialect::StageR3
            && rank == Some(1)
        {
            bail!(
                "Stage 1 ('stage_1_impls/') only allows StageL 3 (.sl3/.sli3) modules, not StageR 3 ('{src_path}')"
            );
        }
        dialects.push(dialect);
        stage_ranks.push(rank);
        let (ns_opt, uses) =
            scan_module_header_paths_with_dialect(inp.source, dialect)?;
        let ns_key = ns_opt.as_ref().map(|segs| segs.join("/"));
        if let (Some(src_path), Some(key)) = (inp.source_path, &ns_key) {
            verify_psr4_path(src_path, key)?;
        }
        if let Some(ref key) = ns_key {
            let strata = ns_strata.entry(key.clone()).or_default();
            for &prev_idx in strata.iter() {
                let prev_rank = *stage_ranks
                    .get(prev_idx)
                    .context("Missing prior stage rank")?;
                let prev_dialect = *dialects
                    .get(prev_idx)
                    .context("Missing prior dialect")?;
                ensure!(
                    can_overlay_namespace(prev_rank, prev_dialect, rank, dialect),
                    "Duplicate namespace declaration '{key}'"
                );
            }
            strata.push(idx);
        }
        headers.push((ns_key, uses));
    }

    for strata in ns_strata.values_mut() {
        strata.sort_by_key(|&i| {
            let r = stage_ranks.get(i).copied().flatten().unwrap_or(u8::MAX); // Reason for fallback: unranked caller modules order after explicit stages.
            let d = match dialects.get(i).copied() {
                Some(StageDialect::StageL3) | None => 0u8,
                Some(StageDialect::StageR3) => 1u8,
            };
            (r, d, i)
        });
    }

    let order_stage = |idx: usize| -> u8 {
        // Reason for fallback: modules outside stage directories validate after explicit staged layers.
        stage_ranks.get(idx).copied().flatten().unwrap_or(u8::MAX)
    };

    // Build dependency edges (module -> prerequisite module indices) per stage
    // barrier, enforcing downward stage visibility and multi-stratum acyclic order.
    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(inputs.len());
    for (idx, (ns_key, uses)) in headers.iter().enumerate() {
        let caller_stage = order_stage(idx);
        // Reason for fallback: Anonymous modules format as '<anonymous>' in diagnostics.
        let caller_ns = ns_key.as_deref().unwrap_or("<anonymous>");
        let mut d = Vec::new();
        for u in uses {
            let target_key = u.join("/");
            let Some(cand_strata) = ns_strata.get(&target_key) else {
                bail!(
                    "Unresolved 'use {target_key}:' import in module {caller_ns}"
                );
            };
            let is_same_ns = ns_key.as_deref() == Some(target_key.as_str());
            let self_stratum_pos = if is_same_ns {
                cand_strata.iter().position(|&s| s == idx)
            } else {
                None
            };
            let mut visible_deps = Vec::new();
            for (pos, &dep_idx) in cand_strata.iter().enumerate() {
                if let Some(self_pos) = self_stratum_pos
                    && pos >= self_pos
                {
                    continue;
                }
                if order_stage(dep_idx) > caller_stage {
                    continue;
                }
                visible_deps.push(dep_idx);
            }
            if visible_deps.is_empty() {
                if is_same_ns {
                    bail!("Module '{target_key}' cannot import itself");
                }
                bail!(
                    "Module '{caller_ns}' cannot import higher-stage namespace '{target_key}'"
                );
            }
            for dep_idx in visible_deps {
                if dep_idx != idx && !d.contains(&dep_idx) {
                    d.push(dep_idx);
                }
            }
        }
        // Within the same stage (e.g. unranked StageL3 + StageR3 sharing a namespace),
        // also ensure consecutive strata of the same namespace validate in order.
        if let Some(key) = ns_key
            && let Some(strata) = ns_strata.get(key)
            && let Some(pos) = strata.iter().position(|&s| s == idx)
            && pos > 0
            && let Some(&prev_idx) = strata.get(pos.saturating_sub(1))
            && !d.contains(&prev_idx)
        {
            d.push(prev_idx);
        }
        deps.push(d);
    }

    let mut stage_buckets: Vec<u8> =
        (0..inputs.len()).map(order_stage).collect();
    stage_buckets.sort_unstable();
    stage_buckets.dedup();

    let mut ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut validated_modules = Vec::with_capacity(inputs.len());

    for stage_val in stage_buckets {
        let bucket_indices: Vec<usize> = (0..inputs.len())
            .filter(|&i| order_stage(i) == stage_val)
            .collect();
        let mut global_to_local: HashMap<usize, usize> =
            HashMap::with_capacity(bucket_indices.len());
        let mut local_headers = Vec::with_capacity(bucket_indices.len());
        for (local_i, &global_i) in bucket_indices.iter().enumerate() {
            global_to_local.insert(global_i, local_i);
            local_headers.push(
                headers
                    .get(global_i)
                    .cloned()
                    .context("Missing local header")?,
            );
        }
        let mut local_deps = Vec::with_capacity(bucket_indices.len());
        for &global_i in &bucket_indices {
            let g_deps = deps.get(global_i).context("Missing global deps")?;
            let l_deps: Vec<usize> = g_deps
                .iter()
                .filter_map(|g_dep| global_to_local.get(g_dep).copied())
                .collect();
            local_deps.push(l_deps);
        }
        let local_order = topo_sort_modules(&local_headers, &local_deps)?;
        for local_i in local_order {
            let idx = *bucket_indices
                .get(local_i)
                .context("Invalid bucket topological index")?;
            let inp = inputs.get(idx).context("Invalid topological index")?;
            let &dialect =
                dialects.get(idx).context("Invalid dialect index")?;
            let (parsed, _) = parse_with_resolved_use_arities_and_dialect(
                inp.source,
                dialect,
                &ns_exports,
            )?;
            let forbid_stagel_internal = dialect == StageDialect::StageR3
                || inp.source_path.is_some_and(is_stage2_or_sr3_path);
            let mut vm =
                validate_module_with_registry_and_dc_imports_and_options(
                    &parsed,
                    &ns_exports,
                    &HashMap::new(),
                    forbid_stagel_internal,
                )?;
            vm.input_index = idx;
            if let Some(ref key) = vm.namespace_key {
                if let Some(existing) = ns_exports.get_mut(key) {
                    existing.merge_overlay(vm.exports.clone(), key)?;
                } else {
                    ns_exports.insert(key.clone(), vm.exports.clone());
                }
            }
            validated_modules.push(vm);
        }
    }

    Ok(ValidatedWorkspace {
        modules: validated_modules,
    })
}

/// Resolves the active [`StageDialect`] for `source_path`, falling back to
/// `default_dialect` when `source_path` is omitted or lacks a known suffix.
#[must_use]
pub(crate) fn dialect_for_source_path(
    source_path: Option<&str>,
    default_dialect: StageDialect,
) -> StageDialect {
    match source_path {
        Some(p) => {
            let norm = p.replace('\\', "/");
            if norm.ends_with(".sr3") || norm.ends_with(".sri3") {
                StageDialect::StageR3
            } else if norm.ends_with(".sl3") || norm.ends_with(".sli3") {
                StageDialect::StageL3
            } else {
                default_dialect
            }
        }
        None => default_dialect,
    }
}

/// Returns `true` when `source_path` designates a Stage 2 (`stage_2_impls/`),
/// general (`impls/`), or StageR 3 (`.sr3`/`.sri3`) source file.
pub(crate) fn is_stage2_or_sr3_path(source_path: &str) -> bool {
    let norm = source_path.replace('\\', "/");
    norm.split('/')
        .any(|seg| seg == "stage_2_impls" || seg == "impls")
        || norm.ends_with(".sr3")
        || norm.ends_with(".sri3")
}

/// Returns `true` for identifiers ending with `StageLInternal`
/// (case-insensitive), which are prohibited outside Stage 1 modules.
pub(crate) fn is_stagel_internal_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with("stagelinternal")
}

/// Verifies that `source_path` matches the PSR-4 `ns_key` suffix.
pub(crate) fn verify_psr4_path(source_path: &str, ns_key: &str) -> Result<()> {
    let norm = source_path.replace('\\', "/");
    let matched = [".sl3", ".sli3", ".sr3", ".sri3"].iter().any(|ext| {
        let expected_suffix = format!("{ns_key}{ext}");
        norm == expected_suffix
            || norm.ends_with(&format!("/{expected_suffix}"))
    });
    ensure!(
        matched,
        "PSR-4 path mismatch: source file '{source_path}' does not match declared 'ns {ns_key}'"
    );
    Ok(())
}

fn topo_sort_modules(
    headers: &[(Option<String>, Vec<Vec<String>>)],
    deps: &[Vec<usize>],
) -> Result<Vec<usize>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Unvisited,
        Visiting,
        Visited,
    }

    fn dfs(
        node: usize,
        headers: &[(Option<String>, Vec<Vec<String>>)],
        deps: &[Vec<usize>],
        marks: &mut [Mark],
        stack: &mut Vec<usize>,
        order: &mut Vec<usize>,
    ) -> Result<()> {
        let mark = *marks.get(node).context("Invalid DFS node")?;
        match mark {
            Mark::Visited => return Ok(()),
            Mark::Visiting => {
                let cycle_names: Vec<String> = stack
                    .iter()
                    .chain(std::iter::once(&node))
                    .map(|&i| {
                        headers
                            .get(i)
                            .and_then(|(k, _)| k.clone())
                            // Reason for fallback: Anonymous modules render as '#module_{i}' in cycle traces.
                            .unwrap_or_else(|| format!("#module_{i}"))
                    })
                    .collect();
                bail!(
                    "Cyclic 'use' dependency detected: {}",
                    cycle_names.join(" -> ")
                );
            }
            Mark::Unvisited => {}
        }
        if let Some(m) = marks.get_mut(node) {
            *m = Mark::Visiting;
        }
        stack.push(node);
        let node_deps = deps.get(node).context("Missing node deps")?;
        for &dep in node_deps {
            dfs(dep, headers, deps, marks, stack, order)?;
        }
        let _ = stack.pop();
        if let Some(m) = marks.get_mut(node) {
            *m = Mark::Visited;
        }
        order.push(node);
        Ok(())
    }

    let mut marks = vec![Mark::Unvisited; deps.len()];
    let mut stack = Vec::new();
    let mut order = Vec::with_capacity(deps.len());
    for i in 0..deps.len() {
        if marks.get(i) == Some(&Mark::Unvisited) {
            dfs(i, headers, deps, &mut marks, &mut stack, &mut order)?;
        }
    }
    Ok(order)
}

fn parse_with_resolved_use_arities_and_dialect(
    source: &str,
    dialect: StageDialect,
    ns_exports: &HashMap<String, NamespaceExports>,
) -> Result<(Module, HashMap<String, usize>)> {
    let empty_dc_imports = HashMap::new();
    let import_ctx =
        build_stager_import_context(source, ns_exports, &empty_dc_imports)?;
    let module = match dialect {
        StageDialect::StageL3 => {
            parse_module_with_imports(source, &import_ctx.verb_arities)?
        }
        StageDialect::StageR3 => {
            parse_sr3_module_with_imports(source, &import_ctx)?
        }
    };
    Ok((module, import_ctx.verb_arities))
}

fn record_enum_variants_for_ctx(
    ctx: &mut StageRImportContext,
    key: StructTypeRef,
    variants: &[EnumVariantDef],
) {
    let map: HashMap<String, bool> = variants
        .iter()
        .map(|v| (v.name.clone(), v.payload.is_some()))
        .collect();
    ctx.enum_variants.insert(key, map);
}

fn record_imported_verb_arity(
    verb_arities: &mut HashMap<String, usize>,
    name: &str,
    arity: usize,
) -> Result<()> {
    if let Some(&prev_arity) = verb_arities.get(name) {
        ensure!(
            prev_arity == arity,
            "Conflicting parameter count for imported routine '{name}': expected {prev_arity}, found {arity}"
        );
    } else {
        verb_arities.insert(name.to_owned(), arity);
    }
    Ok(())
}

/// Scans `source` header `use` clauses plus `extra_dc_imports` to build the
/// parser [`StageRImportContext`] (imported verb arities, StageR type kinds,
/// and enum variant payload arities).
pub(crate) fn build_stager_import_context(
    source: &str,
    ns_exports: &HashMap<String, NamespaceExports>,
    extra_dc_imports: &HashMap<DcChar, ImportedSymbol>,
) -> Result<StageRImportContext> {
    let mut ctx = StageRImportContext::default();
    for (&dc, sym) in extra_dc_imports {
        let dc_word = format!("dc/{}", format_dc_shorthand(dc));
        match sym {
            ImportedSymbol::Routine { sig, .. } => {
                ctx.verb_arities.insert(dc_word, sig.params.len());
            }
            ImportedSymbol::NamedStruct { .. } => {
                ctx.type_kinds_by_dc
                    .insert(dc, StageRTypeKind::NamedStruct);
            }
            ImportedSymbol::BrandType { .. } => {
                ctx.type_kinds_by_dc.insert(dc, StageRTypeKind::Nominal);
            }
            ImportedSymbol::Enum { variants, .. } => {
                ctx.type_kinds_by_dc.insert(dc, StageRTypeKind::Nominal);
                record_enum_variants_for_ctx(
                    &mut ctx,
                    StructTypeRef::Dc(dc),
                    variants,
                );
            }
            ImportedSymbol::Value { .. } | ImportedSymbol::Struct { .. } => {}
        }
    }

    let mut rest = source;
    while !rest.is_empty() {
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }
        if trimmed.starts_with(':') && !trimmed.starts_with("::") {
            if let Some((_, after)) = trimmed.split_once('\n') {
                rest = after;
            } else {
                break;
            }
            continue;
        }
        if trimmed.starts_with("ns ") || trimmed.starts_with("ns\r") {
            let dot = trimmed
                .find(['.', ';'])
                .context("Unterminated 'ns' declaration")?;
            rest = trimmed
                .get(dot.saturating_add(1)..)
                .context("Invalid 'ns' termination slice")?;
            continue;
        }
        if trimmed.starts_with("use ") || trimmed.starts_with("use\r") {
            let after_use = trimmed
                .get(3..)
                .context("Truncated 'use' declaration")?
                .trim_start();
            let colon = after_use
                .find(':')
                .context("Expected ':' in 'use' import")?;
            let path = after_use
                .get(..colon)
                .context("Invalid 'use' namespace slice")?
                .trim();
            let exports = ns_exports.get(path).with_context(|| {
                format!("Unresolved namespace '{path}' in 'use' statement")
            })?;
            let items_start = after_use
                .get(colon.saturating_add(1)..)
                .context("Truncated 'use' item list")?;
            let end_idx = items_start
                .find("::")
                .or_else(|| items_start.find('}'))
                .context("Unclosed 'use' import list")?;
            let items_raw = items_start
                .get(..end_idx)
                .context("Invalid 'use' import item slice")?;
            let words: Vec<&str> = items_raw.split_whitespace().collect();
            let mut wi = 0usize;
            while let Some(&item_w) = words.get(wi) {
                wi = wi.saturating_add(1);
                if let Some(sh) = item_w.strip_prefix("dc/") {
                    let dc = parse_dc_shorthand(sh)?;
                    let sym = exports.by_dc.get(&dc).with_context(|| {
                        format!(
                            "Namespace '{path}' does not export 'dc/{sh}'"
                        )
                    })?;
                    let has_alias = words.get(wi) == Some(&"as");
                    let local_name = if has_alias {
                        wi = wi.saturating_add(1);
                        let a = *words
                            .get(wi)
                            .context("Expected alias after 'as'")?;
                        wi = wi.saturating_add(1);
                        a
                    } else {
                        item_w
                    };
                    match sym {
                        ImportedSymbol::Routine {
                            original_name,
                            sig,
                            ..
                        } => {
                            let new_arity = sig.params.len();
                            record_imported_verb_arity(
                                &mut ctx.verb_arities,
                                local_name,
                                new_arity,
                            )?;
                            if !has_alias {
                                record_imported_verb_arity(
                                    &mut ctx.verb_arities,
                                    original_name,
                                    new_arity,
                                )?;
                            }
                        }
                        ImportedSymbol::NamedStruct {
                            original_name, ..
                        } => {
                            ctx.type_kinds_by_name.insert(
                                local_name.to_owned(),
                                StageRTypeKind::NamedStruct,
                            );
                            if !has_alias {
                                ctx.type_kinds_by_name.insert(
                                    original_name.clone(),
                                    StageRTypeKind::NamedStruct,
                                );
                                ctx.type_kinds_by_dc
                                    .insert(dc, StageRTypeKind::NamedStruct);
                            }
                        }
                        ImportedSymbol::BrandType { original_name, .. } => {
                            ctx.type_kinds_by_name.insert(
                                local_name.to_owned(),
                                StageRTypeKind::Nominal,
                            );
                            if !has_alias {
                                ctx.type_kinds_by_name.insert(
                                    original_name.clone(),
                                    StageRTypeKind::Nominal,
                                );
                                ctx.type_kinds_by_dc
                                    .insert(dc, StageRTypeKind::Nominal);
                            }
                        }
                        ImportedSymbol::Enum {
                            original_name,
                            variants,
                            ..
                        } => {
                            ctx.type_kinds_by_name.insert(
                                local_name.to_owned(),
                                StageRTypeKind::Nominal,
                            );
                            record_enum_variants_for_ctx(
                                &mut ctx,
                                StructTypeRef::Ident(local_name.to_owned()),
                                variants,
                            );
                            if !has_alias {
                                ctx.type_kinds_by_name.insert(
                                    original_name.clone(),
                                    StageRTypeKind::Nominal,
                                );
                                ctx.type_kinds_by_dc
                                    .insert(dc, StageRTypeKind::Nominal);
                                record_enum_variants_for_ctx(
                                    &mut ctx,
                                    StructTypeRef::Ident(original_name.clone()),
                                    variants,
                                );
                                record_enum_variants_for_ctx(
                                    &mut ctx,
                                    StructTypeRef::Dc(dc),
                                    variants,
                                );
                            }
                        }
                        ImportedSymbol::Value { .. }
                        | ImportedSymbol::Struct { .. } => {}
                    }
                } else {
                    let local_name = if words.get(wi) == Some(&"as") {
                        wi = wi.saturating_add(1);
                        let a = *words
                            .get(wi)
                            .context("Expected alias after 'as'")?;
                        wi = wi.saturating_add(1);
                        a
                    } else {
                        item_w
                    };
                    if let Some(sigs) = exports.routines.get(item_w) {
                        ensure!(
                            !sigs.is_empty(),
                            "Empty routine export overload list"
                        );
                        for sig in sigs {
                            record_imported_verb_arity(
                                &mut ctx.verb_arities,
                                local_name,
                                sig.params.len(),
                            )?;
                        }
                    } else if exports.named_structs.contains_key(item_w) {
                        ctx.type_kinds_by_name.insert(
                            local_name.to_owned(),
                            StageRTypeKind::NamedStruct,
                        );
                    } else if exports.brand_types.contains_key(item_w) {
                        ctx.type_kinds_by_name.insert(
                            local_name.to_owned(),
                            StageRTypeKind::Nominal,
                        );
                    } else if let Some(edef) = exports.enums.get(item_w) {
                        ctx.type_kinds_by_name.insert(
                            local_name.to_owned(),
                            StageRTypeKind::Nominal,
                        );
                        record_enum_variants_for_ctx(
                            &mut ctx,
                            StructTypeRef::Ident(local_name.to_owned()),
                            &edef.variants,
                        );
                    } else if !exports.values.contains_key(item_w)
                        && !exports.structs.contains_key(item_w)
                    {
                        bail!(
                            "Namespace '{path}' has no exported item '{item_w}'"
                        );
                    }
                }
            }
            let after_close = items_start
                .get(end_idx..)
                .context("Invalid 'use' closing slice")?;
            let dot = after_close
                .find(['.', ';'])
                .context("Expected '.' ending 'use' import")?;
            rest = after_close
                .get(dot.saturating_add(1)..)
                .context("Invalid 'use' trailing slice")?;
            continue;
        }
        break;
    }
    Ok(ctx)
}

fn type_uses_stager(ty: &StageLType) -> bool {
    match ty {
        StageLType::B
        | StageLType::U8
        | StageLType::I32
        | StageLType::U32
        | StageLType::I64
        | StageLType::U64
        | StageLType::Void
        | StageLType::Never
        | StageLType::Struct(_) => false,
        StageLType::List(inner) | StageLType::Buf(inner) => {
            type_uses_stager(inner)
        }
        StageLType::Tuple(elems) => elems.iter().any(type_uses_stager),
        StageLType::I128
        | StageLType::U128
        | StageLType::Int
        | StageLType::Rat
        | StageLType::Dc
        | StageLType::Dcs
        | StageLType::S
        | StageLType::Error
        | StageLType::View
        | StageLType::Cursor
        | StageLType::Option(_)
        | StageLType::Result(_)
        | StageLType::Nominal(_) => true,
    }
}

fn expr_uses_stager(expr: &Expr) -> bool {
    match expr {
        Expr::Bool { .. }
        | Expr::Bytes { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. } => false,
        Expr::Int { value, .. } => matches!(
            value,
            TypedIntLit::I128(_) | TypedIntLit::U128(_) | TypedIntLit::Int(_)
        ),
        Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::NamedStructNew { .. }
        | Expr::EnumVariant { .. }
        | Expr::OptionNone { .. }
        | Expr::OptionSome { .. }
        | Expr::ResultOk { .. }
        | Expr::ResultErr { .. }
        | Expr::ErrorTag { .. }
        | Expr::NamedField { .. }
        | Expr::NamedWith { .. }
        | Expr::WrapNominal { .. }
        | Expr::UnwrapNominal { .. }
        | Expr::Match { .. }
        | Expr::Propagate { .. } => true,
        Expr::List {
            elem_ty, elements, ..
        } => {
            type_uses_stager(elem_ty) || elements.iter().any(expr_uses_stager)
        }
        Expr::TupleNew { elements, .. } => {
            elements.iter().any(expr_uses_stager)
        }
        Expr::StructNew { fields, .. } => fields.iter().any(expr_uses_stager),
        Expr::Field { base, .. } => expr_uses_stager(base),
        Expr::With { base, value, .. } => {
            expr_uses_stager(base) || expr_uses_stager(value)
        }
        Expr::BufNew {
            elem_ty, capacity, ..
        } => type_uses_stager(elem_ty) || expr_uses_stager(capacity),
        Expr::ValueRef { ty, .. } => ty.as_ref().is_some_and(type_uses_stager),
        Expr::Let { ty, init, .. } => {
            type_uses_stager(ty)
                || init.as_deref().is_some_and(expr_uses_stager)
        }
        Expr::Const { ty, value, .. } => {
            type_uses_stager(ty) || expr_uses_stager(value)
        }
        Expr::Set { ty, value, .. } => {
            ty.as_ref().is_some_and(type_uses_stager) || expr_uses_stager(value)
        }
        Expr::RoutineDecl(r) => routine_uses_stager(r),
        Expr::Call { args, .. } => args.iter().any(expr_uses_stager),
        Expr::Block { stmts, .. } => stmts.iter().any(|item| match item {
            super::super::ast::BlockItem::Expr(e) => expr_uses_stager(e),
            super::super::ast::BlockItem::Comment(_) => false,
        }),
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_uses_stager(cond)
                || expr_uses_stager(then_branch)
                || expr_uses_stager(else_branch)
        }
        Expr::While { cond, body, .. } => {
            expr_uses_stager(cond) || expr_uses_stager(body)
        }
        Expr::ForEach {
            elem_ty,
            list,
            body,
            ..
        } => {
            type_uses_stager(elem_ty)
                || expr_uses_stager(list)
                || expr_uses_stager(body)
        }
        Expr::Return { value, .. } => {
            value.as_deref().is_some_and(expr_uses_stager)
        }
        Expr::Unary { op, operand, .. } => {
            op.dialect_restriction() == Some(StageDialect::StageR3)
                || expr_uses_stager(operand)
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            op.dialect_restriction() == Some(StageDialect::StageR3)
                || expr_uses_stager(left)
                || expr_uses_stager(right)
        }
        Expr::Ternary {
            op,
            first,
            second,
            third,
            ..
        } => {
            op.dialect_restriction() == Some(StageDialect::StageR3)
                || expr_uses_stager(first)
                || expr_uses_stager(second)
                || expr_uses_stager(third)
        }
        Expr::Conv {
            total,
            expr,
            target_ty,
            ..
        } => *total || type_uses_stager(target_ty) || expr_uses_stager(expr),
        Expr::Bitcast {
            expr, target_ty, ..
        } => type_uses_stager(target_ty) || expr_uses_stager(expr),
    }
}

fn routine_uses_stager(r: &RoutineDecl) -> bool {
    type_uses_stager(&r.return_type)
        || r.params.iter().any(|p| type_uses_stager(&p.ty))
        || r.tests.iter().any(|tb| {
            tb.cases.iter().any(|tc| {
                tc.inputs.iter().any(expr_uses_stager)
                    || match &tc.expected {
                        TestExpected::Expr(e) => expr_uses_stager(e),
                        TestExpected::Fault(_) | TestExpected::Void(_) => false,
                    }
            })
        })
        || expr_uses_stager(&r.body)
}

/// Returns `true` if a standalone [`TopDecl`] uses any StageR 3-only
/// declaration, type, or expression construct.
#[must_use]
pub(crate) fn top_decl_uses_stager(decl: &TopDecl) -> bool {
    match decl {
        TopDecl::Struct(_) | TopDecl::Comment(_) => false,
        TopDecl::BrandType(_) | TopDecl::NamedStruct(_) | TopDecl::Enum(_) => {
            true
        }
        TopDecl::Value(v) => {
            type_uses_stager(&v.ty) || expr_uses_stager(&v.value)
        }
        TopDecl::Routine(r) => routine_uses_stager(r),
    }
}
