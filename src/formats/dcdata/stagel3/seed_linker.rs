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

//! Snapshot-pinned StageL 3 linker resolving external PSR-4 `use` namespaces
//! and transitive `dc/<shorthand>` Token-Dc helpers directly from a
//! [`SeedGraphIndex`].
//!
//! Produces a topologically sorted [`ValidatedWorkspace`] ready for execution in
//! [`super::eval`], Rust transpilation, or compilation into a single linked
//! `.WASM` (`Dc 1927`..`Dc 1928`) module with zero runtime host/mailbox calls.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::dc::DC_BEGIN_MODULE;
use crate::seed_graph::SeedGraphIndex;
use crate::{DcChar, DcShorthand};

use super::ast::{
    BlockItem, Expr, ImportTarget, Module, PositionalStructDecl, RoutineDecl,
    Span, StageLType, StructTypeRef, SymbolTarget, TestExpected, TopDecl,
    TopValueDecl,
};
use super::dc_tokens::{
    decode_module_from_dc_tokens, decode_top_decl_from_dc_tokens_at_line,
    encode_module_to_dc_tokens, strip_comment_frames,
};
use super::eval::{
    TestSuiteReport, run_validated_workspace_inline_tests,
};
use super::parser::{
    parse_module_with_imports, scan_direct_dc_words, scan_module_header_paths,
};
use super::validator::{
    ImportedSymbol, ModuleInput, NamespaceExports, RoutineSig, StructDef,
    ValidatedModule, ValidatedWorkspace, collect_use_verb_arities_from_source,
    is_stage2_or_sr3_path,
    validate_module_with_registry_and_dc_imports_and_options, verify_psr4_path,
};

#[derive(Debug, Clone)]
enum GraphOwnerKind {
    Value { ty: StageLType },
    Routine { sig: RoutineSig },
    Struct { fields: Vec<StageLType> },
}

#[derive(Debug, Clone)]
struct GraphDcOwner {
    namespace_key: String,
    original_name: String,
    kind: GraphOwnerKind,
}

impl GraphDcOwner {
    fn to_imported_symbol(&self, dc: DcChar) -> ImportedSymbol {
        match &self.kind {
            GraphOwnerKind::Value { ty } => ImportedSymbol::Value {
                namespace: self.namespace_key.clone(),
                original_name: self.original_name.clone(),
                ty: ty.clone(),
                dc_id: Some(dc),
            },
            GraphOwnerKind::Routine { sig } => ImportedSymbol::Routine {
                namespace: self.namespace_key.clone(),
                original_name: self.original_name.clone(),
                sig: sig.clone(),
            },
            GraphOwnerKind::Struct { fields } => ImportedSymbol::Struct {
                namespace: self.namespace_key.clone(),
                original_name: self.original_name.clone(),
                fields: fields.clone(),
                dc_id: Some(dc),
            },
        }
    }
}

fn collect_dc_refs_in_owner_kind(kind: &GraphOwnerKind, out: &mut Vec<DcChar>) {
    match kind {
        GraphOwnerKind::Value { ty } => collect_dc_refs_in_type(ty, out),
        GraphOwnerKind::Routine { sig } => {
            collect_dc_refs_in_type(&sig.return_type, out);
            for p in &sig.params {
                collect_dc_refs_in_type(p, out);
            }
        }
        GraphOwnerKind::Struct { fields } => {
            for f in fields {
                collect_dc_refs_in_type(f, out);
            }
        }
    }
}

fn sig_for_routine_decl(r: &RoutineDecl) -> RoutineSig {
    RoutineSig {
        return_type: r.return_type.clone(),
        params: r.params.iter().map(|p| p.ty.clone()).collect(),
        is_pub: true,
        dc_id: r.dc_id,
    }
}

fn prepare_graph_top_decl(
    mut decl: TopDecl,
    override_name: Option<&str>,
    target_dc: DcChar,
) -> Result<(TopDecl, String, GraphOwnerKind)> {
    match &mut decl {
        TopDecl::Routine(r) => {
            if let Some(name) = override_name {
                r.name = name.to_owned();
            }
            r.is_pub = true;
            r.pub_dc_syntax = false;
            r.dc_id = Some(target_dc);
            r.tests.clear();
            let orig_name = r.name.clone();
            let sig = sig_for_routine_decl(r);
            Ok((decl, orig_name, GraphOwnerKind::Routine { sig }))
        }
        TopDecl::Value(v) => {
            if let Some(name) = override_name {
                v.name = name.to_owned();
            }
            v.is_pub = true;
            v.pub_dc_syntax = false;
            v.dc_id = Some(target_dc);
            let orig_name = v.name.clone();
            let ty = v.ty.clone();
            Ok((decl, orig_name, GraphOwnerKind::Value { ty }))
        }
        TopDecl::Struct(s) => {
            if let Some(name) = override_name {
                s.name = name.to_owned();
            }
            s.is_pub = true;
            s.pub_dc_syntax = false;
            s.dc_id = Some(target_dc);
            let orig_name = s.name.clone();
            let fields = s.fields.clone();
            Ok((decl, orig_name, GraphOwnerKind::Struct { fields }))
        }
        TopDecl::Comment(_) => {
            bail!("Decoded StageL graph item for {target_dc} cannot be a bare comment")
        }
    }
}

/// Decodes a StageL Token-Dc decomposition from `seed_graph` into either a
/// single `TopDecl` or a `Module`.
fn decode_graph_stagel_entry(
    seed_graph: &SeedGraphIndex,
    target_sh: DcShorthand,
) -> Result<Option<(Vec<DcChar>, EitherDeclOrModule)>> {
    decode_graph_stagel_entry_at_line(seed_graph, target_sh, 1)
}

