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

use crate::seed_graph::SeedGraphIndex;
use crate::{DcChar, DcShorthand};

use super::ast::{
    BlockItem, BrandTypeDecl, EnumDecl, EnumVariantDecl, Expr, ImportTarget,
    Module, NamedFieldDecl, NamedStructDecl, PositionalStructDecl,
    RoutineDecl, Span, StageDialect, StageLType, StructTypeRef, SymbolTarget,
    TestExpected, TopDecl, TopValueDecl,
};
use super::dc_tokens::{
    decode_module_from_dc_tokens, decode_top_decl_from_dc_tokens_at_line,
    encode_module_to_dc_tokens, strip_comment_frames,
};
use super::eval::{
    TestSuiteReport, run_validated_workspace_inline_tests,
};
use super::parser::{
    parse_module_with_imports, parse_sr3_module_with_imports,
    scan_direct_dc_words_with_dialect, scan_module_header_paths_with_dialect,
};
use super::validator::{
    BrandTypeDef, EnumDef, EnumVariantDef, ImportedSymbol, ModuleInput,
    NamedStructDef, NamespaceExports, RoutineSig, StructDef, ValidatedModule,
    ValidatedWorkspace, build_stager_import_context, dialect_for_source_path,
    is_stage2_or_sr3_path, top_decl_uses_stager,
    validate_module_with_registry_and_dc_imports_and_options, verify_psr4_path,
};

#[derive(Debug, Clone)]
enum GraphOwnerKind {
    Value { ty: StageLType },
    Routine { sig: RoutineSig },
    Struct { fields: Vec<StageLType> },
    BrandType { is_opaque: bool, inner_ty: StageLType },
    NamedStruct { fields: Vec<(String, StageLType)> },
    Enum { variants: Vec<EnumVariantDef> },
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
            GraphOwnerKind::BrandType {
                is_opaque,
                inner_ty,
            } => ImportedSymbol::BrandType {
                namespace: self.namespace_key.clone(),
                original_name: self.original_name.clone(),
                is_opaque: *is_opaque,
                inner_ty: inner_ty.clone(),
                dc_id: Some(dc),
            },
            GraphOwnerKind::NamedStruct { fields } => {
                ImportedSymbol::NamedStruct {
                    namespace: self.namespace_key.clone(),
                    original_name: self.original_name.clone(),
                    fields: fields.clone(),
                    dc_id: Some(dc),
                }
            }
            GraphOwnerKind::Enum { variants } => ImportedSymbol::Enum {
                namespace: self.namespace_key.clone(),
                original_name: self.original_name.clone(),
                variants: variants.clone(),
                dc_id: Some(dc),
            },
        }
    }
}

