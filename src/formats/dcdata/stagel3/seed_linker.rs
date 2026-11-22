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
    decode_module_from_dc_tokens, decode_modules_from_dc_tokens,
    decode_top_decl_from_dc_tokens_at_line, encode_module_to_dc_tokens,
    strip_comment_frames,
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
    ValidatedWorkspace, build_stager_import_context, can_overlay_namespace,
    dialect_for_source_path, expr_syntactically_contains_fault,
    is_stage2_or_sr3_path, stage_rank_for_source_path, top_decl_uses_stager,
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

fn top_decl_is_pub(decl: &TopDecl) -> bool {
    match decl {
        TopDecl::Routine(r) => r.is_pub,
        TopDecl::Value(v) => v.is_pub,
        TopDecl::Struct(s) => s.is_pub,
        TopDecl::BrandType(b) => b.is_pub,
        TopDecl::NamedStruct(s) => s.is_pub,
        TopDecl::Enum(e) => e.is_pub,
        TopDecl::Comment(_) => false,
    }
}

fn top_decl_dc_id(decl: &TopDecl) -> Option<DcChar> {
    match decl {
        TopDecl::Routine(r) => r.dc_id,
        TopDecl::Value(v) => v.dc_id,
        TopDecl::Struct(s) => s.dc_id,
        TopDecl::BrandType(b) => b.dc_id,
        TopDecl::NamedStruct(s) => s.dc_id,
        TopDecl::Enum(e) => e.dc_id,
        TopDecl::Comment(_) => None,
    }
}

fn set_top_decl_is_pub(decl: &mut TopDecl, is_pub: bool) {
    match decl {
        TopDecl::Routine(r) => r.is_pub = is_pub,
        TopDecl::Value(v) => v.is_pub = is_pub,
        TopDecl::Struct(s) => s.is_pub = is_pub,
        TopDecl::BrandType(b) => b.is_pub = is_pub,
        TopDecl::NamedStruct(s) => s.is_pub = is_pub,
        TopDecl::Enum(e) => e.is_pub = is_pub,
        TopDecl::Comment(_) => {}
    }
}

fn type_syntactically_contains_buf(ty: &StageLType) -> bool {
    match ty {
        StageLType::Buf(_) => true,
        StageLType::List(inner)
        | StageLType::Option(inner)
        | StageLType::Result(inner) => type_syntactically_contains_buf(inner),
        StageLType::Tuple(elems) => {
            elems.iter().any(type_syntactically_contains_buf)
        }
        _ => false,
    }
}

fn top_decl_syntactically_contains_buf(decl: &TopDecl) -> bool {
    match decl {
        TopDecl::Routine(r) => {
            type_syntactically_contains_buf(&r.return_type)
                || r.params
                    .iter()
                    .any(|p| type_syntactically_contains_buf(&p.ty))
        }
        TopDecl::Value(v) => type_syntactically_contains_buf(&v.ty),
        TopDecl::Struct(s) => {
            s.fields.iter().any(type_syntactically_contains_buf)
        }
        TopDecl::BrandType(b) => type_syntactically_contains_buf(&b.inner_ty),
        TopDecl::NamedStruct(s) => s
            .fields
            .iter()
            .any(|f| type_syntactically_contains_buf(&f.ty)),
        TopDecl::Enum(e) => e.variants.iter().any(|v| {
            v.payload
                .as_ref()
                .is_some_and(type_syntactically_contains_buf)
        }),
        TopDecl::Comment(_) => false,
    }
}