fn decode_graph_stagel_entry_at_line(
    seed_graph: &SeedGraphIndex,
    target_sh: DcShorthand,
    decl_line: usize,
) -> Result<Option<(Vec<DcChar>, EitherDeclOrModule)>> {
    let Some((_, chars)) =
        seed_graph.resolve_stagel_decomposition_tokens(target_sh)?
    else {
        return Ok(None);
    };
    let stripped = strip_comment_frames(&chars);
    if stripped.first().copied() == Some(DC_BEGIN_MODULE) {
        let module = decode_module_from_dc_tokens(&chars).with_context(|| {
            format!("Failed to decode StageL Token-Dc module on {target_sh}")
        })?;
        Ok(Some((chars, EitherDeclOrModule::Module(module))))
    } else {
        let decl = decode_top_decl_from_dc_tokens_at_line(&chars, decl_line)
            .with_context(|| {
                format!(
                    "Failed to decode StageL Token-Dc declaration on {target_sh}"
                )
            })?;
        Ok(Some((chars, EitherDeclOrModule::Decl(decl))))
    }
}

enum EitherDeclOrModule {
    Decl(TopDecl),
    Module(Module),
}

fn extract_decl_from_module(
    module: Module,
    target_dc: DcChar,
    expected_name: Option<&str>,
    target_sh: DcShorthand,
) -> Result<TopDecl> {
    let mut name_matches: Vec<TopDecl> = Vec::new();
    let mut non_comment_items: Vec<TopDecl> = Vec::new();
    for item in module.items {
        match &item {
            TopDecl::Routine(r) => {
                if r.dc_id == Some(target_dc) {
                    return Ok(item);
                }
                if expected_name.is_some_and(|n| r.name == n) {
                    name_matches.push(item);
                } else {
                    non_comment_items.push(item);
                }
            }
            TopDecl::Value(v) => {
                if v.dc_id == Some(target_dc) {
                    return Ok(item);
                }
                if expected_name.is_some_and(|n| v.name == n) {
                    name_matches.push(item);
                } else {
                    non_comment_items.push(item);
                }
            }
            TopDecl::Struct(s) => {
                if s.dc_id == Some(target_dc) {
                    return Ok(item);
                }
                if expected_name.is_some_and(|n| s.name == n) {
                    name_matches.push(item);
                } else {
                    non_comment_items.push(item);
                }
            }
            TopDecl::Comment(_) => {}
        }
    }
    if name_matches.len() == 1
        && let Some(matched) = name_matches.pop()
    {
        return Ok(matched);
    }
    if name_matches.is_empty()
        && non_comment_items.len() == 1
        && let Some(single) = non_comment_items.pop()
    {
        return Ok(single);
    }
    bail!(
        "StageL Token-Dc module on {target_sh} does not contain an unambiguous matching item"
    )
}

fn collect_dc_refs_in_type(ty: &StageLType, out: &mut Vec<DcChar>) {
    match ty {
        StageLType::List(inner) | StageLType::Buf(inner) => {
            collect_dc_refs_in_type(inner, out);
        }
        StageLType::Tuple(elems) => {
            for el in elems {
                collect_dc_refs_in_type(el, out);
            }
        }
        StageLType::Struct(StructTypeRef::Dc(dc)) => {
            out.push(*dc);
        }
        _ => {}
    }
}

fn collect_dc_refs_in_module(module: &Module, out: &mut Vec<DcChar>) {
    for u in &module.uses {
        for item in &u.items {
            if let ImportTarget::Dc(dc) = item.target {
                out.push(dc);
            }
        }
    }
    for decl in &module.items {
        match decl {
            TopDecl::Comment(_) => {}
            TopDecl::Struct(PositionalStructDecl { fields, .. }) => {
                for f in fields {
                    collect_dc_refs_in_type(f, out);
                }
            }
            TopDecl::Value(TopValueDecl { ty, value, .. }) => {
                collect_dc_refs_in_type(ty, out);
                collect_dc_refs_in_expr(value, out);
            }
            TopDecl::Routine(r) => {
                collect_dc_refs_in_type(&r.return_type, out);
                for p in &r.params {
                    collect_dc_refs_in_type(&p.ty, out);
                }
                for tb in &r.tests {
                    for tc in &tb.cases {
                        for arg in &tc.inputs {
                            collect_dc_refs_in_expr(arg, out);
                        }
                        if let TestExpected::Expr(exp) = &tc.expected {
                            collect_dc_refs_in_expr(exp, out);
                        }
                    }
                }
                collect_dc_refs_in_expr(&r.body, out);
            }
        }
    }
}

fn collect_dc_refs_in_expr(expr: &Expr, out: &mut Vec<DcChar>) {
    match expr {
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. } => {}
        Expr::ValueRef { ty, target, .. } => {
            if let Some(t) = ty {
                collect_dc_refs_in_type(t, out);
            }
            if let SymbolTarget::Dc(dc) = target {
                out.push(*dc);
            }
        }
        Expr::List {
            elem_ty, elements, ..
        } => {
            collect_dc_refs_in_type(elem_ty, out);
            for elem in elements {
                collect_dc_refs_in_expr(elem, out);
            }
        }
        Expr::BufNew {
            elem_ty, capacity, ..
        } => {
            collect_dc_refs_in_type(elem_ty, out);
            collect_dc_refs_in_expr(capacity, out);
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    collect_dc_refs_in_expr(e, out);
                }
            }
        }
        Expr::Let { ty, init, .. } => {
            collect_dc_refs_in_type(ty, out);
            if let Some(init_expr) = init {
                collect_dc_refs_in_expr(init_expr, out);
            }
        }
        Expr::Const { ty, value, .. } => {
            collect_dc_refs_in_type(ty, out);
            collect_dc_refs_in_expr(value, out);
        }
        Expr::Set { ty, value, .. } => {
            if let Some(t) = ty {
                collect_dc_refs_in_type(t, out);
            }
            collect_dc_refs_in_expr(value, out);
        }
        Expr::RoutineDecl(r) => {
            collect_dc_refs_in_type(&r.return_type, out);
            for p in &r.params {
                collect_dc_refs_in_type(&p.ty, out);
            }
            collect_dc_refs_in_expr(&r.body, out);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_dc_refs_in_expr(cond, out);
            collect_dc_refs_in_expr(then_branch, out);
            collect_dc_refs_in_expr(else_branch, out);
        }
        Expr::While { cond, body, .. } => {
            collect_dc_refs_in_expr(cond, out);
            collect_dc_refs_in_expr(body, out);
        }
        Expr::ForEach {
            elem_ty,
            list,
            body,
            ..
        } => {
            collect_dc_refs_in_type(elem_ty, out);
            collect_dc_refs_in_expr(list, out);
            collect_dc_refs_in_expr(body, out);
        }
        Expr::Return { value, .. } => {
            if let Some(v) = value {
                collect_dc_refs_in_expr(v, out);
            }
        }
        Expr::Unary { operand, .. } => {
            collect_dc_refs_in_expr(operand, out);
        }
        Expr::Conv {
            expr: inner,
            target_ty,
            ..
        }
        | Expr::Bitcast {
            expr: inner,
            target_ty,
            ..
        } => {
            collect_dc_refs_in_type(target_ty, out);
            collect_dc_refs_in_expr(inner, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_dc_refs_in_expr(left, out);
            collect_dc_refs_in_expr(right, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_dc_refs_in_expr(first, out);
            collect_dc_refs_in_expr(second, out);
            collect_dc_refs_in_expr(third, out);
        }
        Expr::Call { target, args, .. } => {
            if let SymbolTarget::Dc(dc) = target {
                out.push(*dc);
            }
            for arg in args {
                collect_dc_refs_in_expr(arg, out);
            }
        }
        Expr::TupleNew { elements, .. } => {
            for el in elements {
                collect_dc_refs_in_expr(el, out);
            }
        }
        Expr::StructNew { target, fields, .. } => {
            if let SymbolTarget::Dc(dc) = target {
                out.push(*dc);
            }
            for f in fields {
                collect_dc_refs_in_expr(f, out);
            }
        }
        Expr::Field { base, .. } => {
            collect_dc_refs_in_expr(base, out);
        }
        Expr::With { base, value, .. } => {
            collect_dc_refs_in_expr(base, out);
            collect_dc_refs_in_expr(value, out);
        }
    }
}