fn collect_dc_refs_in_owner_kind(kind: &GraphOwnerKind, out: &mut Vec<DcChar>) {
    match kind {
        GraphOwnerKind::Value { ty }
        | GraphOwnerKind::BrandType { inner_ty: ty, .. } => {
            collect_dc_refs_in_type(ty, out);
        }
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
        GraphOwnerKind::NamedStruct { fields } => {
            for (_, fty) in fields {
                collect_dc_refs_in_type(fty, out);
            }
        }
        GraphOwnerKind::Enum { variants } => {
            for v in variants {
                if let Some(p) = &v.payload {
                    collect_dc_refs_in_type(p, out);
                }
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
        TopDecl::BrandType(b) => {
            if let Some(name) = override_name {
                b.name = name.to_owned();
            }
            b.is_pub = true;
            b.pub_dc_syntax = false;
            b.dc_id = Some(target_dc);
            let orig_name = b.name.clone();
            let kind = GraphOwnerKind::BrandType {
                is_opaque: b.is_opaque,
                inner_ty: b.inner_ty.clone(),
            };
            Ok((decl, orig_name, kind))
        }
        TopDecl::NamedStruct(s) => {
            if let Some(name) = override_name {
                s.name = name.to_owned();
            }
            s.is_pub = true;
            s.pub_dc_syntax = false;
            s.dc_id = Some(target_dc);
            let orig_name = s.name.clone();
            let fields = s
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect();
            Ok((decl, orig_name, GraphOwnerKind::NamedStruct { fields }))
        }
        TopDecl::Enum(e) => {
            if let Some(name) = override_name {
                e.name = name.to_owned();
            }
            e.is_pub = true;
            e.pub_dc_syntax = false;
            e.dc_id = Some(target_dc);
            let orig_name = e.name.clone();
            let variants = e
                .variants
                .iter()
                .map(|v| EnumVariantDef {
                    name: v.name.clone(),
                    payload: v.payload.clone(),
                    span: v.span,
                })
                .collect();
            Ok((decl, orig_name, GraphOwnerKind::Enum { variants }))
        }
        TopDecl::Comment(_) => {
            bail!("Decoded StageL/StageR graph comment for {target_dc} cannot be a named declaration")
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
    let is_module = stripped.first().copied().is_some_and(|first| {
        let u = first.to_u128();
        seed_graph.token_fills_canonical_or_equiv_role(
            u,
            crate::syntax::DcRole::BeginModule,
        ) || seed_graph.token_fills_canonical_or_equiv_role(
            u,
            crate::syntax::DcRole::BeginStagerModule,
        )
    });
    if is_module {
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
        let (dc_opt, item_name) = match &item {
            TopDecl::Routine(r) => (r.dc_id, r.name.as_str()),
            TopDecl::Value(v) => (v.dc_id, v.name.as_str()),
            TopDecl::Struct(s) => (s.dc_id, s.name.as_str()),
            TopDecl::BrandType(b) => (b.dc_id, b.name.as_str()),
            TopDecl::NamedStruct(s) => (s.dc_id, s.name.as_str()),
            TopDecl::Enum(e) => (e.dc_id, e.name.as_str()),
            TopDecl::Comment(_) => continue,
        };
        if dc_opt == Some(target_dc) {
            return Ok(item);
        }
        if expected_name.is_some_and(|n| item_name == n) {
            name_matches.push(item);
        } else {
            non_comment_items.push(item);
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
        StageLType::List(inner)
        | StageLType::Buf(inner)
        | StageLType::Option(inner)
        | StageLType::Result(inner) => {
            collect_dc_refs_in_type(inner, out);
        }
        StageLType::Tuple(elems) => {
            for el in elems {
                collect_dc_refs_in_type(el, out);
            }
        }
        StageLType::Struct(StructTypeRef::Dc(dc))
        | StageLType::Nominal(StructTypeRef::Dc(dc)) => {
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
            TopDecl::BrandType(b) => {
                collect_dc_refs_in_type(&b.inner_ty, out);
            }
            TopDecl::NamedStruct(NamedStructDecl { fields, .. }) => {
                for f in fields {
                    collect_dc_refs_in_type(&f.ty, out);
                }
            }
            TopDecl::Enum(EnumDecl { variants, .. }) => {
                for v in variants {
                    if let Some(p) = &v.payload {
                        collect_dc_refs_in_type(p, out);
                    }
                }
            }
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
        Expr::DctsString { .. } | Expr::DcLit { .. } | Expr::ErrorTag { .. } => {
        }
        Expr::NamedStructNew { target, fields, .. } => {
            if let SymbolTarget::Dc(dc) = target {
                out.push(*dc);
            }
            for (_, value, _) in fields {
                collect_dc_refs_in_expr(value, out);
            }
        }
        Expr::EnumVariant {
            enum_ty, payload, ..
        } => {
            if let StructTypeRef::Dc(dc) = enum_ty {
                out.push(*dc);
            }
            if let Some(p) = payload {
                collect_dc_refs_in_expr(p, out);
            }
        }
        Expr::OptionNone { inner_ty, .. } => {
            collect_dc_refs_in_type(inner_ty, out);
        }
        Expr::OptionSome {
            inner_ty, value, ..
        }
        | Expr::ResultOk {
            ok_ty: inner_ty,
            value,
            ..
        }
        | Expr::ResultErr {
            ok_ty: inner_ty,
            error: value,
            ..
        } => {
            collect_dc_refs_in_type(inner_ty, out);
            collect_dc_refs_in_expr(value, out);
        }
        Expr::NamedField { base, .. }
        | Expr::UnwrapNominal { expr: base, .. }
        | Expr::Propagate { expr: base, .. } => {
            collect_dc_refs_in_expr(base, out);
        }
        Expr::NamedWith { base, value, .. } => {
            collect_dc_refs_in_expr(base, out);
            collect_dc_refs_in_expr(value, out);
        }
        Expr::WrapNominal {
            expr: inner,
            target_ty,
            ..
        } => {
            collect_dc_refs_in_type(target_ty, out);
            collect_dc_refs_in_expr(inner, out);
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            collect_dc_refs_in_expr(scrutinee, out);
            for arm in arms {
                if let Some((ty, _, _)) = &arm.binding {
                    collect_dc_refs_in_type(ty, out);
                }
                collect_dc_refs_in_expr(&arm.body, out);
            }
            if let Some(fb) = fallback {
                collect_dc_refs_in_expr(fb, out);
            }
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
            TopDecl::BrandType(b) => {
                if let Some(dc) = b.dc_id {
                    out.insert(dc);
                }
            }
            TopDecl::NamedStruct(s) => {
                if let Some(dc) = s.dc_id {
                    out.insert(dc);
                }
            }
            TopDecl::Enum(e) => {
                if let Some(dc) = e.dc_id {
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
                GraphOwnerKind::BrandType {
                    is_opaque,
                    inner_ty,
                } => {
                    ensure!(
                        !prelim_exports.brand_types.contains_key(&orig_name),
                        "Duplicate exported type '{orig_name}' in namespace '{ns_key}'"
                    );
                    prelim_exports.brand_types.insert(
                        orig_name.clone(),
                        BrandTypeDef {
                            namespace: Some(ns_key.clone()),
                            original_name: orig_name.clone(),
                            is_opaque: *is_opaque,
                            inner_ty: inner_ty.clone(),
                            is_pub: true,
                            dc_id: Some(target_dc),
                            span: Span { line: 1, col: 1 },
                        },
                    );
                }
                GraphOwnerKind::NamedStruct { fields } => {
                    ensure!(
                        !prelim_exports.named_structs.contains_key(&orig_name),
                        "Duplicate exported named struct '{orig_name}' in namespace '{ns_key}'"
                    );
                    prelim_exports.named_structs.insert(
                        orig_name.clone(),
                        NamedStructDef {
                            namespace: Some(ns_key.clone()),
                            original_name: orig_name.clone(),
                            fields: fields.clone(),
                            is_pub: true,
                            dc_id: Some(target_dc),
                            span: Span { line: 1, col: 1 },
                        },
                    );
                }
                GraphOwnerKind::Enum { variants } => {
                    ensure!(
                        !prelim_exports.enums.contains_key(&orig_name),
                        "Duplicate exported enum '{orig_name}' in namespace '{ns_key}'"
                    );
                    prelim_exports.enums.insert(
                        orig_name.clone(),
                        EnumDef {
                            namespace: Some(ns_key.clone()),
                            original_name: orig_name.clone(),
                            variants: variants.clone(),
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

    let dialect = if items.iter().any(top_decl_uses_stager) {
        StageDialect::StageR3
    } else {
        StageDialect::StageL3
    };
    let module = Module {
        dialect,
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

/// Links pre-parsed StageL/StageR `Module`s against external namespaces and
/// transitive `dc/<shorthand>` Token-Dc helpers in `seed_graph`, returning a
/// topologically ordered [`ValidatedWorkspace`].
pub fn link_parsed_modules_with_seed_graph(
    caller_modules: Vec<Module>,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let tagged = caller_modules
        .into_iter()
        .map(|m| {
            let forbid = m.dialect == StageDialect::StageR3;
            (m, forbid)
        })
        .collect();
    link_parsed_modules_with_seed_graph_and_flags(tagged, seed_graph)
}

fn claim_graph_item_owner(
    item: &mut TopDecl,
    target_dc: DcChar,
    ns_key: &str,
    force_claim: bool,
) -> Option<GraphDcOwner> {
    let matches_dc = |dc_id: Option<DcChar>| force_claim || dc_id == Some(target_dc);
    let (original_name, kind) = match item {
        TopDecl::Routine(r) if matches_dc(r.dc_id) => {
            r.is_pub = true;
            r.pub_dc_syntax = false;
            r.dc_id = Some(target_dc);
            let sig = sig_for_routine_decl(r);
            (r.name.clone(), GraphOwnerKind::Routine { sig })
        }
        TopDecl::Value(v) if matches_dc(v.dc_id) => {
            v.is_pub = true;
            v.pub_dc_syntax = false;
            v.dc_id = Some(target_dc);
            (v.name.clone(), GraphOwnerKind::Value { ty: v.ty.clone() })
        }
        TopDecl::Struct(s) if matches_dc(s.dc_id) => {
            s.is_pub = true;
            s.pub_dc_syntax = false;
            s.dc_id = Some(target_dc);
            (
                s.name.clone(),
                GraphOwnerKind::Struct {
                    fields: s.fields.clone(),
                },
            )
        }
        TopDecl::BrandType(b) if matches_dc(b.dc_id) => {
            b.is_pub = true;
            b.pub_dc_syntax = false;
            b.dc_id = Some(target_dc);
            (
                b.name.clone(),
                GraphOwnerKind::BrandType {
                    is_opaque: b.is_opaque,
                    inner_ty: b.inner_ty.clone(),
                },
            )
        }
        TopDecl::NamedStruct(s) if matches_dc(s.dc_id) => {
            s.is_pub = true;
            s.pub_dc_syntax = false;
            s.dc_id = Some(target_dc);
            let fields = s
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect();
            (s.name.clone(), GraphOwnerKind::NamedStruct { fields })
        }
        TopDecl::Enum(e) if matches_dc(e.dc_id) => {
            e.is_pub = true;
            e.pub_dc_syntax = false;
            e.dc_id = Some(target_dc);
            let variants = e
                .variants
                .iter()
                .map(|v| EnumVariantDef {
                    name: v.name.clone(),
                    payload: v.payload.clone(),
                    span: v.span,
                })
                .collect();
            (e.name.clone(), GraphOwnerKind::Enum { variants })
        }
        TopDecl::Routine(_)
        | TopDecl::Value(_)
        | TopDecl::Struct(_)
        | TopDecl::BrandType(_)
        | TopDecl::NamedStruct(_)
        | TopDecl::Enum(_)
        | TopDecl::Comment(_) => return None,
    };
    Some(GraphDcOwner {
        namespace_key: ns_key.to_owned(),
        original_name,
        kind,
    })
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
        if let Some(ns_segs) = seed_graph.resolve_stagel_entity_namespace(sh)? {
            let ns_key = ns_segs.join("/");
            if !caller_ns_set.contains(&ns_key)
                && !synth_ns_modules.contains_key(&ns_key)
            {
                let (ext_mod, _) = reconstruct_external_namespace_module(
                    seed_graph,
                    &ns_segs,
                    "<direct dc import>",
                    &mut dc_owners,
                )?;
                let mut ext_refs = Vec::new();
                collect_dc_refs_in_module(&ext_mod, &mut ext_refs);
                for r_dc in ext_refs {
                    dc_worklist.push_back(r_dc);
                }
                collect_locally_defined_dcs(&ext_mod, &mut local_dcs);
                synth_ns_modules.insert(ns_key, ext_mod);
            }
            if local_dcs.contains(&dc) || dc_owners.contains_key(&dc) {
                continue;
            }
        }
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
                let dialect = if top_decl_uses_stager(&prepared_decl) {
                    StageDialect::StageR3
                } else {
                    StageDialect::StageL3
                };
                (
                    Module {
                        dialect,
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
                    if let TopDecl::Routine(r) = item {
                        r.tests.clear();
                    }
                    if let Some(owner) =
                        claim_graph_item_owner(item, dc, &ns_key, false)
                    {
                        dc_owners.insert(dc, owner);
                        matched = true;
                    }
                }
                if !matched {
                    // Single-item module fallback when `dc_id` was carried on the enclosing entity.
                    for item in &mut m.items {
                        if let Some(owner) =
                            claim_graph_item_owner(item, dc, &ns_key, true)
                        {
                            dc_owners.insert(dc, owner);
                            matched = true;
                            break;
                        }
                    }
                }
                ensure!(
                    matched,
                    "StageL/StageR Token-Dc module on {sh} contains no declaration"
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

fn parse_source_with_seed_graph_context(
    source: &str,
    dialect: StageDialect,
    pre_ns_exports: &HashMap<String, NamespaceExports>,
    seed_graph: &SeedGraphIndex,
) -> Result<Module> {
    let mut dc_import_map: HashMap<DcChar, ImportedSymbol> = HashMap::new();
    for (_dc_word, dc) in scan_direct_dc_words_with_dialect(source, dialect)? {
        if dc_import_map.contains_key(&dc) {
            continue;
        }
        if let Some(sym) =
            pre_ns_exports.values().find_map(|exp| exp.by_dc.get(&dc))
        {
            dc_import_map.insert(dc, sym.clone());
            continue;
        }
        let sh = DcShorthand::from_global_id(dc.to_u128());
        if let Some((_, entry)) = decode_graph_stagel_entry(seed_graph, sh)? {
            let decl_res = match entry {
                EitherDeclOrModule::Decl(d) => Ok(d),
                EitherDeclOrModule::Module(m) => {
                    extract_decl_from_module(m, dc, None, sh)
                }
            };
            if let Ok(raw_decl) = decl_res
                && let Ok((_, orig_name, kind)) =
                    prepare_graph_top_decl(raw_decl, None, dc)
            {
                // Reason for fallback: Standalone non-namespaced graph decompositions have no FilePath context and use a synthetic per-Dc module key.
                let ns_key = seed_graph
                    .resolve_stagel_entity_namespace(sh)?
                    .map_or_else(
                        || format!("$graph_dc_{sh}"),
                        |segs| segs.join("/"),
                    );
                let owner = GraphDcOwner {
                    namespace_key: ns_key,
                    original_name: orig_name,
                    kind,
                };
                dc_import_map.insert(dc, owner.to_imported_symbol(dc));
            }
        }
    }
    let import_ctx =
        build_stager_import_context(source, pre_ns_exports, &dc_import_map)?;
    match dialect {
        StageDialect::StageL3 => {
            parse_module_with_imports(source, &import_ctx.verb_arities)
        }
        StageDialect::StageR3 => {
            parse_sr3_module_with_imports(source, &import_ctx)
        }
    }
}

fn record_caller_prelim_exports(
    parsed: &Module,
    pre_ns_exports: &mut HashMap<String, NamespaceExports>,
) {
    let Some(segs) = &parsed.namespace else {
        return;
    };
    let mut exp = NamespaceExports::default();
    let ns_key = segs.join("/");
    for item in &parsed.items {
        match item {
            TopDecl::Routine(r) if r.is_pub => {
                let sig = RoutineSig {
                    return_type: r.return_type.clone(),
                    params: r.params.iter().map(|p| p.ty.clone()).collect(),
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
                exp.values.insert(v.name.clone(), (v.ty.clone(), v.dc_id));
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
            TopDecl::BrandType(b) if b.is_pub => {
                exp.brand_types.insert(
                    b.name.clone(),
                    BrandTypeDef {
                        namespace: Some(ns_key.clone()),
                        original_name: b.name.clone(),
                        is_opaque: b.is_opaque,
                        inner_ty: b.inner_ty.clone(),
                        is_pub: true,
                        dc_id: b.dc_id,
                        span: b.span,
                    },
                );
                if let Some(dc) = b.dc_id {
                    exp.by_dc.insert(
                        dc,
                        ImportedSymbol::BrandType {
                            namespace: ns_key.clone(),
                            original_name: b.name.clone(),
                            is_opaque: b.is_opaque,
                            inner_ty: b.inner_ty.clone(),
                            dc_id: Some(dc),
                        },
                    );
                }
            }
            TopDecl::NamedStruct(s) if s.is_pub => {
                let fields: Vec<(String, StageLType)> = s
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect();
                exp.named_structs.insert(
                    s.name.clone(),
                    NamedStructDef {
                        namespace: Some(ns_key.clone()),
                        original_name: s.name.clone(),
                        fields: fields.clone(),
                        is_pub: true,
                        dc_id: s.dc_id,
                        span: s.span,
                    },
                );
                if let Some(dc) = s.dc_id {
                    exp.by_dc.insert(
                        dc,
                        ImportedSymbol::NamedStruct {
                            namespace: ns_key.clone(),
                            original_name: s.name.clone(),
                            fields,
                            dc_id: Some(dc),
                        },
                    );
                }
            }
            TopDecl::Enum(e) if e.is_pub => {
                let variants: Vec<EnumVariantDef> = e
                    .variants
                    .iter()
                    .map(|v| EnumVariantDef {
                        name: v.name.clone(),
                        payload: v.payload.clone(),
                        span: v.span,
                    })
                    .collect();
                exp.enums.insert(
                    e.name.clone(),
                    EnumDef {
                        namespace: Some(ns_key.clone()),
                        original_name: e.name.clone(),
                        variants: variants.clone(),
                        is_pub: true,
                        dc_id: e.dc_id,
                        span: e.span,
                    },
                );
                if let Some(dc) = e.dc_id {
                    exp.by_dc.insert(
                        dc,
                        ImportedSymbol::Enum {
                            namespace: ns_key.clone(),
                            original_name: e.name.clone(),
                            variants,
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

fn parse_and_validate_workspace_with_seed_graph_and_dialect(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
    default_dialect: StageDialect,
) -> Result<ValidatedWorkspace> {
    let mut headers = Vec::with_capacity(inputs.len());
    let mut caller_ns_to_idx: HashMap<String, usize> = HashMap::new();

    for (idx, inp) in inputs.iter().enumerate() {
        let dialect = dialect_for_source_path(inp.source_path, default_dialect);
        let (ns_opt, uses) =
            scan_module_header_paths_with_dialect(inp.source, dialect)?;
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
        headers.push((dialect, ns_key, uses));
    }

    // Pre-populate `ns_exports` for external `seed_graph` namespaces so caller
    // `use <ns>: ... ::.` headers have exact verb arities and nominal/struct kinds before parsing.
    let mut pre_ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut dummy_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    for (_dialect, ns_key, uses) in &headers {
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
    // imports are known when parsing dependent caller modules.
    let mut caller_deps: Vec<Vec<usize>> = Vec::with_capacity(inputs.len());
    let mut caller_labels: Vec<String> = Vec::with_capacity(inputs.len());
    for (idx, (_dialect, ns_key, uses)) in headers.iter().enumerate() {
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
        let dialect = headers
            .get(idx)
            .map(|(d, _, _)| *d)
            .context("Invalid caller header index")?;
        let parsed = parse_source_with_seed_graph_context(
            inp.source,
            dialect,
            &pre_ns_exports,
            seed_graph,
        )?;
        record_caller_prelim_exports(&parsed, &mut pre_ns_exports);
        if let Some(slot) = parsed_caller_modules.get_mut(idx) {
            *slot = Some(parsed);
        }
    }

    let mut ordered_callers = Vec::with_capacity(inputs.len());
    for (idx, opt) in parsed_caller_modules.into_iter().enumerate() {
        let parsed = opt.context("Missing parsed caller module")?;
        let forbid = parsed.dialect == StageDialect::StageR3
            || inputs
                .get(idx)
                .and_then(|inp| inp.source_path)
                .is_some_and(is_stage2_or_sr3_path);
        ordered_callers.push((parsed, forbid));
    }

    link_parsed_modules_with_seed_graph_and_flags(ordered_callers, seed_graph)
}

/// Parses and validates a multi-module `.sl3` workspace (and any `.sr3` /
/// `.sri3` files identified by extension), resolving any external `use`
/// namespaces and transitive `dc/<shorthand>` Token-Dc helpers from
/// `seed_graph`.
pub fn parse_and_validate_workspace_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    parse_and_validate_workspace_with_seed_graph_and_dialect(
        inputs,
        seed_graph,
        StageDialect::StageL3,
    )
}

/// Parses and validates a multi-module StageR 3 `.sr3` / mixed `.sl3`+`.sr3`
/// workspace against `seed_graph`.
pub fn parse_and_validate_stager_workspace_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    parse_and_validate_workspace_with_seed_graph_and_dialect(
        inputs,
        seed_graph,
        StageDialect::StageR3,
    )
}

fn parse_module_with_seed_graph_and_dialect(
    source: &str,
    seed_graph: &SeedGraphIndex,
    dialect: StageDialect,
) -> Result<Module> {
    let (ns_opt, uses) = scan_module_header_paths_with_dialect(source, dialect)?;
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
    parse_source_with_seed_graph_context(
        source,
        dialect,
        &pre_ns_exports,
        seed_graph,
    )
}

/// Parses a single `.sl3` source file using imported routine arities resolved
/// from `seed_graph` (both `use` namespaces and direct `dc/<shorthand>` calls).
pub fn parse_module_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Module> {
    parse_module_with_seed_graph_and_dialect(
        source,
        seed_graph,
        StageDialect::StageL3,
    )
}

/// Parses a single StageR 3 `.sr3` source file using imported verb arities and
/// nominal/struct kinds resolved from `seed_graph`.
pub fn parse_sr3_module_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Module> {
    parse_module_with_seed_graph_and_dialect(
        source,
        seed_graph,
        StageDialect::StageR3,
    )
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

/// Converts StageR 3 `.sr3` source (resolving external `use` and `dc/<shorthand>`
/// imports against `seed_graph`) into canonical StageR Token Dcs.
pub fn sr3_to_dc_tokens_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let module = parse_sr3_module_with_seed_graph(source, seed_graph)?;
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

/// Parses and validates a single StageR 3 `.sr3` source file against
/// `seed_graph`.
pub fn parse_and_validate_sr3_module_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    parse_and_validate_stager_workspace_with_seed_graph(
        &[ModuleInput {
            source_path: None,
            source,
        }],
        seed_graph,
    )
}

/// Validates a pre-parsed StageL/StageR `Module` against `seed_graph`.
pub fn validate_parsed_module_with_seed_graph(
    module: &Module,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    link_parsed_modules_with_seed_graph(vec![module.clone()], seed_graph)
}

/// Decodes a StageL/StageR Token-Dc stream (`Dc 2192`..`Dc 2193` or
/// `Dc 2218`..`Dc 2193`) and links/validates it against `seed_graph`.
pub fn validate_dc_tokens_with_seed_graph(
    tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let module = decode_module_from_dc_tokens(tokens)?;
    validate_parsed_module_with_seed_graph(&module, seed_graph)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks on a
/// single `.sl3` source file using the deterministic reference evaluator.
pub fn eval_sl3_inline_tests_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_module_with_seed_graph(source, seed_graph)?;
    run_validated_workspace_inline_tests(&ws)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks on a
/// single StageR 3 `.sr3` source file using the deterministic reference
/// evaluator.
pub fn eval_sr3_inline_tests_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws =
        parse_and_validate_sr3_module_with_seed_graph(source, seed_graph)?;
    run_validated_workspace_inline_tests(&ws)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks across
/// a multi-module `.sl3` workspace using the reference evaluator.
pub fn eval_workspace_inline_tests_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_workspace_with_seed_graph(inputs, seed_graph)?;
    run_validated_workspace_inline_tests(&ws)
}

/// Parses, links against `seed_graph`, and executes inline `test` blocks across
/// a multi-module StageR 3 `.sr3` / mixed `.sl3`+`.sr3` workspace using the
/// reference evaluator.
pub fn eval_stager_workspace_inline_tests_with_seed_graph(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
) -> Result<TestSuiteReport> {
    let ws =
        parse_and_validate_stager_workspace_with_seed_graph(inputs, seed_graph)?;
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
    use crate::support::graph_ledger::single_valued_property_set;
    use crate::support::stagel_dc_alloc::{
        DiscoveredStagelFile, ImplStage, reconcile_stagel_and_csv_in_memory,
    };
    use std::collections::BTreeSet;
    use std::path::PathBuf;

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

    fn extract_assigned_dc_shorthand(
        source: &str,
        decl_prefix: &str,
    ) -> Result<String> {
        let lines: Vec<&str> = source.lines().collect();
        for pair in lines.windows(2) {
            if pair[1].trim_start().starts_with(decl_prefix) {
                let dc = pair[0]
                    .trim()
                    .strip_prefix(": @dc ")
                    .with_context(|| {
                        format!(
                            "Missing ': @dc ' comment above '{decl_prefix}' (saw '{}')",
                            pair[0]
                        )
                    })?;
                return Ok(dc.to_owned());
            }
        }
        bail!("Declaration '{decl_prefix}' not found in reconciled source")
    }

    #[crate::ctb_test]
    fn test_stager_workspace_linking_and_dc_token_roundtrip() -> Result<()> {
        let scripts = BTreeSet::new();
        let mut files = vec![
            make_mem_file(
                ImplStage::Stage1,
                "std/base_vals.sl3",
                concat!(
                    "ns std/base_vals.\n\n",
                    "pub u64/BASE_OFFSET u64:10.\n",
                    "pub v/u64/addBase u64/x\n",
                    "    wrapAdd /x /BASE_OFFSET.\n",
                ),
            ),
            make_mem_file(
                ImplStage::Stage2,
                "impls_demo/shapes.sr3",
                concat!(
                    "ns impls_demo/shapes.\n",
                    "use std/base_vals: addBase ::.\n\n",
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
                    "pub v/u64/areaPlusBase shape/sh\n",
                    "    match /sh\n",
                    "        case empty addBase u64:0.\n",
                    "        case boxShape rect/r addBase wrapMul unwrap field /r w unwrap field /r h.\n",
                    "    ::.\n",
                ),
            ),
        ];

        let (nodes, _alloc, _stats) = reconcile_stagel_and_csv_in_memory(
            Vec::new(),
            &mut files,
            &[],
            &[],
            &scripts,
            None,
        )?;
        let shapes_src = &files[1].source;
        let meter_dc = extract_assigned_dc_shorthand(shapes_src, "pub brand u64/meter")?;
        let nz_dc = extract_assigned_dc_shorthand(shapes_src, "pub type u64/nonZero")?;
        let rect_dc = extract_assigned_dc_shorthand(shapes_src, "pub struct rect")?;
        let shape_dc = extract_assigned_dc_shorthand(shapes_src, "pub enum shape")?;
        let unit_sq_dc =
            extract_assigned_dc_shorthand(shapes_src, "pub rect/UNIT_SQUARE")?;
        let mk_nz_dc =
            extract_assigned_dc_shorthand(shapes_src, "pub v/result/nonZero/mkNz")?;
        let area_dc =
            extract_assigned_dc_shorthand(shapes_src, "pub v/u64/areaPlusBase")?;

        let single_props = single_valued_property_set();
        let idx = SeedGraphIndex::from_nodes_with_single_props(nodes, &single_props);

        // 1. Direct `dc/<shorthand>` imports (no `use` header) across brand,
        //    opaque type, named struct, enum, const, and routines + Token-Dc roundtrip.
        let dc_only_caller = format!(
            concat!(
                "ns app/dc_caller.\n\n",
                "pub v/result/u64/runDc u64/raw\n",
                "    test runs direct Dc imports across nominal types and Stage1 dep\n",
                "        in u64:3\n",
                "        out result/u64:ok u64:27\n",
                "    const dc/{nz}/_checked dc/{mknz}? /raw.\n",
                "    const dc/{rect}/r1 structNew dc/{rect}: w wrap u64:3 dc/{meter} h wrap u64:2 dc/{meter} ::.\n",
                "    const dc/{shape}/s1 dc/{shape}:boxShape /r1.\n",
                "    const u64/a1 dc/{area} /s1.\n",
                "    const u64/a2 dc/{area} dc/{shape}:boxShape /dc/{unit_sq}.\n",
                "    result/u64:ok wrapAdd /a1 /a2.\n",
            ),
            meter = meter_dc,
            nz = nz_dc,
            rect = rect_dc,
            shape = shape_dc,
            unit_sq = unit_sq_dc,
            mknz = mk_nz_dc,
            area = area_dc,
        );
        let tokens = sr3_to_dc_tokens_with_seed_graph(&dc_only_caller, &idx)?;
        let dc_ws = validate_dc_tokens_with_seed_graph(&tokens, &idx)?;
        assert_eq!(dc_ws.modules.len(), 3);
        let dc_report = eval_sr3_inline_tests_with_seed_graph(&dc_only_caller, &idx)?;
        assert_eq!(dc_report.ensure_all_passed()?, 1);

        // 2. Multi-module `.sr3` workspace (`use` + direct `dc/<shorthand>` combined)
        let helper_src = concat!(
            "ns app/helper.\n",
            "use impls_demo/shapes: meter rect shape areaPlusBase ::.\n\n",
            "pub v/u64/scaledArea rect/r u64/k\n",
            "    wrapMul areaPlusBase shape:boxShape /r /k.\n",
        );
        let main_src = format!(
            concat!(
                "ns app/main.\n",
                "use app/helper: scaledArea ::.\n",
                "use impls_demo/shapes: meter rect ::.\n\n",
                "pub v/result/u64/compute u64/w u64/h\n",
                "    test combines multi-module use and direct dc linking\n",
                "        in u64:4 u64:5\n",
                "        out result/u64:ok u64:41\n",
                "    const dc/{nz}/_nz dc/{mknz}? /w.\n",
                "    const rect/r structNew rect: w wrap /w meter h wrap /h meter ::.\n",
                "    const u64/scaled scaledArea /r u64:1.\n",
                "    const u64/base dc/{area} dc/{shape}:empty.\n",
                "    result/u64:ok wrapAdd wrapAdd /scaled /base unwrap field /dc/{unit_sq} w.\n",
            ),
            nz = nz_dc,
            mknz = mk_nz_dc,
            area = area_dc,
            shape = shape_dc,
            unit_sq = unit_sq_dc,
        );
        let inputs = [
            ModuleInput {
                source_path: Some("app/helper.sr3"),
                source: helper_src,
            },
            ModuleInput {
                source_path: Some("app/main.sr3"),
                source: &main_src,
            },
        ];
        let ws = parse_and_validate_stager_workspace_with_seed_graph(&inputs, &idx)?;
        assert_eq!(ws.modules.len(), 4);
        let ws_report =
            eval_stager_workspace_inline_tests_with_seed_graph(&inputs, &idx)?;
        assert_eq!(ws_report.ensure_all_passed()?, 1);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager_shared_seed_graph_import_and_stage1_internal_gating()
    -> Result<()> {
        let seed_graph = SeedGraphIndex::shared()?;

        // Positive: StageR 3 module imports public non-internal `std/math::u8Max`
        // both via `use std/math:` and direct `u8/dc/l8589978265`.
        let good_src = r#"ns app/math_ok.
use std/math: u8Max ::.

pub v/b/checkMax
    test imports public u8Max from shared seed graph
        in.
        out true
    eq /u8Max u8/dc/l8589978265.
"#;
        let rep = eval_sr3_inline_tests_with_seed_graph(good_src, &seed_graph)?;
        assert_eq!(rep.ensure_all_passed()?, 1);

        // Negative: StageR 3 cannot import `*StageLInternal` via `use`...
        let bad_use_src = r#"ns app/math_bad_use.
use std/math: modStageLInternal ::.

pub v/u8/bad u8/a u8/rhs
    modStageLInternal /a /rhs.
"#;
        let err_use =
            parse_and_validate_sr3_module_with_seed_graph(bad_use_src, &seed_graph)
                .unwrap_err()
                .to_string();
        assert!(
            err_use.contains("StageL-internal"),
            "Expected StageL-internal error for 'use', got: {err_use}"
        );

        // ...or bypass gating via direct `dc/<shorthand>` (`u8/modStageLInternal` = l8589978542).
        let bad_dc_src = r#"ns app/math_bad_dc.

pub v/u8/bad
    dc/l8589978542 u8:7 u8:3.
"#;
        let err_dc =
            parse_and_validate_sr3_module_with_seed_graph(bad_dc_src, &seed_graph)
                .unwrap_err()
                .to_string();
        assert!(
            err_dc.contains("StageL-internal"),
            "Expected StageL-internal error for 'dc/...', got: {err_dc}"
        );
        Ok(())
    }
}