fn sig_for_routine_decl(r: &RoutineDecl) -> RoutineSig {
    RoutineSig {
        return_type: r.return_type.clone(),
        params: r.params.iter().map(|p| p.ty.clone()).collect(),
        is_pub: true,
        dc_id: r.dc_id,
        may_fault: expr_syntactically_contains_fault(&r.body),
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

fn collect_dc_refs_in_top_decl_sig(decl: &TopDecl, out: &mut Vec<DcChar>) {
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
        TopDecl::Value(TopValueDecl { ty, .. }) => {
            collect_dc_refs_in_type(ty, out);
        }
        TopDecl::Routine(r) => {
            collect_dc_refs_in_type(&r.return_type, out);
            for p in &r.params {
                collect_dc_refs_in_type(&p.ty, out);
            }
        }
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
        collect_dc_refs_in_top_decl_sig(decl, out);
        match decl {
            TopDecl::Comment(_)
            | TopDecl::BrandType(_)
            | TopDecl::NamedStruct(_)
            | TopDecl::Enum(_)
            | TopDecl::Struct(_) => {}
            TopDecl::Value(TopValueDecl { value, .. }) => {
                collect_dc_refs_in_expr(value, out);
            }
            TopDecl::Routine(r) => {
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
        Expr::AnonRoutine(r) => {
            collect_dc_refs_in_type(&r.return_type, out);
            for p in &r.params {
                collect_dc_refs_in_type(&p.ty, out);
            }
            collect_dc_refs_in_expr(&r.body, out);
        }
        Expr::InlineCall { routine, args, .. } => {
            collect_dc_refs_in_type(&routine.return_type, out);
            for p in &routine.params {
                collect_dc_refs_in_type(&p.ty, out);
            }
            collect_dc_refs_in_expr(&routine.body, out);
            for arg in args {
                collect_dc_refs_in_expr(arg, out);
            }
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
                use crate::syntax::SyntaxRuleResolver as _;
                if !crate::syntax::default_syntax_resolver()
                    .token_fills_role(dc.to_u128(), crate::syntax::DcRole::SelfReference)
                {
                    out.push(*dc);
                }
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

fn reconstruct_external_namespace_modules(
    seed_graph: &SeedGraphIndex,
    ns_segs: &[String],
    caller_ns_for_diag: &str,
    allow_stager_deps: bool,
    dc_owners: &mut HashMap<DcChar, GraphDcOwner>,
) -> Result<(Vec<Module>, NamespaceExports)> {
    let mut line_counter = 1usize;
    reconstruct_external_namespace_modules_with_line(
        seed_graph,
        ns_segs,
        caller_ns_for_diag,
        allow_stager_deps,
        dc_owners,
        &mut line_counter,
    )
}

#[allow(
    clippy::too_many_lines,
    reason = "Reconstructs graph-exported declarations and splits mixed StageL/StageR namespaces into dialect strata"
)]
fn reconstruct_external_namespace_modules_with_line(
    seed_graph: &SeedGraphIndex,
    ns_segs: &[String],
    caller_ns_for_diag: &str,
    allow_stager_deps: bool,
    dc_owners: &mut HashMap<DcChar, GraphDcOwner>,
    next_decl_line: &mut usize,
) -> Result<(Vec<Module>, NamespaceExports)> {
    let ns_key = ns_segs.join("/");
    let exports_map = seed_graph.resolve_stagel_namespace_exports(ns_segs)?;
    if exports_map.is_empty() {
        bail!(
            "Unresolved 'use {ns_key}:' import in module {caller_ns_for_diag}"
        );
    }

    let mut sl3_items = Vec::new();
    let mut sr3_items = Vec::new();
    let mut prelim_exports = NamespaceExports::default();
    let mut any_skipped_stager = false;

    for (export_name, target_sh_list) in exports_map {
        for target_sh in target_sh_list {
            let decl_line = *next_decl_line;
            *next_decl_line = next_decl_line.saturating_add(1);
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
            let is_stager_item = top_decl_uses_stager(&prepared_decl);
            if is_stager_item && !allow_stager_deps {
                any_skipped_stager = true;
                continue;
            }
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
            if is_stager_item {
                sr3_items.push(prepared_decl);
            } else {
                sl3_items.push(prepared_decl);
            }
        }
    }

    if sl3_items.is_empty() && sr3_items.is_empty() && any_skipped_stager {
        bail!(
            "Namespace '{ns_key}' imported by {caller_ns_for_diag} only exports StageR 3 items, which cannot be used by a StageL 3 caller"
        );
    }

    let mut modules = Vec::new();
    if !sl3_items.is_empty() {
        modules.push(Module {
            dialect: StageDialect::StageL3,
            leading_comments: Vec::new(),
            namespace: Some(ns_segs.to_vec()),
            uses: Vec::new(),
            items: sl3_items,
        });
    }
    if !sr3_items.is_empty() {
        modules.push(Module {
            dialect: StageDialect::StageR3,
            leading_comments: Vec::new(),
            namespace: Some(ns_segs.to_vec()),
            uses: Vec::new(),
            items: sr3_items,
        });
    }
    Ok((modules, prelim_exports))
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

#[derive(Debug, Clone)]
struct CallerSpec {
    module: Module,
    forbid_stagel_internal: bool,
    stage_rank: Option<u8>,
    ordered_stream: bool,
    input_index: usize,
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
        .enumerate()
        .map(|(input_index, module)| {
            let forbid_stagel_internal =
                module.dialect == StageDialect::StageR3;
            CallerSpec {
                module,
                forbid_stagel_internal,
                stage_rank: None,
                ordered_stream: false,
                input_index,
            }
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
    all_input_indices: Vec<usize>,
    module_extra_dc_imports: Vec<HashMap<DcChar, ImportedSymbol>>,
    order: Vec<usize>,
}

fn tarjan_sccs(
    node_count: usize,
    deps: &[Vec<usize>],
) -> Result<Vec<Vec<usize>>> {
    struct TarjanState {
        next_idx: usize,
        indices: Vec<Option<usize>>,
        lowlink: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<usize>,
        sccs: Vec<Vec<usize>>,
    }

    fn strongconnect(
        v: usize,
        deps: &[Vec<usize>],
        st: &mut TarjanState,
    ) -> Result<()> {
        let cur_idx = st.next_idx;
        st.next_idx = st.next_idx.saturating_add(1);
        *st.indices.get_mut(v).context("Invalid Tarjan index slot")? =
            Some(cur_idx);
        *st.lowlink.get_mut(v).context("Invalid Tarjan lowlink slot")? = cur_idx;
        st.stack.push(v);
        *st.on_stack.get_mut(v).context("Invalid Tarjan on_stack slot")? = true;

        let v_deps = deps.get(v).context("Missing Tarjan dependency list")?;
        for &w in v_deps {
            let w_idx = *st.indices.get(w).context("Invalid Tarjan neighbor")?;
            if let Some(wi) = w_idx {
                if *st.on_stack.get(w).context("Invalid Tarjan neighbor stack")? {
                    let v_low = st
                        .lowlink
                        .get_mut(v)
                        .context("Invalid Tarjan lowlink")?;
                    *v_low = (*v_low).min(wi);
                }
            } else {
                strongconnect(w, deps, st)?;
                let w_low =
                    *st.lowlink.get(w).context("Invalid Tarjan child lowlink")?;
                let v_low =
                    st.lowlink.get_mut(v).context("Invalid Tarjan lowlink")?;
                *v_low = (*v_low).min(w_low);
            }
        }

        let is_root =
            st.lowlink.get(v).copied() == st.indices.get(v).copied().flatten();
        if is_root {
            let mut comp = Vec::new();
            loop {
                let w = st
                    .stack
                    .pop()
                    .context("Unexpected empty Tarjan stack")?;
                *st.on_stack
                    .get_mut(w)
                    .context("Invalid Tarjan pop slot")? = false;
                comp.push(w);
                if w == v {
                    break;
                }
            }
            st.sccs.push(comp);
        }
        Ok(())
    }

    let mut st = TarjanState {
        next_idx: 0,
        indices: vec![None; node_count],
        lowlink: vec![0; node_count],
        on_stack: vec![false; node_count],
        stack: Vec::with_capacity(node_count),
        sccs: Vec::new(),
    };
    for v in 0..node_count {
        if st.indices.get(v).copied().flatten().is_none() {
            strongconnect(v, deps, &mut st)?;
        }
    }
    Ok(st.sccs)
}

fn order_merged_module_items(items: impl IntoIterator<Item = TopDecl>) -> Vec<TopDecl> {
    let mut type_items = Vec::new();
    let mut val_items = Vec::new();
    let mut rout_items = Vec::new();
    let mut other_items = Vec::new();
    for item in items {
        match item {
            TopDecl::Struct(_)
            | TopDecl::BrandType(_)
            | TopDecl::NamedStruct(_)
            | TopDecl::Enum(_) => type_items.push(item),
            TopDecl::Value(_) => val_items.push(item),
            TopDecl::Routine(_) => rout_items.push(item),
            TopDecl::Comment(_) => other_items.push(item),
        }
    }
    let mut all_items = Vec::with_capacity(
        type_items
            .len()
            .saturating_add(val_items.len())
            .saturating_add(rout_items.len())
            .saturating_add(other_items.len()),
    );
    all_items.append(&mut type_items);
    all_items.append(&mut val_items);
    all_items.append(&mut rout_items);
    all_items.append(&mut other_items);
    all_items
}

fn compute_synth_module_deps(
    modules: &[(String, bool, bool, Module)],
    dc_owners: &HashMap<DcChar, GraphDcOwner>,
    local_dc_to_idx: &HashMap<DcChar, usize>,
) -> Vec<Vec<usize>> {
    let mut ns_to_indices: HashMap<String, Vec<usize>> =
        HashMap::with_capacity(modules.len());
    for (idx, (k, _, _, _)) in modules.iter().enumerate() {
        ns_to_indices.entry(k.clone()).or_default().push(idx);
    }
    let mut synth_deps: Vec<Vec<usize>> = Vec::with_capacity(modules.len());
    for (idx, (self_k, _, _, m)) in modules.iter().enumerate() {
        let mut mod_local_dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut mod_local_dcs);
        let mut d = Vec::new();
        if let Some(strata) = ns_to_indices.get(self_k)
            && let Some(pos) = strata.iter().position(|&s| s == idx)
            && pos > 0
            && let Some(&prev_idx) = strata.get(pos.saturating_sub(1))
            && !d.contains(&prev_idx)
        {
            d.push(prev_idx);
        }
        for u in &m.uses {
            let u_key = u.path.join("/");
            if let Some(cand_indices) = ns_to_indices.get(&u_key) {
                for &dep_idx in cand_indices {
                    if dep_idx == idx {
                        continue;
                    }
                    if u_key == *self_k
                        && let Some(self_pos) =
                            cand_indices.iter().position(|&s| s == idx)
                        && let Some(dep_pos) =
                            cand_indices.iter().position(|&s| s == dep_idx)
                        && dep_pos >= self_pos
                    {
                        continue;
                    }
                    if m.dialect == StageDialect::StageL3
                        && modules
                            .get(dep_idx)
                            .is_some_and(|dm| dm.3.dialect == StageDialect::StageR3)
                        && cand_indices.iter().any(|&ci| {
                            modules
                                .get(ci)
                                .is_some_and(|dm| dm.3.dialect == StageDialect::StageL3)
                        })
                    {
                        continue;
                    }
                    if !d.contains(&dep_idx) {
                        d.push(dep_idx);
                    }
                }
            }
        }
        let mut refs = Vec::new();
        collect_dc_refs_in_module(m, &mut refs);
        let mut ref_queue: VecDeque<DcChar> = refs.into_iter().collect();
        let mut visited = HashSet::new();
        while let Some(dc) = ref_queue.pop_front() {
            if mod_local_dcs.contains(&dc) || !visited.insert(dc) {
                continue;
            }
            if let Some(owner) = dc_owners.get(&dc) {
                let mut sig_refs = Vec::new();
                collect_dc_refs_in_owner_kind(&owner.kind, &mut sig_refs);
                for s_dc in sig_refs {
                    ref_queue.push_back(s_dc);
                }
                if let Some(&dep_idx) = local_dc_to_idx.get(&dc) {
                    if dep_idx != idx && !d.contains(&dep_idx) {
                        d.push(dep_idx);
                    }
                } else if let Some(cand_indices) =
                    ns_to_indices.get(&owner.namespace_key)
                {
                    for &dep_idx in cand_indices {
                        if dep_idx != idx && !d.contains(&dep_idx) {
                            d.push(dep_idx);
                        }
                    }
                }
            } else if let Some(&dep_idx) = local_dc_to_idx.get(&dc)
                && dep_idx != idx
                && !d.contains(&dep_idx)
            {
                d.push(dep_idx);
            }
        }
        synth_deps.push(d);
    }
    synth_deps
}

#[allow(
    clippy::too_many_lines,
    reason = "Absorbs single-owner file-private graph helpers, condenses synthetic SCCs, and restores module-private visibility"
)]
fn coalesce_synthetic_sccs(
    synth_helper_modules: BTreeMap<String, (Module, bool)>,
    synth_ns_modules: BTreeMap<String, Vec<Module>>,
    orig_private_dcs: &HashSet<DcChar>,
    caller_modules: &[CallerSpec],
    dc_owners: &mut HashMap<DcChar, GraphDcOwner>,
) -> Result<Vec<Module>> {
    let ns_total: usize = synth_ns_modules.values().map(Vec::len).sum();
    let total = synth_helper_modules.len().saturating_add(ns_total);
    if total == 0 {
        return Ok(Vec::new());
    }

    // Tuple: `(ns_key, is_real_ns, is_anchor, module)`.
    let mut raw_synth: Vec<(String, bool, bool, Module)> =
        Vec::with_capacity(total);
    for (k, (m, is_pub_anchor)) in synth_helper_modules {
        raw_synth.push((k, false, is_pub_anchor, m));
    }
    for (k, mods) in synth_ns_modules {
        for m in mods {
            raw_synth.push((k.clone(), true, true, m));
        }
    }

    let mut raw_local_dc_to_idx: HashMap<DcChar, usize> = HashMap::new();
    for (idx, (_, _, _, m)) in raw_synth.iter().enumerate() {
        let mut dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut dcs);
        for dc in dcs {
            raw_local_dc_to_idx.insert(dc, idx);
        }
    }

    // Phase 1: Absorb non-`pub` `$graph_dc_*` helpers (`!is_anchor`) that are not directly
    // or transitively reached from `caller_modules` (without crossing an anchor) and belong
    // to a unique synthetic anchor module of matching dialect.
    let phase1_deps = compute_synth_module_deps(
        &raw_synth,
        dc_owners,
        &raw_local_dc_to_idx,
    );
    let mut ns_to_indices: HashMap<String, Vec<usize>> =
        HashMap::with_capacity(raw_synth.len());
    for (idx, (k, _, _, _)) in raw_synth.iter().enumerate() {
        ns_to_indices.entry(k.clone()).or_default().push(idx);
    }

    let mut reached_by_caller = vec![false; raw_synth.len()];
    {
        let mut caller_refs = Vec::new();
        let mut caller_local_dcs = HashSet::new();
        for cs in caller_modules {
            collect_locally_defined_dcs(&cs.module, &mut caller_local_dcs);
            collect_dc_refs_in_module(&cs.module, &mut caller_refs);
        }
        let mut dc_q: VecDeque<DcChar> = caller_refs.into_iter().collect();
        let mut seen_dc = HashSet::new();
        let mut helper_q: VecDeque<usize> = VecDeque::new();
        while let Some(dc) = dc_q.pop_front() {
            if caller_local_dcs.contains(&dc) || !seen_dc.insert(dc) {
                continue;
            }
            if let Some(owner) = dc_owners.get(&dc) {
                let mut sig_refs = Vec::new();
                collect_dc_refs_in_owner_kind(&owner.kind, &mut sig_refs);
                for s_dc in sig_refs {
                    dc_q.push_back(s_dc);
                }
                if let Some(&h_idx) = raw_local_dc_to_idx.get(&dc)
                    && let Some((_, _, is_anchor, _)) = raw_synth.get(h_idx)
                    && !*is_anchor
                    && let Some(flag) = reached_by_caller.get_mut(h_idx)
                    && !*flag
                {
                    *flag = true;
                    helper_q.push_back(h_idx);
                } else if let Some(cand_indices) =
                    ns_to_indices.get(&owner.namespace_key)
                {
                    for &h_idx in cand_indices {
                        if let Some((_, _, is_anchor, _)) = raw_synth.get(h_idx)
                            && !*is_anchor
                            && let Some(flag) = reached_by_caller.get_mut(h_idx)
                            && !*flag
                        {
                            *flag = true;
                            helper_q.push_back(h_idx);
                        }
                    }
                }
            }
        }
        while let Some(h_idx) = helper_q.pop_front() {
            if let Some(neighbors) = phase1_deps.get(h_idx) {
                for &n_idx in neighbors {
                    if let Some((_, _, is_anchor, _)) = raw_synth.get(n_idx)
                        && !*is_anchor
                        && let Some(flag) = reached_by_caller.get_mut(n_idx)
                        && !*flag
                    {
                        *flag = true;
                        helper_q.push_back(n_idx);
                    }
                }
            }
        }
    }

    let mut anchor_owners: Vec<HashSet<usize>> =
        vec![HashSet::new(); raw_synth.len()];
    for a_idx in 0..raw_synth.len() {
        let Some((_, _, true, _)) = raw_synth.get(a_idx) else {
            continue;
        };
        let mut helper_q: VecDeque<usize> = VecDeque::new();
        if let Some(neighbors) = phase1_deps.get(a_idx) {
            for &n_idx in neighbors {
                if let Some((_, _, false, _)) = raw_synth.get(n_idx)
                    && let Some(owners) = anchor_owners.get_mut(n_idx)
                    && owners.insert(a_idx)
                {
                    helper_q.push_back(n_idx);
                }
            }
        }
        while let Some(h_idx) = helper_q.pop_front() {
            if let Some(neighbors) = phase1_deps.get(h_idx) {
                for &n_idx in neighbors {
                    if let Some((_, _, false, _)) = raw_synth.get(n_idx)
                        && let Some(owners) = anchor_owners.get_mut(n_idx)
                        && owners.insert(a_idx)
                    {
                        helper_q.push_back(n_idx);
                    }
                }
            }
        }
    }

    let mut absorbed_into: Vec<Option<usize>> = vec![None; raw_synth.len()];
    for h_idx in 0..raw_synth.len() {
        let Some((_, _, false, helper_mod)) = raw_synth.get(h_idx) else {
            continue;
        };
        let by_caller = reached_by_caller.get(h_idx).copied().unwrap_or(false); // Reason for fallback: h_idx is bounded by raw_synth.len() == reached_by_caller.len()
        if by_caller {
            continue;
        }
        if let Some(owners) = anchor_owners.get(h_idx)
            && owners.len() == 1
            && let Some(&sole_anchor) = owners.iter().next()
            && let Some((_, _, _, anchor_mod)) = raw_synth.get(sole_anchor)
            && anchor_mod.dialect == helper_mod.dialect
            && let Some(slot) = absorbed_into.get_mut(h_idx)
        {
            *slot = Some(sole_anchor);
        }
    }

    let mut phase1_slots: Vec<Option<(String, bool, bool, Module)>> =
        raw_synth.into_iter().map(Some).collect();
    for h_idx in 0..phase1_slots.len() {
        let Some(Some(a_idx)) = absorbed_into.get(h_idx).copied() else {
            continue;
        };
        let Some((_, _, _, helper_mod)) =
            phase1_slots.get_mut(h_idx).and_then(Option::take)
        else {
            continue;
        };
        let Some((anchor_key, _, _, anchor_mod)) =
            phase1_slots.get_mut(a_idx).and_then(Option::as_mut)
        else {
            continue;
        };
        let anchor_key_cloned = anchor_key.clone();
        let mut helper_dcs = HashSet::new();
        collect_locally_defined_dcs(&helper_mod, &mut helper_dcs);
        for dc in helper_dcs {
            if let Some(owner) = dc_owners.get_mut(&dc) {
                owner.namespace_key.clone_from(&anchor_key_cloned);
            }
        }
        for u in helper_mod.uses {
            if u.path.join("/") != anchor_key_cloned {
                anchor_mod.uses.push(u);
            }
        }
        let combined = std::mem::take(&mut anchor_mod.items)
            .into_iter()
            .chain(helper_mod.items);
        anchor_mod.items = order_merged_module_items(combined);
        anchor_mod.dialect = if anchor_mod.items.iter().any(top_decl_uses_stager)
        {
            StageDialect::StageR3
        } else {
            StageDialect::StageL3
        };
    }

    let post_absorb: Vec<(String, bool, bool, Module)> =
        phase1_slots.into_iter().flatten().collect();

    // Phase 2: Condense any remaining synthetic SCCs (`<= 1` real namespace per SCC).
    let mut post_dc_to_idx: HashMap<DcChar, usize> = HashMap::new();
    for (idx, (_, _, _, m)) in post_absorb.iter().enumerate() {
        let mut dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut dcs);
        for dc in dcs {
            post_dc_to_idx.insert(dc, idx);
        }
    }
    let synth_deps =
        compute_synth_module_deps(&post_absorb, dc_owners, &post_dc_to_idx);
    let sccs = tarjan_sccs(post_absorb.len(), &synth_deps)?;
    let mut slots: Vec<Option<(String, bool, bool, Module)>> =
        post_absorb.into_iter().map(Some).collect();
    let mut merged = Vec::with_capacity(sccs.len());

    for mut comp in sccs {
        comp.sort_unstable();
        if comp.len() == 1 {
            let only_idx = *comp.first().context("Empty Tarjan component")?;
            let (_, _, _, m) = slots
                .get_mut(only_idx)
                .and_then(Option::take)
                .context("Missing synthetic module slot")?;
            merged.push(m);
            continue;
        }

        let mut entries = Vec::with_capacity(comp.len());
        let mut real_ns_count = 0usize;
        for idx in comp {
            let entry = slots
                .get_mut(idx)
                .and_then(Option::take)
                .context("Missing synthetic SCC module slot")?;
            if entry.1 {
                real_ns_count = real_ns_count.saturating_add(1);
            }
            entries.push(entry);
        }

        // Never merge a cycle spanning two or more real namespaces; leave them
        // separate so `topo_sort_all_modules` reports the namespace cycle.
        if real_ns_count > 1 {
            for (_, _, _, m) in entries {
                merged.push(m);
            }
            continue;
        }

        let chosen_pos = entries
            .iter()
            .position(|(_, is_real, _, _)| *is_real)
            .or_else(|| entries.iter().position(|(_, _, is_anchor, _)| *is_anchor))
            .unwrap_or(0); // Reason for fallback: all-helper SCCs deterministically adopt their first `$graph_dc_*` namespace
        let chosen_entry = entries
            .get(chosen_pos)
            .context("Invalid chosen SCC entry")?;
        let chosen_key = chosen_entry.0.clone();
        let chosen_ns = chosen_entry.3.namespace.clone();

        let mut merged_uses = Vec::new();
        let mut raw_items = Vec::new();
        for (_, _, _, m) in entries {
            for u in m.uses {
                if u.path.join("/") != chosen_key {
                    merged_uses.push(u);
                }
            }
            raw_items.extend(m.items);
        }
        let all_items = order_merged_module_items(raw_items);
        let dialect = if all_items.iter().any(top_decl_uses_stager) {
            StageDialect::StageR3
        } else {
            StageDialect::StageL3
        };
        let merged_mod = Module {
            dialect,
            leading_comments: Vec::new(),
            namespace: chosen_ns,
            uses: merged_uses,
            items: all_items,
        };
        let mut merged_dcs = HashSet::new();
        collect_locally_defined_dcs(&merged_mod, &mut merged_dcs);
        for dc in merged_dcs {
            if let Some(owner) = dc_owners.get_mut(&dc) {
                owner.namespace_key.clone_from(&chosen_key);
            }
        }
        merged.push(merged_mod);
    }

    // Phase 3: Restore `is_pub = false` on any originally file-private `@dc` helper
    // that has no external references outside its owning module (or carries `buf/T`).
    let mut externally_referenced_dcs: HashSet<DcChar> = HashSet::new();
    for m in merged.iter().chain(caller_modules.iter().map(|cs| &cs.module)) {
        let mut mod_local_dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut mod_local_dcs);
        let mut refs = Vec::new();
        collect_dc_refs_in_module(m, &mut refs);
        let mut q: VecDeque<DcChar> = refs.into_iter().collect();
        let mut seen = HashSet::new();
        while let Some(dc) = q.pop_front() {
            if mod_local_dcs.contains(&dc) || !seen.insert(dc) {
                continue;
            }
            externally_referenced_dcs.insert(dc);
            if let Some(owner) = dc_owners.get(&dc) {
                let mut sig_refs = Vec::new();
                collect_dc_refs_in_owner_kind(&owner.kind, &mut sig_refs);
                for s_dc in sig_refs {
                    q.push_back(s_dc);
                }
            }
        }
    }

    for m in &mut merged {
        for item in &mut m.items {
            let Some(dc) = top_decl_dc_id(item) else {
                continue;
            };
            if (orig_private_dcs.contains(&dc)
                && !externally_referenced_dcs.contains(&dc))
                || top_decl_syntactically_contains_buf(item)
            {
                set_top_decl_is_pub(item, false);
                if let Some(owner) = dc_owners.get_mut(&dc)
                    && let GraphOwnerKind::Routine { sig } = &mut owner.kind
                {
                    sig.is_pub = false;
                }
            }
        }
    }

    Ok(merged)
}

#[allow(
    clippy::too_many_lines,
    reason = "Assembles reconstructed multi-stratum graph namespaces, transitive Dc helpers, SCC condensation, and topological ordering"
)]
struct SynthLoadState<'a> {
    seed_graph: &'a SeedGraphIndex,
    dc_owners: &'a mut HashMap<DcChar, GraphDcOwner>,
    next_synth_decl_line: &'a mut usize,
    caller_ns_strata: &'a HashMap<String, Vec<(Option<u8>, StageDialect, bool)>>,
    synth_ns_modules: &'a mut BTreeMap<String, Vec<Module>>,
    local_dcs: &'a mut HashSet<DcChar>,
}