fn collect_locally_defined_dcs(module: &Module, out: &mut HashSet<DcChar>) {
    for item in &module.items {
        match item {
            TopDecl::Routine(r) => {
                if let Some(dc) = r.dc_id {
                    out.insert(dc);
                }
            }
            TopDecl::Value(v) => {
                if let Some(dc) = v.dc_id {
                    out.insert(dc);
                }
            }
            TopDecl::Struct(s) => {
                if let Some(dc) = s.dc_id {
                    out.insert(dc);
                }
            }
            TopDecl::Comment(_) => {}
        }
    }
}

fn reconstruct_external_namespace_module(
    seed_graph: &SeedGraphIndex,
    ns_segs: &[String],
    caller_ns_for_diag: &str,
    dc_owners: &mut HashMap<DcChar, GraphDcOwner>,
) -> Result<(Module, NamespaceExports)> {
    let ns_key = ns_segs.join("/");
    let exports_map = seed_graph.resolve_stagel_namespace_exports(ns_segs)?;
    if exports_map.is_empty() {
        bail!(
            "Unresolved 'use {ns_key}:' import in module {caller_ns_for_diag}"
        );
    }

    let mut items = Vec::with_capacity(exports_map.len());
    let mut prelim_exports = NamespaceExports::default();
    for (export_name, target_sh_list) in exports_map {
        for target_sh in target_sh_list {
            let decl_line = items.len().saturating_add(1);
            let gid = target_sh.to_global_id()?;
            let target_dc = DcChar::from_u128(gid);
            let (_, entry) = decode_graph_stagel_entry_at_line(
                seed_graph, target_sh, decl_line,
            )?
            .with_context(|| {
                format!(
                    "Exported symbol '{export_name}' ({target_sh}) in namespace '{ns_key}' has no active StageL Token-Dc decomposition"
                )
            })?;
            let raw_decl = match entry {
                EitherDeclOrModule::Decl(d) => d,
                EitherDeclOrModule::Module(m) => extract_decl_from_module(
                    m,
                    target_dc,
                    Some(&export_name),
                    target_sh,
                )?,
            };
            let (prepared_decl, orig_name, kind) =
                prepare_graph_top_decl(raw_decl, Some(&export_name), target_dc)?;
            let owner = GraphDcOwner {
                namespace_key: ns_key.clone(),
                original_name: orig_name.clone(),
                kind: kind.clone(),
            };
            match &kind {
                GraphOwnerKind::Value { ty } => {
                    ensure!(
                        !prelim_exports.values.contains_key(&orig_name),
                        "Duplicate exported constant '{orig_name}' in namespace '{ns_key}'"
                    );
                    prelim_exports
                        .values
                        .insert(orig_name.clone(), (ty.clone(), Some(target_dc)));
                }
                GraphOwnerKind::Routine { sig } => {
                    prelim_exports
                        .routines
                        .entry(orig_name.clone())
                        .or_default()
                        .push(sig.clone());
                }
                GraphOwnerKind::Struct { fields } => {
                    ensure!(
                        !prelim_exports.structs.contains_key(&orig_name),
                        "Duplicate exported struct '{orig_name}' in namespace '{ns_key}'"
                    );
                    prelim_exports.structs.insert(
                        orig_name.clone(),
                        StructDef {
                            namespace: Some(ns_key.clone()),
                            original_name: orig_name.clone(),
                            fields: fields.clone(),
                            is_pub: true,
                            dc_id: Some(target_dc),
                            span: Span { line: 1, col: 1 },
                        },
                    );
                }
            }
            prelim_exports
                .by_dc
                .insert(target_dc, owner.to_imported_symbol(target_dc));
            dc_owners.insert(target_dc, owner);
            items.push(prepared_decl);
        }
    }

    let module = Module {
        leading_comments: Vec::new(),
        namespace: Some(ns_segs.to_vec()),
        uses: Vec::new(),
        items,
    };
    Ok((module, prelim_exports))
}

fn topo_sort_all_modules(
    labels: &[String],
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
        labels: &[String],
        deps: &[Vec<usize>],
        marks: &mut [Mark],
        stack: &mut Vec<usize>,
        order: &mut Vec<usize>,
    ) -> Result<()> {
        let mark = *marks.get(node).context("Invalid DFS node")?;
        match mark {
            Mark::Visited => return Ok(()),
            Mark::Visiting => {
                let mut cycle_names = Vec::new();
                for &idx in stack.iter() {
                    if let Some(lbl) = labels.get(idx) {
                        cycle_names.push(lbl.clone());
                    }
                }
                if let Some(lbl) = labels.get(node) {
                    cycle_names.push(lbl.clone());
                }
                bail!(
                    "Cyclic StageL 3 dependency cycle detected: {}",
                    cycle_names.join(" -> ")
                );
            }
            Mark::Unvisited => {}
        }
        if let Some(slot) = marks.get_mut(node) {
            *slot = Mark::Visiting;
        }
        stack.push(node);
        let node_deps = deps.get(node).context("Missing DFS dependency list")?;
        for &dep in node_deps {
            dfs(dep, labels, deps, marks, stack, order)?;
        }
        stack.pop();
        if let Some(slot) = marks.get_mut(node) {
            *slot = Mark::Visited;
        }
        order.push(node);
        Ok(())
    }

    let mut marks = vec![Mark::Unvisited; labels.len()];
    let mut stack = Vec::new();
    let mut order = Vec::with_capacity(labels.len());
    for i in 0..labels.len() {
        if marks.get(i) == Some(&Mark::Unvisited) {
            dfs(i, labels, deps, &mut marks, &mut stack, &mut order)?;
        }
    }
    Ok(order)
}

/// Links pre-parsed StageL `Module`s against external namespaces and transitive
/// `dc/<shorthand>` Token-Dc helpers in `seed_graph`, returning a topologically
/// ordered [`ValidatedWorkspace`].
pub fn link_parsed_modules_with_seed_graph(
    caller_modules: Vec<Module>,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let tagged = caller_modules.into_iter().map(|m| (m, false)).collect();
    link_parsed_modules_with_seed_graph_and_flags(tagged, seed_graph)
}

struct AssembledGraphModules {
    all_modules: Vec<Module>,
    all_forbid_flags: Vec<bool>,
    module_extra_dc_imports: Vec<HashMap<DcChar, ImportedSymbol>>,
    order: Vec<usize>,
}