fn ensure_external_namespace_loaded(
    state: &mut SynthLoadState<'_>,
    ns_segs: &[String],
    caller_label: &str,
    need_stager: bool,
    mut dc_worklist: Option<&mut VecDeque<(DcChar, bool)>>,
) -> Result<()> {
    let ns_key = ns_segs.join("/");
    if state.caller_ns_strata.contains_key(&ns_key) {
        return Ok(());
    }
    if let Some(existing_mods) = state.synth_ns_modules.get(&ns_key) {
        if !need_stager
            || existing_mods
                .iter()
                .any(|m| m.dialect == StageDialect::StageR3)
        {
            return Ok(());
        }
        let (mut ext_mods, _) =
            reconstruct_external_namespace_modules_with_line(
                state.seed_graph,
                ns_segs,
                caller_label,
                true,
                state.dc_owners,
                state.next_synth_decl_line,
            )?;
        ext_mods.retain(|m| m.dialect == StageDialect::StageR3);
        for ext_mod in &ext_mods {
            collect_locally_defined_dcs(ext_mod, state.local_dcs);
            if let Some(ref mut q) = dc_worklist {
                let mut ext_refs = Vec::new();
                collect_dc_refs_in_module(ext_mod, &mut ext_refs);
                for r_dc in ext_refs {
                    q.push_back((r_dc, true));
                }
            }
        }
        if let Some(slot) = state.synth_ns_modules.get_mut(&ns_key) {
            slot.extend(ext_mods);
        }
        return Ok(());
    }
    let (ext_mods, _) = reconstruct_external_namespace_modules_with_line(
        state.seed_graph,
        ns_segs,
        caller_label,
        need_stager,
        state.dc_owners,
        state.next_synth_decl_line,
    )?;
    for ext_mod in &ext_mods {
        collect_locally_defined_dcs(ext_mod, state.local_dcs);
        if let Some(ref mut q) = dc_worklist {
            let is_sr = ext_mod.dialect == StageDialect::StageR3;
            let mut ext_refs = Vec::new();
            collect_dc_refs_in_module(ext_mod, &mut ext_refs);
            for r_dc in ext_refs {
                q.push_back((r_dc, is_sr));
            }
        }
    }
    state.synth_ns_modules.insert(ns_key, ext_mods);
    Ok(())
}

fn assemble_topo_sorted_modules_with_seed_graph(
    caller_modules: Vec<CallerSpec>,
    seed_graph: &SeedGraphIndex,
) -> Result<AssembledGraphModules> {
    let allow_stager_deps = caller_modules
        .iter()
        .any(|c| c.module.dialect == StageDialect::StageR3);
    let mut dc_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    let mut caller_ns_strata: HashMap<
        String,
        Vec<(Option<u8>, StageDialect, bool)>,
    > = HashMap::new();
    let mut local_dcs: HashSet<DcChar> = HashSet::new();

    for cs in &caller_modules {
        if let Some(segs) = &cs.module.namespace {
            let key = segs.join("/");
            let strata = caller_ns_strata.entry(key.clone()).or_default();
            for &(prev_rank, prev_dialect, prev_ordered) in strata.iter() {
                if cs.ordered_stream && prev_ordered {
                    ensure!(
                        !(prev_dialect == StageDialect::StageR3
                            && cs.module.dialect == StageDialect::StageL3),
                        "StageL 3 module cannot overlay StageR 3 module in '{key}'"
                    );
                } else {
                    ensure!(
                        can_overlay_namespace(
                            prev_rank,
                            prev_dialect,
                            cs.stage_rank,
                            cs.module.dialect,
                        ),
                        "Duplicate namespace declaration '{key}'"
                    );
                }
            }
            strata.push((cs.stage_rank, cs.module.dialect, cs.ordered_stream));
        }
        collect_locally_defined_dcs(&cs.module, &mut local_dcs);
    }

    // 1. Reconstruct any external `use <ns>:` namespaces from `seed_graph`.
    let mut next_synth_decl_line: usize = 100_000;
    let mut synth_ns_modules: BTreeMap<String, Vec<Module>> = BTreeMap::new();
    let mut load_state = SynthLoadState {
        seed_graph,
        dc_owners: &mut dc_owners,
        next_synth_decl_line: &mut next_synth_decl_line,
        caller_ns_strata: &caller_ns_strata,
        synth_ns_modules: &mut synth_ns_modules,
        local_dcs: &mut local_dcs,
    };
    for cs in &caller_modules {
        // Reason for fallback: Anonymous caller modules format as '<anonymous>' in diagnostics.
        let caller_label = cs
            .module
            .namespace
            .as_ref()
            .map_or_else(|| String::from("<anonymous>"), |s| s.join("/"));
        let need_stager =
            allow_stager_deps && cs.module.dialect == StageDialect::StageR3;
        for u in &cs.module.uses {
            ensure_external_namespace_loaded(
                &mut load_state,
                &u.path,
                &caller_label,
                need_stager,
                None,
            )?;
        }
    }

    // 2. Transitively resolve all `SymbolTarget::Dc` / `ImportTarget::Dc` helpers from `seed_graph`.
    let mut dc_worklist: VecDeque<(DcChar, bool)> = VecDeque::new();
    for m in load_state
        .synth_ns_modules
        .values()
        .flatten()
        .chain(caller_modules.iter().map(|cs| &cs.module))
    {
        let from_sr = m.dialect == StageDialect::StageR3;
        let mut initial_refs = Vec::new();
        collect_dc_refs_in_module(m, &mut initial_refs);
        for dc in initial_refs {
            dc_worklist.push_back((dc, from_sr));
        }
    }

    let mut synth_helper_modules: BTreeMap<String, (Module, bool)> =
        BTreeMap::new();
    let mut orig_private_dcs: HashSet<DcChar> = HashSet::new();
    let mut checked_missing_dcs: HashSet<DcChar> = HashSet::new();

    while let Some((dc, from_stager)) = dc_worklist.pop_front() {
        if load_state.local_dcs.contains(&dc)
            || load_state.dc_owners.contains_key(&dc)
            || (!from_stager && checked_missing_dcs.contains(&dc))
        {
            continue;
        }
        let sh = DcShorthand::from_global_id(dc.to_u128());
        if let Some(ns_segs) = seed_graph.resolve_stagel_entity_namespace(sh)? {
            ensure_external_namespace_loaded(
                &mut load_state,
                &ns_segs,
                "<direct dc import>",
                from_stager && allow_stager_deps,
                Some(&mut dc_worklist),
            )?;
            if load_state.local_dcs.contains(&dc)
                || load_state.dc_owners.contains_key(&dc)
            {
                continue;
            }
        }
        let decl_line = *load_state.next_synth_decl_line;
        *load_state.next_synth_decl_line =
            load_state.next_synth_decl_line.saturating_add(1);
        let Some((_, entry)) =
            decode_graph_stagel_entry_at_line(seed_graph, sh, decl_line)?
        else {
            if !from_stager {
                checked_missing_dcs.insert(dc);
            }
            continue;
        };

        let synth_ns_seg = format!("$graph_dc_{sh}");
        let (synth_mod, key, is_pub_anchor) = match entry {
            EitherDeclOrModule::Decl(raw_decl) => {
                let raw_was_pub = top_decl_is_pub(&raw_decl);
                if !raw_was_pub {
                    orig_private_dcs.insert(dc);
                }
                let (prepared_decl, orig_name, kind) =
                    prepare_graph_top_decl(raw_decl, None, dc)?;
                load_state.dc_owners.insert(
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
                    raw_was_pub,
                )
            }
            EitherDeclOrModule::Module(mut m) => {
                let ns_key = if let Some(existing_ns) = &m.namespace {
                    let joined = existing_ns.join("/");
                    if caller_ns_strata.contains_key(&joined)
                        || load_state.synth_ns_modules.contains_key(&joined)
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
                let mut any_pub = false;
                for item in &mut m.items {
                    if top_decl_is_pub(item) {
                        any_pub = true;
                    } else if let Some(item_dc) = top_decl_dc_id(item) {
                        orig_private_dcs.insert(item_dc);
                    }
                    if let TopDecl::Routine(r) = item {
                        r.tests.clear();
                    }
                    if let Some(owner) =
                        claim_graph_item_owner(item, dc, &ns_key, false)
                    {
                        load_state.dc_owners.insert(dc, owner);
                        matched = true;
                    }
                }
                if !matched {
                    // Single-item module fallback when `dc_id` was carried on the enclosing entity.
                    for item in &mut m.items {
                        if let Some(owner) =
                            claim_graph_item_owner(item, dc, &ns_key, true)
                        {
                            load_state.dc_owners.insert(dc, owner);
                            matched = true;
                            break;
                        }
                    }
                }
                ensure!(
                    matched,
                    "StageL/StageR Token-Dc module on {sh} contains no declaration"
                );
                let mod_need_stager =
                    allow_stager_deps && m.dialect == StageDialect::StageR3;
                // If this graph module itself has external `use` declarations, resolve them too.
                for u in &m.uses {
                    ensure_external_namespace_loaded(
                        &mut load_state,
                        &u.path,
                        &ns_key,
                        mod_need_stager,
                        Some(&mut dc_worklist),
                    )?;
                }
                (m, ns_key, any_pub)
            }
        };

        collect_locally_defined_dcs(&synth_mod, load_state.local_dcs);
        let synth_is_sr = synth_mod.dialect == StageDialect::StageR3;
        let mut nested_refs = Vec::new();
        collect_dc_refs_in_module(&synth_mod, &mut nested_refs);
        for nested_dc in nested_refs {
            dc_worklist.push_back((nested_dc, synth_is_sr));
        }
        synth_helper_modules.insert(key, (synth_mod, is_pub_anchor));
    }

    // 3. Absorb file-private helpers into their unique owning synthetic anchors and
    // coalesce any remaining Strongly Connected Components among synthetic graph modules,
    // then assemble with caller modules, compute exact dependency edges, and topologically sort!
    let merged_synth = coalesce_synthetic_sccs(
        synth_helper_modules,
        synth_ns_modules,
        &orig_private_dcs,
        &caller_modules,
        &mut dc_owners,
    )?;
    let total_cap = merged_synth.len().saturating_add(caller_modules.len());
    let mut all_modules: Vec<Module> = Vec::with_capacity(total_cap);
    let mut all_forbid_flags: Vec<bool> = Vec::with_capacity(total_cap);
    let mut all_stage_ranks: Vec<Option<u8>> = Vec::with_capacity(total_cap);
    let mut all_input_indices: Vec<usize> = Vec::with_capacity(total_cap);
    for m in merged_synth {
        all_modules.push(m);
        all_forbid_flags.push(false);
        all_stage_ranks.push(None);
        all_input_indices.push(0);
    }
    for cs in caller_modules {
        all_modules.push(cs.module);
        all_forbid_flags.push(cs.forbid_stagel_internal);
        all_stage_ranks.push(cs.stage_rank);
        all_input_indices.push(cs.input_index);
    }

    let mut ns_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
    let mut labels: Vec<String> = Vec::with_capacity(all_modules.len());
    for (idx, m) in all_modules.iter().enumerate() {
        if let Some(segs) = &m.namespace {
            let key = segs.join("/");
            ns_to_indices.entry(key.clone()).or_default().push(idx);
            labels.push(key);
        } else {
            labels.push(format!("<anonymous#{idx}>"));
        }
    }
    for strata in ns_to_indices.values_mut() {
        strata.sort_by_key(|&i| {
            // Reason for fallback: unranked modules order after explicit staged layers.
            let r = all_stage_ranks
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(u8::MAX);
            let d = match all_modules.get(i).map(|m| m.dialect) {
                Some(StageDialect::StageL3) | None => 0u8,
                Some(StageDialect::StageR3) => 1u8,
            };
            (r, d, i)
        });
    }

    let mut local_dc_to_mod_idx: HashMap<DcChar, usize> = HashMap::new();
    let mut local_dc_sig_refs: HashMap<DcChar, Vec<DcChar>> = HashMap::new();
    for (m_idx, m) in all_modules.iter().enumerate() {
        for item in &m.items {
            if let Some(dc) = top_decl_dc_id(item) {
                local_dc_to_mod_idx.insert(dc, m_idx);
                let mut sig_refs = Vec::new();
                collect_dc_refs_in_top_decl_sig(item, &mut sig_refs);
                local_dc_sig_refs.insert(dc, sig_refs);
            }
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
        if let Some(ref key) = self_ns
            && let Some(strata) = ns_to_indices.get(key)
            && let Some(pos) = strata.iter().position(|&s| s == idx)
            && pos > 0
            && let Some(&prev_idx) = strata.get(pos.saturating_sub(1))
            && !d.contains(&prev_idx)
        {
            d.push(prev_idx);
        }

        for u in &m.uses {
            let target_key = u.path.join("/");
            let Some(cand_strata) = ns_to_indices.get(&target_key) else {
                let caller_lbl = labels
                    .get(idx)
                    .context("Missing caller module label")?;
                bail!("Unresolved 'use {target_key}:' import in module {caller_lbl}");
            };
            let is_same_ns = self_ns.as_deref() == Some(target_key.as_str());
            let self_stratum_pos = if is_same_ns {
                cand_strata.iter().position(|&s| s == idx)
            } else {
                None
            };
            let caller_rank = *all_stage_ranks
                .get(idx)
                .context("Missing caller module stage rank")?;
            let has_sl3_stratum = cand_strata.iter().any(|&ci| {
                all_modules
                    .get(ci)
                    .is_some_and(|cm| cm.dialect == StageDialect::StageL3)
            });
            let mut visible_deps = Vec::new();
            for (pos, &dep_idx) in cand_strata.iter().enumerate() {
                if let Some(self_pos) = self_stratum_pos
                    && pos >= self_pos
                {
                    continue;
                }
                let dep_rank = *all_stage_ranks
                    .get(dep_idx)
                    .context("Missing dependency module stage rank")?;
                if let (Some(dr), Some(cr)) = (dep_rank, caller_rank)
                    && dr > cr
                {
                    continue;
                }
                if m.dialect == StageDialect::StageL3
                    && has_sl3_stratum
                    && all_modules
                        .get(dep_idx)
                        .is_some_and(|dm| dm.dialect == StageDialect::StageR3)
                {
                    continue;
                }
                visible_deps.push(dep_idx);
            }
            ensure!(
                !visible_deps.is_empty(),
                "Module '{target_key}' cannot import itself"
            );
            for dep_idx in visible_deps {
                if dep_idx != idx && !d.contains(&dep_idx) {
                    d.push(dep_idx);
                }
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
                if let Some(&dep_idx) = local_dc_to_mod_idx.get(&dc) {
                    if dep_idx != idx && !d.contains(&dep_idx) {
                        d.push(dep_idx);
                    }
                } else if let Some(cand_indices) =
                    ns_to_indices.get(&owner.namespace_key)
                {
                    for &dep_idx in cand_indices {
                        if dep_idx != idx && !d.contains(&dep_idx) {
                            d.push(dep_idx);
                        }
                    }
                }
                extra_dc_map.insert(dc, owner.to_imported_symbol(dc));
            } else if let Some(&dep_idx) = local_dc_to_mod_idx.get(&dc) {
                if let Some(sig_refs) = local_dc_sig_refs.get(&dc) {
                    for &s_dc in sig_refs {
                        ref_queue.push_back(s_dc);
                    }
                }
                if dep_idx != idx && !d.contains(&dep_idx) {
                    d.push(dep_idx);
                }
            }
        }
        deps.push(d);
        module_extra_dc_imports.push(extra_dc_map);
    }

    let order = topo_sort_all_modules(&labels, &deps)?;
    Ok(AssembledGraphModules {
        all_modules,
        all_forbid_flags,
        all_input_indices,
        module_extra_dc_imports,
        order,
    })
}

fn link_parsed_modules_with_seed_graph_and_flags(
    caller_modules: Vec<CallerSpec>,
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let assembled =
        assemble_topo_sorted_modules_with_seed_graph(caller_modules, seed_graph)?;
    let mut ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut validated_by_dc: HashMap<DcChar, ImportedSymbol> = HashMap::new();
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
        let mut mod_local_dcs = HashSet::new();
        collect_locally_defined_dcs(m, &mut mod_local_dcs);
        let mut refs = Vec::new();
        collect_dc_refs_in_module(m, &mut refs);
        let mut merged_extra_dc = extra_dc.clone();
        for dc in refs {
            if !mod_local_dcs.contains(&dc)
                && !merged_extra_dc.contains_key(&dc)
                && let Some(sym) = validated_by_dc.get(&dc)
            {
                merged_extra_dc.insert(dc, sym.clone());
            }
        }
        let forbid_stagel_internal = assembled
            .all_forbid_flags
            .get(idx)
            .copied()
            .context("Invalid module forbid_stagel_internal index")?;
        let mut vm =
            validate_module_with_registry_and_dc_imports_and_options(
                m,
                &ns_exports,
                &merged_extra_dc,
                forbid_stagel_internal,
            )?;
        vm.input_index = assembled
            .all_input_indices
            .get(idx)
            .copied()
            .context("Invalid module input_index")?;
        // Reason for fallback: anonymous caller modules still contribute transitive struct/nominal definitions to `ns_exports` without colliding with `use` paths.
        let export_key = vm
            .namespace_key
            .clone()
            .unwrap_or_else(|| format!("$anon_caller_{idx}"));
        if let Some(existing) = ns_exports.get_mut(&export_key) {
            existing.merge_overlay(vm.exports.clone(), &export_key)?;
        } else {
            ns_exports.insert(export_key, vm.exports.clone());
        }
        for (&dc, sym) in &vm.exports.by_dc {
            validated_by_dc.insert(dc, sym.clone());
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
                    may_fault: expr_syntactically_contains_fault(&r.body),
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
    if let Some(existing) = pre_ns_exports.get_mut(&ns_key) {
        let _ = existing.merge_overlay(exp, &ns_key);
    } else {
        pre_ns_exports.insert(ns_key, exp);
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "Orchestrates pre-scan reconstruction, multi-stratum DAG resolution, parsing, and linking against SeedGraphIndex"
)]
fn parse_and_validate_workspace_with_seed_graph_and_dialect(
    inputs: &[ModuleInput<'_>],
    seed_graph: &SeedGraphIndex,
    default_dialect: StageDialect,
) -> Result<ValidatedWorkspace> {
    let mut headers = Vec::with_capacity(inputs.len());
    let mut caller_ns_strata: HashMap<String, Vec<usize>> = HashMap::new();
    let mut stage_ranks = Vec::with_capacity(inputs.len());

    for (idx, inp) in inputs.iter().enumerate() {
        let dialect = dialect_for_source_path(inp.source_path, default_dialect);
        let rank = stage_rank_for_source_path(inp.source_path);
        let (ns_opt, uses) =
            scan_module_header_paths_with_dialect(inp.source, dialect)?;
        let ns_key = ns_opt.as_ref().map(|segs| segs.join("/"));
        if let (Some(src_path), Some(key)) = (inp.source_path, &ns_key) {
            verify_psr4_path(src_path, key)?;
        }
        if let Some(ref key) = ns_key {
            let strata = caller_ns_strata.entry(key.clone()).or_default();
            for &prev_idx in strata.iter() {
                let prev_rank = *stage_ranks
                    .get(prev_idx)
                    .context("Missing prior stage rank")?;
                let prev_dialect = headers
                    .get(prev_idx)
                    .map(|(d, _, _)| *d)
                    .context("Missing prior dialect")?;
                ensure!(
                    can_overlay_namespace(prev_rank, prev_dialect, rank, dialect),
                    "Duplicate namespace declaration '{key}'"
                );
            }
            strata.push(idx);
        }
        stage_ranks.push(rank);
        headers.push((dialect, ns_key, uses));
    }

    for strata in caller_ns_strata.values_mut() {
        strata.sort_by_key(|&i| {
            // Reason for fallback: unranked caller modules order after explicit staged layers.
            let r = stage_ranks.get(i).copied().flatten().unwrap_or(u8::MAX);
            let d = match headers.get(i).map(|(dl, _, _)| *dl) {
                Some(StageDialect::StageL3) | None => 0u8,
                Some(StageDialect::StageR3) => 1u8,
            };
            (r, d, i)
        });
    }

    // Pre-populate `ns_exports` for external `seed_graph` namespaces so caller
    // `use <ns>: ... ::.` headers have exact verb arities and nominal/struct kinds before parsing.
    let any_stager = headers
        .iter()
        .any(|(d, _, _)| *d == StageDialect::StageR3);
    let mut pre_ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut dummy_owners: HashMap<DcChar, GraphDcOwner> = HashMap::new();
    for (_dialect, ns_key, uses) in &headers {
        // Reason for fallback: Anonymous caller modules format as '<anonymous>' in diagnostics.
        let caller_label = ns_key.as_deref().unwrap_or("<anonymous>");
        for u in uses {
            let target_key = u.join("/");
            if caller_ns_strata.contains_key(&target_key)
                || pre_ns_exports.contains_key(&target_key)
            {
                continue;
            }
            let (_, ext_exports) = reconstruct_external_namespace_modules(
                seed_graph,
                u,
                caller_label,
                any_stager,
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
        if let Some(key) = ns_key
            && let Some(strata) = caller_ns_strata.get(key)
            && let Some(pos) = strata.iter().position(|&s| s == idx)
            && pos > 0
            && let Some(&prev_idx) = strata.get(pos.saturating_sub(1))
            && !d.contains(&prev_idx)
        {
            d.push(prev_idx);
        }
        for u in uses {
            let target_key = u.join("/");
            if let Some(cand_strata) = caller_ns_strata.get(&target_key) {
                let is_same_ns = ns_key.as_deref() == Some(target_key.as_str());
                let self_pos = if is_same_ns {
                    cand_strata.iter().position(|&s| s == idx)
                } else {
                    None
                };
                let mut added = false;
                for (pos, &dep_idx) in cand_strata.iter().enumerate() {
                    if let Some(sp) = self_pos
                        && pos >= sp
                    {
                        continue;
                    }
                    if dep_idx != idx && !d.contains(&dep_idx) {
                        d.push(dep_idx);
                    }
                    added = true;
                }
                ensure!(
                    added,
                    "Module '{target_key}' cannot import itself"
                );
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
        let stage_rank = stage_ranks.get(idx).copied().flatten();
        ordered_callers.push(CallerSpec {
            module: parsed,
            forbid_stagel_internal: forbid,
            stage_rank,
            ordered_stream: false,
            input_index: idx,
        });
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
        let (_, ext_exports) = reconstruct_external_namespace_modules(
            seed_graph,
            u,
            caller_label,
            dialect == StageDialect::StageR3,
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

/// Converts StageL 3 `.sl3` source (validating, resolving `use` / `dc/<shorthand>`
/// targets, and desugaring StageL 3 overloaded std operators into direct calls)
/// into canonical StageL Token Dcs.
pub fn sl3_to_dc_tokens_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let ws = parse_and_validate_module_with_seed_graph(source, seed_graph)?;
    let module = ws
        .modules
        .last()
        .context("Missing validated caller module during StageL 3 encoding")?;
    encode_module_to_dc_tokens(&module.module)
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

/// Decodes one or more StageL/StageR Token-Dc modules (`Dc 2192`..`Dc 2193` or
/// `Dc 2221`..`Dc 2222`) and links/validates them against `seed_graph`.
pub fn validate_dc_tokens_with_seed_graph(
    tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
) -> Result<ValidatedWorkspace> {
    let modules = decode_modules_from_dc_tokens(tokens)?;
    let tagged = modules
        .into_iter()
        .enumerate()
        .map(|(input_index, module)| {
            let forbid_stagel_internal =
                module.dialect == StageDialect::StageR3;
            CallerSpec {
                module,
                forbid_stagel_internal,
                stage_rank: None,
                ordered_stream: true,
                input_index,
            }
        })
        .collect();
    link_parsed_modules_with_seed_graph_and_flags(tagged, seed_graph)
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
    crate::dc::DC_REQUEST_HOST_CAPABILITY_OP,
];

/// Synthesizes a minimal StageL 3 Token-Dc entry module (`v/l/u8/step l/u8/input`)
/// that delegates directly to `routine_dc` (`pub dc/<shorthand> v/l/u8/... l/u8/input`)
/// in `seed_graph`.
///
/// # Errors
/// Returns an error if `routine_dc` cannot be resolved in `seed_graph` or does not have
/// a single-parameter routine signature.
pub fn build_entry_module_for_graph_routine_dc(
    routine_dc: DcChar,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let sh = super::ast::format_dc_shorthand(routine_dc);
    let src = format!("v/l/u8/step l/u8/input :: dc/{sh} /input.\n");
    sl3_to_dc_tokens_with_seed_graph(&src, seed_graph)
}

/// Synthesizes a minimal StageR 3 Token-Dc entry module (`v/l/u8/step l/u8/input`)
/// that delegates directly to `routine_dc` (`pub dc/<shorthand> v/l/u8/... l/u8/input`)
/// in `seed_graph`.
///
/// # Errors
/// Returns an error if `routine_dc` cannot be resolved in `seed_graph` or fails
/// StageR 3 validation.
pub fn build_stager_bytes_entry_module_for_graph_routine_dc(
    routine_dc: DcChar,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let sh = super::ast::format_dc_shorthand(routine_dc);
    let src = format!("v/l/u8/step l/u8/input :: dc/{sh} /input.\n");
    sr3_to_dc_tokens_with_seed_graph(&src, seed_graph)
}

/// Inspects a standalone StageL 3 / StageR 3 decomposition attached to `target_sh`,
/// returning `(dialect, routine)` when `target_sh` exposes an unambiguous public
/// routine declaration.
///
/// # Errors
/// Returns an error if reading or decoding the decomposition tokens fails.
pub(crate) fn inspect_standalone_graph_routine_signature(
    seed_graph: &SeedGraphIndex,
    target_sh: DcShorthand,
) -> Result<Option<(StageDialect, RoutineDecl)>> {
    let Ok(target_gid) = target_sh.to_global_id() else {
        return Ok(None);
    };
    let target_dc = DcChar::from_u128(target_gid);
    let Some((_, entry)) =
        decode_graph_stagel_entry_at_line(seed_graph, target_sh, 1)?
    else {
        return Ok(None);
    };
    let (mod_dialect, routine_opt) = match entry {
        EitherDeclOrModule::Decl(TopDecl::Routine(r)) => {
            if r.is_pub && (r.dc_id == Some(target_dc) || r.dc_id.is_none()) {
                (StageDialect::StageL3, Some(r))
            } else {
                (StageDialect::StageL3, None)
            }
        }
        EitherDeclOrModule::Decl(_) => (StageDialect::StageL3, None),
        EitherDeclOrModule::Module(m) => {
            let mod_dialect = m.dialect;
            let mut explicit_matches = Vec::new();
            let mut non_comment_count = 0usize;
            let mut single_unnamed_pub = None;
            for item in m.items {
                if matches!(item, TopDecl::Comment(_)) {
                    continue;
                }
                non_comment_count = non_comment_count.saturating_add(1);
                if let TopDecl::Routine(r) = item {
                    if r.is_pub && r.dc_id == Some(target_dc) {
                        explicit_matches.push(r);
                    } else if r.is_pub && r.dc_id.is_none() {
                        single_unnamed_pub = Some(r);
                    }
                }
            }
            let picked = if explicit_matches.len() == 1 {
                explicit_matches.pop()
            } else if explicit_matches.is_empty() && non_comment_count == 1 {
                single_unnamed_pub
            } else {
                None
            };
            (mod_dialect, picked)
        }
    };
    let Some(routine) = routine_opt else {
        return Ok(None);
    };
    let uses_stager = mod_dialect == StageDialect::StageR3
        || top_decl_uses_stager(&TopDecl::Routine(routine.clone()));
    let dialect = if uses_stager {
        StageDialect::StageR3
    } else {
        StageDialect::StageL3
    };
    Ok(Some((dialect, routine)))
}

/// Synthesizes a minimal StageR 3 Token-Dc entry module (`v/l/u8/step
/// l/u8/input`) that decodes DcTF bytes into `dcs` and delegates to
/// `routine_dc` (`pub dc/<shorthand> v/dcs/... dcs/input`) in `seed_graph`.
///
/// # Errors
/// Returns an error if `routine_dc` cannot be resolved in `seed_graph` or fails
/// StageR 3 validation.
pub fn build_stager_dcs_entry_module_for_graph_routine_dc(
    routine_dc: DcChar,
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let sh = super::ast::format_dc_shorthand(routine_dc);
    let src = format!(
        concat!(
            "use std/dcs: decodeDcsDctf encodeDcsDctf ::.\n",
            "v/l/u8/step l/u8/input\n",
            "    match decodeDcsDctf /input\n",
            "        case ok dcs/d encodeDcsDctf dc/{sh} /d.\n",
            "        case err error/_e l/u8: ::.\n",
            "    ::.\n",
        ),
        sh = sh,
    );
    sr3_to_dc_tokens_with_seed_graph(&src, seed_graph)
}

/// Prepends topologically sorted transitive `use` / `dc/<shorthand>` StageL or StageR
/// Token-Dc dependency modules from `seed_graph` ahead of `entry_tokens`, without
/// prepending the Stage 0 `.WASM` operator KV map.
///
/// # Errors
/// Returns an error if a transitive dependency in `seed_graph` fails to resolve or encode.
pub fn bundle_dc_module_dependencies(
    entry_tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
) -> Result<Vec<DcChar>> {
    let mut module_stream: Vec<DcChar> = Vec::new();
    if let Ok(caller_mods) = decode_modules_from_dc_tokens(entry_tokens) {
        let single_caller = caller_mods.len() == 1;
        let tagged: Vec<CallerSpec> = caller_mods
            .into_iter()
            .enumerate()
            .map(|(input_index, module)| CallerSpec {
                module,
                forbid_stagel_internal: false,
                stage_rank: None,
                ordered_stream: true,
                input_index,
            })
            .collect();
        let assembled =
            assemble_topo_sorted_modules_with_seed_graph(tagged, seed_graph)?;
        let entry_idx = assembled.all_modules.len().saturating_sub(1);
        for &idx in &assembled.order {
            if single_caller && idx == entry_idx {
                continue;
            }
            let dep_mod = assembled
                .all_modules
                .get(idx)
                .context("Invalid dependency module index")?;
            module_stream.extend(encode_module_to_dc_tokens(dep_mod)?);
        }
        if single_caller {
            module_stream.extend_from_slice(entry_tokens);
        }
    } else {
        // Malformed Token-Dc input is forwarded directly so the downstream converter
        // traps under the active task quota rather than failing in host Rust.
        module_stream.extend_from_slice(entry_tokens);
    }
    Ok(module_stream)
}

/// Bundles an entry StageL 3 Token-Dc module (`Dc 2192`..`Dc 2193`) with its
/// transitive `use` / `dc/<shorthand>` StageL Token-Dc dependency modules and
/// active contextual Stage 0 `.WASM` built-ins from `seed_graph` so the pure
/// linear-memory `f1680_stagel3_to_wasm.wat` transformation can compile the
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

    let module_stream = bundle_dc_module_dependencies(entry_tokens, seed_graph)?;

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

    struct Stager3VmCompiler {
        vm: super::super::eval::StageLWorkspaceVm,
        mod_idx: usize,
        eval_cfg: super::super::eval::EvalConfig,
    }

    impl Stager3VmCompiler {
        fn new(seed_graph: &SeedGraphIndex) -> Result<Self> {
            Self::new_with_quota(seed_graph, 50_000_000)
        }

        fn new_with_quota(
            seed_graph: &SeedGraphIndex,
            compute_quota: u64,
        ) -> Result<Self> {
            use super::super::eval::{EvalConfig, StageLWorkspaceVm};

            let compiler_src = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data-src/stage_1_impls/std/stager3_to_stagel3.sl3"
            ))?;
            let compiler_ws = parse_and_validate_module_with_seed_graph(
                &compiler_src,
                seed_graph,
            )?;
            let mod_idx = compiler_ws
                .modules
                .iter()
                .position(|m| {
                    m.exports.routines.contains_key("stager3ToStagel3")
                })
                .context("Expected stager3ToStagel3 module in workspace")?;
            let eval_cfg = EvalConfig {
                compute_quota,
                max_call_depth: 50,
                max_bigint_bits: 1_048_576,
                max_seq_elems: 4_194_304,
            };
            let vm = StageLWorkspaceVm::from_validated_workspace(
                &compiler_ws,
                eval_cfg,
            )?;
            Ok(Self {
                vm,
                mod_idx,
                eval_cfg,
            })
        }

        fn run_raw(
            &self,
            in_tokens: &[DcChar],
        ) -> Result<super::super::eval::RoutineOutcome> {
            use std::rc::Rc;

            use super::super::eval::Value;

            let mut in_bytes = Vec::with_capacity(in_tokens.len() * 4);
            for t in in_tokens {
                in_bytes.extend_from_slice(&ctb_formats_dctf::encode_dctf(t.0));
            }
            let arg = Value::List(Rc::new(
                in_bytes.into_iter().map(Value::U8).collect(),
            ));
            self.vm
                .call_routine(self.mod_idx, "stager3ToStagel3", vec![arg])
        }

        fn decode_tokens(bytes: &[u8]) -> Result<Vec<DcChar>> {
            let mut out = Vec::new();
            let mut pos = 0usize;
            while pos < bytes.len() {
                let (cp, used) = ctb_formats_dctf::decode_dctf_buf(&bytes[pos..])
                    .with_context(|| format!("Invalid DcTF at byte {pos}"))?;
                out.push(DcChar(cp));
                pos += used;
            }
            Ok(out)
        }

        fn compile_tokens(&self, in_tokens: &[DcChar]) -> Result<Vec<DcChar>> {
            use super::super::eval::{RoutineOutcome, Value};

            let out = self.run_raw(in_tokens)?;
            let out_val = match out {
                RoutineOutcome::Returned(v) => v,
                RoutineOutcome::Trapped(msg) => {
                    bail!("stager3ToStagel3 trapped: {msg}");
                }
            };
            let Value::List(out_list) = out_val else {
                bail!(
                    "Expected List(u8) from stager3ToStagel3, got {out_val:?}"
                );
            };
            let out_bytes: Vec<u8> = out_list
                .iter()
                .map(|v| match v {
                    Value::U8(b) => Ok(*b),
                    other => bail!("Expected U8 element, got {other:?}"),
                })
                .collect::<Result<_>>()?;
            Self::decode_tokens(&out_bytes)
        }
    }

    #[crate::ctb_test]
    fn test_stager3_to_stagel3_vm_parity_suite() -> Result<()> {
        let handle = std::thread::Builder::new()
            .name(String::from("sr3_to_sl3_parity"))
            .stack_size(64 * 1024 * 1024)
            .spawn(|| -> Result<()> {
                use std::rc::Rc;

                use super::super::dc_tokens::encode_module_to_dc_tokens;
                use super::super::eval::{
                    RoutineOutcome, StageLWorkspaceVm, Value,
                };
                use super::super::parser::parse_sr3_module;

                let seed_graph = SeedGraphIndex::shared()?;
                let compiler = Stager3VmCompiler::new(&seed_graph)?;
                let eval_cfg = compiler.eval_cfg;
                let run_compiler =
                    |in_tokens: &[DcChar]| compiler.run_raw(in_tokens);

                let verify_workspace = |label: &str,
                                        inputs: &[ModuleInput<'_>],
                                        calls: &[(&str, Vec<Value>)]|
                 -> Result<()> {
                    let sr3_ws =
                        parse_and_validate_stager_workspace_with_seed_graph(
                            inputs,
                            &seed_graph,
                        )
                        .with_context(|| {
                            format!("[{label}] validating StageR workspace")
                        })?;
                    let sr3_vm = StageLWorkspaceVm::from_validated_workspace(
                        &sr3_ws, eval_cfg,
                    )
                    .with_context(|| format!("[{label}] building StageR VM"))?;
                    let sr3_rep = sr3_vm.run_all_tests(&sr3_ws)?;
                    sr3_rep.ensure_all_passed().with_context(|| {
                        format!("[{label}] StageR inline tests failed")
                    })?;

                    let mut raw_tokens = Vec::new();
                    for vm in &sr3_ws.modules {
                        raw_tokens
                            .extend(encode_module_to_dc_tokens(&vm.module)?);
                    }
                    let bundled_tokens =
                        bundle_dc_module_dependencies(&raw_tokens, &seed_graph)?;
                    let sl3_tokens = compiler
                        .compile_tokens(&bundled_tokens)
                        .with_context(|| {
                            format!("[{label}] running stager3ToStagel3")
                        })?;
                    let sl3_ws = validate_dc_tokens_with_seed_graph(
                        &sl3_tokens,
                        &seed_graph,
                    )
                    .with_context(|| {
                        format!(
                            "[{label}] validating lowered StageL 3 Token-Dc stream"
                        )
                    })?;
                    let sl3_vm = StageLWorkspaceVm::from_validated_workspace(
                        &sl3_ws, eval_cfg,
                    )
                    .with_context(|| {
                        format!("[{label}] building lowered StageL VM")
                    })?;
                    let sl3_rep = sl3_vm.run_all_tests(&sl3_ws)?;
                    sl3_rep.ensure_all_passed().with_context(|| {
                        format!(
                            "[{label}] Lowered StageL 3 inline tests failed"
                        )
                    })?;

                    let sr3_entry = sr3_ws.modules.len() - 1;
                    let sl3_entry = sl3_ws.modules.len() - 1;
                    for (fn_name, args) in calls {
                        let expected = sr3_vm.call_routine(
                            sr3_entry,
                            fn_name,
                            args.clone(),
                        )?;
                        let actual = sl3_vm.call_routine(
                            sl3_entry,
                            fn_name,
                            args.clone(),
                        )?;
                        assert_eq!(
                            actual, expected,
                            "[{label}] mismatch calling '{fn_name}' with {args:?}"
                        );
                    }
                    Ok(())
                };

                // 1. Pipeline transform test (`brand`, named `struct`, `enum`, `?`, `match`, `okOr`, `unwrapOr`)
                let sr3_transform_src = r#"ns app/stager_pipe.

brand u32/tag.

struct boxPair
    tag/id.
    i32/delta.

enum op
    keep.
    i32/shift.

v/result/i32/applyOp boxPair/bp op/act
    const u32/raw unwrap field /bp id.
    const i32/base conv? /raw i32.
    const i32/sum add? /base field /bp delta.
    match /act
        case keep result/i32:ok /sum.
        case shift i32/s add /sum /s.
    ::.

pub v/l/u8/step l/u8/input
    test
        in b"hi"
        out b"<hi>".
    const boxPair/bp structNew boxPair:
        delta i32:-5
        id wrap u32:40 tag
    ::.
    const result/i32/res applyOp /bp op:shift i32:7.
    if isOk /res
        do
            const i32/v unwrapOr okOr get l/i32: i32:42 :: u64:0 error:unexpectedNone i32:0.
            if and eq /res result/i32:ok i32:42 eq /v i32:42
                concat b"<" concat /input b">"
                b"bad"
        ::
        b"err".
"#;
                verify_workspace(
                    "sr3_transform",
                    &[ModuleInput {
                        source_path: Some("app/stager_pipe.sr3"),
                        source: sr3_transform_src,
                    }],
                    &[(
                        "step",
                        vec![Value::List(Rc::new(vec![
                            Value::U8(b'A'),
                            Value::U8(b'!'),
                        ]))],
                    )],
                )?;

                // 2. Anonymous StageR 3 module + tail-recursive `match` (depth 200 > max_call_depth 50)
                let anon_tail_src = r#"
enum counter
    done.
    u64/more.

v/u64/sumDown counter/c u64/acc
    match /c
        case done /acc.
        case more u64/n
            if eq /n u64:0
                sumDown counter:done /acc
                sumDown counter:more wrapSub /n u64:1 wrapAdd /acc /n.
    ::.

v/u64/runTail u64/n
    test
        in u64:200
        out u64:20100.
    sumDown counter:more /n u64:0.
"#;
                verify_workspace(
                    "anon_tail_match",
                    &[ModuleInput {
                        source_path: None,
                        source: anon_tail_src,
                    }],
                    &[("runTail", vec![Value::U64(200)])],
                )?;

                // 3. Multi-module StageR 3: transparent `brand` vs opaque `type`, `struct`, `enum`, overloaded `pub` verbs, `std/math` import
                let mod_defs = r#"ns demo/defs.
use std/math: u8Max ::.

pub brand u64/meter.
pub type u64/secretToken.

pub struct point
    meter/x.
    meter/y.

pub enum action
    idle.
    i32/stepBy.
    i32/jumpBy.
    u64/resetTo.

pub meter/unitMeter wrap u64:1 meter.

pub v/result/secretToken/mintToken u64/raw
    if eq /raw u64:0
        result/secretToken:err error:outOfBounds
        result/secretToken:ok wrap wrapAdd /raw conv /u8Max u64 secretToken.

pub v/u64/revealToken secretToken/tok
    unwrap /tok.

pub v/i64/describe i32/x
    conv /x i64.

pub v/i64/describe u64/x
    unwrapOr conv /x i64 i64:-1.
"#;
                let mod_user = r#"ns demo/user.
use demo/defs: meter secretToken point action unitMeter mintToken revealToken describe ::.

v/i64/evalAction point/p action/a
    const u64/sumM wrapAdd unwrap field /p x unwrap field /p y.
    match /a
        case idle describe /sumM.
        case stepBy i32/dx wrapAdd describe /sumM describe /dx.
        case jumpBy i32/jx wrapSub describe /sumM describe /jx.
        else i64:999.
    ::.

pub v/result/i64/runMulti u64/seed
    test
        in u64:10
        out result/i64:ok i64:1278.
        in u64:0
        out result/i64:err error:outOfBounds.
    const secretToken/t mintToken? /seed.
    const secretToken/t2 mintToken? /seed.
    if not eq /t /t2
        return result/i64:err error:unexpectedNone
        nop.
    const point/pt structNew point: y wrap u64:7 meter x /unitMeter ::.
    const i64/a1 evalAction /pt action:stepBy i32:3.
    const i64/a2 evalAction /pt action:jumpBy i32:5.
    const i64/a3 evalAction /pt action:resetTo u64:99.
    const i64/tokVal conv? revealToken /t i64.
    result/i64:ok wrapAdd wrapAdd /a1 /a2 wrapAdd /a3 /tokVal.
"#;
                verify_workspace(
                    "multi_module_nominal_overload",
                    &[
                        ModuleInput {
                            source_path: Some("demo/defs.sr3"),
                            source: mod_defs,
                        },
                        ModuleInput {
                            source_path: Some("demo/user.sr3"),
                            source: mod_user,
                        },
                    ],
                    &[],
                )?;

                // 4. All 4 `match` domains (`option`, `result`, `error`, `enum`) + `?` (`option`, `result`, `result/void`) + `bufSet`
                let flow_and_buf_src = r#"ns demo/flow.

v/option/i32/doubleOpt option/i32/mx
    const i32/x /mx?.
    option/i32:some unwrapOr mul /x i32:2 i32:0.

v/result/l/i32/buildTwo u64/idx i32/val
    let buf/i32/b bufFill i32:0 u64:3.
    bufSet? /b /idx /val.
    bufSet? /b wrapAdd /idx u64:1 wrapAdd /val i32:10.
    result/l/i32:ok bufFreeze /b.

v/result/void/checkWriteBounds u64/idx
    let buf/i32/b bufFill i32:0 u64:3.
    bufSet? /b /idx i32:9.
    result/void:ok nop.

v/i32/codeForErr error/e
    match /e
        case divisionByZero i32:10.
        case overflow i32:20.
        case outOfBounds i32:30.
        else i32:40.
    ::.

pub v/b/checkFlow
    test
        in.
        out true.
    const option/i32/o1 doubleOpt option/i32:some i32:21.
    const option/i32/o2 doubleOpt option/i32:none.
    const i32/m1 match /o1
        case some i32/v /v.
        case none i32:-1.
    ::.
    const i32/m2 match /o2
        case some i32/v /v.
        else i32:77.
    ::.
    const result/l/i32/wOk buildTwo u64:0 i32:5.
    const result/void/wBad checkWriteBounds u64:3.
    const l/i32/frozen unwrapOr /wOk l/i32: ::.
    const i32/wErrCode match /wBad
        case ok i32:0.
        case err error/e codeForErr /e.
    ::.
    const option/l/i32/slOk slice /frozen u64:0 u64:2.
    const option/l/i32/slBad slice /frozen u64:2 u64:5.
    and eq /m1 i32:42
        and eq /m2 i32:77
            and isOk /wOk
                and eq /wErrCode i32:30
                    and eq /slOk option/l/i32:some l/i32: i32:5 i32:15 ::
                        isNone /slBad.
"#;
                verify_workspace(
                    "flow_match_and_buf",
                    &[ModuleInput {
                        source_path: Some("demo/flow.sr3"),
                        source: flow_and_buf_src,
                    }],
                    &[("checkFlow", vec![])],
                )?;

                // 5. Numeric suite: `i128`, `u128`, `int`, `rat`, total arithmetic, `truncRem` vs `mod`, shifts, bitops, `conv`
                let numeric_src = r#"ns demo/nums.

pub v/b/checkWideInts
    test
        in.
        out true.
    const i128/a i128:-170141183460469231731687303715884105728.
    const result/i128/negMin neg /a.
    const result/i128/divMin div /a i128:-1.
    const result/i128/modMin mod /a i128:-1.
    const result/i128/remNeg truncRem i128:-7 i128:3.
    const result/i128/modNeg mod i128:-7 i128:3.
    const u128/uMax u128:340282366920938463463374607431768211455.
    const result/u128/uOv add /uMax u128:1.
    const result/u128/shBad shl u128:1 u32:128.
    const result/u128/shOk shl u128:1 u32:100.
    const u32/pc popcnt /uMax.
    and isErr /negMin
        and isErr /divMin
            and eq /modMin result/i128:ok i128:0
                and eq /remNeg result/i128:ok i128:-1
                    and eq /modNeg result/i128:ok i128:2
                        and isErr /uOv
                            and isErr /shBad
                                and isOk /shOk
                                    eq /pc u32:128.

pub v/result/b/checkBigIntAndRat
    test
        in.
        out result/b:ok true.
    const int/big mul int:1000000000000000000000000000000000000000 int:-3.
    const result/int/q div /big int:1000000000000000000000000000000000000000.
    const result/int/em mod int:-7 int:-3.
    const result/int/tr truncRem int:-7 int:-3.
    const rat/r1 makeRat? int:-6 int:8.
    const rat/r2 makeRat? int:1 int:4.
    const rat/rSum add /r1 /r2.
    const rat/rWhole mul /rSum makeRat? int:-4 int:1.
    const result/int/cBad conv /rSum int.
    const result/int/cGood conv /rWhole int.
    const result/rat/rDiv0 div /r1 makeRat? int:0 int:1.
    const rat/wide conv int:-42 rat.
    result/b:ok
        and eq /q result/int:ok int:-3
            and eq /em result/int:ok int:2
                and eq /tr result/int:ok int:-1
                    and eq /rSum makeRat? int:-1 int:2
                        and isErr /cBad
                            and eq /cGood result/int:ok int:2
                                and eq /wide makeRat? int:-42 int:1
                                    isErr /rDiv0.
"#;
                verify_workspace(
                    "numeric_wide_bigint_rat",
                    &[ModuleInput {
                        source_path: Some("demo/nums.sr3"),
                        source: numeric_src,
                    }],
                    &[("checkWideInts", vec![])],
                )?;

                // 6. Document Character & string suite: `dcFromU128`, `u128FromDc`, `dcsFromS`, `sFromDcs`, `byteLen` (including embedded Base64 binary runs), `explode`/`implode` (`s`, `dcs`, `l/i32`)
                let dc_strings_src = r#"ns demo/dcstrings.

pub v/result/b/checkDcAndStrings
    test
        in.
        out result/b:ok true.
    const dc/cu100 dcFromU128 u128:256.
    const dc/c256 dcFromU128 u128:1114368.
    const dc/csur dcFromU128 u128:55296.
    const b/okDc and eq /cu100 dc:u100 and eq /c256 dc:256 and eq u128FromDc /c256 u128:1114368 eq u128FromDc /csur u128:55296.
    const b/okSurrLit and eq /csur dc:ud800 eq len dcsFromS? "@ud800@" u64:1.

    const dcs/pure dcsFromS? "hi @256@".
    const b/okLen and eq len /pure u64:4 eq get /pure u64:3 option/dc:some dc:256.
    const b/okOob and eq get /pure u64:9 option/dc:none eq slice /pure u64:3 u64:1 option/dcs:none.
    const dcs/sl slice? /pure u64:1 u64:3.
    const s/back sFromDcs /sl.
    const b/okRound eq /back "i ".

    const dcs/dsub dcsFromS? "a:b:c".
    const dcs/dcolon dcsFromS? ":".
    const dcs/ddash dcsFromS? "-".
    const l/dcs/dparts explode /dcolon /dsub.
    const dcs/drejoined implode /ddash /dparts.
    const b/okDcsSplit eq sFromDcs /drejoined "a-b-c".

    const s/withBin "pre:@203@@127@@127@@127@@127@@204@:post".
    const b/binRejected eq dcsFromS /withBin result/dcs:err error:invalidEncoding.
    const u64/blen byteLen /withBin.
    const l/s/parts explode ":" /withBin.
    const s/rejoined implode ":" /parts.
    const l/s/chars explode "" "a@256@@203@@127@@127@@127@@127@@204@".
    const b/okBinSplit and eq /rejoined /withBin and eq len /parts u64:3 and eq len /chars u64:3 eq get /parts u64:1 option/s:some "@203@@127@@127@@127@@127@@204@".

    const l/l/i32/lparts explode l/i32: i32:0 :: l/i32: i32:1 i32:0 i32:2 i32:3 ::.
    const l/i32/ljoined implode l/i32: i32:9 :: /lparts.
    const l/l/i32/charsSplit explode l/i32: :: l/i32: i32:7 i32:8 ::.
    const b/okListSplit and eq /ljoined l/i32: i32:1 i32:9 i32:2 i32:3 :: eq len /charsSplit u64:2.

    result/b:ok and /okDc and /okSurrLit and /okLen and /okOob and /okRound and /okDcsSplit and /binRejected and gt /blen u64:24 and /okBinSplit /okListSplit.
"#;
                verify_workspace(
                    "dc_and_strings",
                    &[ModuleInput {
                        source_path: Some("demo/dcstrings.sr3"),
                        source: dc_strings_src,
                    }],
                    &[],
                )?;

                // 7. Negative compile-time rejection tests in `stager3ToStagel3`:
                // 7a. `pub` item in anonymous StageR 3 module (no `ns`) must trap!
                let bad_anon_pub = sr3_to_dc_tokens_with_seed_graph(
                    "pub v/i32/bad\n    i32:1.\n",
                    &seed_graph,
                )?;
                assert!(
                    matches!(
                        run_compiler(&bad_anon_pub)?,
                        RoutineOutcome::Trapped(_)
                    ),
                    "Expected anonymous StageR 'pub' item to trap in stager3ToStagel3"
                );

                // 7b. Non-exhaustive `enum` match without `else` must trap!
                let bad_non_exhaustive = sr3_to_dc_tokens_with_seed_graph(
                    concat!(
                        "ns bad/m.\n",
                        "enum color\n    red.\n    blue.\n\n",
                        "pub v/i32/f color/c\n",
                        "    match /c\n        case red i32:1.\n    ::.\n",
                    ),
                    &seed_graph,
                )?;
                assert!(
                    matches!(
                        run_compiler(&bad_non_exhaustive)?,
                        RoutineOutcome::Trapped(_)
                    ),
                    "Expected non-exhaustive enum match to trap in stager3ToStagel3"
                );

                // 7c. Cross-module `unwrap` on opaque `type` must trap!
                let opq_def = parse_sr3_module(
                    "ns bad/opq.\npub type u64/tok.\npub v/tok/mk u64/x\n    wrap /x tok.\n",
                )?;
                let mut import_ctx = HashMap::new();
                let mut exp = NamespaceExports::default();
                exp.brand_types.insert(
                    String::from("tok"),
                    BrandTypeDef {
                        namespace: Some(String::from("bad/opq")),
                        original_name: String::from("tok"),
                        is_opaque: true,
                        inner_ty: StageLType::U64,
                        is_pub: true,
                        dc_id: None,
                        span: super::super::ast::Span { line: 1, col: 1 },
                    },
                );
                exp.routines.insert(
                    String::from("mk"),
                    vec![RoutineSig {
                        return_type: StageLType::Struct(
                            super::super::ast::StructTypeRef::Ident(
                                String::from("tok"),
                            ),
                        ),
                        params: vec![StageLType::U64],
                        is_pub: true,
                        dc_id: None,
                        may_fault: false,
                    }],
                );
                import_ctx.insert(String::from("bad/opq"), exp);
                let sr_import_ctx = build_stager_import_context(
                    "ns bad/caller.\nuse bad/opq: tok mk ::.\npub v/u64/steal\n    unwrap mk u64:5.\n",
                    &import_ctx,
                    &HashMap::new(),
                )?;
                let opq_caller = parse_sr3_module_with_imports(
                    "ns bad/caller.\nuse bad/opq: tok mk ::.\npub v/u64/steal\n    unwrap mk u64:5.\n",
                    &sr_import_ctx,
                )?;
                let mut bad_opq_tokens = encode_module_to_dc_tokens(&opq_def)?;
                bad_opq_tokens
                    .extend(encode_module_to_dc_tokens(&opq_caller)?);
                assert!(
                    matches!(
                        run_compiler(&bad_opq_tokens)?,
                        RoutineOutcome::Trapped(_)
                    ),
                    "Expected cross-module unwrap on opaque 'type' to trap in stager3ToStagel3"
                );

                // 7d. `buf/T` parameter in StageR 3 routine must trap!
                let bad_buf_param_mod = parse_sr3_module(
                    "ns bad/bufp.\nv/void/bad buf/i32/b\n    nop.\n",
                )?;
                let bad_buf_tokens =
                    encode_module_to_dc_tokens(&bad_buf_param_mod)?;
                assert!(
                    matches!(
                        run_compiler(&bad_buf_tokens)?,
                        RoutineOutcome::Trapped(_)
                    ),
                    "Expected StageR routine 'buf/T' parameter to trap in stager3ToStagel3"
                );

                Ok(())
            })
            .context("Failed to spawn sr3_to_sl3_parity thread")?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("sr3_to_sl3_parity thread panicked"))?
    }

    #[crate::ctb_test]
    fn test_stager3_std_math_overloads_carrier_bridging_and_may_fault()
    -> Result<()> {
        let handle = std::thread::Builder::new()
            .name(String::from("sr3_std_overloads"))
            .stack_size(64 * 1024 * 1024)
            .spawn(|| -> Result<()> {
                use super::super::eval::{RoutineOutcome, StageLWorkspaceVm};

                let seed_graph = SeedGraphIndex::shared()?;
                let compiler = Stager3VmCompiler::new(&seed_graph)?;
                let eval_cfg = compiler.eval_cfg;

                // 1. StageR 3 importing unified `std/math`, `std/u128`, `std/i128`, `std/bigint`
                //    overloads + StageR built-in `add` + aliased total `add as totalAdd`.
                let sr3_src = r#"ns demo/std_math_sr.
use std/math: add sub mul neg canAdd canSub canMul canDiv canNeg wrapAdd wrapNeg bitAnd rotl clz eq lt i128Min u128Max ::.
use std/math: add as totalAdd ::.
use std/u128: from as u128From ::.
use std/i128: from as i128From ::.
use std/bigint: abs gcd ::.

pub v/result/b/checkStdMath
    test
        in.
        out result/b:ok true.
    const i128/negMin /i128Min.
    const u128/uMax /u128Max.
    const b/okCan and canAdd int:5 int:7
        and not canAdd /negMin i128:-1
        and not canAdd /uMax u128:1
        and not canNeg /negMin
        and canNeg int:-5
        canDiv makeRat? int:1 int:2 makeRat? int:3 int:4.
    const b/okWrap and eq wrapNeg /negMin /negMin eq wrapAdd u8:250 u8:10 u8:4.
    const b/okBit and eq bitAnd i128:15 i128:7 i128:7
        and eq rotl u128:1 u32:64 u128:18446744073709551616
        eq clz u128:1 u32:127.
    const b/okTotal and eq totalAdd int:5 int:7 int:12
        eq totalAdd makeRat? int:1 int:2 makeRat? int:1 int:3 makeRat? int:5 int:6.
    const b/okConv and eq u128From u64:42 u128:42
        and eq i128From i32:-7 i128:-7
        and eq gcd int:-12 int:18 int:6
        eq abs int:-9 int:9.
    const i128/biSum add? i128:5 i128:7.
    const b/okRel and lt /negMin i128:0 and eq /biSum i128:12 eq eq i32:3 i32:3 true.
    result/b:ok and /okCan and /okWrap and /okBit and /okTotal and /okConv /okRel.
"#;
                let sr3_ws = parse_and_validate_stager_workspace_with_seed_graph(
                    &[ModuleInput {
                        source_path: Some("demo/std_math_sr.sr3"),
                        source: sr3_src,
                    }],
                    &seed_graph,
                )?;
                let sr3_vm = StageLWorkspaceVm::from_validated_workspace(
                    &sr3_ws,
                    eval_cfg,
                )?;
                assert_eq!(sr3_vm.run_all_tests(&sr3_ws)?.ensure_all_passed()?, 1);

                let sr3_tokens =
                    sr3_to_dc_tokens_with_seed_graph(sr3_src, &seed_graph)?;
                let bundled_sr3 =
                    bundle_dc_module_dependencies(&sr3_tokens, &seed_graph)?;
                let sl3_tokens = compiler.compile_tokens(&bundled_sr3)?;
                let sl3_ws =
                    validate_dc_tokens_with_seed_graph(&sl3_tokens, &seed_graph)?;
                let sl3_vm = StageLWorkspaceVm::from_validated_workspace(
                    &sl3_ws,
                    eval_cfg,
                )?;
                assert_eq!(sl3_vm.run_all_tests(&sl3_ws)?.ensure_all_passed()?, 1);

                // 2. Negative totality tests in BOTH host validator AND self-hosted `stager3ToStagel3`:
                for (label, bad_src) in [
                    (
                        "StageL faulting add(i32, i32) via alias",
                        "ns bad/sl_add_i32.\nuse std/math: add as slAdd ::.\npub v/i32/boom\n    slAdd i32:1 i32:2.\n",
                    ),
                    (
                        "StageL faulting add(i128, i128) via alias",
                        "ns bad/sl_add_i128.\nuse std/math: add as slAdd ::.\npub v/i128/boom\n    slAdd i128:1 i128:2.\n",
                    ),
                    (
                        "StageL faulting dcsFromS(s) via alias",
                        "ns bad/sl_dc.\nuse std/dcmst: dcsFromS as slDcsFromS ::.\npub v/dcs/boom\n    conv slDcsFromS \"x\" dcs.\n",
                    ),
                ] {
                    assert!(
                        parse_and_validate_sr3_module_with_seed_graph(
                            bad_src,
                            &seed_graph,
                        )
                        .is_err(),
                        "[{label}] Host validator must reject StageL-faulting call in StageR 3"
                    );
                    let bad_tokens =
                        sr3_to_dc_tokens_with_seed_graph(bad_src, &seed_graph)?;
                    let bundled_bad =
                        bundle_dc_module_dependencies(&bad_tokens, &seed_graph)?;
                    assert!(
                        matches!(
                            compiler.run_raw(&bundled_bad)?,
                            RoutineOutcome::Trapped(_)
                        ),
                        "[{label}] Self-hosted stager3ToStagel3 must trap on StageL-faulting call"
                    );
                }

                // 3. Stage 2 `impls/` module must also reject StageL-faulting stdlib routines.
                assert!(
                    parse_and_validate_workspace_with_seed_graph(
                        &[ModuleInput {
                            source_path: Some("impls/app/bad_fault.sl3"),
                            source: concat!(
                                "use std/u128: u128Val u128One add ::.\n",
                                "pub v/u128Val/boom :: add /u128One /u128One.\n",
                            ),
                        }],
                        &seed_graph,
                    )
                    .is_err(),
                    "Stage 2 impls/ must reject faulting std/u128::add"
                );

                // 4. StageL 3 direct overloaded operator syntax across `std/{u128,i128,bigint,rat,dcs,dcmst,math}`.
                let sl3_src = r#"ns demo/std_math_sl3.
use std/u128: u128Val u128New from as u128From ::.
use std/i128: i128Val from as i128From ::.
use std/bigint: bigInt from as intFrom abs ::.
use std/rat: ratVal from as ratFrom makeRat ::.
use std/dctf: dcVal ::.
use std/dcs: dcFromU128 u128FromDc explode as dcsExplode implode as dcsImplode ::.
use std/dcmst: byteLen dcsFromS sFromDcs explode as sExplode implode as sImplode ::.
use std/math: add sub mul div rem mod truncRem wrapAdd wrapNeg bitAnd rotl shl clz lt neg ::.

pub v/b/checkStageLOverloads
    test
        in.
        out true.
    const u128Val/ua add u128From u64:10 u128From u64:32.
    const u128Val/ur rem div u128From u64:43 u128From u64:5 u128From u64:3.
    const u128Val/ub rotl shl u128From u64:1 u32:2 u32:62.
    const i128Val/ia neg sub i128From i32:10 i128From i32:17.
    const i128Val/im mod truncRem /ia i128From i32:5 i128From i32:4.
    const bigInt/bi mul intFrom i32:-6 intFrom i32:7.
    const bigInt/bq add div /bi intFrom i32:2 intFrom i32:100.
    const ratVal/ra makeRat intFrom i32:6 intFrom i32:4.
    const ratVal/rb div sub mul /ra ratFrom i32:2 ratFrom i32:1 ratFrom i32:2.
    const dcVal/ch dcFromU128 u128From u64:65.
    const l/dcVal/colon l/dcVal: dcFromU128 u128From u64:58 ::.
    const l/dcVal/joined dcsImplode /colon dcsExplode /colon l/dcVal: /ch /ch ::.
    const l/dcVal/pure dcsFromS sFromDcs /joined.
    and eq /ua u128From u64:42
        and eq /ur u128From u64:2
            and eq /ub u128New u64:0 u64:1
                and eq clz /ua u32:122
                    and eq wrapNeg i128From i32:5 i128From i32:-5
                        and lt /im i128From i32:4
                            and eq abs /bi intFrom i32:42
                                and eq /bq intFrom i32:79
                                    and eq /rb ratFrom i32:1
                                        and eq u128FromDc /ch u128From u64:65
                                            and eq byteLen sImplode /colon sExplode /colon /pure len /pure
                                                eq bitAnd u128From u64:15 u128From u64:6 u128From u64:6.
"#;
                let sl3_rep =
                    eval_sl3_inline_tests_with_seed_graph(sl3_src, &seed_graph)?;
                assert_eq!(sl3_rep.ensure_all_passed()?, 1);
                Ok(())
            })
            .context("Failed to spawn sr3_std_overloads thread")?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("sr3_std_overloads thread panicked"))?
    }

    #[crate::ctb_test]
    fn test_stage2_overlays_surrogate_dcs_and_range_errors() -> Result<()> {
        let handle = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || -> Result<()> {
                use super::super::eval::{
                    EvalConfig, RoutineOutcome, StageLWorkspaceVm,
                };
                let sg = SeedGraphIndex::shared()?;
                let compiler = Stager3VmCompiler::new(&sg)?;

                // 1. Mixed Stage 1 + Stage 2 overlays (`use std/math:` & `use std/dcs:`),
                //    surrogate `0xD800` roundtrip through `dc`/`dcs`/`s`/`DcTF`, and all 4 `numberOutside*Range` errors
                //    in BOTH Host VM and self-hosted `stager3ToStagel3` -> lowered StageL 3 VM.
                let clean_mix_src = r#"ns app/stage2_mix.
use std/math: canAdd wrapAdd bitAnd rotl min max signum abs gcd lcm ::.
use std/dcs: dcFromU128 u128FromDc classifyDc dcKind dcIsUnicode dcUnicode dcShort dcFormat dcLong formatId dcFromInt dcFromUnicode dcFromShort dcFromFormat formatIdFromShorthand formatIdShorthand parseDcShorthand formatDcShorthand dcsSingleton dcsFromList dcsToList encodeDcsDctf decodeDcsDctf ::.

v/u32/errCode error/e
    match /e
        case numberOutsideDcRange u32:1.
        case numberOutsideShortDcRange u32:2.
        case numberOutsideFormatRange u32:3.
        case numberOutsideUnicodeRange u32:4.
        else u32:0.
    ::.

pub v/b/verifyAll
    test
        in.
        out true.
    const b/okMath and canAdd u64:1 u64:2
        and eq wrapAdd u32:4294967295 u32:1 u32:0
        and eq bitAnd i128:15 i128:6 i128:6
        and eq rotl u64:1 u32:4 u64:16
        and eq min i32:-5 i32:3 i32:-5
        and eq max u128:4 u128:9 u128:9
        and eq signum i64:-7 i64:-1
        and eq gcd int:-12 int:18 int:6
        and eq lcm int:-12 int:18 int:36
        eq abs i32:-42 result/i32:ok i32:42.

    const dc/surr1 dcFromU128 u128:55296.
    const dc/surr2 dc:ud800.
    const dc/surrParsed unwrapOr parseDcShorthand "ud800" dc:u0.
    const dcs/surrS unwrapOr dcsFromS "@ud800@" dcsSingleton dc:u0.
    const dcs/seq dcsFromList l/dc: /surr1 dc:256 dc:f657 ::.
    const l/u8/enc encodeDcsDctf /seq.
    const b/okSurrDctf
        match decodeDcsDctf /enc
            case ok dcs/dec eq /dec /seq.
            case err error/ignoredErr false.
        ::.
    const b/okKind
        match classifyDc /surr2
            case unicode u32/cp eq /cp u32:55296.
            else false.
        ::.
    const b/okFmt
        match formatIdFromShorthand "f657"
            case ok formatId/fid and eq formatIdShorthand /fid "f657" eq dcFromFormat /fid dc:f657.
            case err error/ignoredErr false.
        ::.
    const b/okOpts and eq dcUnicode /surr1 option/u32:some u32:55296
        and eq dcShort dc:256 option/u32:some u32:256
        and isSome dcFormat dc:f657
        and eq dcLong dc:l8589934592 option/u128:some u128:8589934592
        dcIsUnicode /surr1.

    const b/okErr1 match dcFromInt int:-1 case err error/e eq errCode /e u32:1. else false. ::.
    const b/okErr2 match dcFromShort u32:1114112 case err error/e eq errCode /e u32:2. else false. ::.
    const b/okErr3 match formatIdFromShorthand "f1114112" case err error/e eq errCode /e u32:3. else false. ::.
    const b/okErr4 match dcFromUnicode u32:1114112 case err error/e eq errCode /e u32:4. else false. ::.

    and /okMath
        and eq /surr1 /surr2
            and eq /surrParsed /surr1
                and eq /surrS dcsSingleton /surr1
                    and eq u128FromDc /surr1 u128:55296
                        and eq len dcsToList /seq u64:3
                            and eq formatDcShorthand /surr1 "ud800"
                                and /okSurrDctf
                                    and /okKind
                                        and /okFmt
                                            and /okOpts
                                                and /okErr1
                                                    and /okErr2
                                                        and /okErr3
                                                            /okErr4.
"#;
                let sr3_rep =
                    eval_sr3_inline_tests_with_seed_graph(clean_mix_src, &sg)?;
                assert_eq!(sr3_rep.ensure_all_passed()?, 1);

                let sr3_tokens =
                    sr3_to_dc_tokens_with_seed_graph(clean_mix_src, &sg)?;
                let bundled_sr3 =
                    bundle_dc_module_dependencies(&sr3_tokens, &sg)?;
                let lowered_sl3 = compiler.compile_tokens(&bundled_sr3)?;
                let sl3_ws =
                    validate_dc_tokens_with_seed_graph(&lowered_sl3, &sg)?;
                let sl3_vm = StageLWorkspaceVm::from_validated_workspace(
                    &sl3_ws,
                    crate::stagel3::eval::EvalConfig::default(),
                )?;
                let sl3_rep = sl3_vm.run_all_tests(&sl3_ws)?;
                assert_eq!(sl3_rep.ensure_all_passed()?, 1);

                // 2. Pure `.sl3` caller bundled for `f1680` pulls ONLY StageL 3 modules (no Stage 2 `.sr3`).
                let pure_sl3_src = concat!(
                    "ns app/pure_sl3.\n",
                    "use std/u128: u128New ::.\n",
                    "use std/dctf: dcNew ::.\n",
                    "use std/math: canAdd ::.\n",
                    "use std/dcs: dcFromU128 ::.\n",
                    "pub v/b/ok :: and canAdd u64:1 u64:2 eq dcFromU128 u128New u64:55296 u64:0 dcNew u64:55296 u64:0.\n",
                );
                let pure_tokens =
                    sl3_to_dc_tokens_with_seed_graph(pure_sl3_src, &sg)?;
                let pure_bundled =
                    bundle_dc_module_dependencies(&pure_tokens, &sg)?;
                let pure_mods = decode_modules_from_dc_tokens(&pure_bundled)?;
                assert!(
                    pure_mods
                        .iter()
                        .all(|m| m.dialect == StageDialect::StageL3),
                    "Pure StageL 3 bundle must not pull StageR 3 strata"
                );

                // 3. Negative: Stage 1 module cannot import Stage 2-only exports, and same-stage duplicate `ns` fails.
                assert!(
                    parse_and_validate_workspace_with_seed_graph(
                        &[ModuleInput {
                            source_path: Some("stage_1_impls/app/s1_bad_ovl.sl3"),
                            source: "ns app/s1_bad_ovl.\nuse std/math: gcd ::.\npub v/u32/f :: gcd u32:4 u32:6.\n",
                        }],
                        &sg,
                    )
                    .is_err(),
                    "Stage 1 must not import Stage 2 std/math::gcd"
                );
                assert!(
                    parse_and_validate_workspace_with_seed_graph(
                        &[
                            ModuleInput {
                                source_path: Some("stage_1_impls/app/dupns.sl3"),
                                source: "ns app/dupns.\npub v/u32/a :: u32:1.\n",
                            },
                            ModuleInput {
                                source_path: Some("stage_1_impls/app/dupns.sl3"),
                                source: "ns app/dupns.\npub v/u32/b :: u32:2.\n",
                            },
                        ],
                        &sg,
                    )
                    .is_err(),
                    "Duplicate same-stage namespace must be rejected"
                );

                // 4. Negative: cross-stratum `pub` collisions rejected by both Host validator and self-hosted `f1681`.
                for (label, s1_col, s2_col) in [
                    (
                        "pub type",
                        "ns app/colmod.\npub struct item :: u32.\n",
                        "ns app/colmod.\npub type u32/item.\n",
                    ),
                    (
                        "pub const",
                        "ns app/colmod.\npub u32/VAL u32:1.\n",
                        "ns app/colmod.\npub u32/VAL u32:2.\n",
                    ),
                    (
                        "same-param routine",
                        "ns app/colmod.\npub v/u32/op u32/x :: /x.\n",
                        "ns app/colmod.\npub v/u32/op u32/x :: /x.\n",
                    ),
                    (
                        "differing-arity routine",
                        "ns app/colmod.\npub v/u32/op u32/x :: /x.\n",
                        "ns app/colmod.\npub v/u32/op u32/x u32/y :: /x.\n",
                    ),
                ] {
                    assert!(
                        parse_and_validate_workspace_with_seed_graph(
                            &[
                                ModuleInput {
                                    source_path: Some("stage_1_impls/app/colmod.sl3"),
                                    source: s1_col,
                                },
                                ModuleInput {
                                    source_path: Some("stage_2_impls/app/colmod.sr3"),
                                    source: s2_col,
                                },
                            ],
                            &sg,
                        )
                        .is_err(),
                        "[{label}] Host validator must reject cross-stratum pub collision"
                    );
                    let m1 = super::super::parser::parse_module(s1_col)?;
                    let m2 = super::super::parser::parse_sr3_module(s2_col)?;
                    let mut raw_col = encode_module_to_dc_tokens(&m1)?;
                    raw_col.extend(encode_module_to_dc_tokens(&m2)?);
                    assert!(
                        matches!(
                            compiler.run_raw(&raw_col)?,
                            RoutineOutcome::Trapped(_)
                        ),
                        "[{label}] Self-hosted stager3ToStagel3 must trap on cross-stratum pub collision"
                    );
                }

                Ok(())
            })
            .context("Failed to spawn stage2 overlay test thread")?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("stage2 overlay test thread panicked"))?
    }

    #[crate::ctb_test]
    fn test_stager3_to_stagel3_lowers_dc_2608_entry_module() -> Result<()> {
        let handle = std::thread::Builder::new()
            .name(String::from("sr3_to_sl3_dc2608_entry"))
            .stack_size(64 * 1024 * 1024)
            .spawn(|| -> Result<()> {
                let seed_graph = SeedGraphIndex::shared()?;
                let compiler =
                    Stager3VmCompiler::new_with_quota(seed_graph, 500_000_000)?;
                let entry_tokens =
                    build_stager_dcs_entry_module_for_graph_routine_dc(
                        crate::dc::DC_CONVERT_DC_STRING_TO_WASM,
                        seed_graph,
                    )?;
                let bundled =
                    bundle_dc_module_dependencies(&entry_tokens, seed_graph)?;
                let lowered = compiler.compile_tokens(&bundled)?;
                let decoded_mods = decode_modules_from_dc_tokens(&lowered)?;
                assert!(
                    !decoded_mods.is_empty(),
                    "Expected at least 1 lowered StageL 3 module for Dc 2608"
                );
                for m in &decoded_mods {
                    assert_eq!(
                        m.dialect,
                        StageDialect::StageL3,
                        "Expected StageL 3 module after lowering Dc 2608 bundle"
                    );
                }
                let last_mod = decoded_mods
                    .last()
                    .context("Missing final entry module in lowered Dc 2608 bundle")?;
                let has_step = last_mod.items.iter().any(|item| {
                    matches!(item, TopDecl::Routine(r) if r.name == "step")
                });
                assert!(
                    has_step,
                    "Expected final lowered Dc 2608 entry module to define 'step'"
                );
                Ok(())
            })
            .context("Failed to spawn dc2608 StageL VM lowering test thread")?;
        handle
            .join()
            .map_err(|_| anyhow::anyhow!("dc2608 StageL VM lowering thread panicked"))?
    }
}