fn assemble_topo_sorted_modules_with_seed_graph(
    caller_modules: Vec<(Module, bool)>,
    seed_graph: &SeedGraphIndex,
) -> Result<AssembledGraphModules> {
    let mut dc_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    let mut caller_ns_set: HashSet<String> = HashSet::new();
    let mut local_dcs: HashSet<DcChar> = HashSet::new();

    for (m, _) in &caller_modules {
        if let Some(segs) = &m.namespace {
            let key = segs.join("/");
            ensure!(
                caller_ns_set.insert(key.clone()),
                "Duplicate namespace declaration '{key}'"
            );
        }
        collect_locally_defined_dcs(m, &mut local_dcs);
    }

    // 1. Reconstruct any external `use <ns>:` namespaces from `seed_graph`.
    let mut synth_ns_modules: BTreeMap<String, Module> = BTreeMap::new();
    for (m, _) in &caller_modules {
        // Reason for fallback: Anonymous caller modules format as '<anonymous>' in diagnostics.
        let caller_label = m
            .namespace
            .as_ref()
            .map_or_else(|| String::from("<anonymous>"), |s| s.join("/"));
        for u in &m.uses {
            let target_key = u.path.join("/");
            if caller_ns_set.contains(&target_key)
                || synth_ns_modules.contains_key(&target_key)
            {
                continue;
            }
            let (ext_mod, _) = reconstruct_external_namespace_module(
                seed_graph,
                &u.path,
                &caller_label,
                &mut dc_owners,
            )?;
            collect_locally_defined_dcs(&ext_mod, &mut local_dcs);
            synth_ns_modules.insert(target_key, ext_mod);
        }
    }

    // 2. Transitively resolve all `SymbolTarget::Dc` / `ImportTarget::Dc` helpers from `seed_graph`.
    let mut dc_worklist: VecDeque<DcChar> = VecDeque::new();
    let mut initial_refs = Vec::new();
    for m in synth_ns_modules
        .values()
        .chain(caller_modules.iter().map(|(m, _)| m))
    {
        collect_dc_refs_in_module(m, &mut initial_refs);
    }
    for dc in initial_refs {
        dc_worklist.push_back(dc);
    }

    let mut synth_helper_modules: BTreeMap<String, Module> = BTreeMap::new();
    let mut checked_missing_dcs: HashSet<DcChar> = HashSet::new();

    while let Some(dc) = dc_worklist.pop_front() {
        if local_dcs.contains(&dc)
            || dc_owners.contains_key(&dc)
            || checked_missing_dcs.contains(&dc)
        {
            continue;
        }
        let sh = DcShorthand::from_global_id(dc.to_u128());
        let Some((_, entry)) = decode_graph_stagel_entry(seed_graph, sh)?
        else {
            checked_missing_dcs.insert(dc);
            continue;
        };

        let synth_ns_seg = format!("$graph_dc_{sh}");
        let (synth_mod, key) = match entry {
            EitherDeclOrModule::Decl(raw_decl) => {
                let (prepared_decl, orig_name, kind) =
                    prepare_graph_top_decl(raw_decl, None, dc)?;
                dc_owners.insert(
                    dc,
                    GraphDcOwner {
                        namespace_key: synth_ns_seg.clone(),
                        original_name: orig_name,
                        kind,
                    },
                );
                (
                    Module {
                        leading_comments: Vec::new(),
                        namespace: Some(vec![synth_ns_seg.clone()]),
                        uses: Vec::new(),
                        items: vec![prepared_decl],
                    },
                    synth_ns_seg,
                )
            }
            EitherDeclOrModule::Module(mut m) => {
                let ns_key = if let Some(existing_ns) = &m.namespace {
                    let joined = existing_ns.join("/");
                    if caller_ns_set.contains(&joined)
                        || synth_ns_modules.contains_key(&joined)
                        || synth_helper_modules.contains_key(&joined)
                    {
                        m.namespace = Some(vec![synth_ns_seg.clone()]);
                        synth_ns_seg.clone()
                    } else {
                        joined
                    }
                } else {
                    m.namespace = Some(vec![synth_ns_seg.clone()]);
                    synth_ns_seg.clone()
                };
                let mut matched = false;
                for item in &mut m.items {
                    match item {
                        TopDecl::Routine(r) => {
                            r.tests.clear();
                            if r.dc_id == Some(dc) {
                                r.is_pub = true;
                                r.pub_dc_syntax = false;
                                let sig = sig_for_routine_decl(r);
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: r.name.clone(),
                                        kind: GraphOwnerKind::Routine { sig },
                                    },
                                );
                                matched = true;
                            }
                        }
                        TopDecl::Value(v) => {
                            if v.dc_id == Some(dc) {
                                v.is_pub = true;
                                v.pub_dc_syntax = false;
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: v.name.clone(),
                                        kind: GraphOwnerKind::Value {
                                            ty: v.ty.clone(),
                                        },
                                    },
                                );
                                matched = true;
                            }
                        }
                        TopDecl::Struct(s) => {
                            if s.dc_id == Some(dc) {
                                s.is_pub = true;
                                s.pub_dc_syntax = false;
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: s.name.clone(),
                                        kind: GraphOwnerKind::Struct {
                                            fields: s.fields.clone(),
                                        },
                                    },
                                );
                                matched = true;
                            }
                        }
                        TopDecl::Comment(_) => {}
                    }
                }
                if !matched {
                    // Single-item module fallback when `dc_id` was carried on the enclosing entity.
                    for item in &mut m.items {
                        match item {
                            TopDecl::Routine(r) => {
                                r.is_pub = true;
                                r.pub_dc_syntax = false;
                                r.dc_id = Some(dc);
                                let sig = sig_for_routine_decl(r);
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: r.name.clone(),
                                        kind: GraphOwnerKind::Routine { sig },
                                    },
                                );
                                matched = true;
                                break;
                            }
                            TopDecl::Value(v) => {
                                v.is_pub = true;
                                v.pub_dc_syntax = false;
                                v.dc_id = Some(dc);
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: v.name.clone(),
                                        kind: GraphOwnerKind::Value {
                                            ty: v.ty.clone(),
                                        },
                                    },
                                );
                                matched = true;
                                break;
                            }
                            TopDecl::Struct(s) => {
                                s.is_pub = true;
                                s.pub_dc_syntax = false;
                                s.dc_id = Some(dc);
                                dc_owners.insert(
                                    dc,
                                    GraphDcOwner {
                                        namespace_key: ns_key.clone(),
                                        original_name: s.name.clone(),
                                        kind: GraphOwnerKind::Struct {
                                            fields: s.fields.clone(),
                                        },
                                    },
                                );
                                matched = true;
                                break;
                            }
                            TopDecl::Comment(_) => {}
                        }
                    }
                }
                ensure!(
                    matched,
                    "StageL Token-Dc module on {sh} contains no routine, struct, or constant declaration"
                );
                // If this graph module itself has external `use` declarations, resolve them too.
                for u in &m.uses {
                    let u_key = u.path.join("/");
                    if !caller_ns_set.contains(&u_key)
                        && !synth_ns_modules.contains_key(&u_key)
                    {
                        let (ext_mod, _) = reconstruct_external_namespace_module(
                            seed_graph,
                            &u.path,
                            &ns_key,
                            &mut dc_owners,
                        )?;
                        let mut ext_refs = Vec::new();
                        collect_dc_refs_in_module(&ext_mod, &mut ext_refs);
                        for r_dc in ext_refs {
                            dc_worklist.push_back(r_dc);
                        }
                        collect_locally_defined_dcs(&ext_mod, &mut local_dcs);
                        synth_ns_modules.insert(u_key, ext_mod);
                    }
                }
                (m, ns_key)
            }
        };

        collect_locally_defined_dcs(&synth_mod, &mut local_dcs);
        let mut nested_refs = Vec::new();
        collect_dc_refs_in_module(&synth_mod, &mut nested_refs);
        for nested_dc in nested_refs {
            dc_worklist.push_back(nested_dc);
        }
        synth_helper_modules.insert(key, synth_mod);
    }

    // 3. Assemble all modules (synthetic helpers + synthetic namespaces + caller modules),
    // compute exact dependency edges, topologically sort, and validate!
    let total_cap = synth_helper_modules
        .len()
        .saturating_add(synth_ns_modules.len())
        .saturating_add(caller_modules.len());
    let mut all_modules: Vec<Module> = Vec::with_capacity(total_cap);
    let mut all_forbid_flags: Vec<bool> = Vec::with_capacity(total_cap);
    for m in synth_helper_modules.into_values() {
        all_modules.push(m);
        all_forbid_flags.push(false);
    }
    for m in synth_ns_modules.into_values() {
        all_modules.push(m);
        all_forbid_flags.push(false);
    }
    for (m, forbid_flag) in caller_modules {
        all_modules.push(m);
        all_forbid_flags.push(forbid_flag);
    }

    let mut ns_to_idx: HashMap<String, usize> = HashMap::new();
    let mut labels: Vec<String> = Vec::with_capacity(all_modules.len());
    for (idx, m) in all_modules.iter().enumerate() {
        if let Some(segs) = &m.namespace {
            let key = segs.join("/");
            ns_to_idx.insert(key.clone(), idx);
            labels.push(key);
        } else {
            labels.push(format!("<anonymous#{idx}>"));
        }
    }

    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(all_modules.len());
    let mut module_extra_dc_imports: Vec<HashMap<DcChar, ImportedSymbol>> =
        Vec::with_capacity(all_modules.len());

    for (idx, m) in all_modules.iter().enumerate() {
        let self_ns = m.namespace.as_ref().map(|s| s.join("/"));
        let mut mod_local_dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut mod_local_dcs);

        let mut d = Vec::new();
        for u in &m.uses {
            let target_key = u.path.join("/");
            let Some(&dep_idx) = ns_to_idx.get(&target_key) else {
                let caller_lbl = labels
                    .get(idx)
                    .context("Missing caller module label")?;
                bail!("Unresolved 'use {target_key}:' import in module {caller_lbl}");
            };
            ensure!(
                dep_idx != idx,
                "Module '{target_key}' cannot import itself"
            );
            if !d.contains(&dep_idx) {
                d.push(dep_idx);
            }
        }

        let mut refs = Vec::new();
        collect_dc_refs_in_module(m, &mut refs);
        let mut ref_queue: VecDeque<DcChar> = refs.into_iter().collect();
        let mut visited_extra = HashSet::new();
        let mut extra_dc_map = HashMap::new();
        while let Some(dc) = ref_queue.pop_front() {
            if mod_local_dcs.contains(&dc) || !visited_extra.insert(dc) {
                continue;
            }
            if let Some(owner) = dc_owners.get(&dc) {
                let mut sig_refs = Vec::new();
                collect_dc_refs_in_owner_kind(&owner.kind, &mut sig_refs);
                for s_dc in sig_refs {
                    ref_queue.push_back(s_dc);
                }
                if self_ns.as_deref() == Some(owner.namespace_key.as_str()) {
                    continue;
                }
                if let Some(&dep_idx) = ns_to_idx.get(&owner.namespace_key)
                    && dep_idx != idx
                    && !d.contains(&dep_idx)
                {
                    d.push(dep_idx);
                }
                extra_dc_map.insert(dc, owner.to_imported_symbol(dc));
            }
        }
        deps.push(d);
        module_extra_dc_imports.push(extra_dc_map);
    }

    let order = topo_sort_all_modules(&labels, &deps)?;
    Ok(AssembledGraphModules {
        all_modules,
        all_forbid_flags,
        module_extra_dc_imports,
        order,
    })
}

fn link_parsed_modules_with_seed_graph_and_flags(
    caller_modules: Vec<(Module, bool)>,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let assembled =
        assemble_topo_sorted_modules_with_seed_graph(caller_modules, seed_graph)?;
    let mut ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut validated_modules =
        Vec::with_capacity(assembled.all_modules.len());

    for idx in assembled.order {
        let m = assembled
            .all_modules
            .get(idx)
            .context("Invalid topological module index")?;
        let extra_dc = assembled
            .module_extra_dc_imports
            .get(idx)
            .context("Invalid module extra dc_imports index")?;
        let forbid_stagel_internal = assembled
            .all_forbid_flags
            .get(idx)
            .copied()
            .context("Invalid module forbid_stagel_internal index")?;
        let vm = validate_module_with_registry_and_dc_imports_and_options(
            m,
            &ns_exports,
            extra_dc,
            forbid_stagel_internal,
        )?;
        if let Some(ref key) = vm.namespace_key {
            ns_exports.insert(key.clone(), vm.exports.clone());
        }
        validated_modules.push(vm);
    }

    Ok(ValidatedWorkspace {
        modules: validated_modules,
    })
}

/// Parses and validates a multi-module `.sl3` workspace, resolving any external
/// `use` namespaces and transitive `dc/<shorthand>` Token-Dc helpers from
/// `seed_graph`.
pub fn parse_and_validate_workspace_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let mut headers = Vec::with_capacity(inputs.len());
    let mut caller_ns_to_idx: HashMap<String, usize> = HashMap::new();

    for (idx, inp) in inputs.iter().enumerate() {
        let (ns_opt, uses) = scan_module_header_paths(inp.source)?;
        let ns_key = ns_opt.as_ref().map(|segs| segs.join("/"));
        if let (Some(src_path), Some(key)) = (inp.source_path, &ns_key) {
            verify_psr4_path(src_path, key)?;
        }
        if let Some(ref key) = ns_key {
            ensure!(
                !caller_ns_to_idx.contains_key(key),
                "Duplicate namespace declaration '{key}'"
            );
            caller_ns_to_idx.insert(key.clone(), idx);
        }
        headers.push((ns_key, uses));
    }

    // Pre-populate `ns_exports` for external `seed_graph` namespaces so caller
    // `use <ns>: verb ::.` headers have exact verb arities before parsing!
    let mut pre_ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut dummy_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    for (ns_key, uses) in &headers {
        // Reason for fallback: Anonymous caller modules format as '<anonymous>' in diagnostics.
        let caller_label = ns_key.as_deref().unwrap_or("<anonymous>");
        for u in uses {
            let target_key = u.join("/");
            if caller_ns_to_idx.contains_key(&target_key)
                || pre_ns_exports.contains_key(&target_key)
            {
                continue;
            }
            let (_, ext_exports) = reconstruct_external_namespace_module(
                seed_graph,
                u,
                caller_label,
                &mut dummy_owners,
            )?;
            pre_ns_exports.insert(target_key, ext_exports);
        }
    }

    // Topologically order the caller modules among themselves so intra-caller
    // `use` verb arities are known when parsing dependent caller modules.
    let mut caller_deps: Vec<Vec<usize>> = Vec::with_capacity(inputs.len());
    let mut caller_labels: Vec<String> = Vec::with_capacity(inputs.len());
    for (idx, (ns_key, uses)) in headers.iter().enumerate() {
        // Reason for fallback: anonymous caller modules without a `namespace` declaration are labeled by index for diagnostics.
        let label = ns_key
            .clone()
            .unwrap_or_else(|| format!("<anonymous#{idx}>"));
        caller_labels.push(label);
        let mut d = Vec::new();
        for u in uses {
            let target_key = u.join("/");
            if let Some(&dep_idx) = caller_ns_to_idx.get(&target_key) {
                ensure!(
                    dep_idx != idx,
                    "Module '{target_key}' cannot import itself"
                );
                if !d.contains(&dep_idx) {
                    d.push(dep_idx);
                }
            }
        }
        caller_deps.push(d);
    }
    let caller_order = topo_sort_all_modules(&caller_labels, &caller_deps)?;

    let mut parsed_caller_modules: Vec<Option<Module>> =
        vec![None; inputs.len()];
    for idx in caller_order {
        let inp = inputs.get(idx).context("Invalid caller module index")?;
        let mut arities =
            collect_use_verb_arities_from_source(inp.source, &pre_ns_exports)?;
        for (dc_word, dc) in scan_direct_dc_words(inp.source)? {
            if arities.contains_key(&dc_word) {
                continue;
            }
            let sh = DcShorthand::from_global_id(dc.to_u128());
            if let Some((_, entry)) = decode_graph_stagel_entry(seed_graph, sh)?
            {
                let decl_opt = match entry {
                    EitherDeclOrModule::Decl(d) => Some(d),
                    EitherDeclOrModule::Module(m) => {
                        extract_decl_from_module(m, dc, None, sh).ok()
                    }
                };
                if let Some(TopDecl::Routine(r)) = decl_opt {
                    arities.insert(dc_word, r.params.len());
                }
            }
        }

        let parsed = parse_module_with_imports(inp.source, &arities)?;
        // Record caller exports in `pre_ns_exports` so downstream caller modules can parse `use` calls.
        if let Some(segs) = &parsed.namespace {
            let mut exp = NamespaceExports::default();
            let ns_key = segs.join("/");
            for item in &parsed.items {
                match item {
                    TopDecl::Routine(r) if r.is_pub => {
                        let sig = RoutineSig {
                            return_type: r.return_type.clone(),
                            params: r
                                .params
                                .iter()
                                .map(|p| p.ty.clone())
                                .collect(),
                            is_pub: true,
                            dc_id: r.dc_id,
                        };
                        exp.routines
                            .entry(r.name.clone())
                            .or_default()
                            .push(sig.clone());
                        if let Some(dc) = r.dc_id {
                            exp.by_dc.insert(
                                dc,
                                ImportedSymbol::Routine {
                                    namespace: ns_key.clone(),
                                    original_name: r.name.clone(),
                                    sig,
                                },
                            );
                        }
                    }
                    TopDecl::Value(v) if v.is_pub => {
                        exp.values
                            .insert(v.name.clone(), (v.ty.clone(), v.dc_id));
                        if let Some(dc) = v.dc_id {
                            exp.by_dc.insert(
                                dc,
                                ImportedSymbol::Value {
                                    namespace: ns_key.clone(),
                                    original_name: v.name.clone(),
                                    ty: v.ty.clone(),
                                    dc_id: Some(dc),
                                },
                            );
                        }
                    }
                    TopDecl::Struct(s) if s.is_pub => {
                        exp.structs.insert(
                            s.name.clone(),
                            StructDef {
                                namespace: Some(ns_key.clone()),
                                original_name: s.name.clone(),
                                fields: s.fields.clone(),
                                is_pub: true,
                                dc_id: s.dc_id,
                                span: s.span,
                            },
                        );
                        if let Some(dc) = s.dc_id {
                            exp.by_dc.insert(
                                dc,
                                ImportedSymbol::Struct {
                                    namespace: ns_key.clone(),
                                    original_name: s.name.clone(),
                                    fields: s.fields.clone(),
                                    dc_id: Some(dc),
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
            pre_ns_exports.insert(ns_key, exp);
        }
        if let Some(slot) = parsed_caller_modules.get_mut(idx) {
            *slot = Some(parsed);
        }
    }

    let mut ordered_callers = Vec::with_capacity(inputs.len());
    for (idx, opt) in parsed_caller_modules.into_iter().enumerate() {
        let parsed = opt.context("Missing parsed caller module")?;
        let forbid = inputs
            .get(idx)
            .and_then(|inp| inp.source_path)
            .is_some_and(is_stage2_or_sr3_path);
        ordered_callers.push((parsed, forbid));
    }

    link_parsed_modules_with_seed_graph_and_flags(ordered_callers, seed_graph)
}

/// Parses a single `.sl3` source file using imported routine arities resolved
/// from `seed_graph` (both `use` namespaces and direct `dc/<shorthand>` calls).
pub fn parse_module_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Module> {
    let (ns_opt, uses) = scan_module_header_paths(source)?;
    let ns_key = ns_opt.as_ref().map(|segs| segs.join("/"));
    // Reason for fallback: Anonymous caller modules format as '<anonymous>' in diagnostics.
    let caller_label = ns_key.as_deref().unwrap_or("<anonymous>");
    let mut pre_ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut dummy_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    for u in &uses {
        let target_key = u.join("/");
        if pre_ns_exports.contains_key(&target_key) {
            continue;
        }
        let (_, ext_exports) = reconstruct_external_namespace_module(
            seed_graph,
            u,
            caller_label,
            &mut dummy_owners,
        )?;
        pre_ns_exports.insert(target_key, ext_exports);
    }
    let mut arities =
        collect_use_verb_arities_from_source(source, &pre_ns_exports)?;
    for (dc_word, dc) in scan_direct_dc_words(source)? {
        if arities.contains_key(&dc_word) {
            continue;
        }
        let sh = DcShorthand::from_global_id(dc.to_u128());
        if let Some((_, entry)) = decode_graph_stagel_entry(seed_graph, sh)? {
            let decl_opt = match entry {
                EitherDeclOrModule::Decl(d) => Some(d),
                EitherDeclOrModule::Module(m) => {
                    extract_decl_from_module(m, dc, None, sh).ok()
                }
            };
            if let Some(TopDecl::Routine(r)) = decl_opt {
                arities.insert(dc_word, r.params.len());
            }
        }
    }
    parse_module_with_imports(source, &arities)
}

/// Converts StageL 3 `.sl3` source (resolving external `use` and `dc/<shorthand>`
/// routine arities against `seed_graph`) into canonical StageL Token Dcs.
pub fn sl3_to_dc_tokens_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let module = parse_module_with_seed_graph(source, seed_graph)?;
    encode_module_to_dc_tokens(&module)
}

/// Parses and validates a single `.sl3` source file against `seed_graph`.
pub fn parse_and_validate_module_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    parse_and_validate_workspace_with_seed_graph(
        &[ModuleInput {
            source_path: None,
            source,
        }],
        seed_graph,
    )
}

/// Validates a pre-parsed StageL `Module` against `seed_graph`.
pub fn validate_parsed_module_with_seed_graph(
    module: &Module,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    link_parsed_modules_with_seed_graph(vec![module.clone()], seed_graph)
}

/// Decodes a StageL 3 Token-Dc stream (`Dc 2192`..`Dc 2193`) and links/validates
/// it against `seed_graph`.
pub fn validate_dc_tokens_with_seed_graph(
    tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let module = decode_module_from_dc_tokens(tokens)?;
    validate_parsed_module_with_seed_graph(&module, seed_graph)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks on a
/// single `.sl3` source file using the deterministic StageL reference evaluator.
pub fn eval_sl3_inline_tests_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_module_with_seed_graph(source, seed_graph)?;
    run_validated_workspace_inline_tests(&ws)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks across
/// a multi-module `.sl3` workspace using the StageL reference evaluator.
pub fn eval_workspace_inline_tests_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_workspace_with_seed_graph(inputs, seed_graph)?;
    run_validated_workspace_inline_tests(&ws)
}

const STAGE0_WASM_OPERATOR_DCS: &[DcChar] = &[
    crate::dc::DC_ADD_OP,
    crate::dc::DC_SUB_OP,
    crate::dc::DC_MUL_OP,
    crate::dc::DC_DIV_OP,
    crate::dc::DC_REM_OP,
    crate::dc::DC_NEG_OP,
    crate::dc::DC_WRAP_ADD_OP,
    crate::dc::DC_WRAP_SUB_OP,
    crate::dc::DC_WRAP_MUL_OP,
    crate::dc::DC_BIT_AND_OP,
    crate::dc::DC_BIT_OR_OP,
    crate::dc::DC_BIT_XOR_OP,
    crate::dc::DC_BIT_NOT_OP,
    crate::dc::DC_SHL_OP,
    crate::dc::DC_SHR_OP,
    crate::dc::DC_ROTL_OP,
    crate::dc::DC_ROTR_OP,
    crate::dc::DC_CLZ_OP,
    crate::dc::DC_CTZ_OP,
    crate::dc::DC_POPCNT_OP,
    crate::dc::DC_LOGICAL_NOT_OP,
    crate::dc::DC_EQ_OP,
    crate::dc::DC_NE_OP,
    crate::dc::DC_LT_OP,
    crate::dc::DC_LE_OP,
    crate::dc::DC_GT_OP,
    crate::dc::DC_GE_OP,
    crate::dc::DC_CONVERT_OP,
    crate::dc::DC_BITCAST_OP,
    crate::dc::DC_SEQ_LEN_OP,
    crate::dc::DC_SEQ_GET_OP,
    crate::dc::DC_SEQ_SLICE_OP,
    crate::dc::DC_SEQ_CONCAT_OP,
    crate::dc::DC_STAGEL_BUF_NEW,
    crate::dc::DC_STAGEL_BUF_FILL,
    crate::dc::DC_STAGEL_BUF_PUSH,
    crate::dc::DC_STAGEL_BUF_EXTEND,
    crate::dc::DC_STAGEL_BUF_SET,
    crate::dc::DC_STAGEL_BUF_FREEZE,
];

/// Bundles an entry StageL 3 Token-Dc module (`Dc 2192`..`Dc 2193`) with its
/// transitive `use` / `dc/<shorthand>` StageL Token-Dc dependency modules and
/// active contextual Stage 0 `.WASM` built-ins from `seed_graph` so the pure
/// linear-memory `stagel3_to_wasm.wat` converter routine can compile the
/// document deterministically at `seed_graph`'s pinned snapshot without host
/// callbacks or Rust AST validation.
pub fn bundle_dc_module_for_wasm_converter(
    entry_tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
    resolver: &dyn crate::syntax::SyntaxRuleResolver,
) -> Result<Vec<DcChar>> {
    use crate::dc::{
        DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LIST, DC_END_KV_MAP,
        DC_END_KV_PAIR, DC_END_LIST, DC_EQ_OP, DC_LIST_ELEMENT_SEPARATOR,
        DC_NE_OP, DC_STAGEL_BUF_NEW, DC_TYPE_ANY,
    };

    let mut module_stream: Vec<DcChar> = Vec::new();
    if let Ok(entry_mod) = decode_module_from_dc_tokens(entry_tokens) {
        let assembled = assemble_topo_sorted_modules_with_seed_graph(
            vec![(entry_mod, false)],
            seed_graph,
        )?;
        let entry_idx = assembled.all_modules.len().saturating_sub(1);
        for idx in assembled.order {
            if idx == entry_idx {
                continue;
            }
            let dep_mod = assembled
                .all_modules
                .get(idx)
                .context("Invalid dependency module index")?;
            module_stream.extend(encode_module_to_dc_tokens(dep_mod)?);
        }
        module_stream.extend_from_slice(entry_tokens);
    } else {
        // Malformed Token-Dc input is forwarded directly so `stagel3_to_wasm.wat`
        // traps under the active task quota rather than failing in host Rust.
        module_stream.extend_from_slice(entry_tokens);
    }

    let mut present_ops: HashSet<DcChar> = HashSet::new();
    present_ops.insert(DC_STAGEL_BUF_NEW);
    for &tok in &module_stream {
        for &op_dc in STAGE0_WASM_OPERATOR_DCS {
            if tok == op_dc
                || resolver.is_equivalent_to(tok.to_u128(), op_dc.to_u128())
            {
                present_ops.insert(op_dc);
                if op_dc == DC_EQ_OP || op_dc == DC_NE_OP {
                    present_ops.insert(DC_EQ_OP);
                    present_ops.insert(DC_NE_OP);
                }
            }
        }
    }

    let mut out = Vec::new();
    out.push(DC_BEGIN_KV_MAP);
    out.push(DC_TYPE_ANY);
    out.push(DC_TYPE_ANY);

    for &op_dc in STAGE0_WASM_OPERATOR_DCS {
        if !present_ops.contains(&op_dc) {
            continue;
        }
        let op_sh = DcShorthand::from_global_id(op_dc.to_u128());
        for (ctx_tokens, wasm_tokens) in
            seed_graph.list_active_contextual_wasm_definitions(op_sh)
        {
            out.push(DC_BEGIN_KV_PAIR);
            out.push(DC_BEGIN_LIST);
            out.push(DC_TYPE_ANY);
            out.push(op_dc);
            for ctx_sh in ctx_tokens {
                out.push(DC_LIST_ELEMENT_SEPARATOR);
                out.push(DcChar::from_u128(ctx_sh.to_global_id()?));
            }
            out.push(DC_END_LIST);
            out.push(DC_LIST_ELEMENT_SEPARATOR);
            for w_sh in wasm_tokens {
                out.push(DcChar::from_u128(w_sh.to_global_id()?));
            }
            out.push(DC_END_KV_PAIR);
        }
    }

    out.push(DC_END_KV_MAP);
    out.extend(module_stream);
    Ok(out)
}
