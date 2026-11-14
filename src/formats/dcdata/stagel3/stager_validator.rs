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

//! Static totality, nominal type, linear buffer, and exhaustiveness validator
//! for StageR 3 (`f657` `.sr3`) modules.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, BrandTypeDecl, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp,
    EnumDecl, Expr, ImportTarget, MatchArm, Module, NamedStructDecl,
    RoutineDecl, Span, StageDialect, StageLType, StageRBinaryOp,
    StageRErrorTag, StageRTernaryOp, StageRUnaryOp, StructTypeRef,
    SymbolTarget, TestExpected, TopDecl, TopValueDecl, format_dc_shorthand,
};
use super::stager_strings::is_valid_dc_u128;
use super::validator::{
    AssignState, BindingEntry, BindingKind, BrandTypeDef, EnumDef,
    EnumVariantDef, ImportedSymbol, NamedStructDef, NamespaceExports,
    NestedVerbSummary, ResolvedCallTarget, ResolvedValueTarget, RoutineSig,
    StructDef, ValidatedModule, VerbTarget, check_pub_dc_shorthand,
    collect_value_deps, format_param_types, is_stagel_internal_name,
    types_compatible,
};
use crate::DcChar;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufState {
    Unfrozen,
    Frozen,
    MaybeFrozen,
}

impl BufState {
    fn join(self, other: Self) -> Self {
        if self == other {
            self
        } else {
            Self::MaybeFrozen
        }
    }
}

#[derive(Debug, Clone)]
struct FlowState {
    assign: AssignState,
    bufs: BTreeMap<usize, BufState>,
}

impl FlowState {
    fn new() -> Self {
        Self {
            assign: AssignState::new(),
            bufs: BTreeMap::new(),
        }
    }

    fn intersect_with(&mut self, other: &Self) {
        self.assign.intersect_with(&other.assign);
        let keys: Vec<usize> = self.bufs.keys().copied().collect();
        for id in keys {
            match (self.bufs.get(&id).copied(), other.bufs.get(&id).copied()) {
                (Some(a), Some(b)) => {
                    self.bufs.insert(id, a.join(b));
                }
                (Some(_), None) => {
                    self.bufs.insert(id, BufState::MaybeFrozen);
                }
                (None, _) => {}
            }
        }
    }
}

struct StageRCtx {
    return_type: StageLType,
    loop_depth: usize,
    in_tail: bool,
    allow_buf: bool,
    allow_propagate: bool,
    outer_ids: HashSet<usize>,
    loop_protected_bufs: HashSet<usize>,
    captured_reads: HashSet<usize>,
    always_written: HashSet<usize>,
}

impl StageRCtx {
    fn child(&self, in_tail: bool, loop_depth: usize, allow_buf: bool) -> Self {
        Self {
            return_type: self.return_type.clone(),
            loop_depth,
            in_tail,
            allow_buf,
            allow_propagate: self.allow_propagate,
            outer_ids: self.outer_ids.clone(),
            loop_protected_bufs: self.loop_protected_bufs.clone(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        }
    }
}

enum MatchDomain {
    Option(StageLType),
    Result(StageLType),
    Error,
    Enum(Vec<EnumVariantDef>),
}

struct StageRValidator<'a> {
    value_candidates: HashMap<String, ResolvedValueTarget>,
    routine_candidates: HashMap<String, Vec<ResolvedCallTarget>>,
    local_structs: HashMap<String, (Option<String>, String)>,
    dc_structs: HashMap<DcChar, (Option<String>, String)>,
    local_nominals: HashMap<String, (Option<String>, String)>,
    dc_nominals: HashMap<DcChar, (Option<String>, String)>,
    struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    brand_types: BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: BTreeMap<(Option<String>, String), EnumDef>,
    local_declared_opaque: HashSet<(Option<String>, String)>,
    top_dc_values: HashMap<DcChar, StageLType>,
    top_dc_routines: HashMap<DcChar, RoutineSig>,
    top_dc_names: HashMap<DcChar, String>,
    dc_imports: HashMap<DcChar, ImportedSymbol>,
    next_binding_id: usize,
    local_scopes: Vec<HashMap<String, BindingEntry>>,
    routine_scopes: Vec<HashMap<String, VerbTarget>>,
    nested_summaries: HashMap<usize, NestedVerbSummary>,
    pending_nested_calls: Vec<(usize, Span, AssignState)>,
    tail_calls: HashSet<(usize, usize)>,
    resolved_calls: HashMap<(usize, usize), ResolvedCallTarget>,
    resolved_values: HashMap<(usize, usize), ResolvedValueTarget>,
    resolved_named_structs: HashMap<(usize, usize), NamedStructDef>,
    resolved_enum_variants:
        HashMap<(usize, usize), ((Option<String>, String), String)>,
    value_ref_types: HashMap<(usize, usize), StageLType>,
    expr_types: HashMap<(usize, usize), StageLType>,
    _marker: std::marker::PhantomData<&'a ()>,
}

/// Validates a parsed StageR 3 [`Module`] against resolved dependency exports
/// and pre-resolved direct `DcChar` symbol targets.
pub(super) fn validate_stager_module(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
    extra_dc_imports: &HashMap<DcChar, ImportedSymbol>,
) -> Result<ValidatedModule> {
    ensure!(
        module.dialect == StageDialect::StageR3,
        "validate_stager_module called on non-StageR 3 module"
    );

    let namespace_key = module.namespace.as_ref().map(|segs| segs.join("/"));
    let mut exports = NamespaceExports::default();
    let mut imports: HashMap<String, Vec<ImportedSymbol>> = HashMap::new();
    let mut dc_imports = extra_dc_imports.clone();
    let mut value_candidates: HashMap<String, ResolvedValueTarget> =
        HashMap::new();
    let mut routine_candidates: HashMap<String, Vec<ResolvedCallTarget>> =
        HashMap::new();
    let mut local_structs: HashMap<String, (Option<String>, String)> =
        HashMap::new();
    let mut dc_structs: HashMap<DcChar, (Option<String>, String)> =
        HashMap::new();
    let mut local_nominals: HashMap<String, (Option<String>, String)> =
        HashMap::new();
    let mut dc_nominals: HashMap<DcChar, (Option<String>, String)> =
        HashMap::new();
    let mut struct_defs: BTreeMap<(Option<String>, String), StructDef> =
        BTreeMap::new();
    let mut brand_types: BTreeMap<(Option<String>, String), BrandTypeDef> =
        BTreeMap::new();
    let mut named_structs: BTreeMap<(Option<String>, String), NamedStructDef> =
        BTreeMap::new();
    let mut enums: BTreeMap<(Option<String>, String), EnumDef> =
        BTreeMap::new();
    let mut local_declared_opaque: HashSet<(Option<String>, String)> =
        HashSet::new();

    for dep_exp in ns_exports.values() {
        for (k, d) in &dep_exp.all_structs {
            struct_defs.insert(k.clone(), d.clone());
        }
        for (k, d) in &dep_exp.all_brand_types {
            brand_types.insert(k.clone(), d.clone());
        }
        for (k, d) in &dep_exp.all_named_structs {
            named_structs.insert(k.clone(), d.clone());
        }
        for (k, d) in &dep_exp.all_enums {
            enums.insert(k.clone(), d.clone());
        }
    }

    seed_dc_import_type_tables(
        &dc_imports,
        &mut dc_structs,
        &mut dc_nominals,
        &mut struct_defs,
        &mut brand_types,
        &mut named_structs,
        &mut enums,
    );

    canonicalize_dc_import_tables(
        &mut dc_imports,
        &dc_structs,
        &dc_nominals,
        &mut struct_defs,
        &mut brand_types,
        &mut named_structs,
        &mut enums,
    )?;

    // 1. Resolve `use` imports.
    for u in &module.uses {
        let target_ns = u.path.join("/");
        let dep_exp = ns_exports.get(&target_ns).with_context(|| {
            format!("Unresolved namespace '{target_ns}' in 'use' at {}", u.span)
        })?;
        for item in &u.items {
            let resolved_list = resolve_use_item_symbols(
                &target_ns,
                dep_exp,
                item,
                &mut dc_imports,
                &mut dc_structs,
                &mut dc_nominals,
            )?;
            let first_sym = resolved_list
                .first()
                .context("Empty exported symbol list")?;
            let local_name = match (&item.alias, &item.target) {
                (Some(a), _) => a.clone(),
                (None, ImportTarget::Name(n)) => n.clone(),
                (None, ImportTarget::Dc(_)) => {
                    imported_symbol_original_name(first_sym).to_owned()
                }
            };
            if is_stagel_internal_name(&local_name) {
                bail!(
                    "StageL-internal symbol '{local_name}' cannot be imported in StageR 3 module at {}",
                    item.span
                );
            }

            for resolved in resolved_list {
                register_imported_symbol(
                    &resolved,
                    &local_name,
                    item.span,
                    &mut value_candidates,
                    &mut routine_candidates,
                    &mut local_structs,
                    &mut local_nominals,
                    &mut struct_defs,
                    &mut brand_types,
                    &mut named_structs,
                    &mut enums,
                )?;
                imports
                    .entry(local_name.clone())
                    .or_default()
                    .push(resolved);
            }
        }
    }

    // 2a. Collect top-level type declarations (Pass 2a-1: register names & @dc).
    let mut top_dc_values = HashMap::new();
    let mut top_dc_routines = HashMap::new();
    let mut top_dc_names = HashMap::new();
    let mut top_value_order: Vec<&TopValueDecl> = Vec::new();
    let mut top_routine_order: Vec<&RoutineDecl> = Vec::new();
    let mut brand_decls: Vec<&BrandTypeDecl> = Vec::new();
    let mut named_struct_decls: Vec<&NamedStructDecl> = Vec::new();
    let mut enum_decls: Vec<&EnumDecl> = Vec::new();

    for decl in &module.items {
        match decl {
            TopDecl::Struct(s) => {
                bail!(
                    "Positional StageL 'struct {}' at {} is forbidden in StageR 3 (use named-field 'struct' instead)",
                    s.name,
                    s.span
                );
            }
            TopDecl::BrandType(b) => {
                ensure!(
                    !is_stagel_internal_name(&b.name),
                    "StageL-internal type '{}' cannot be declared in StageR 3 module at {}",
                    b.name,
                    b.span
                );
                ensure!(
                    !name_conflicts_top(
                        &b.name,
                        &value_candidates,
                        &routine_candidates,
                        &local_structs,
                        &local_nominals,
                    ),
                    "Duplicate or conflicting declaration name '{}' at {}",
                    b.name,
                    b.span
                );
                if b.pub_dc_syntax
                    && let Some(dc) = b.dc_id
                {
                    check_pub_dc_shorthand(dc, b.span)?;
                }
                if let Some(dc) = b.dc_id {
                    register_top_dc_name(&mut top_dc_names, dc, &b.name, b.span)?;
                }
                let key = (namespace_key.clone(), b.name.clone());
                local_nominals.insert(b.name.clone(), key.clone());
                if let Some(dc) = b.dc_id {
                    dc_nominals.insert(dc, key.clone());
                }
                if b.is_opaque {
                    local_declared_opaque.insert(key);
                }
                brand_decls.push(b);
            }
            TopDecl::NamedStruct(ns) => {
                ensure!(
                    !is_stagel_internal_name(&ns.name),
                    "StageL-internal struct '{}' cannot be declared in StageR 3 module at {}",
                    ns.name,
                    ns.span
                );
                ensure!(
                    !name_conflicts_top(
                        &ns.name,
                        &value_candidates,
                        &routine_candidates,
                        &local_structs,
                        &local_nominals,
                    ),
                    "Duplicate or conflicting declaration name '{}' at {}",
                    ns.name,
                    ns.span
                );
                if ns.pub_dc_syntax
                    && let Some(dc) = ns.dc_id
                {
                    check_pub_dc_shorthand(dc, ns.span)?;
                }
                if let Some(dc) = ns.dc_id {
                    register_top_dc_name(
                        &mut top_dc_names,
                        dc,
                        &ns.name,
                        ns.span,
                    )?;
                }
                let key = (namespace_key.clone(), ns.name.clone());
                local_structs.insert(ns.name.clone(), key.clone());
                if let Some(dc) = ns.dc_id {
                    dc_structs.insert(dc, key);
                }
                named_struct_decls.push(ns);
            }
            TopDecl::Enum(en) => {
                ensure!(
                    !is_stagel_internal_name(&en.name),
                    "StageL-internal enum '{}' cannot be declared in StageR 3 module at {}",
                    en.name,
                    en.span
                );
                ensure!(
                    !name_conflicts_top(
                        &en.name,
                        &value_candidates,
                        &routine_candidates,
                        &local_structs,
                        &local_nominals,
                    ),
                    "Duplicate or conflicting declaration name '{}' at {}",
                    en.name,
                    en.span
                );
                if en.pub_dc_syntax
                    && let Some(dc) = en.dc_id
                {
                    check_pub_dc_shorthand(dc, en.span)?;
                }
                if let Some(dc) = en.dc_id {
                    register_top_dc_name(
                        &mut top_dc_names,
                        dc,
                        &en.name,
                        en.span,
                    )?;
                }
                let key = (namespace_key.clone(), en.name.clone());
                local_nominals.insert(en.name.clone(), key.clone());
                if let Some(dc) = en.dc_id {
                    dc_nominals.insert(dc, key);
                }
                enum_decls.push(en);
            }
            TopDecl::Value(_) | TopDecl::Routine(_) | TopDecl::Comment(_) => {}
        }
    }

    // Pass 2a-2: Resolve underlying types / fields / variants for declared types.
    for b in brand_decls {
        let canon_inner = canonicalize_stager_type(
            &b.inner_ty,
            &local_structs,
            &dc_structs,
            &local_nominals,
            &dc_nominals,
            &struct_defs,
            &brand_types,
            &named_structs,
            &enums,
            &top_dc_names,
            b.span,
        )?;
        check_stager_value_type_shape(&canon_inner, b.span)?;
        let key = (namespace_key.clone(), b.name.clone());
        let def = BrandTypeDef {
            namespace: namespace_key.clone(),
            original_name: b.name.clone(),
            is_opaque: b.is_opaque,
            inner_ty: canon_inner.clone(),
            is_pub: b.is_pub,
            dc_id: b.dc_id,
            span: b.span,
        };
        brand_types.insert(key.clone(), def.clone());
        if b.is_pub {
            let export_ns = namespace_key.as_ref().with_context(|| {
                format!(
                    "'pub {} {}' requires a module 'ns' declaration (at {})",
                    if b.is_opaque { "type" } else { "brand" },
                    b.name,
                    b.span
                )
            })?;
            exports.brand_types.insert(b.name.clone(), def.clone());
            exports.all_brand_types.insert(key, def);
            if let Some(dc) = b.dc_id {
                let sym = ImportedSymbol::BrandType {
                    namespace: export_ns.clone(),
                    original_name: b.name.clone(),
                    is_opaque: b.is_opaque,
                    inner_ty: canon_inner,
                    dc_id: Some(dc),
                };
                if exports.by_dc.insert(dc, sym).is_some() {
                    bail!(
                        "Duplicate exported Dc shorthand 'dc/{}' at {}",
                        format_dc_shorthand(dc),
                        b.span
                    );
                }
            }
        }
    }

    for ns in named_struct_decls {
        ensure!(
            !ns.fields.is_empty(),
            "Named struct '{}' at {} must have at least 1 field",
            ns.name,
            ns.span
        );
        let mut seen_fields = HashSet::new();
        let mut canon_fields = Vec::with_capacity(ns.fields.len());
        for f in &ns.fields {
            ensure!(
                !is_stagel_internal_name(&f.name),
                "StageL-internal field name '{}' in struct '{}' at {} is forbidden in StageR 3",
                f.name,
                ns.name,
                f.span
            );
            ensure!(
                seen_fields.insert(f.name.clone()),
                "Duplicate field '{}' in struct '{}' at {}",
                f.name,
                ns.name,
                f.span
            );
            let cf = canonicalize_stager_type(
                &f.ty,
                &local_structs,
                &dc_structs,
                &local_nominals,
                &dc_nominals,
                &struct_defs,
                &brand_types,
                &named_structs,
                &enums,
                &top_dc_names,
                f.span,
            )?;
            check_stager_value_type_shape(&cf, f.span)?;
            canon_fields.push((f.name.clone(), cf));
        }
        let key = (namespace_key.clone(), ns.name.clone());
        let def = NamedStructDef {
            namespace: namespace_key.clone(),
            original_name: ns.name.clone(),
            fields: canon_fields.clone(),
            is_pub: ns.is_pub,
            dc_id: ns.dc_id,
            span: ns.span,
        };
        named_structs.insert(key.clone(), def.clone());
        if ns.is_pub {
            let export_ns = namespace_key.as_ref().with_context(|| {
                format!(
                    "'pub struct {}' requires a module 'ns' declaration (at {})",
                    ns.name,
                    ns.span
                )
            })?;
            exports.named_structs.insert(ns.name.clone(), def.clone());
            exports.all_named_structs.insert(key, def);
            if let Some(dc) = ns.dc_id {
                let sym = ImportedSymbol::NamedStruct {
                    namespace: export_ns.clone(),
                    original_name: ns.name.clone(),
                    fields: canon_fields,
                    dc_id: Some(dc),
                };
                if exports.by_dc.insert(dc, sym).is_some() {
                    bail!(
                        "Duplicate exported Dc shorthand 'dc/{}' at {}",
                        format_dc_shorthand(dc),
                        ns.span
                    );
                }
            }
        }
    }

    for en in enum_decls {
        let mut seen_vars = HashSet::new();
        let mut canon_vars = Vec::with_capacity(en.variants.len());
        for v in &en.variants {
            ensure!(
                !is_stagel_internal_name(&v.name),
                "StageL-internal variant name '{}' in enum '{}' at {} is forbidden in StageR 3",
                v.name,
                en.name,
                v.span
            );
            ensure!(
                seen_vars.insert(v.name.clone()),
                "Duplicate variant '{}' in enum '{}' at {}",
                v.name,
                en.name,
                v.span
            );
            let canon_payload = match &v.payload {
                Some(pty) => {
                    let cp = canonicalize_stager_type(
                        pty,
                        &local_structs,
                        &dc_structs,
                        &local_nominals,
                        &dc_nominals,
                        &struct_defs,
                        &brand_types,
                        &named_structs,
                        &enums,
                        &top_dc_names,
                        v.span,
                    )?;
                    check_stager_value_type_shape(&cp, v.span)?;
                    Some(cp)
                }
                None => None,
            };
            canon_vars.push(EnumVariantDef {
                name: v.name.clone(),
                payload: canon_payload,
                span: v.span,
            });
        }
        let key = (namespace_key.clone(), en.name.clone());
        let def = EnumDef {
            namespace: namespace_key.clone(),
            original_name: en.name.clone(),
            variants: canon_vars.clone(),
            is_pub: en.is_pub,
            dc_id: en.dc_id,
            span: en.span,
        };
        enums.insert(key.clone(), def.clone());
        if en.is_pub {
            let export_ns = namespace_key.as_ref().with_context(|| {
                format!(
                    "'pub enum {}' requires a module 'ns' declaration (at {})",
                    en.name,
                    en.span
                )
            })?;
            exports.enums.insert(en.name.clone(), def.clone());
            exports.all_enums.insert(key, def);
            if let Some(dc) = en.dc_id {
                let sym = ImportedSymbol::Enum {
                    namespace: export_ns.clone(),
                    original_name: en.name.clone(),
                    variants: canon_vars,
                    dc_id: Some(dc),
                };
                if exports.by_dc.insert(dc, sym).is_some() {
                    bail!(
                        "Duplicate exported Dc shorthand 'dc/{}' at {}",
                        format_dc_shorthand(dc),
                        en.span
                    );
                }
            }
        }
    }

    // Pass 2a-3: Forbid zero-indirection recursive cycles.
    check_stager_unguarded_type_cycles(&brand_types, &named_structs, &enums)?;

    // Ensure transitive type closure is available to downstream importers when
    // exporting `pub` items.
    if !exports.brand_types.is_empty()
        || !exports.named_structs.is_empty()
        || !exports.enums.is_empty()
        || module
            .items
            .iter()
            .any(|d| matches!(d, TopDecl::Value(v) if v.is_pub) || matches!(d, TopDecl::Routine(r) if r.is_pub))
    {
        for (k, d) in &struct_defs {
            exports.all_structs.insert(k.clone(), d.clone());
        }
        for (k, d) in &brand_types {
            exports.all_brand_types.insert(k.clone(), d.clone());
        }
        for (k, d) in &named_structs {
            exports.all_named_structs.insert(k.clone(), d.clone());
        }
        for (k, d) in &enums {
            exports.all_enums.insert(k.clone(), d.clone());
        }
    }

    // Pass 2b: Collect top-level constants and routine signatures.
    let mut local_val_names = HashSet::new();
    let mut local_routine_sigs: HashMap<String, Vec<RoutineSig>> =
        HashMap::new();

    for decl in &module.items {
        match decl {
            TopDecl::Value(v) => {
                ensure!(
                    !is_stagel_internal_name(&v.name),
                    "StageL-internal constant '{}' cannot be declared in StageR 3 module at {}",
                    v.name,
                    v.span
                );
                ensure!(
                    !value_candidates.contains_key(&v.name)
                        && !routine_candidates.contains_key(&v.name)
                        && !local_structs.contains_key(&v.name)
                        && !local_nominals.contains_key(&v.name)
                        && local_val_names.insert(v.name.clone()),
                    "Duplicate or conflicting constant declaration '{}' at {}",
                    v.name,
                    v.span
                );
                let canon_ty = canonicalize_stager_type(
                    &v.ty,
                    &local_structs,
                    &dc_structs,
                    &local_nominals,
                    &dc_nominals,
                    &struct_defs,
                    &brand_types,
                    &named_structs,
                    &enums,
                    &top_dc_names,
                    v.span,
                )?;
                check_stager_value_type_shape(&canon_ty, v.span)?;
                if v.pub_dc_syntax
                    && let Some(dc) = v.dc_id
                {
                    check_pub_dc_shorthand(dc, v.span)?;
                }
                if let Some(dc) = v.dc_id {
                    register_top_dc_name(&mut top_dc_names, dc, &v.name, v.span)?;
                    top_dc_values.insert(dc, canon_ty.clone());
                }
                value_candidates.insert(
                    v.name.clone(),
                    ResolvedValueTarget {
                        namespace: None,
                        original_name: v.name.clone(),
                        ty: canon_ty.clone(),
                        dc_id: v.dc_id,
                    },
                );
                if v.is_pub {
                    let export_ns = namespace_key.as_ref().with_context(|| {
                        format!(
                            "'pub' constant '{}' requires a module 'ns' declaration (at {})",
                            v.name,
                            v.span
                        )
                    })?;
                    exports
                        .values
                        .insert(v.name.clone(), (canon_ty.clone(), v.dc_id));
                    if let Some(dc) = v.dc_id {
                        let sym = ImportedSymbol::Value {
                            namespace: export_ns.clone(),
                            original_name: v.name.clone(),
                            ty: canon_ty,
                            dc_id: Some(dc),
                        };
                        if exports.by_dc.insert(dc, sym).is_some() {
                            bail!(
                                "Duplicate exported Dc shorthand 'dc/{}' at {}",
                                format_dc_shorthand(dc),
                                v.span
                            );
                        }
                    }
                }
                top_value_order.push(v);
            }
            TopDecl::Routine(r) => {
                ensure!(
                    !is_stagel_internal_name(&r.name),
                    "StageL-internal routine '{}' cannot be declared in StageR 3 module at {}",
                    r.name,
                    r.span
                );
                ensure!(
                    !value_candidates.contains_key(&r.name)
                        && !local_structs.contains_key(&r.name)
                        && !local_nominals.contains_key(&r.name),
                    "Routine '{}' at {} conflicts with an existing constant or type name",
                    r.name,
                    r.span
                );
                let canon_ret = canonicalize_stager_type(
                    &r.return_type,
                    &local_structs,
                    &dc_structs,
                    &local_nominals,
                    &dc_nominals,
                    &struct_defs,
                    &brand_types,
                    &named_structs,
                    &enums,
                    &top_dc_names,
                    r.span,
                )?;
                ensure!(
                    !canon_ret.is_or_contains_buf(),
                    "Routine '{}' return type '{}' cannot be or contain 'buf/T' in StageR 3 at {}",
                    r.name,
                    canon_ret,
                    r.span
                );
                let mut seen_p = HashSet::new();
                let mut canon_params = Vec::with_capacity(r.params.len());
                for p in &r.params {
                    ensure!(
                        !is_stagel_internal_name(&p.name),
                        "StageL-internal parameter name '{}' in '{}' at {} is forbidden in StageR 3",
                        p.name,
                        r.name,
                        p.span
                    );
                    ensure!(
                        seen_p.insert(p.name.clone()),
                        "Duplicate parameter '{}' in routine '{}' at {}",
                        p.name,
                        r.name,
                        p.span
                    );
                    let canon_pty = canonicalize_stager_type(
                        &p.ty,
                        &local_structs,
                        &dc_structs,
                        &local_nominals,
                        &dc_nominals,
                        &struct_defs,
                        &brand_types,
                        &named_structs,
                        &enums,
                        &top_dc_names,
                        p.span,
                    )?;
                    check_stager_value_type_shape(&canon_pty, p.span)?;
                    canon_params.push(canon_pty);
                }

                if let Some(existing) = routine_candidates.get(&r.name) {
                    let existing_arity = existing
                        .first()
                        .context("Empty routine candidate list")?
                        .sig
                        .params
                        .len();
                    ensure!(
                        existing_arity == canon_params.len(),
                        "Overloaded routine '{}' at {} must share fixed arity {existing_arity}, found {}",
                        r.name,
                        r.span,
                        canon_params.len()
                    );
                    for prev in existing {
                        ensure!(
                            prev.sig.params != canon_params,
                            "Duplicate or conflicting routine overload '{}/{}({})' at {}",
                            r.name,
                            r.params.len(),
                            format_param_types(&canon_params),
                            r.span
                        );
                    }
                }
                if r.pub_dc_syntax
                    && let Some(dc) = r.dc_id
                {
                    check_pub_dc_shorthand(dc, r.span)?;
                }
                let sig = RoutineSig {
                    return_type: canon_ret,
                    params: canon_params,
                    is_pub: r.is_pub,
                    dc_id: r.dc_id,
                };
                if let Some(dc) = r.dc_id {
                    register_top_dc_name(&mut top_dc_names, dc, &r.name, r.span)?;
                    top_dc_routines.insert(dc, sig.clone());
                }
                routine_candidates
                    .entry(r.name.clone())
                    .or_default()
                    .push(ResolvedCallTarget {
                        namespace: None,
                        routine_name: r.name.clone(),
                        sig: sig.clone(),
                    });
                local_routine_sigs
                    .entry(r.name.clone())
                    .or_default()
                    .push(sig.clone());
                if r.is_pub {
                    let export_ns = namespace_key.as_ref().with_context(|| {
                        format!(
                            "'pub' routine '{}' requires a module 'ns' declaration (at {})",
                            r.name,
                            r.span
                        )
                    })?;
                    exports
                        .routines
                        .entry(r.name.clone())
                        .or_default()
                        .push(sig.clone());
                    if let Some(dc) = r.dc_id {
                        let sym = ImportedSymbol::Routine {
                            namespace: export_ns.clone(),
                            original_name: r.name.clone(),
                            sig,
                        };
                        if exports.by_dc.insert(dc, sym).is_some() {
                            bail!(
                                "Duplicate exported Dc shorthand 'dc/{}' at {}",
                                format_dc_shorthand(dc),
                                r.span
                            );
                        }
                    }
                }
                top_routine_order.push(r);
            }
            TopDecl::BrandType(_)
            | TopDecl::NamedStruct(_)
            | TopDecl::Enum(_)
            | TopDecl::Struct(_)
            | TopDecl::Comment(_) => {}
        }
    }

    let mut validator = StageRValidator {
        value_candidates,
        routine_candidates,
        local_structs,
        dc_structs,
        local_nominals,
        dc_nominals,
        struct_defs,
        brand_types,
        named_structs,
        enums,
        local_declared_opaque,
        top_dc_values,
        top_dc_routines,
        top_dc_names,
        dc_imports,
        next_binding_id: 1,
        local_scopes: Vec::new(),
        routine_scopes: Vec::new(),
        nested_summaries: HashMap::new(),
        pending_nested_calls: Vec::new(),
        tail_calls: HashSet::new(),
        resolved_calls: HashMap::new(),
        resolved_values: HashMap::new(),
        resolved_named_structs: HashMap::new(),
        resolved_enum_variants: HashMap::new(),
        value_ref_types: HashMap::new(),
        expr_types: HashMap::new(),
        _marker: std::marker::PhantomData,
    };

    // 3. Validate top-level constant initializers (acyclic & pure).
    validator.validate_top_values_acyclic(&top_value_order)?;

    // 4. Validate inline tests & bodies for all top-level routines.
    for r in top_routine_order {
        validator.validate_top_routine(r)?;
    }

    Ok(ValidatedModule {
        module: module.clone(),
        namespace_key,
        imports,
        dc_imports: validator.dc_imports,
        local_structs: validator.local_structs,
        dc_structs: validator.dc_structs,
        local_nominals: validator.local_nominals,
        dc_nominals: validator.dc_nominals,
        struct_defs: validator.struct_defs,
        brand_types: validator.brand_types,
        named_structs: validator.named_structs,
        enums: validator.enums,
        tail_calls: validator.tail_calls,
        resolved_calls: validator.resolved_calls,
        resolved_values: validator.resolved_values,
        resolved_structs: HashMap::new(),
        resolved_named_structs: validator.resolved_named_structs,
        resolved_enum_variants: validator.resolved_enum_variants,
        value_ref_types: validator.value_ref_types,
        expr_types: validator.expr_types,
        exports,
    })
}

fn name_conflicts_top(
    name: &str,
    values: &HashMap<String, ResolvedValueTarget>,
    routines: &HashMap<String, Vec<ResolvedCallTarget>>,
    local_structs: &HashMap<String, (Option<String>, String)>,
    local_nominals: &HashMap<String, (Option<String>, String)>,
) -> bool {
    values.contains_key(name)
        || routines.contains_key(name)
        || local_structs.contains_key(name)
        || local_nominals.contains_key(name)
}

fn register_top_dc_name(
    top_dc_names: &mut HashMap<DcChar, String>,
    dc: DcChar,
    name: &str,
    span: Span,
) -> Result<()> {
    if let Some(prev) = top_dc_names.insert(dc, name.to_owned()) {
        bail!(
            "Duplicate top-level @dc assignment 'dc/{}' on '{prev}' and '{name}' at {span}",
            format_dc_shorthand(dc)
        );
    }
    Ok(())
}

fn imported_symbol_original_name(sym: &ImportedSymbol) -> &str {
    match sym {
        ImportedSymbol::Value { original_name, .. }
        | ImportedSymbol::Routine { original_name, .. }
        | ImportedSymbol::Struct { original_name, .. }
        | ImportedSymbol::BrandType { original_name, .. }
        | ImportedSymbol::NamedStruct { original_name, .. }
        | ImportedSymbol::Enum { original_name, .. } => original_name,
    }
}

fn seed_dc_import_type_tables(
    dc_imports: &HashMap<DcChar, ImportedSymbol>,
    dc_structs: &mut HashMap<DcChar, (Option<String>, String)>,
    dc_nominals: &mut HashMap<DcChar, (Option<String>, String)>,
    struct_defs: &mut BTreeMap<(Option<String>, String), StructDef>,
    brand_types: &mut BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: &mut BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: &mut BTreeMap<(Option<String>, String), EnumDef>,
) {
    let synth_span = Span { line: 1, col: 1 };
    for (&dc, sym) in dc_imports {
        match sym {
            ImportedSymbol::Struct {
                namespace,
                original_name,
                fields,
                dc_id,
            } => {
                let key = (Some(namespace.clone()), original_name.clone());
                dc_structs.insert(dc, key.clone());
                struct_defs.entry(key).or_insert_with(|| StructDef {
                    namespace: Some(namespace.clone()),
                    original_name: original_name.clone(),
                    fields: fields.clone(),
                    is_pub: true,
                    dc_id: *dc_id,
                    span: synth_span,
                });
            }
            ImportedSymbol::BrandType {
                namespace,
                original_name,
                is_opaque,
                inner_ty,
                dc_id,
            } => {
                let key = (Some(namespace.clone()), original_name.clone());
                dc_nominals.insert(dc, key.clone());
                brand_types.entry(key).or_insert_with(|| BrandTypeDef {
                    namespace: Some(namespace.clone()),
                    original_name: original_name.clone(),
                    is_opaque: *is_opaque,
                    inner_ty: inner_ty.clone(),
                    is_pub: true,
                    dc_id: *dc_id,
                    span: synth_span,
                });
            }
            ImportedSymbol::NamedStruct {
                namespace,
                original_name,
                fields,
                dc_id,
            } => {
                let key = (Some(namespace.clone()), original_name.clone());
                dc_structs.insert(dc, key.clone());
                named_structs.entry(key).or_insert_with(|| NamedStructDef {
                    namespace: Some(namespace.clone()),
                    original_name: original_name.clone(),
                    fields: fields.clone(),
                    is_pub: true,
                    dc_id: *dc_id,
                    span: synth_span,
                });
            }
            ImportedSymbol::Enum {
                namespace,
                original_name,
                variants,
                dc_id,
            } => {
                let key = (Some(namespace.clone()), original_name.clone());
                dc_nominals.insert(dc, key.clone());
                enums.entry(key).or_insert_with(|| EnumDef {
                    namespace: Some(namespace.clone()),
                    original_name: original_name.clone(),
                    variants: variants.clone(),
                    is_pub: true,
                    dc_id: *dc_id,
                    span: synth_span,
                });
            }
            ImportedSymbol::Value { .. } | ImportedSymbol::Routine { .. } => {}
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Canonicalizes transitive dc_import tables with full type registries"
)]
fn canonicalize_dc_import_tables(
    dc_imports: &mut HashMap<DcChar, ImportedSymbol>,
    dc_structs: &HashMap<DcChar, (Option<String>, String)>,
    dc_nominals: &HashMap<DcChar, (Option<String>, String)>,
    struct_defs: &mut BTreeMap<(Option<String>, String), StructDef>,
    brand_types: &mut BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: &mut BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: &mut BTreeMap<(Option<String>, String), EnumDef>,
) -> Result<()> {
    let empty_local_structs = HashMap::new();
    let empty_local_nominals = HashMap::new();
    let empty_top_dc = HashMap::new();
    let sp = Span { line: 1, col: 1 };

    let struct_keys: Vec<_> = struct_defs.keys().cloned().collect();
    for k in struct_keys {
        if let Some(existing) = struct_defs.get(&k).cloned() {
            let mut out = Vec::with_capacity(existing.fields.len());
            for f in &existing.fields {
                out.push(canonicalize_stager_type(
                    f,
                    &empty_local_structs,
                    dc_structs,
                    &empty_local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    &empty_top_dc,
                    sp,
                )?);
            }
            if let Some(slot) = struct_defs.get_mut(&k) {
                slot.fields = out;
            }
        }
    }

    let brand_keys: Vec<_> = brand_types.keys().cloned().collect();
    for k in brand_keys {
        if let Some(existing) = brand_types.get(&k).cloned() {
            let ci = canonicalize_stager_type(
                &existing.inner_ty,
                &empty_local_structs,
                dc_structs,
                &empty_local_nominals,
                dc_nominals,
                struct_defs,
                brand_types,
                named_structs,
                enums,
                &empty_top_dc,
                sp,
            )?;
            if let Some(slot) = brand_types.get_mut(&k) {
                slot.inner_ty = ci;
            }
        }
    }

    let ns_keys: Vec<_> = named_structs.keys().cloned().collect();
    for k in ns_keys {
        if let Some(existing) = named_structs.get(&k).cloned() {
            let mut out = Vec::with_capacity(existing.fields.len());
            for (fname, fty) in &existing.fields {
                let cf = canonicalize_stager_type(
                    fty,
                    &empty_local_structs,
                    dc_structs,
                    &empty_local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    &empty_top_dc,
                    sp,
                )?;
                out.push((fname.clone(), cf));
            }
            if let Some(slot) = named_structs.get_mut(&k) {
                slot.fields = out;
            }
        }
    }

    let enum_keys: Vec<_> = enums.keys().cloned().collect();
    for k in enum_keys {
        if let Some(existing) = enums.get(&k).cloned() {
            let mut out = Vec::with_capacity(existing.variants.len());
            for v in &existing.variants {
                let cp = match &v.payload {
                    Some(pty) => Some(canonicalize_stager_type(
                        pty,
                        &empty_local_structs,
                        dc_structs,
                        &empty_local_nominals,
                        dc_nominals,
                        struct_defs,
                        brand_types,
                        named_structs,
                        enums,
                        &empty_top_dc,
                        sp,
                    )?),
                    None => None,
                };
                out.push(EnumVariantDef {
                    name: v.name.clone(),
                    payload: cp,
                    span: v.span,
                });
            }
            if let Some(slot) = enums.get_mut(&k) {
                slot.variants = out;
            }
        }
    }

    for sym in dc_imports.values_mut() {
        match sym {
            ImportedSymbol::Value { ty, .. } => {
                *ty = canonicalize_stager_type(
                    ty,
                    &empty_local_structs,
                    dc_structs,
                    &empty_local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    &empty_top_dc,
                    sp,
                )?;
            }
            ImportedSymbol::Routine { sig, .. } => {
                sig.return_type = canonicalize_stager_type(
                    &sig.return_type,
                    &empty_local_structs,
                    dc_structs,
                    &empty_local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    &empty_top_dc,
                    sp,
                )?;
                let mut cp = Vec::with_capacity(sig.params.len());
                for p in &sig.params {
                    cp.push(canonicalize_stager_type(
                        p,
                        &empty_local_structs,
                        dc_structs,
                        &empty_local_nominals,
                        dc_nominals,
                        struct_defs,
                        brand_types,
                        named_structs,
                        enums,
                        &empty_top_dc,
                        sp,
                    )?);
                }
                sig.params = cp;
            }
            ImportedSymbol::Struct { fields, .. } => {
                let mut cf = Vec::with_capacity(fields.len());
                for f in fields.iter() {
                    cf.push(canonicalize_stager_type(
                        f,
                        &empty_local_structs,
                        dc_structs,
                        &empty_local_nominals,
                        dc_nominals,
                        struct_defs,
                        brand_types,
                        named_structs,
                        enums,
                        &empty_top_dc,
                        sp,
                    )?);
                }
                *fields = cf;
            }
            ImportedSymbol::BrandType { inner_ty, .. } => {
                *inner_ty = canonicalize_stager_type(
                    inner_ty,
                    &empty_local_structs,
                    dc_structs,
                    &empty_local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    &empty_top_dc,
                    sp,
                )?;
            }
            ImportedSymbol::NamedStruct { fields, .. } => {
                let mut cf = Vec::with_capacity(fields.len());
                for (fname, fty) in fields.iter() {
                    let c = canonicalize_stager_type(
                        fty,
                        &empty_local_structs,
                        dc_structs,
                        &empty_local_nominals,
                        dc_nominals,
                        struct_defs,
                        brand_types,
                        named_structs,
                        enums,
                        &empty_top_dc,
                        sp,
                    )?;
                    cf.push((fname.clone(), c));
                }
                *fields = cf;
            }
            ImportedSymbol::Enum { variants, .. } => {
                let mut cv = Vec::with_capacity(variants.len());
                for v in variants.iter() {
                    let cp = match &v.payload {
                        Some(pty) => Some(canonicalize_stager_type(
                            pty,
                            &empty_local_structs,
                            dc_structs,
                            &empty_local_nominals,
                            dc_nominals,
                            struct_defs,
                            brand_types,
                            named_structs,
                            enums,
                            &empty_top_dc,
                            sp,
                        )?),
                        None => None,
                    };
                    cv.push(EnumVariantDef {
                        name: v.name.clone(),
                        payload: cp,
                        span: v.span,
                    });
                }
                *variants = cv;
            }
        }
    }

    Ok(())
}

fn resolve_use_item_symbols(
    target_ns: &str,
    dep_exp: &NamespaceExports,
    item: &super::ast::UseImportItem,
    dc_imports: &mut HashMap<DcChar, ImportedSymbol>,
    dc_structs: &mut HashMap<DcChar, (Option<String>, String)>,
    dc_nominals: &mut HashMap<DcChar, (Option<String>, String)>,
) -> Result<Vec<ImportedSymbol>> {
    match &item.target {
        ImportTarget::Name(name) => {
            if let Some((ty, dc_id)) = dep_exp.values.get(name) {
                Ok(vec![ImportedSymbol::Value {
                    namespace: target_ns.to_owned(),
                    original_name: name.clone(),
                    ty: ty.clone(),
                    dc_id: *dc_id,
                }])
            } else if let Some(sig_list) = dep_exp.routines.get(name) {
                Ok(sig_list
                    .iter()
                    .map(|sig| ImportedSymbol::Routine {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        sig: sig.clone(),
                    })
                    .collect())
            } else if let Some(bdef) = dep_exp.brand_types.get(name) {
                Ok(vec![ImportedSymbol::BrandType {
                    namespace: target_ns.to_owned(),
                    original_name: name.clone(),
                    is_opaque: bdef.is_opaque,
                    inner_ty: bdef.inner_ty.clone(),
                    dc_id: bdef.dc_id,
                }])
            } else if let Some(nsdef) = dep_exp.named_structs.get(name) {
                Ok(vec![ImportedSymbol::NamedStruct {
                    namespace: target_ns.to_owned(),
                    original_name: name.clone(),
                    fields: nsdef.fields.clone(),
                    dc_id: nsdef.dc_id,
                }])
            } else if let Some(edef) = dep_exp.enums.get(name) {
                Ok(vec![ImportedSymbol::Enum {
                    namespace: target_ns.to_owned(),
                    original_name: name.clone(),
                    variants: edef.variants.clone(),
                    dc_id: edef.dc_id,
                }])
            } else if let Some(sdef) = dep_exp.structs.get(name) {
                Ok(vec![ImportedSymbol::Struct {
                    namespace: target_ns.to_owned(),
                    original_name: name.clone(),
                    fields: sdef.fields.clone(),
                    dc_id: sdef.dc_id,
                }])
            } else {
                bail!(
                    "Namespace '{target_ns}' has no exported 'pub' symbol named '{name}' (at {})",
                    item.span
                );
            }
        }
        ImportTarget::Dc(dc) => {
            let single = dep_exp.by_dc.get(dc).cloned().with_context(|| {
                format!(
                    "Namespace '{target_ns}' has no exported 'pub dc/{}' (at {})",
                    format_dc_shorthand(*dc),
                    item.span
                )
            })?;
            if item.alias.is_none() {
                dc_imports.insert(*dc, single.clone());
                match &single {
                    ImportedSymbol::Struct {
                        namespace,
                        original_name,
                        ..
                    }
                    | ImportedSymbol::NamedStruct {
                        namespace,
                        original_name,
                        ..
                    } => {
                        dc_structs.insert(
                            *dc,
                            (Some(namespace.clone()), original_name.clone()),
                        );
                    }
                    ImportedSymbol::BrandType {
                        namespace,
                        original_name,
                        ..
                    }
                    | ImportedSymbol::Enum {
                        namespace,
                        original_name,
                        ..
                    } => {
                        dc_nominals.insert(
                            *dc,
                            (Some(namespace.clone()), original_name.clone()),
                        );
                    }
                    ImportedSymbol::Value { .. }
                    | ImportedSymbol::Routine { .. } => {}
                }
            }
            Ok(vec![single])
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "Registers one resolved ImportedSymbol into module scope tables"
)]
fn register_imported_symbol(
    resolved: &ImportedSymbol,
    local_name: &str,
    span: Span,
    value_candidates: &mut HashMap<String, ResolvedValueTarget>,
    routine_candidates: &mut HashMap<String, Vec<ResolvedCallTarget>>,
    local_structs: &mut HashMap<String, (Option<String>, String)>,
    local_nominals: &mut HashMap<String, (Option<String>, String)>,
    struct_defs: &mut BTreeMap<(Option<String>, String), StructDef>,
    brand_types: &mut BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: &mut BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: &mut BTreeMap<(Option<String>, String), EnumDef>,
) -> Result<()> {
    let orig = imported_symbol_original_name(resolved);
    if is_stagel_internal_name(orig) {
        bail!(
            "StageL-internal symbol '{orig}' cannot be imported into StageR 3 module at {span}"
        );
    }
    match resolved {
        ImportedSymbol::Value {
            namespace,
            original_name,
            ty,
            dc_id,
        } => {
            ensure!(
                !value_candidates.contains_key(local_name)
                    && !routine_candidates.contains_key(local_name)
                    && !local_structs.contains_key(local_name)
                    && !local_nominals.contains_key(local_name),
                "Duplicate or conflicting imported value '{local_name}' at {span}"
            );
            value_candidates.insert(
                local_name.to_owned(),
                ResolvedValueTarget {
                    namespace: Some(namespace.clone()),
                    original_name: original_name.clone(),
                    ty: ty.clone(),
                    dc_id: *dc_id,
                },
            );
        }
        ImportedSymbol::Routine {
            namespace,
            original_name,
            sig,
        } => {
            ensure!(
                !value_candidates.contains_key(local_name)
                    && !local_structs.contains_key(local_name)
                    && !local_nominals.contains_key(local_name),
                "Imported routine '{local_name}' conflicts with value or type name at {span}"
            );
            let list = routine_candidates
                .entry(local_name.to_owned())
                .or_default();
            if let Some(first) = list.first() {
                ensure!(
                    first.sig.params.len() == sig.params.len(),
                    "Conflicting overload arity for imported routine '{local_name}' at {span}"
                );
                for prev in list.iter() {
                    ensure!(
                        prev.sig.params != sig.params,
                        "Duplicate imported routine overload '{local_name}({})' at {span}",
                        format_param_types(&sig.params)
                    );
                }
            }
            list.push(ResolvedCallTarget {
                namespace: Some(namespace.clone()),
                routine_name: original_name.clone(),
                sig: sig.clone(),
            });
        }
        ImportedSymbol::Struct {
            namespace,
            original_name,
            fields,
            dc_id,
        } => {
            ensure!(
                !name_conflicts_top(
                    local_name,
                    value_candidates,
                    routine_candidates,
                    local_structs,
                    local_nominals,
                ),
                "Duplicate or conflicting imported struct '{local_name}' at {span}"
            );
            let key = (Some(namespace.clone()), original_name.clone());
            local_structs.insert(local_name.to_owned(), key.clone());
            struct_defs.entry(key).or_insert_with(|| StructDef {
                namespace: Some(namespace.clone()),
                original_name: original_name.clone(),
                fields: fields.clone(),
                is_pub: true,
                dc_id: *dc_id,
                span,
            });
        }
        ImportedSymbol::BrandType {
            namespace,
            original_name,
            is_opaque,
            inner_ty,
            dc_id,
        } => {
            ensure!(
                !name_conflicts_top(
                    local_name,
                    value_candidates,
                    routine_candidates,
                    local_structs,
                    local_nominals,
                ),
                "Duplicate or conflicting imported nominal type '{local_name}' at {span}"
            );
            let key = (Some(namespace.clone()), original_name.clone());
            local_nominals.insert(local_name.to_owned(), key.clone());
            brand_types.entry(key).or_insert_with(|| BrandTypeDef {
                namespace: Some(namespace.clone()),
                original_name: original_name.clone(),
                is_opaque: *is_opaque,
                inner_ty: inner_ty.clone(),
                is_pub: true,
                dc_id: *dc_id,
                span,
            });
        }
        ImportedSymbol::NamedStruct {
            namespace,
            original_name,
            fields,
            dc_id,
        } => {
            ensure!(
                !name_conflicts_top(
                    local_name,
                    value_candidates,
                    routine_candidates,
                    local_structs,
                    local_nominals,
                ),
                "Duplicate or conflicting imported struct '{local_name}' at {span}"
            );
            let key = (Some(namespace.clone()), original_name.clone());
            local_structs.insert(local_name.to_owned(), key.clone());
            named_structs.entry(key).or_insert_with(|| NamedStructDef {
                namespace: Some(namespace.clone()),
                original_name: original_name.clone(),
                fields: fields.clone(),
                is_pub: true,
                dc_id: *dc_id,
                span,
            });
        }
        ImportedSymbol::Enum {
            namespace,
            original_name,
            variants,
            dc_id,
        } => {
            ensure!(
                !name_conflicts_top(
                    local_name,
                    value_candidates,
                    routine_candidates,
                    local_structs,
                    local_nominals,
                ),
                "Duplicate or conflicting imported enum '{local_name}' at {span}"
            );
            let key = (Some(namespace.clone()), original_name.clone());
            local_nominals.insert(local_name.to_owned(), key.clone());
            enums.entry(key).or_insert_with(|| EnumDef {
                namespace: Some(namespace.clone()),
                original_name: original_name.clone(),
                variants: variants.clone(),
                is_pub: true,
                dc_id: *dc_id,
                span,
            });
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "Canonicalizes syntactic struct/nominal refs across module type tables"
)]
fn canonicalize_stager_type(
    ty: &StageLType,
    local_structs: &HashMap<String, (Option<String>, String)>,
    dc_structs: &HashMap<DcChar, (Option<String>, String)>,
    local_nominals: &HashMap<String, (Option<String>, String)>,
    dc_nominals: &HashMap<DcChar, (Option<String>, String)>,
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
    brand_types: &BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: &BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: &BTreeMap<(Option<String>, String), EnumDef>,
    top_dc_names: &HashMap<DcChar, String>,
    span: Span,
) -> Result<StageLType> {
    match ty {
        StageLType::B
        | StageLType::U8
        | StageLType::I32
        | StageLType::U32
        | StageLType::I64
        | StageLType::U64
        | StageLType::I128
        | StageLType::U128
        | StageLType::Int
        | StageLType::Rat
        | StageLType::Dc
        | StageLType::Dcs
        | StageLType::S
        | StageLType::Error
        | StageLType::View
        | StageLType::Cursor
        | StageLType::Void
        | StageLType::Never => Ok(ty.clone()),
        StageLType::List(inner) => Ok(StageLType::List(Box::new(
            canonicalize_stager_type(
                inner,
                local_structs,
                dc_structs,
                local_nominals,
                dc_nominals,
                struct_defs,
                brand_types,
                named_structs,
                enums,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Buf(inner) => Ok(StageLType::Buf(Box::new(
            canonicalize_stager_type(
                inner,
                local_structs,
                dc_structs,
                local_nominals,
                dc_nominals,
                struct_defs,
                brand_types,
                named_structs,
                enums,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Option(inner) => Ok(StageLType::Option(Box::new(
            canonicalize_stager_type(
                inner,
                local_structs,
                dc_structs,
                local_nominals,
                dc_nominals,
                struct_defs,
                brand_types,
                named_structs,
                enums,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Result(inner) => Ok(StageLType::Result(Box::new(
            canonicalize_stager_type(
                inner,
                local_structs,
                dc_structs,
                local_nominals,
                dc_nominals,
                struct_defs,
                brand_types,
                named_structs,
                enums,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Tuple(elems) => {
            let mut out = Vec::with_capacity(elems.len());
            for el in elems {
                out.push(canonicalize_stager_type(
                    el,
                    local_structs,
                    dc_structs,
                    local_nominals,
                    dc_nominals,
                    struct_defs,
                    brand_types,
                    named_structs,
                    enums,
                    top_dc_names,
                    span,
                )?);
            }
            Ok(StageLType::Tuple(out))
        }
        StageLType::Struct(sref) => match sref {
            StructTypeRef::Ident(name) => {
                ensure!(
                    !is_stagel_internal_name(name),
                    "StageL-internal type '{name}' is forbidden in StageR 3 at {span}"
                );
                if let Some((ns, orig)) = local_structs.get(name) {
                    ensure!(
                        !is_stagel_internal_name(orig),
                        "StageL-internal type '{orig}' is forbidden in StageR 3 at {span}"
                    );
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else if let Some((ns, orig)) = local_nominals.get(name) {
                    ensure!(
                        !is_stagel_internal_name(orig),
                        "StageL-internal type '{orig}' is forbidden in StageR 3 at {span}"
                    );
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else {
                    bail!("Unknown StageR struct/nominal type '{name}' at {span}");
                }
            }
            StructTypeRef::Dc(dc) => {
                let sh = format_dc_shorthand(*dc);
                if let Some(orig) = top_dc_names.get(dc) {
                    ensure!(
                        !is_stagel_internal_name(orig),
                        "StageL-internal type '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
                    );
                }
                if let Some((ns, orig)) = dc_structs.get(dc) {
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else if let Some((ns, orig)) = dc_nominals.get(dc) {
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else {
                    bail!("Unknown StageR struct/nominal Dc reference 'dc/{sh}' at {span}");
                }
            }
            StructTypeRef::Canonical { namespace, name } => {
                let key = (namespace.clone(), name.clone());
                if named_structs.contains_key(&key)
                    || struct_defs.contains_key(&key)
                {
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: namespace.clone(),
                        name: name.clone(),
                    }))
                } else if brand_types.contains_key(&key)
                    || enums.contains_key(&key)
                {
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: namespace.clone(),
                        name: name.clone(),
                    }))
                } else {
                    bail!("Unknown canonical struct/nominal '{name}' at {span}");
                }
            }
        },
        StageLType::Nominal(nref) => match nref {
            StructTypeRef::Ident(name) => {
                ensure!(
                    !is_stagel_internal_name(name),
                    "StageL-internal type '{name}' is forbidden in StageR 3 at {span}"
                );
                if let Some((ns, orig)) = local_nominals.get(name) {
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else if let Some((ns, orig)) = local_structs.get(name) {
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else {
                    bail!("Unknown StageR nominal type '{name}' at {span}");
                }
            }
            StructTypeRef::Dc(dc) => {
                let sh = format_dc_shorthand(*dc);
                if let Some((ns, orig)) = dc_nominals.get(dc) {
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else if let Some((ns, orig)) = dc_structs.get(dc) {
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    }))
                } else {
                    bail!("Unknown StageR nominal Dc reference 'dc/{sh}' at {span}");
                }
            }
            StructTypeRef::Canonical { namespace, name } => {
                let key = (namespace.clone(), name.clone());
                if brand_types.contains_key(&key) || enums.contains_key(&key) {
                    Ok(StageLType::Nominal(StructTypeRef::Canonical {
                        namespace: namespace.clone(),
                        name: name.clone(),
                    }))
                } else if named_structs.contains_key(&key) {
                    Ok(StageLType::Struct(StructTypeRef::Canonical {
                        namespace: namespace.clone(),
                        name: name.clone(),
                    }))
                } else {
                    bail!("Unknown canonical nominal '{name}' at {span}");
                }
            }
        },
    }
}

fn check_stager_value_type_shape(ty: &StageLType, span: Span) -> Result<()> {
    ensure!(
        ty.can_be_element(),
        "Type '{ty}' at {span} is not valid as a stored value, field, parameter, or element type"
    );
    Ok(())
}

fn check_stager_unguarded_type_cycles(
    brand_types: &BTreeMap<(Option<String>, String), BrandTypeDef>,
    named_structs: &BTreeMap<(Option<String>, String), NamedStructDef>,
    enums: &BTreeMap<(Option<String>, String), EnumDef>,
) -> Result<()> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Visited,
    }

    fn collect_direct_edges(
        ty: &StageLType,
        out: &mut Vec<(Option<String>, String)>,
    ) {
        match ty {
            StageLType::Struct(StructTypeRef::Canonical { namespace, name })
            | StageLType::Nominal(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                out.push((namespace.clone(), name.clone()));
            }
            StageLType::Tuple(elems) => {
                for el in elems {
                    collect_direct_edges(el, out);
                }
            }
            StageLType::Option(inner) | StageLType::Result(inner) => {
                collect_direct_edges(inner, out);
            }
            // `List(T)` and `Buf(T)` are heap-indirected sequence headers.
            _ => {}
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Recursive cycle DFS over nominal brands, named structs, and enums"
    )]
    fn dfs(
        key: &(Option<String>, String),
        brand_types: &BTreeMap<(Option<String>, String), BrandTypeDef>,
        named_structs: &BTreeMap<(Option<String>, String), NamedStructDef>,
        enums: &BTreeMap<(Option<String>, String), EnumDef>,
        marks: &mut HashMap<(Option<String>, String), Mark>,
        stack: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(&m) = marks.get(key) {
            match m {
                Mark::Visited => return Ok(()),
                Mark::Visiting => {
                    stack.push(key.1.clone());
                    let sp = named_structs
                        .get(key)
                        .map(|d| d.span)
                        .or_else(|| brand_types.get(key).map(|d| d.span))
                        .or_else(|| enums.get(key).map(|d| d.span))
                        // Reason for fallback: Default span only if definition table is missing.
                        .unwrap_or(Span { line: 1, col: 1 });
                    bail!(
                        "Infinite-size unguarded recursive type cycle '{}' at {sp} (wrap recursive references in 'l/...')",
                        stack.join(" -> ")
                    );
                }
            }
        }
        marks.insert(key.clone(), Mark::Visiting);
        stack.push(key.1.clone());
        let mut edges = Vec::new();
        if let Some(nsdef) = named_structs.get(key) {
            for (_, fty) in &nsdef.fields {
                collect_direct_edges(fty, &mut edges);
            }
        }
        if let Some(bdef) = brand_types.get(key) {
            collect_direct_edges(&bdef.inner_ty, &mut edges);
        }
        if let Some(edef) = enums.get(key) {
            for v in &edef.variants {
                if let Some(ref pty) = v.payload {
                    collect_direct_edges(pty, &mut edges);
                }
            }
        }
        for target in edges {
            dfs(&target, brand_types, named_structs, enums, marks, stack)?;
        }
        let _ = stack.pop();
        marks.insert(key.clone(), Mark::Visited);
        Ok(())
    }

    let mut marks = HashMap::new();
    let mut stack = Vec::new();
    for key in named_structs
        .keys()
        .chain(brand_types.keys())
        .chain(enums.keys())
    {
        if !marks.contains_key(key) {
            dfs(
                key,
                brand_types,
                named_structs,
                enums,
                &mut marks,
                &mut stack,
            )?;
        }
    }
    Ok(())
}

impl<'a> StageRValidator<'a> {
    fn alloc_binding_id(&mut self) -> usize {
        let id = self.next_binding_id;
        self.next_binding_id = self.next_binding_id.saturating_add(1);
        id
    }

    fn lookup_local(&self, name: &str) -> Option<&BindingEntry> {
        for scope in self.local_scopes.iter().rev() {
            if let Some(e) = scope.get(name) {
                return Some(e);
            }
        }
        None
    }

    fn lookup_nested_routine(&self, name: &str) -> Option<usize> {
        for scope in self.routine_scopes.iter().rev() {
            if let Some(VerbTarget::Nested(id)) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }

    fn name_exists_in_local_or_top(&self, name: &str) -> bool {
        self.lookup_local(name).is_some()
            || self.lookup_nested_routine(name).is_some()
            || self.value_candidates.contains_key(name)
            || self.routine_candidates.contains_key(name)
            || self.local_structs.contains_key(name)
            || self.local_nominals.contains_key(name)
    }

    fn lookup_is_declared_in_scope(
        scope: Option<&HashMap<String, BindingEntry>>,
        id: usize,
    ) -> bool {
        scope.is_some_and(|s| s.values().any(|e| e.id == id))
    }

    fn canonicalize_type(
        &self,
        ty: &StageLType,
        span: Span,
    ) -> Result<StageLType> {
        canonicalize_stager_type(
            ty,
            &self.local_structs,
            &self.dc_structs,
            &self.local_nominals,
            &self.dc_nominals,
            &self.struct_defs,
            &self.brand_types,
            &self.named_structs,
            &self.enums,
            &self.top_dc_names,
            span,
        )
    }

    fn check_and_canonicalize_type(
        &self,
        ty: &StageLType,
        allow_direct_buf: bool,
        span: Span,
    ) -> Result<StageLType> {
        let canon = self.canonicalize_type(ty, span)?;
        match &canon {
            StageLType::Buf(elem) => {
                ensure!(
                    allow_direct_buf,
                    "'buf/T' type '{canon}' is not permitted in this position at {span}"
                );
                ensure!(
                    elem.can_be_element(),
                    "Buffer element type '{elem}' at {span} is invalid"
                );
            }
            StageLType::Void | StageLType::Never => {}
            other => {
                ensure!(
                    !other.is_or_contains_buf(),
                    "'buf/T' cannot be stored inside compound type '{other}' at {span}"
                );
            }
        }
        Ok(canon)
    }

    fn type_supports_eq(&self, ty: &StageLType) -> bool {
        let mut visiting = HashSet::new();
        self.type_supports_eq_rec(ty, &mut visiting)
    }

    fn type_supports_eq_rec(
        &self,
        ty: &StageLType,
        visiting: &mut HashSet<(Option<String>, String)>,
    ) -> bool {
        match ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat
            | StageLType::Dc
            | StageLType::Dcs
            | StageLType::S
            | StageLType::Error => true,
            StageLType::Void
            | StageLType::Never
            | StageLType::Buf(_)
            | StageLType::View
            | StageLType::Cursor => false,
            StageLType::List(inner)
            | StageLType::Option(inner)
            | StageLType::Result(inner) => {
                self.type_supports_eq_rec(inner, visiting)
            }
            StageLType::Tuple(elems) => elems
                .iter()
                .all(|e| self.type_supports_eq_rec(e, visiting)),
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace.clone(), name.clone());
                if !visiting.insert(key.clone()) {
                    return true;
                }
                let ok = if let Some(nsdef) = self.named_structs.get(&key) {
                    nsdef
                        .fields
                        .iter()
                        .all(|(_, fty)| self.type_supports_eq_rec(fty, visiting))
                } else if let Some(sdef) = self.struct_defs.get(&key) {
                    sdef.fields
                        .iter()
                        .all(|fty| self.type_supports_eq_rec(fty, visiting))
                } else {
                    false
                };
                visiting.remove(&key);
                ok
            }
            StageLType::Nominal(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace.clone(), name.clone());
                if !visiting.insert(key.clone()) {
                    return true;
                }
                let ok = if let Some(bdef) = self.brand_types.get(&key) {
                    self.type_supports_eq_rec(&bdef.inner_ty, visiting)
                } else if let Some(edef) = self.enums.get(&key) {
                    edef.variants.iter().all(|v| {
                        v.payload.as_ref().is_none_or(|pty| {
                            self.type_supports_eq_rec(pty, visiting)
                        })
                    })
                } else {
                    false
                };
                visiting.remove(&key);
                ok
            }
            StageLType::Struct(_) | StageLType::Nominal(_) => false,
        }
    }

    fn validate_top_values_acyclic(
        &mut self,
        values: &[&'a TopValueDecl],
    ) -> Result<()> {
        let mut by_name: HashMap<String, &'a TopValueDecl> = HashMap::new();
        let mut dc_to_name: HashMap<DcChar, String> = HashMap::new();
        for &v in values {
            if let Some(dc) = v.dc_id {
                dc_to_name.insert(dc, v.name.clone());
            }
            by_name.insert(v.name.clone(), v);
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Visiting,
            Done,
        }
        let mut visit: HashMap<String, State> = HashMap::new();
        let mut stack: Vec<String> = Vec::new();

        fn visit_const<'a>(
            v: &'a TopValueDecl,
            by_name: &HashMap<String, &'a TopValueDecl>,
            dc_to_name: &HashMap<DcChar, String>,
            validator: &mut StageRValidator<'a>,
            visit: &mut HashMap<String, State>,
            stack: &mut Vec<String>,
        ) -> Result<()> {
            let label = format!("{}/{}", v.ty, v.name);
            if let Some(st) = visit.get(&v.name) {
                match st {
                    State::Done => return Ok(()),
                    State::Visiting => {
                        stack.push(label);
                        bail!(
                            "Cyclic constant initializer dependency: {}",
                            stack.join(" -> ")
                        );
                    }
                }
            }
            visit.insert(v.name.clone(), State::Visiting);
            stack.push(label);

            let mut deps = Vec::new();
            collect_value_deps(&v.value, dc_to_name, &mut deps);
            for dep in deps {
                if let Some(&dep_decl) = by_name.get(&dep) {
                    visit_const(
                        dep_decl,
                        by_name,
                        dc_to_name,
                        validator,
                        visit,
                        stack,
                    )?;
                }
            }

            let _ = stack.pop();
            let expected_ty = validator.canonicalize_type(&v.ty, v.span)?;
            let mut state = FlowState::new();
            let mut ctx = StageRCtx {
                return_type: expected_ty.clone(),
                loop_depth: 0,
                in_tail: false,
                allow_buf: false,
                allow_propagate: false,
                outer_ids: HashSet::new(),
                loop_protected_bufs: HashSet::new(),
                captured_reads: HashSet::new(),
                always_written: HashSet::new(),
            };
            validator.check_const_purity(&v.value)?;
            let actual_ty =
                validator.check_expr(&v.value, &mut state, &mut ctx)?;
            ensure!(
                types_compatible(&actual_ty, &expected_ty),
                "Constant '{}' initializer at {} has type '{actual_ty}', expected '{expected_ty}'",
                v.name,
                v.span
            );
            visit.insert(v.name.clone(), State::Done);
            Ok(())
        }

        for &v in values {
            visit_const(
                v,
                &by_name,
                &dc_to_name,
                self,
                &mut visit,
                &mut stack,
            )?;
        }
        Ok(())
    }

    fn check_const_purity(&self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Bool { .. }
            | Expr::Int { .. }
            | Expr::Bytes { .. }
            | Expr::DctsString { .. }
            | Expr::DcLit { .. }
            | Expr::OptionNone { .. }
            | Expr::ErrorTag { .. }
            | Expr::ValueRef { .. }
            | Expr::Nop { .. } => Ok(()),
            Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
                for e in elements {
                    self.check_const_purity(e)?;
                }
                Ok(())
            }
            Expr::NamedStructNew { fields, .. } => {
                for (_, e, _) in fields {
                    self.check_const_purity(e)?;
                }
                Ok(())
            }
            Expr::EnumVariant { payload, .. } => {
                if let Some(e) = payload {
                    self.check_const_purity(e)?;
                }
                Ok(())
            }
            Expr::OptionSome { value, .. }
            | Expr::ResultOk { value, .. } => self.check_const_purity(value),
            Expr::ResultErr { error, .. } => self.check_const_purity(error),
            Expr::Field { base, .. }
            | Expr::NamedField { base, .. } => self.check_const_purity(base),
            Expr::With { base, value, .. }
            | Expr::NamedWith { base, value, .. } => {
                self.check_const_purity(base)?;
                self.check_const_purity(value)
            }
            Expr::Unary { op, operand, span } => {
                ensure!(
                    *op != CoreUnaryOp::BufFreeze,
                    "'bufFreeze' is disallowed in module-level constant initializer at {span}"
                );
                self.check_const_purity(operand)
            }
            Expr::StageRUnary { operand, .. } => {
                self.check_const_purity(operand)
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => {
                ensure!(
                    !matches!(
                        op,
                        CoreBinaryOp::BufFill
                            | CoreBinaryOp::BufPush
                            | CoreBinaryOp::BufExtend
                    ),
                    "Buffer operation '{}' is disallowed in module-level constant initializer at {span}",
                    op.keyword()
                );
                self.check_const_purity(left)?;
                self.check_const_purity(right)
            }
            Expr::StageRBinary { left, right, .. } => {
                self.check_const_purity(left)?;
                self.check_const_purity(right)
            }
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => {
                ensure!(
                    *op != CoreTernaryOp::BufSet,
                    "'bufSet' is disallowed in module-level constant initializer at {span}"
                );
                self.check_const_purity(first)?;
                self.check_const_purity(second)?;
                self.check_const_purity(third)
            }
            Expr::StageRTernary {
                op,
                first,
                second,
                third,
                span,
            } => {
                ensure!(
                    *op != StageRTernaryOp::TotalBufSet,
                    "'bufSet' is disallowed in module-level constant initializer at {span}"
                );
                self.check_const_purity(first)?;
                self.check_const_purity(second)?;
                self.check_const_purity(third)
            }
            Expr::TotalConv { expr, .. }
            | Expr::WrapNominal { expr, .. }
            | Expr::UnwrapNominal { expr, .. } => self.check_const_purity(expr),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                ..
            } => {
                self.check_const_purity(cond)?;
                self.check_const_purity(then_branch)?;
                self.check_const_purity(else_branch)
            }
            Expr::Match {
                scrutinee,
                arms,
                fallback,
                ..
            } => {
                self.check_const_purity(scrutinee)?;
                for arm in arms {
                    self.check_const_purity(&arm.body)?;
                }
                if let Some(fb) = fallback {
                    self.check_const_purity(fb)?;
                }
                Ok(())
            }
            other => bail!(
                "StageR 3 module-level constant initializer must be a pure constant expression (disallowed construct at {})",
                other.span()
            ),
        }
    }

    fn validate_top_routine(&mut self, r: &'a RoutineDecl) -> Result<()> {
        let canon_ret = self.canonicalize_type(&r.return_type, r.span)?;
        let mut canon_params = Vec::with_capacity(r.params.len());
        for p in &r.params {
            canon_params.push(self.canonicalize_type(&p.ty, p.span)?);
        }

        for tb in &r.tests {
            for tc in &tb.cases {
                ensure!(
                    tc.inputs.len() == r.params.len(),
                    "Test case for routine '{}' at {} passed {} input(s), expected {}",
                    r.name,
                    tc.span,
                    tc.inputs.len(),
                    r.params.len()
                );
                for ((inp_expr, param), pty) in
                    tc.inputs.iter().zip(&r.params).zip(&canon_params)
                {
                    let mut st = FlowState::new();
                    let mut ctx = StageRCtx {
                        return_type: pty.clone(),
                        loop_depth: 0,
                        in_tail: false,
                        allow_buf: true,
                        allow_propagate: false,
                        outer_ids: HashSet::new(),
                        loop_protected_bufs: HashSet::new(),
                        captured_reads: HashSet::new(),
                        always_written: HashSet::new(),
                    };
                    let ty = self.check_expr(inp_expr, &mut st, &mut ctx)?;
                    ensure!(
                        types_compatible(&ty, pty),
                        "Test input type mismatch at {} for parameter '{}' (expected '{}', got '{}')",
                        inp_expr.span(),
                        param.name,
                        pty,
                        ty
                    );
                }
                match &tc.expected {
                    TestExpected::Expr(exp_expr) => {
                        let mut st = FlowState::new();
                        let mut ctx = StageRCtx {
                            return_type: canon_ret.clone(),
                            loop_depth: 0,
                            in_tail: false,
                            allow_buf: true,
                            allow_propagate: false,
                            outer_ids: HashSet::new(),
                            loop_protected_bufs: HashSet::new(),
                            captured_reads: HashSet::new(),
                            always_written: HashSet::new(),
                        };
                        let ty =
                            self.check_expr(exp_expr, &mut st, &mut ctx)?;
                        ensure!(
                            types_compatible(&ty, &canon_ret),
                            "Test expected output type mismatch for '{}' at {} (expected '{}', got '{}')",
                            r.name,
                            exp_expr.span(),
                            canon_ret,
                            ty
                        );
                    }
                    TestExpected::Fault(sp) => {
                        bail!(
                            "'out fault' test expectation for '{}' at {sp} is forbidden in StageR 3 (StageR 3 is total)",
                            r.name
                        );
                    }
                    TestExpected::Void(sp) => {
                        ensure!(
                            canon_ret == StageLType::Void
                                || canon_ret == StageLType::Never,
                            "Omitted/void test output at {sp} is only valid for 'void' routines ('{}' returns '{}')",
                            r.name,
                            canon_ret
                        );
                    }
                }
            }
        }

        self.validate_routine_body(r, false)
    }

    fn validate_routine_body(
        &mut self,
        r: &RoutineDecl,
        is_nested: bool,
    ) -> Result<()> {
        let canon_ret = self.canonicalize_type(&r.return_type, r.span)?;
        let mut outer_ids = HashSet::new();
        if is_nested {
            for scope in &self.local_scopes {
                for entry in scope.values() {
                    outer_ids.insert(entry.id);
                }
            }
        }

        let mut param_scope = HashMap::new();
        let mut state = FlowState::new();
        if is_nested {
            for scope in &self.local_scopes {
                for entry in scope.values() {
                    state.assign.set_state(entry.id, true);
                }
            }
        }
        for p in &r.params {
            ensure!(
                !self.name_exists_in_local_or_top(&p.name)
                    && !param_scope.contains_key(&p.name),
                "Parameter '{}' of '{}' at {} shadows or duplicates an existing declaration",
                p.name,
                r.name,
                p.span
            );
            let pty = self.check_and_canonicalize_type(&p.ty, false, p.span)?;
            check_stager_value_type_shape(&pty, p.span)?;
            let id = self.alloc_binding_id();
            state.assign.set_state(id, true);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id,
                    ty: pty,
                    kind: BindingKind::Param,
                    span: p.span,
                },
            );
        }

        self.local_scopes.push(param_scope);
        let mut ctx = StageRCtx {
            return_type: canon_ret.clone(),
            loop_depth: 0,
            in_tail: true,
            allow_buf: true,
            allow_propagate: true,
            outer_ids,
            loop_protected_bufs: HashSet::new(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        };
        let actual_ty = self.check_expr(&r.body, &mut state, &mut ctx)?;
        let _ = self.local_scopes.pop();

        ensure!(
            types_compatible(&actual_ty, &canon_ret),
            "Routine '{}' body returns '{}' which is incompatible with declared return type '{}' at {}",
            r.name,
            actual_ty,
            canon_ret,
            r.span
        );
        Ok(())
    }

    fn validate_nested_routine_at_decl(
        &mut self,
        nid: usize,
        r: &RoutineDecl,
        enclosing_state: &FlowState,
    ) -> Result<()> {
        let canon_ret = self.canonicalize_type(&r.return_type, r.span)?;
        let mut outer_ids = HashSet::new();
        for sc in &self.local_scopes {
            for ent in sc.values() {
                outer_ids.insert(ent.id);
            }
        }
        let mut nested_state = enclosing_state.clone();
        for &id in &outer_ids {
            nested_state.assign.set_state(id, true);
        }
        let mut param_scope = HashMap::new();
        for p in &r.params {
            ensure!(
                !self.name_exists_in_local_or_top(&p.name)
                    && !param_scope.contains_key(&p.name),
                "Parameter '{}' of nested routine '{}' at {} shadows an enclosing binding",
                p.name,
                r.name,
                p.span
            );
            let pty = self.check_and_canonicalize_type(&p.ty, false, p.span)?;
            check_stager_value_type_shape(&pty, p.span)?;
            let pid = self.alloc_binding_id();
            nested_state.assign.set_state(pid, true);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id: pid,
                    ty: pty,
                    kind: BindingKind::Param,
                    span: p.span,
                },
            );
        }

        self.local_scopes.push(param_scope);
        let mut ctx = StageRCtx {
            return_type: canon_ret.clone(),
            loop_depth: 0,
            in_tail: true,
            allow_buf: true,
            allow_propagate: true,
            outer_ids,
            loop_protected_bufs: HashSet::new(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        };
        let body_ty = self.check_expr(&r.body, &mut nested_state, &mut ctx)?;
        let _ = self.local_scopes.pop();
        ensure!(
            types_compatible(&body_ty, &canon_ret),
            "Nested routine '{}' body returns '{}' (expected '{}') at {}",
            r.name,
            body_ty,
            canon_ret,
            r.span
        );
        if let Some(sum) = self.nested_summaries.get_mut(&nid) {
            sum.captured_reads = ctx.captured_reads;
            if canon_ret != StageLType::Never {
                sum.always_written = ctx.always_written;
            }
            sum.captures_resolved = true;
        }
        Ok(())
    }

    fn check_expr(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let ty = self.check_expr_inner(expr, state, ctx)?;
        let sp = expr.span();
        self.expr_types.insert((sp.line, sp.col), ty.clone());
        Ok(ty)
    }

    fn check_subexpr(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let mut child_ctx = ctx.child(false, ctx.loop_depth, ctx.allow_buf);
        let ty = self.check_expr(expr, state, &mut child_ctx)?;
        ctx.captured_reads.extend(child_ctx.captured_reads);
        ctx.always_written.extend(child_ctx.always_written);
        Ok(ty)
    }

    /// Evaluates a direct operand expected to denote an unfrozen local `buf/T`
    /// variable or a temporary `bufNew`/`bufFill` allocation. Returns
    /// `(type, Some(binding_id))` when referencing a local `buf` variable.
    fn check_buf_target(
        &mut self,
        expr: &Expr,
        op_name: &str,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<(StageLType, Option<usize>)> {
        if let Expr::ValueRef {
            ty,
            target: SymbolTarget::Ident(name),
            span,
        } = expr
        {
            let Some(entry) = self.lookup_local(name).cloned() else {
                bail!("Undeclared identifier '{name}' at {span}");
            };
            ensure!(
                !matches!(entry.ty, StageLType::Buf(_))
                    || !ctx.outer_ids.contains(&entry.id),
                "Linear 'buf' variable '{name}' cannot be captured by a nested routine at {span}"
            );
            if let Some(ann) = ty {
                let canon_ann =
                    self.check_and_canonicalize_type(ann, true, *span)?;
                ensure!(
                    entry.ty == canon_ann,
                    "Type annotation mismatch on '{name}' at {span}: declared '{}', referenced as '{canon_ann}'",
                    entry.ty
                );
            }
            ensure!(
                state.assign.is_assigned(entry.id),
                "Use of potentially uninitialized variable '{name}' at {span}"
            );
            if matches!(entry.ty, StageLType::Buf(_)) {
                let bstate = state
                    .bufs
                    .get(&entry.id)
                    .copied()
                    // Reason for fallback: Any buf binding missing from state is treated as unfrozen only at initial creation; otherwise require tracking entry.
                    .unwrap_or(BufState::MaybeFrozen);
                ensure!(
                    bstate == BufState::Unfrozen,
                    "Linear buffer '{name}' is already frozen or may have been frozen before '{op_name}' at {span}"
                );
                self.value_ref_types
                    .insert((span.line, span.col), entry.ty.clone());
                self.expr_types
                    .insert((span.line, span.col), entry.ty.clone());
                return Ok((entry.ty, Some(entry.id)));
            }
            if ctx.outer_ids.contains(&entry.id)
                && !ctx.always_written.contains(&entry.id)
            {
                ctx.captured_reads.insert(entry.id);
            }
            self.value_ref_types
                .insert((span.line, span.col), entry.ty.clone());
            self.expr_types
                .insert((span.line, span.col), entry.ty.clone());
            return Ok((entry.ty, None));
        }

        let ty = self.check_subexpr(expr, state, ctx)?;
        Ok((ty, None))
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Uniformly dispatches StageR 3 AST expression variants"
    )]
    fn check_expr_inner(
        &mut self,
        expr: &Expr,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match expr {
            Expr::Bool { .. } => Ok(StageLType::B),
            Expr::Int { value, .. } => Ok(value.stage_l_type()),
            Expr::Bytes { .. } => Ok(StageLType::List(Box::new(StageLType::U8))),
            Expr::DctsString { value, span, .. } => {
                super::stager_strings::validate_and_measure_s_byte_len(value)
                    .with_context(|| {
                        format!(
                            "Invalid mixed-mode string (s) framing in literal at {span}"
                        )
                    })?;
                Ok(StageLType::S)
            }
            Expr::DcLit { value, span } => {
                ensure!(
                    is_valid_dc_u128(value.to_u128()),
                    "Invalid Document Character scalar in literal at {span}"
                );
                Ok(StageLType::Dc)
            }
            Expr::ErrorTag { .. } => Ok(StageLType::Error),
            Expr::Nop { .. } => Ok(StageLType::Void),
            Expr::Break { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'break' at {span} is outside any enclosing loop"
                );
                Ok(StageLType::Never)
            }
            Expr::Continue { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'continue' at {span} is outside any enclosing loop"
                );
                Ok(StageLType::Never)
            }
            Expr::Fault { span } => {
                bail!("'fault' trap at {span} is forbidden in total StageR 3")
            }
            Expr::StructNew { span, .. } => {
                bail!(
                    "Positional StageL 'structNew' at {span} is forbidden in StageR 3 (use named-field 'structNew')"
                )
            }
            Expr::Conv { span, .. } => {
                bail!(
                    "Partial StageL 'conv' at {span} is invalid in StageR 3 (use StageR total 'conv')"
                )
            }
            Expr::Bitcast { span, .. } => {
                bail!("'bitcast' at {span} is forbidden in StageR 3")
            }
            Expr::List {
                elem_ty,
                elements,
                span,
            } => {
                let canon_elem =
                    self.check_and_canonicalize_type(elem_ty, false, *span)?;
                check_stager_value_type_shape(&canon_elem, *span)?;
                for el in elements {
                    let ety = self.check_subexpr(el, state, ctx)?;
                    ensure!(
                        types_compatible(&ety, &canon_elem),
                        "List element at {} has type '{ety}', expected '{canon_elem}'",
                        el.span()
                    );
                }
                Ok(StageLType::List(Box::new(canon_elem)))
            }
            Expr::TupleNew { elements, span } => {
                let mut tys = Vec::with_capacity(elements.len());
                for el in elements {
                    let ety = self.check_subexpr(el, state, ctx)?;
                    ensure!(
                        !ety.is_or_contains_buf(),
                        "'buf/T' cannot be stored inside tuple at {}",
                        el.span()
                    );
                    ensure!(
                        ety != StageLType::Void,
                        "Tuple element at {} cannot have type 'void'",
                        el.span()
                    );
                    tys.push(ety);
                }
                let _ = span;
                Ok(StageLType::Tuple(tys))
            }
            Expr::NamedStructNew {
                target,
                fields,
                span,
            } => self.check_named_struct_new(target, fields, *span, state, ctx),
            Expr::NamedField { base, field, span } => {
                self.check_named_field(base, field, *span, state, ctx)
            }
            Expr::NamedWith {
                base,
                field,
                value,
                span,
            } => self.check_named_with(base, field, value, *span, state, ctx),
            Expr::Field { index, base, span } => {
                let idx_usize = usize::try_from(*index)
                    .context("Field index exceeds usize")?;
                let bty = self.check_subexpr(base, state, ctx)?;
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Tuple(ref slots) => {
                        let slot_ty =
                            slots.get(idx_usize).cloned().with_context(|| {
                                format!(
                                    "Tuple field index {index} out of bounds for '{bty}' ({} slot(s)) at {span}",
                                    slots.len()
                                )
                            })?;
                        Ok(slot_ty)
                    }
                    other => bail!(
                        "Positional 'field' with index '{index}' requires a tuple operand in StageR 3, found '{other}' at {span}"
                    ),
                }
            }
            Expr::With {
                index,
                base,
                value,
                span,
            } => {
                let idx_usize = usize::try_from(*index)
                    .context("Field index exceeds usize")?;
                let bty = self.check_subexpr(base, state, ctx)?;
                let vty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    !vty.is_or_contains_buf(),
                    "'buf/T' cannot be stored in tuple at {}",
                    value.span()
                );
                match &bty {
                    StageLType::Never => {
                        ensure!(
                            vty != StageLType::Void,
                            "'with' replacement value at {} cannot have type 'void'",
                            value.span()
                        );
                        Ok(StageLType::Never)
                    }
                    StageLType::Tuple(slots) => {
                        let slot_ty = slots.get(idx_usize).with_context(|| {
                            format!(
                                "Tuple field index {index} out of bounds in 'with' for '{bty}' ({} slot(s)) at {span}",
                                slots.len()
                            )
                        })?;
                        ensure!(
                            types_compatible(&vty, slot_ty),
                            "'with' field {index} type mismatch at {}: expected '{slot_ty}', found '{vty}'",
                            value.span()
                        );
                        Ok(bty)
                    }
                    other => bail!(
                        "Positional 'with' with index '{index}' requires a tuple operand in StageR 3, found '{other}' at {span}"
                    ),
                }
            }
            Expr::EnumVariant {
                enum_ty,
                variant,
                payload,
                span,
            } => self.check_enum_variant(
                enum_ty,
                variant,
                payload.as_deref(),
                *span,
                state,
                ctx,
            ),
            Expr::OptionNone { inner_ty, span } => {
                let canon_inner =
                    self.check_and_canonicalize_type(inner_ty, false, *span)?;
                check_stager_value_type_shape(&canon_inner, *span)?;
                Ok(StageLType::Option(Box::new(canon_inner)))
            }
            Expr::OptionSome {
                inner_ty,
                value,
                span,
            } => {
                let canon_inner =
                    self.check_and_canonicalize_type(inner_ty, false, *span)?;
                check_stager_value_type_shape(&canon_inner, *span)?;
                let vty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    types_compatible(&vty, &canon_inner),
                    "'option/T:some' payload at {} has type '{vty}', expected '{canon_inner}'",
                    value.span()
                );
                Ok(StageLType::Option(Box::new(canon_inner)))
            }
            Expr::ResultOk { ok_ty, value, span } => {
                let canon_ok =
                    self.check_and_canonicalize_type(ok_ty, false, *span)?;
                check_stager_value_type_shape(&canon_ok, *span)?;
                let vty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    types_compatible(&vty, &canon_ok),
                    "'result/T:ok' payload at {} has type '{vty}', expected '{canon_ok}'",
                    value.span()
                );
                Ok(StageLType::Result(Box::new(canon_ok)))
            }
            Expr::ResultErr { ok_ty, error, span } => {
                let canon_ok =
                    self.check_and_canonicalize_type(ok_ty, false, *span)?;
                check_stager_value_type_shape(&canon_ok, *span)?;
                let ety = self.check_subexpr(error, state, ctx)?;
                ensure!(
                    types_compatible(&ety, &StageLType::Error),
                    "'result/T:err' error operand at {} has type '{ety}', expected 'error'",
                    error.span()
                );
                Ok(StageLType::Result(Box::new(canon_ok)))
            }
            Expr::WrapNominal {
                expr: inner,
                target_ty,
                span,
            } => {
                let canon_target =
                    self.check_and_canonicalize_type(target_ty, false, *span)?;
                let StageLType::Nominal(StructTypeRef::Canonical {
                    namespace,
                    name,
                }) = &canon_target
                else {
                    bail!(
                        "'wrap' target '{canon_target}' at {span} must be a declared 'brand' or 'type'"
                    );
                };
                let key = (namespace.clone(), name.clone());
                let bdef = self
                    .brand_types
                    .get(&key)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "'wrap' at {span} requires a 'brand' or 'type' nominal type ('{name}' is not a brand/type)"
                        )
                    })?;
                if bdef.is_opaque {
                    ensure!(
                        self.local_declared_opaque.contains(&key),
                        "Cannot construct module-opaque type '{}' outside its defining module at {span}",
                        bdef.original_name
                    );
                }
                let vty = self.check_subexpr(inner, state, ctx)?;
                ensure!(
                    types_compatible(&vty, &bdef.inner_ty),
                    "'wrap' operand at {} has type '{vty}', expected '{}' for '{}'",
                    inner.span(),
                    bdef.inner_ty,
                    bdef.original_name
                );
                Ok(canon_target)
            }
            Expr::UnwrapNominal { expr: inner, span } => {
                let vty = self.check_subexpr(inner, state, ctx)?;
                let StageLType::Nominal(StructTypeRef::Canonical {
                    namespace,
                    name,
                }) = vty
                else {
                    bail!(
                        "'unwrap' operand at {span} must be a declared 'brand' or 'type', found '{vty}'"
                    );
                };
                let key = (namespace, name);
                let bdef = self
                    .brand_types
                    .get(&key)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "'unwrap' at {span} requires a 'brand' or 'type' nominal value"
                        )
                    })?;
                if bdef.is_opaque {
                    ensure!(
                        self.local_declared_opaque.contains(&key),
                        "Cannot unwrap module-opaque type '{}' outside its defining module at {span}",
                        bdef.original_name
                    );
                }
                Ok(bdef.inner_ty)
            }
            Expr::BufNew {
                elem_ty,
                capacity,
                span,
            } => {
                ensure!(
                    ctx.allow_buf,
                    "'bufNew' is disallowed in constant context at {span}"
                );
                let canon_elem =
                    self.check_and_canonicalize_type(elem_ty, false, *span)?;
                check_stager_value_type_shape(&canon_elem, *span)?;
                let cap_ty = self.check_subexpr(capacity, state, ctx)?;
                ensure!(
                    types_compatible(&cap_ty, &StageLType::U64),
                    "'bufNew' capacity at {} must have type 'u64', found '{cap_ty}'",
                    capacity.span()
                );
                Ok(StageLType::Buf(Box::new(canon_elem)))
            }
            Expr::ValueRef { ty, target, span } => {
                self.check_value_ref(ty.as_ref(), target, *span, state, ctx)
            }
            Expr::Let {
                ty,
                name,
                init,
                span,
            } => self.check_let(ty, name, init.as_deref(), *span, state, ctx),
            Expr::Const {
                ty,
                name,
                value,
                span,
                ..
            } => self.check_local_const(ty, name, value, *span, state, ctx),
            Expr::Set {
                ty,
                name,
                value,
                span,
            } => self.check_set(ty.as_ref(), name, value, *span, state, ctx),
            Expr::RoutineDecl(r) => {
                ensure!(
                    !is_stagel_internal_name(&r.name),
                    "StageL-internal nested routine '{}' cannot be declared in StageR 3 at {}",
                    r.name,
                    r.span
                );
                ensure!(
                    !r.is_pub && !r.pub_dc_syntax && r.dc_id.is_none(),
                    "Nested routine '{}' at {} cannot carry 'pub' or Dc annotations",
                    r.name,
                    r.span
                );
                let nid = self
                    .lookup_nested_routine(&r.name)
                    .with_context(|| {
                        format!(
                            "Nested routine '{}' at {} missing from enclosing block signature table",
                            r.name,
                            r.span
                        )
                    })?;
                self.validate_nested_routine_at_decl(nid, r, state)?;
                Ok(StageLType::Void)
            }
            Expr::Call { target, args, span } => {
                self.check_call(target, args, *span, state, ctx)
            }
            Expr::Block { stmts, .. } => self.check_block(stmts, state, ctx),
            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                self.check_if(cond, then_branch, else_branch, *span, state, ctx)
            }
            Expr::While { cond, body, span } => {
                self.check_while(cond, body, *span, state, ctx)
            }
            Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                span,
            } => self.check_foreach(
                elem_ty,
                elem_name,
                list,
                body,
                *span,
                state,
                ctx,
            ),
            Expr::Match {
                scrutinee,
                arms,
                fallback,
                span,
            } => self.check_match(
                scrutinee,
                arms,
                fallback.as_deref(),
                *span,
                state,
                ctx,
            ),
            Expr::Propagate { expr: inner, span } => {
                self.check_propagate(inner, *span, state, ctx)
            }
            Expr::Return { value, span } => {
                self.check_return(value.as_deref(), *span, state, ctx)
            }
            Expr::Unary { op, operand, span } => {
                self.check_core_unary(*op, operand, *span, state, ctx)
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => self.check_core_binary(*op, left, right, *span, state, ctx),
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => self.check_core_ternary(
                *op,
                first,
                second,
                third,
                *span,
                state,
                ctx,
            ),
            Expr::StageRUnary { op, operand, span } => {
                self.check_stager_unary(*op, operand, *span, state, ctx)
            }
            Expr::StageRBinary {
                op,
                left,
                right,
                span,
            } => self.check_stager_binary(*op, left, right, *span, state, ctx),
            Expr::StageRTernary {
                op,
                first,
                second,
                third,
                span,
            } => self.check_stager_ternary(
                *op,
                first,
                second,
                third,
                *span,
                state,
                ctx,
            ),
            Expr::TotalConv {
                expr: inner,
                target_ty,
                span,
            } => self.check_total_conv(inner, target_ty, *span, state, ctx),
        }
    }

    fn check_named_struct_new(
        &mut self,
        target: &SymbolTarget,
        fields: &[(String, Expr, Span)],
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let key = match target {
            SymbolTarget::Ident(name) => {
                ensure!(
                    !is_stagel_internal_name(name),
                    "StageL-internal struct '{name}' cannot be constructed in StageR 3 at {span}"
                );
                self.local_structs.get(name).cloned().with_context(|| {
                    format!("Unknown named struct '{name}' in 'structNew' at {span}")
                })?
            }
            SymbolTarget::Dc(dc) => {
                let sh = format_dc_shorthand(*dc);
                self.dc_structs.get(dc).cloned().with_context(|| {
                    format!("Unknown named struct Dc 'dc/{sh}' in 'structNew' at {span}")
                })?
            }
        };
        let sdef = self
            .named_structs
            .get(&key)
            .cloned()
            .with_context(|| {
                format!(
                    "'{}' at {span} is not a StageR 3 named-field struct",
                    key.1
                )
            })?;
        let mut expected_map: HashMap<&str, &StageLType> =
            HashMap::with_capacity(sdef.fields.len());
        for (fname, fty) in &sdef.fields {
            expected_map.insert(fname.as_str(), fty);
        }
        let mut provided: HashSet<&str> = HashSet::with_capacity(fields.len());
        for (fname, fexpr, fspan) in fields {
            let expected_fty =
                expected_map.get(fname.as_str()).with_context(|| {
                    format!(
                        "Unknown field '{fname}' on struct '{}' at {fspan}",
                        sdef.original_name
                    )
                })?;
            ensure!(
                provided.insert(fname.as_str()),
                "Duplicate initializer for field '{fname}' in struct '{}' at {fspan}",
                sdef.original_name
            );
            let fty = self.check_subexpr(fexpr, state, ctx)?;
            ensure!(
                types_compatible(&fty, expected_fty),
                "Field '{fname}' on struct '{}' at {fspan} has type '{fty}', expected '{}'",
                sdef.original_name,
                expected_fty
            );
        }
        if provided.len() != sdef.fields.len() {
            let missing: Vec<&str> = sdef
                .fields
                .iter()
                .filter(|(n, _)| !provided.contains(n.as_str()))
                .map(|(n, _)| n.as_str())
                .collect();
            bail!(
                "Missing field(s) {} in 'structNew {}' at {span}",
                missing.join(", "),
                sdef.original_name
            );
        }
        let struct_ty = StageLType::Struct(StructTypeRef::Canonical {
            namespace: sdef.namespace.clone(),
            name: sdef.original_name.clone(),
        });
        self.resolved_named_structs.insert((span.line, span.col), sdef);
        Ok(struct_ty)
    }

    fn check_named_field(
        &mut self,
        base: &Expr,
        field: &str,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let bty = self.check_subexpr(base, state, ctx)?;
        match bty {
            StageLType::Never => Ok(StageLType::Never),
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace, name);
                let sdef = self
                    .named_structs
                    .get(&key)
                    .with_context(|| {
                        format!(
                            "Named 'field {field}' requires a StageR named-field struct at {span}"
                        )
                    })?;
                let (_, fty) = sdef
                    .fields
                    .iter()
                    .find(|(n, _)| n == field)
                    .with_context(|| {
                        format!(
                            "Struct '{}' has no field named '{field}' at {span}",
                            sdef.original_name
                        )
                    })?;
                Ok(fty.clone())
            }
            other => bail!(
                "Named 'field {field}' requires a named-field struct operand, found '{other}' at {span}"
            ),
        }
    }

    fn check_named_with(
        &mut self,
        base: &Expr,
        field: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let bty = self.check_subexpr(base, state, ctx)?;
        let vty = self.check_subexpr(value, state, ctx)?;
        ensure!(
            !vty.is_or_contains_buf(),
            "'buf/T' cannot be stored via 'with' at {}",
            value.span()
        );
        match &bty {
            StageLType::Never => {
                ensure!(
                    vty != StageLType::Void,
                    "'with' replacement value at {} cannot have type 'void'",
                    value.span()
                );
                Ok(StageLType::Never)
            }
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace.clone(), name.clone());
                let sdef = self
                    .named_structs
                    .get(&key)
                    .with_context(|| {
                        format!(
                            "Named 'with {field}' requires a StageR named-field struct at {span}"
                        )
                    })?;
                let (_, expected_fty) = sdef
                    .fields
                    .iter()
                    .find(|(n, _)| n == field)
                    .with_context(|| {
                        format!(
                            "Struct '{}' has no field named '{field}' at {span}",
                            sdef.original_name
                        )
                    })?;
                ensure!(
                    types_compatible(&vty, expected_fty),
                    "'with {field}' type mismatch at {}: expected '{expected_fty}', found '{vty}'",
                    value.span()
                );
                Ok(bty)
            }
            other => bail!(
                "Named 'with {field}' requires a named-field struct operand, found '{other}' at {span}"
            ),
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Validates closed enum variant constructor and optional payload"
    )]
    fn check_enum_variant(
        &mut self,
        enum_ty: &StructTypeRef,
        variant: &str,
        payload: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let canon = self.check_and_canonicalize_type(
            &StageLType::Nominal(enum_ty.clone()),
            false,
            span,
        )?;
        let StageLType::Nominal(StructTypeRef::Canonical {
            ref namespace,
            ref name,
        }) = canon
        else {
            bail!("Target '{canon}' at {span} is not a declared enum");
        };
        let key = (namespace.clone(), name.clone());
        let edef = self.enums.get(&key).cloned().with_context(|| {
            format!("'{name}' at {span} is not a StageR 'enum'")
        })?;
        let vdef = edef
            .variants
            .iter()
            .find(|v| v.name == variant)
            .with_context(|| {
                format!(
                    "Enum '{}' has no variant '{variant}' at {span}",
                    edef.original_name
                )
            })?;
        match (&vdef.payload, payload) {
            (None, None) => {}
            (None, Some(p)) => {
                bail!(
                    "Unit enum variant '{}:{variant}' takes no payload at {}",
                    edef.original_name,
                    p.span()
                );
            }
            (Some(exp_ty), None) => {
                bail!(
                    "Enum variant '{}:{variant}' requires payload of type '{exp_ty}' at {span}",
                    edef.original_name
                );
            }
            (Some(exp_ty), Some(p)) => {
                let pty = self.check_subexpr(p, state, ctx)?;
                ensure!(
                    types_compatible(&pty, exp_ty),
                    "Payload type mismatch for '{}:{variant}' at {}: expected '{exp_ty}', found '{pty}'",
                    edef.original_name,
                    p.span()
                );
            }
        }
        self.resolved_enum_variants
            .insert((span.line, span.col), (key, variant.to_owned()));
        Ok(canon)
    }

    fn check_value_ref(
        &mut self,
        ty: Option<&StageLType>,
        target: &SymbolTarget,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let canon_expected = match ty {
            Some(t) => Some(self.check_and_canonicalize_type(t, true, span)?),
            None => None,
        };
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(entry) = self.lookup_local(name).cloned() {
                    ensure!(
                        !matches!(entry.ty, StageLType::Buf(_)),
                        "Linear 'buf/T' variable '{name}' cannot be moved, aliased, captured, returned, or stored outside direct buffer operations at {span}"
                    );
                    if let Some(ref exp) = canon_expected {
                        ensure!(
                            &entry.ty == exp,
                            "Type annotation mismatch on '{name}' at {span}: declared '{}', referenced as '{exp}'",
                            entry.ty
                        );
                    }
                    ensure!(
                        state.assign.is_assigned(entry.id),
                        "Use of potentially uninitialized variable '{name}' at {span}"
                    );
                    if ctx.outer_ids.contains(&entry.id)
                        && !ctx.always_written.contains(&entry.id)
                    {
                        ctx.captured_reads.insert(entry.id);
                    }
                    self.value_ref_types
                        .insert((span.line, span.col), entry.ty.clone());
                    return Ok(entry.ty);
                }
                if let Some(cand) = self.value_candidates.get(name).cloned() {
                    if let Some(ref exp) = canon_expected {
                        ensure!(
                            &cand.ty == exp,
                            "Type annotation mismatch on constant '{name}' at {span}: declared '{}', referenced as '{exp}'",
                            cand.ty
                        );
                    }
                    let cty = cand.ty.clone();
                    self.resolved_values.insert((span.line, span.col), cand);
                    self.value_ref_types
                        .insert((span.line, span.col), cty.clone());
                    return Ok(cty);
                }
                bail!("Undeclared identifier '{name}' at {span}");
            }
            SymbolTarget::Dc(dc) => {
                let sh = format_dc_shorthand(*dc);
                if let Some(orig) = self.top_dc_names.get(dc)
                    && is_stagel_internal_name(orig)
                {
                    bail!(
                        "StageL-internal symbol '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
                    );
                }
                if let Some(v_ty) = self.top_dc_values.get(dc) {
                    if let Some(ref exp) = canon_expected {
                        ensure!(
                            v_ty == exp,
                            "Type mismatch on Dc reference 'dc/{sh}' at {span}"
                        );
                    }
                    self.value_ref_types
                        .insert((span.line, span.col), v_ty.clone());
                    return Ok(v_ty.clone());
                }
                if let Some(ImportedSymbol::Value {
                    original_name,
                    ty: imp_ty,
                    ..
                }) = self.dc_imports.get(dc)
                {
                    ensure!(
                        !is_stagel_internal_name(original_name),
                        "StageL-internal symbol '{original_name}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
                    );
                    let canon_imp = self.canonicalize_type(imp_ty, span)?;
                    if let Some(ref exp) = canon_expected {
                        ensure!(
                            &canon_imp == exp,
                            "Type mismatch on imported Dc reference 'dc/{sh}' at {span}"
                        );
                    }
                    self.value_ref_types
                        .insert((span.line, span.col), canon_imp.clone());
                    return Ok(canon_imp);
                }
                bail!("Unresolved value Dc reference 'dc/{sh}' at {span}");
            }
        }
    }

    fn check_let(
        &mut self,
        ty: &StageLType,
        name: &str,
        init: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        ensure!(
            !is_stagel_internal_name(name),
            "StageL-internal variable '{name}' cannot be declared in StageR 3 at {span}"
        );
        ensure!(
            *ty != StageLType::Void && *ty != StageLType::Never,
            "'let' variable '{name}' at {span} cannot have type '{ty}'"
        );
        let canon_ty = self.check_and_canonicalize_type(ty, true, span)?;
        ensure!(
            !self.name_exists_in_local_or_top(name),
            "Cannot re-declare or shadow '{name}' with 'let' at {span} (use 'set' to mutate)"
        );

        let is_buf = matches!(canon_ty, StageLType::Buf(_));
        let is_init = if is_buf {
            let init_expr = init.with_context(|| {
                format!(
                    "'let {canon_ty}/{name}' at {span} must immediately initialize a fresh 'bufNew' or 'bufFill' allocation (linear 'buf/T' cannot be uninitialized)"
                )
            })?;
            ensure!(
                matches!(
                    init_expr,
                    Expr::BufNew { .. }
                        | Expr::Binary {
                            op: CoreBinaryOp::BufFill,
                            ..
                        }
                ),
                "Linear 'let {canon_ty}/{name}' initializer at {} must be a fresh 'bufNew' or 'bufFill' allocation (aliasing is forbidden)",
                init_expr.span()
            );
            let rhs_ty = self.check_subexpr(init_expr, state, ctx)?;
            ensure!(
                types_compatible(&rhs_ty, &canon_ty),
                "Initializer type mismatch for 'let {canon_ty}/{name}' at {span}: expected '{canon_ty}', found '{rhs_ty}'"
            );
            true
        } else if let Some(init_expr) = init {
            let rhs_ty = self.check_subexpr(init_expr, state, ctx)?;
            ensure!(
                types_compatible(&rhs_ty, &canon_ty),
                "Initializer type mismatch for 'let {canon_ty}/{name}' at {span}: expected '{canon_ty}', found '{rhs_ty}'"
            );
            true
        } else {
            false
        };

        let id = self.alloc_binding_id();
        state.assign.set_state(id, is_init);
        if is_buf {
            state.bufs.insert(id, BufState::Unfrozen);
        }
        let scope = self
            .local_scopes
            .last_mut()
            .context("'let' is only allowed inside routine/code blocks")?;
        scope.insert(
            name.to_owned(),
            BindingEntry {
                id,
                ty: canon_ty,
                kind: BindingKind::MutableLet,
                span,
            },
        );
        Ok(StageLType::Void)
    }

    fn check_local_const(
        &mut self,
        ty: &StageLType,
        name: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        ensure!(
            !is_stagel_internal_name(name),
            "StageL-internal constant '{name}' cannot be declared in StageR 3 at {span}"
        );
        ensure!(
            *ty != StageLType::Void && *ty != StageLType::Never,
            "'const' binding '{name}' at {span} cannot have type '{ty}'"
        );
        let canon_ty = self.check_and_canonicalize_type(ty, false, span)?;
        ensure!(
            !self.name_exists_in_local_or_top(name),
            "Cannot re-declare or shadow '{name}' with 'const' at {span}"
        );
        let rhs_ty = self.check_subexpr(value, state, ctx)?;
        ensure!(
            types_compatible(&rhs_ty, &canon_ty),
            "Initializer type mismatch for 'const {canon_ty}/{name}' at {span}: expected '{canon_ty}', found '{rhs_ty}'"
        );
        let id = self.alloc_binding_id();
        state.assign.set_state(id, true);
        let scope = self
            .local_scopes
            .last_mut()
            .context("Local 'const' requires an enclosing block")?;
        scope.insert(
            name.to_owned(),
            BindingEntry {
                id,
                ty: canon_ty,
                kind: BindingKind::Const,
                span,
            },
        );
        Ok(StageLType::Void)
    }

    fn check_set(
        &mut self,
        ty: Option<&StageLType>,
        name: &str,
        value: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let canon_expected = match ty {
            Some(t) => Some(self.check_and_canonicalize_type(t, false, span)?),
            None => None,
        };
        let entry = self
            .lookup_local(name)
            .cloned()
            .with_context(|| {
                format!(
                    "Cannot 'set' undeclared variable '{name}' at {span} (must declare with 'let' first)"
                )
            })?;
        ensure!(
            !matches!(entry.ty, StageLType::Buf(_)),
            "Cannot re-assign linear buffer '{name}' with 'set' at {span}"
        );
        ensure!(
            matches!(entry.kind, BindingKind::MutableLet),
            "Cannot 'set' immutable binding '{name}' ({:?}) at {span}",
            entry.kind
        );
        if let Some(ref exp) = canon_expected {
            ensure!(
                &entry.ty == exp,
                "Type mismatch in 'set {exp}/{name}' at {span}: declared type is '{}'",
                entry.ty
            );
        }
        let rhs_ty = self.check_subexpr(value, state, ctx)?;
        ensure!(
            types_compatible(&rhs_ty, &entry.ty),
            "Assigned value type mismatch in 'set' on '{name}' at {span}: expected '{}', found '{rhs_ty}'",
            entry.ty
        );
        state.assign.set_state(entry.id, true);
        ctx.always_written.insert(entry.id);
        Ok(StageLType::Void)
    }

    fn check_call(
        &mut self,
        target: &SymbolTarget,
        args: &[Expr],
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match target {
            SymbolTarget::Ident(name) => {
                if let Some(nid) = self.lookup_nested_routine(name) {
                    let summary = self
                        .nested_summaries
                        .get(&nid)
                        .cloned()
                        .with_context(|| {
                            format!("Missing nested verb summary for '{name}' at {span}")
                        })?;
                    self.pending_nested_calls
                        .push((nid, span, state.assign.clone()));
                    if summary.captures_resolved {
                        for cap_id in &summary.captured_reads {
                            if !state.assign.is_assigned(*cap_id) {
                                bail!(
                                    "Calling nested verb '{name}' at {span} before captured 'let' variable (binding #{cap_id}) is definitely assigned"
                                );
                            }
                            if ctx.outer_ids.contains(cap_id)
                                && !ctx.always_written.contains(cap_id)
                            {
                                ctx.captured_reads.insert(*cap_id);
                            }
                        }
                        for written_id in &summary.always_written {
                            state.assign.set_state(*written_id, true);
                            ctx.always_written.insert(*written_id);
                        }
                    }
                    let sig = summary.sig;
                    ensure!(
                        args.len() == sig.params.len(),
                        "Wrong argument count in call at {span}: expected {}, got {}",
                        sig.params.len(),
                        args.len()
                    );
                    for (arg_expr, param_ty) in args.iter().zip(&sig.params) {
                        let aty = self.check_subexpr(arg_expr, state, ctx)?;
                        ensure!(
                            types_compatible(&aty, param_ty),
                            "Call argument type mismatch at {}: expected '{param_ty}', found '{aty}'",
                            arg_expr.span()
                        );
                    }
                    if ctx.in_tail {
                        self.tail_calls.insert((span.line, span.col));
                    }
                    return Ok(sig.return_type);
                }

                let cands = self
                    .routine_candidates
                    .get(name)
                    .cloned()
                    .with_context(|| format!("Unknown routine '{name}' at {span}"))?;
                let first_cand = cands
                    .first()
                    .context("Empty routine candidate list")?;
                let expected_arity = first_cand.sig.params.len();
                ensure!(
                    args.len() == expected_arity,
                    "Wrong argument count in call at {span}: expected {expected_arity}, got {}",
                    args.len()
                );

                if cands.len() == 1 {
                    for (arg_expr, param_ty) in
                        args.iter().zip(&first_cand.sig.params)
                    {
                        let aty = self.check_subexpr(arg_expr, state, ctx)?;
                        ensure!(
                            types_compatible(&aty, param_ty),
                            "Call argument type mismatch at {}: expected '{param_ty}', found '{aty}'",
                            arg_expr.span()
                        );
                    }
                    let ret_ty = first_cand.sig.return_type.clone();
                    self.resolved_calls
                        .insert((span.line, span.col), first_cand.clone());
                    if ctx.in_tail {
                        self.tail_calls.insert((span.line, span.col));
                    }
                    return Ok(ret_ty);
                }

                let mut arg_tys = Vec::with_capacity(args.len());
                for arg_expr in args {
                    arg_tys.push(self.check_subexpr(arg_expr, state, ctx)?);
                }
                let matches: Vec<&ResolvedCallTarget> = cands
                    .iter()
                    .filter(|c| {
                        arg_tys
                            .iter()
                            .zip(&c.sig.params)
                            .all(|(aty, pty)| types_compatible(aty, pty))
                    })
                    .collect();
                if matches.is_empty() {
                    let avail: Vec<String> = cands
                        .iter()
                        .map(|c| format!("({})", format_param_types(&c.sig.params)))
                        .collect();
                    bail!(
                        "No matching overload for routine '{name}' at {span} with argument types ({}) (available: {})",
                        format_param_types(&arg_tys),
                        avail.join(", ")
                    );
                }
                ensure!(
                    matches.len() == 1,
                    "Ambiguous call to overloaded routine '{name}' at {span} with argument types ({})",
                    format_param_types(&arg_tys)
                );
                let chosen = (*matches
                    .first()
                    .context("Missing matched routine overload")?)
                .clone();
                let ret_ty = chosen.sig.return_type.clone();
                self.resolved_calls.insert((span.line, span.col), chosen);
                if ctx.in_tail {
                    self.tail_calls.insert((span.line, span.col));
                }
                Ok(ret_ty)
            }
            SymbolTarget::Dc(dc) => {
                let sh = format_dc_shorthand(*dc);
                if let Some(orig) = self.top_dc_names.get(dc) {
                    ensure!(
                        !is_stagel_internal_name(orig),
                        "StageL-internal routine '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
                    );
                }
                let sig = if let Some(s) = self.top_dc_routines.get(dc) {
                    s.clone()
                } else if let Some(ImportedSymbol::Routine {
                    original_name,
                    sig,
                    ..
                }) = self.dc_imports.get(dc)
                {
                    ensure!(
                        !is_stagel_internal_name(original_name),
                        "StageL-internal routine '{original_name}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
                    );
                    sig.clone()
                } else {
                    bail!("Unknown routine Dc reference 'dc/{sh}' at {span}");
                };
                ensure!(
                    args.len() == sig.params.len(),
                    "Wrong argument count in call at {span}: expected {}, got {}",
                    sig.params.len(),
                    args.len()
                );
                for (arg_expr, param_ty) in args.iter().zip(&sig.params) {
                    let aty = self.check_subexpr(arg_expr, state, ctx)?;
                    ensure!(
                        types_compatible(&aty, param_ty),
                        "Call argument type mismatch at {}: expected '{param_ty}', found '{aty}'",
                        arg_expr.span()
                    );
                }
                if ctx.in_tail {
                    self.tail_calls.insert((span.line, span.col));
                }
                Ok(sig.return_type)
            }
        }
    }

    fn check_block(
        &mut self,
        stmts: &[BlockItem],
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let mut block_routines: HashMap<String, VerbTarget> = HashMap::new();
        let mut nested_decls: Vec<(usize, &RoutineDecl)> = Vec::new();

        for item in stmts {
            if let BlockItem::Expr(Expr::RoutineDecl(r)) = item {
                ensure!(
                    !self.name_exists_in_local_or_top(&r.name)
                        && !block_routines.contains_key(&r.name),
                    "Nested routine '{}' at {} duplicates or shadows an existing identifier",
                    r.name,
                    r.span
                );
                let canon_ret = self.check_and_canonicalize_type(
                    &r.return_type,
                    false,
                    r.span,
                )?;
                let mut seen_p = HashSet::new();
                let mut ptys = Vec::with_capacity(r.params.len());
                for p in &r.params {
                    ensure!(
                        p.ty != StageLType::Void && p.ty != StageLType::Never,
                        "Parameter '{}' of '{}' at {} cannot have type '{}'",
                        p.name,
                        r.name,
                        p.span,
                        p.ty
                    );
                    let canon_pty = self.check_and_canonicalize_type(
                        &p.ty, false, p.span,
                    )?;
                    ensure!(
                        seen_p.insert(p.name.clone()),
                        "Duplicate parameter '{}' in nested routine '{}' at {}",
                        p.name,
                        r.name,
                        p.span
                    );
                    ptys.push(canon_pty);
                }
                let nid = self.alloc_binding_id();
                let sig = RoutineSig {
                    return_type: canon_ret,
                    params: ptys,
                    is_pub: false,
                    dc_id: None,
                };
                self.nested_summaries.insert(
                    nid,
                    NestedVerbSummary {
                        sig,
                        captured_reads: HashSet::new(),
                        always_written: HashSet::new(),
                        captures_resolved: false,
                        span: r.span,
                    },
                );
                block_routines.insert(r.name.clone(), VerbTarget::Nested(nid));
                nested_decls.push((nid, r));
            }
        }

        self.routine_scopes.push(block_routines);
        self.local_scopes.push(HashMap::new());
        let saved_pending = std::mem::take(&mut self.pending_nested_calls);

        let last_expr_idx =
            stmts.iter().rposition(|it| matches!(it, BlockItem::Expr(_)));
        let mut result_ty = StageLType::Void;
        let mut terminated = false;
        for (idx, item) in stmts.iter().enumerate() {
            let BlockItem::Expr(e) = item else {
                continue;
            };
            let is_last = Some(idx) == last_expr_idx;
            let mut item_ctx =
                ctx.child(ctx.in_tail && is_last, ctx.loop_depth, ctx.allow_buf);
            let ety = self.check_expr(e, state, &mut item_ctx)?;
            ctx.captured_reads.extend(item_ctx.captured_reads);
            ctx.always_written.extend(item_ctx.always_written);
            if ety == StageLType::Never {
                terminated = true;
            }
            if is_last {
                ensure!(
                    !ety.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be returned from a 'do' block at {}",
                    e.span()
                );
                result_ty = if terminated { StageLType::Never } else { ety };
            }
        }

        let block_calls =
            std::mem::replace(&mut self.pending_nested_calls, saved_pending);
        let mut called_nids = HashSet::new();
        for (nid, call_span, call_state) in block_calls {
            called_nids.insert(nid);
            if let Some(sum) = self.nested_summaries.get(&nid) {
                for cap_id in &sum.captured_reads {
                    if !call_state.is_assigned(*cap_id) {
                        bail!(
                            "Calling nested verb (at {}) at {call_span} before captured 'let' variable (binding #{cap_id}) is definitely assigned",
                            sum.span
                        );
                    }
                    if ctx.outer_ids.contains(cap_id)
                        && !ctx.always_written.contains(cap_id)
                    {
                        ctx.captured_reads.insert(*cap_id);
                    }
                }
            }
        }

        for (nid, r) in nested_decls {
            if let Some(sum) = self.nested_summaries.get(&nid)
                && !called_nids.contains(&nid)
            {
                for cap_id in &sum.captured_reads {
                    if Self::lookup_is_declared_in_scope(
                        self.local_scopes.last(),
                        *cap_id,
                    ) && !state.assign.is_assigned(*cap_id)
                    {
                        bail!(
                            "Nested routine '{}' at {} reads uninitialized 'let' variable (binding #{cap_id})",
                            r.name,
                            r.span
                        );
                    }
                }
            }
        }

        if let Some(popped) = self.local_scopes.pop() {
            for entry in popped.into_values() {
                state.bufs.remove(&entry.id);
            }
        }
        let _ = self.routine_scopes.pop();
        Ok(result_ty)
    }

    fn check_if(
        &mut self,
        cond: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let cty = self.check_subexpr(cond, state, ctx)?;
        ensure!(
            types_compatible(&cty, &StageLType::B),
            "'if' condition at {} must have type 'b', found '{cty}'",
            cond.span()
        );

        let mut then_state = state.clone();
        let mut then_ctx = ctx.child(ctx.in_tail, ctx.loop_depth, ctx.allow_buf);
        let tty = self.check_expr(then_branch, &mut then_state, &mut then_ctx)?;

        let mut else_state = state.clone();
        let mut else_ctx = ctx.child(ctx.in_tail, ctx.loop_depth, ctx.allow_buf);
        let ety = self.check_expr(else_branch, &mut else_state, &mut else_ctx)?;

        ctx.captured_reads.extend(then_ctx.captured_reads);
        ctx.captured_reads.extend(else_ctx.captured_reads);

        ensure!(
            !tty.is_or_contains_buf() && !ety.is_or_contains_buf(),
            "Linear 'buf/T' cannot be returned from 'if' expression at {span}"
        );

        match (tty == StageLType::Never, ety == StageLType::Never) {
            (true, true) => Ok(StageLType::Never),
            (true, false) => {
                *state = else_state;
                ctx.always_written.extend(else_ctx.always_written);
                Ok(ety)
            }
            (false, true) => {
                *state = then_state;
                ctx.always_written.extend(then_ctx.always_written);
                Ok(tty)
            }
            (false, false) => {
                ensure!(
                    tty == ety,
                    "'if' branch type mismatch at {span}: then has '{tty}', else has '{ety}'"
                );
                then_state.intersect_with(&else_state);
                *state = then_state;
                for id in &then_ctx.always_written {
                    if else_ctx.always_written.contains(id) {
                        ctx.always_written.insert(*id);
                    }
                }
                Ok(tty)
            }
        }
    }

    fn check_while(
        &mut self,
        cond: &Expr,
        body: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let cty = self.check_subexpr(cond, state, ctx)?;
        ensure!(
            types_compatible(&cty, &StageLType::B),
            "'while' condition at {} must have type 'b', found '{cty}'",
            cond.span()
        );
        let mut body_state = state.clone();
        let mut body_ctx =
            ctx.child(false, ctx.loop_depth.saturating_add(1), ctx.allow_buf);
        body_ctx.loop_protected_bufs.extend(state.bufs.keys().copied());
        let bty = self.check_expr(body, &mut body_state, &mut body_ctx)?;
        ctx.captured_reads.extend(body_ctx.captured_reads);
        ensure!(
            bty == StageLType::Void || bty == StageLType::Never,
            "'while' body at {span} must evaluate to 'void' (or 'never'), found '{bty}'"
        );
        Ok(StageLType::Void)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Validates StageR foreach over l/T and dcs"
    )]
    fn check_foreach(
        &mut self,
        elem_ty: &StageLType,
        elem_name: &str,
        list: &Expr,
        body: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        ensure!(
            !is_stagel_internal_name(elem_name),
            "StageL-internal variable '{elem_name}' cannot be declared in StageR 3 at {span}"
        );
        let canon_elem =
            self.check_and_canonicalize_type(elem_ty, false, span)?;
        check_stager_value_type_shape(&canon_elem, span)?;
        ensure!(
            !self.name_exists_in_local_or_top(elem_name),
            "'foreach' element binding '{elem_name}' at {span} shadows an existing declaration"
        );
        let lty = self.check_subexpr(list, state, ctx)?;
        match &lty {
            StageLType::Never => {}
            StageLType::List(el) if **el == canon_elem => {}
            StageLType::Dcs if canon_elem == StageLType::Dc => {}
            other => {
                if canon_elem == StageLType::Dc {
                    bail!(
                        "'foreach' iterable at {} must have type 'l/dc' or 'dcs', found '{other}'",
                        list.span()
                    );
                }
                bail!(
                    "'foreach' iterable at {} must have type 'l/{canon_elem}', found '{other}'",
                    list.span()
                );
            }
        }
        let elem_id = self.alloc_binding_id();
        let mut body_state = state.clone();
        body_state.assign.set_state(elem_id, true);
        let mut elem_scope = HashMap::new();
        elem_scope.insert(
            elem_name.to_owned(),
            BindingEntry {
                id: elem_id,
                ty: canon_elem,
                kind: BindingKind::ForEachElem,
                span,
            },
        );
        self.local_scopes.push(elem_scope);
        let mut body_ctx =
            ctx.child(false, ctx.loop_depth.saturating_add(1), ctx.allow_buf);
        body_ctx.loop_protected_bufs.extend(state.bufs.keys().copied());
        let bty = self.check_expr(body, &mut body_state, &mut body_ctx)?;
        let _ = self.local_scopes.pop();
        ctx.captured_reads.extend(body_ctx.captured_reads);
        ensure!(
            bty == StageLType::Void || bty == StageLType::Never,
            "'foreach' body at {span} must evaluate to 'void' (or 'never'), found '{bty}'"
        );
        Ok(StageLType::Void)
    }

    fn resolve_match_domain(
        &self,
        sty: &StageLType,
        span: Span,
    ) -> Result<MatchDomain> {
        match sty {
            StageLType::Option(inner) => Ok(MatchDomain::Option((**inner).clone())),
            StageLType::Result(ok_ty) => Ok(MatchDomain::Result((**ok_ty).clone())),
            StageLType::Error => Ok(MatchDomain::Error),
            StageLType::Nominal(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace.clone(), name.clone());
                if let Some(edef) = self.enums.get(&key) {
                    Ok(MatchDomain::Enum(edef.variants.clone()))
                } else {
                    bail!(
                        "'match' scrutinee type '{sty}' at {span} is not an 'enum' ('brand'/'type' values cannot be matched)"
                    );
                }
            }
            StageLType::Never => {
                bail!("Cannot 'match' on diverging 'never' expression at {span}")
            }
            other => bail!(
                "'match' requires an 'option/T', 'result/T', 'error', or 'enum' scrutinee, found '{other}' at {span}"
            ),
        }
    }

    fn check_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        fallback: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let sty = self.check_subexpr(scrutinee, state, ctx)?;
        let domain = self.resolve_match_domain(&sty, scrutinee.span())?;

        let all_variants: Vec<(&str, Option<StageLType>)> = match &domain {
            MatchDomain::Option(inner) => {
                vec![("none", None), ("some", Some(inner.clone()))]
            }
            MatchDomain::Result(ok_ty) => vec![
                ("ok", Some(ok_ty.clone())),
                ("err", Some(StageLType::Error)),
            ],
            MatchDomain::Error => StageRErrorTag::all_keywords()
                .iter()
                .map(|&kw| (kw, None))
                .collect(),
            MatchDomain::Enum(vars) => vars
                .iter()
                .map(|v| (v.name.as_str(), v.payload.clone()))
                .collect(),
        };

        let mut seen = HashSet::new();
        let mut survivors: Vec<(StageLType, FlowState, HashSet<usize>)> =
            Vec::new();

        for arm in arms {
            let expected_payload = all_variants
                .iter()
                .find(|(vname, _)| *vname == arm.variant)
                .map(|(_, p)| p.clone())
                .with_context(|| {
                    format!(
                        "Unknown 'case {}' for scrutinee type '{sty}' at {}",
                        arm.variant, arm.span
                    )
                })?;
            ensure!(
                seen.insert(arm.variant.clone()),
                "Duplicate 'case {}' in 'match' at {}",
                arm.variant,
                arm.span
            );

            let mut arm_state = state.clone();
            let mut arm_ctx = ctx.child(ctx.in_tail, ctx.loop_depth, ctx.allow_buf);
            let pushed_scope = match (expected_payload, &arm.binding) {
                (None, None) => false,
                (None, Some((_, bname, bsp))) => {
                    bail!(
                        "Unit variant '{}' takes no payload binding '{bname}' at {bsp}",
                        arm.variant
                    );
                }
                (Some(exp_ty), None) => {
                    bail!(
                        "Payload variant '{}' requires binding '{exp_ty}/<name>' at {}",
                        arm.variant,
                        arm.span
                    );
                }
                (Some(exp_ty), Some((ann_ty, bname, bsp))) => {
                    ensure!(
                        !is_stagel_internal_name(bname),
                        "StageL-internal pattern binding '{bname}' cannot be declared in StageR 3 at {bsp}"
                    );
                    let canon_ann =
                        self.check_and_canonicalize_type(ann_ty, false, *bsp)?;
                    ensure!(
                        canon_ann == exp_ty,
                        "Pattern binding type mismatch for 'case {}' at {bsp}: expected '{exp_ty}', found '{canon_ann}'",
                        arm.variant
                    );
                    ensure!(
                        !self.name_exists_in_local_or_top(bname),
                        "Pattern binding '{bname}' at {bsp} shadows an existing declaration"
                    );
                    let bid = self.alloc_binding_id();
                    arm_state.assign.set_state(bid, true);
                    let mut sc = HashMap::new();
                    sc.insert(
                        bname.clone(),
                        BindingEntry {
                            id: bid,
                            ty: canon_ann,
                            kind: BindingKind::MatchArm,
                            span: *bsp,
                        },
                    );
                    self.local_scopes.push(sc);
                    true
                }
            };

            let arm_res =
                self.check_expr(&arm.body, &mut arm_state, &mut arm_ctx);
            if pushed_scope {
                let _ = self.local_scopes.pop();
            }
            let arm_ty = arm_res?;
            ctx.captured_reads.extend(arm_ctx.captured_reads);
            ensure!(
                !arm_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from 'match' arm at {}",
                arm.body.span()
            );
            if arm_ty != StageLType::Never {
                survivors.push((arm_ty, arm_state, arm_ctx.always_written));
            }
        }

        let missing: Vec<&str> = all_variants
            .iter()
            .filter(|(vname, _)| !seen.contains(*vname))
            .map(|(vname, _)| *vname)
            .collect();

        if missing.is_empty() && fallback.is_some() {
            bail!(
                "Redundant 'else' fallback arm in 'match' at {span} when all variants of '{sty}' are already covered"
            );
        }
        if !missing.is_empty() && fallback.is_none() {
            bail!(
                "Non-exhaustive 'match' on '{sty}' at {span}: missing variant(s): {}",
                missing.join(", ")
            );
        }

        if let Some(fb) = fallback {
            let mut fb_state = state.clone();
            let mut fb_ctx = ctx.child(ctx.in_tail, ctx.loop_depth, ctx.allow_buf);
            let fb_ty = self.check_expr(fb, &mut fb_state, &mut fb_ctx)?;
            ctx.captured_reads.extend(fb_ctx.captured_reads);
            ensure!(
                !fb_ty.is_or_contains_buf(),
                "Linear 'buf/T' cannot be returned from 'match' else arm at {}",
                fb.span()
            );
            if fb_ty != StageLType::Never {
                survivors.push((fb_ty, fb_state, fb_ctx.always_written));
            }
        }

        let Some((first_ty, mut joined_state, first_written)) =
            survivors.first().cloned()
        else {
            return Ok(StageLType::Never);
        };
        let mut common_written = first_written;
        for (aty, ast, aw) in survivors.into_iter().skip(1) {
            ensure!(
                aty == first_ty,
                "'match' branch type mismatch at {span}: expected '{first_ty}', found '{aty}'"
            );
            joined_state.intersect_with(&ast);
            common_written.retain(|id| aw.contains(id));
        }
        *state = joined_state;
        ctx.always_written.extend(common_written);
        Ok(first_ty)
    }

    fn check_propagate(
        &mut self,
        inner: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        ensure!(
            ctx.allow_propagate,
            "Error propagation operator '/' ('?') at {span} is only allowed inside a routine body"
        );
        let ity = self.check_subexpr(inner, state, ctx)?;
        match ity {
            StageLType::Never => Ok(StageLType::Never),
            StageLType::Option(ok) => match &ctx.return_type {
                StageLType::Option(_) | StageLType::Result(_) => Ok(*ok),
                other => bail!(
                    "Cannot propagate 'option/...' with '/' ('?') at {span} inside routine returning '{other}' (routine must return 'option/...' or 'result/...')"
                ),
            },
            StageLType::Result(ok) => match &ctx.return_type {
                StageLType::Result(_) => Ok(*ok),
                other => bail!(
                    "Cannot propagate 'result/...' with '/' ('?') at {span} inside routine returning '{other}' (routine must return 'result/...')"
                ),
            },
            other => bail!(
                "Propagation operator '/' ('?') at {span} requires 'option/T' or 'result/T' operand, found '{other}'"
            ),
        }
    }

    fn check_return(
        &mut self,
        value: Option<&Expr>,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let ret_ty = ctx.return_type.clone();
        match (ret_ty, value) {
            (StageLType::Void, None) => Ok(StageLType::Never),
            (StageLType::Void, Some(v)) => bail!(
                "Cannot return a value from 'void' routine at {}",
                v.span()
            ),
            (expected, None) => bail!(
                "Missing return value expression at {span} (routine returns '{expected}')"
            ),
            (expected, Some(v)) => {
                let mut ret_ctx = ctx.child(true, ctx.loop_depth, ctx.allow_buf);
                let vty = self.check_expr(v, state, &mut ret_ctx)?;
                ctx.captured_reads.extend(ret_ctx.captured_reads);
                ensure!(
                    !vty.is_or_contains_buf(),
                    "Linear 'buf/T' cannot be returned at {span}"
                );
                ensure!(
                    types_compatible(&vty, &expected),
                    "Return value type mismatch at {span}: expected '{expected}', found '{vty}'"
                );
                Ok(StageLType::Never)
            }
        }
    }

    fn check_core_unary(
        &mut self,
        op: CoreUnaryOp,
        operand: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match op {
            CoreUnaryOp::Neg => bail!(
                "Trapping unary 'neg' at {span} is forbidden in StageR 3 (use StageR total 'neg')"
            ),
            CoreUnaryOp::Not => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                ensure!(
                    types_compatible(&ty, &StageLType::B),
                    "'not' requires 'b', found '{ty}' at {span}"
                );
                Ok(StageLType::B)
            }
            CoreUnaryOp::BitNot => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                if ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                ensure!(
                    ty.is_stager_fixed_int(),
                    "'bitNot' requires fixed-width integer, found '{ty}' at {span}"
                );
                Ok(ty)
            }
            CoreUnaryOp::Clz | CoreUnaryOp::Ctz | CoreUnaryOp::Popcnt => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                if ty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                ensure!(
                    ty.is_stager_fixed_int(),
                    "'{}' requires fixed-width integer, found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::U32)
            }
            CoreUnaryOp::Len => {
                let (ty, _) =
                    self.check_buf_target(operand, op.keyword(), state, ctx)?;
                match ty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::List(_)
                    | StageLType::Buf(_)
                    | StageLType::Dcs => Ok(StageLType::U64),
                    other => bail!(
                        "'len' requires 'l/T', 'buf/T', or 'dcs', found '{other}' at {span}"
                    ),
                }
            }
            CoreUnaryOp::BufFreeze => {
                ensure!(
                    ctx.allow_buf,
                    "'bufFreeze' is disallowed in constant context at {span}"
                );
                let (ty, buf_id) =
                    self.check_buf_target(operand, op.keyword(), state, ctx)?;
                if let Some(id) = buf_id {
                    ensure!(
                        !ctx.loop_protected_bufs.contains(&id),
                        "Cannot 'bufFreeze' buffer declared outside loop inside loop body at {span}"
                    );
                    state.bufs.insert(id, BufState::Frozen);
                }
                match ty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => Ok(StageLType::List(elem)),
                    other => bail!(
                        "'bufFreeze' requires 'buf/T', found '{other}' at {span}"
                    ),
                }
            }
        }
    }

    fn check_core_binary(
        &mut self,
        op: CoreBinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match op {
            CoreBinaryOp::Add
            | CoreBinaryOp::Sub
            | CoreBinaryOp::Mul
            | CoreBinaryOp::Div
            | CoreBinaryOp::Rem
            | CoreBinaryOp::Shl
            | CoreBinaryOp::Shr
            | CoreBinaryOp::Get => bail!(
                "Partial StageL operator '{}' at {span} is forbidden in StageR 3 (use StageR total '{}')",
                op.keyword(),
                op.keyword()
            ),
            CoreBinaryOp::And | CoreBinaryOp::Or => {
                let lty = self.check_subexpr(left, state, ctx)?;
                ensure!(
                    types_compatible(&lty, &StageLType::B),
                    "'{}' left operand at {} must be 'b', found '{lty}'",
                    op.keyword(),
                    left.span()
                );
                let mut right_state = state.clone();
                let mut right_ctx =
                    ctx.child(false, ctx.loop_depth, ctx.allow_buf);
                let rty =
                    self.check_expr(right, &mut right_state, &mut right_ctx)?;
                ctx.captured_reads.extend(right_ctx.captured_reads);
                ensure!(
                    types_compatible(&rty, &StageLType::B),
                    "'{}' right operand at {} must be 'b', found '{rty}'",
                    op.keyword(),
                    right.span()
                );
                Ok(StageLType::B)
            }
            CoreBinaryOp::WrapAdd
            | CoreBinaryOp::WrapSub
            | CoreBinaryOp::WrapMul
            | CoreBinaryOp::BitAnd
            | CoreBinaryOp::BitOr
            | CoreBinaryOp::BitXor => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    StageLType::is_stager_fixed_int,
                    "fixed-width integer",
                    span,
                )
            }
            CoreBinaryOp::Rotl | CoreBinaryOp::Rotr => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&rty, &StageLType::U32),
                    "'{}' rotate count at {} must be 'u32', found '{rty}'",
                    op.keyword(),
                    right.span()
                );
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                ensure!(
                    lty.is_stager_unsigned_int(),
                    "'{}' requires unsigned fixed integer ('u8', 'u32', 'u64', 'u128'), found '{lty}' at {span}",
                    op.keyword()
                );
                Ok(lty)
            }
            CoreBinaryOp::Lt
            | CoreBinaryOp::Le
            | CoreBinaryOp::Gt
            | CoreBinaryOp::Ge => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                let _ = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    StageLType::is_stager_numeric,
                    "numeric",
                    span,
                )?;
                Ok(StageLType::B)
            }
            CoreBinaryOp::Eq | CoreBinaryOp::Ne => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                match (lty == StageLType::Never, rty == StageLType::Never) {
                    (true, true) => {}
                    (true, false) => ensure!(
                        self.type_supports_eq(&rty),
                        "'{}' does not support type '{rty}' at {span}",
                        op.keyword()
                    ),
                    (false, true) => ensure!(
                        self.type_supports_eq(&lty),
                        "'{}' does not support type '{lty}' at {span}",
                        op.keyword()
                    ),
                    (false, false) => {
                        ensure!(
                            lty == rty,
                            "'{}' operand type mismatch at {span}: '{lty}' vs '{rty}'",
                            op.keyword()
                        );
                        ensure!(
                            self.type_supports_eq(&lty),
                            "'{}' does not support type '{lty}' at {span}",
                            op.keyword()
                        );
                    }
                }
                Ok(StageLType::B)
            }
            CoreBinaryOp::Concat => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                match (lty, rty) {
                    (StageLType::Never, StageLType::Never) => {
                        Ok(StageLType::Never)
                    }
                    (StageLType::Never, r @ (StageLType::List(_) | StageLType::S | StageLType::Dcs)) => {
                        Ok(r)
                    }
                    (l @ (StageLType::List(_) | StageLType::S | StageLType::Dcs), StageLType::Never) => {
                        Ok(l)
                    }
                    (l @ StageLType::List(_), r @ StageLType::List(_))
                    | (l @ StageLType::S, r @ StageLType::S)
                    | (l @ StageLType::Dcs, r @ StageLType::Dcs) => {
                        ensure!(
                            l == r,
                            "'concat' sequence types must match at {span}: '{l}' vs '{r}'"
                        );
                        Ok(l)
                    }
                    (l, r) => bail!(
                        "'concat' requires matching 'l/T', 's', or 'dcs' operands at {span}, found '{l}' and '{r}'"
                    ),
                }
            }
            CoreBinaryOp::BufFill => {
                ensure!(
                    ctx.allow_buf,
                    "'bufFill' is disallowed in constant context at {span}"
                );
                let vty = self.check_subexpr(left, state, ctx)?;
                let nty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&nty, &StageLType::U64),
                    "'bufFill' length at {} must be 'u64', found '{nty}'",
                    right.span()
                );
                if vty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                check_stager_value_type_shape(&vty, span)?;
                Ok(StageLType::Buf(Box::new(vty)))
            }
            CoreBinaryOp::BufPush => {
                let (bty, _) =
                    self.check_buf_target(left, op.keyword(), state, ctx)?;
                let vty = self.check_subexpr(right, state, ctx)?;
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        ensure!(
                            types_compatible(&vty, &elem),
                            "'bufPush' value at {} has type '{vty}', expected '{elem}'",
                            right.span()
                        );
                        Ok(StageLType::Void)
                    }
                    other => bail!(
                        "'bufPush' target at {} must be 'buf/T', found '{other}'",
                        left.span()
                    ),
                }
            }
            CoreBinaryOp::BufExtend => {
                let (bty, _) =
                    self.check_buf_target(left, op.keyword(), state, ctx)?;
                let xty = self.check_subexpr(right, state, ctx)?;
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        let expected_list = StageLType::List(elem);
                        ensure!(
                            types_compatible(&xty, &expected_list),
                            "'bufExtend' list at {} has type '{xty}', expected '{expected_list}'",
                            right.span()
                        );
                        Ok(StageLType::Void)
                    }
                    other => bail!(
                        "'bufExtend' target at {} must be 'buf/T', found '{other}'",
                        left.span()
                    ),
                }
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Rejects partial Core ternary ops in StageR 3"
    )]
    fn check_core_ternary(
        &mut self,
        op: CoreTernaryOp,
        _first: &Expr,
        _second: &Expr,
        _third: &Expr,
        span: Span,
        _state: &mut FlowState,
        _ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        bail!(
            "Partial StageL operator '{}' at {span} is forbidden in StageR 3 (use StageR total '{}')",
            op.keyword(),
            op.keyword()
        )
    }

    fn check_stager_unary(
        &mut self,
        op: StageRUnaryOp,
        operand: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let ty = self.check_subexpr(operand, state, ctx)?;
        if ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        match op {
            StageRUnaryOp::TotalNeg => match &ty {
                StageLType::I32 | StageLType::I64 | StageLType::I128 => {
                    Ok(StageLType::Result(Box::new(ty)))
                }
                StageLType::Int | StageLType::Rat => Ok(ty),
                other => bail!(
                    "'neg' requires signed numeric operand ('i32', 'i64', 'i128', 'int', 'rat'), found '{other}' at {span}"
                ),
            },
            StageRUnaryOp::IsSome | StageRUnaryOp::IsNone => {
                ensure!(
                    matches!(ty, StageLType::Option(_)),
                    "'{}' requires 'option/T', found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::B)
            }
            StageRUnaryOp::IsOk | StageRUnaryOp::IsErr => {
                ensure!(
                    matches!(ty, StageLType::Result(_)),
                    "'{}' requires 'result/T', found '{ty}' at {span}",
                    op.keyword()
                );
                Ok(StageLType::B)
            }
            StageRUnaryOp::ByteLen => {
                ensure!(
                    ty == StageLType::S,
                    "'byteLen' requires 's', found '{ty}' at {span}"
                );
                Ok(StageLType::U64)
            }
            StageRUnaryOp::DcFromU128 => {
                ensure!(
                    ty == StageLType::U128,
                    "'dcFromU128' requires 'u128', found '{ty}' at {span}"
                );
                Ok(StageLType::Result(Box::new(StageLType::Dc)))
            }
            StageRUnaryOp::U128FromDc => {
                ensure!(
                    ty == StageLType::Dc,
                    "'u128FromDc' requires 'dc', found '{ty}' at {span}"
                );
                Ok(StageLType::U128)
            }
            StageRUnaryOp::DcsFromS => {
                ensure!(
                    ty == StageLType::S,
                    "'dcsFromS' requires 's', found '{ty}' at {span}"
                );
                Ok(StageLType::Result(Box::new(StageLType::Dcs)))
            }
            StageRUnaryOp::SFromDcs => {
                ensure!(
                    ty == StageLType::Dcs,
                    "'sFromDcs' requires 'dcs', found '{ty}' at {span}"
                );
                Ok(StageLType::S)
            }
        }
    }

    fn check_stager_binary(
        &mut self,
        op: StageRBinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match op {
            StageRBinaryOp::TotalAdd
            | StageRBinaryOp::TotalSub
            | StageRBinaryOp::TotalMul => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                let base = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    StageLType::is_stager_numeric,
                    "numeric",
                    span,
                )?;
                if base == StageLType::Never
                    || matches!(base, StageLType::Int | StageLType::Rat)
                {
                    Ok(base)
                } else {
                    Ok(StageLType::Result(Box::new(base)))
                }
            }
            StageRBinaryOp::TotalDiv => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                let base = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    StageLType::is_stager_numeric,
                    "numeric",
                    span,
                )?;
                if base == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(base)))
                }
            }
            StageRBinaryOp::TotalMod | StageRBinaryOp::TotalTruncRem => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                let base = unify_matching_pair(
                    op.keyword(),
                    &lty,
                    &rty,
                    |t| t.is_stager_fixed_int() || *t == StageLType::Int,
                    "integer",
                    span,
                )?;
                if base == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(base)))
                }
            }
            StageRBinaryOp::TotalShl | StageRBinaryOp::TotalShr => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&rty, &StageLType::U32),
                    "'{}' shift count at {} must be 'u32', found '{rty}'",
                    op.keyword(),
                    right.span()
                );
                if lty == StageLType::Never {
                    return Ok(StageLType::Never);
                }
                ensure!(
                    lty.is_stager_fixed_int(),
                    "'{}' value operand at {span} must be fixed-width integer, found '{lty}'",
                    op.keyword()
                );
                Ok(StageLType::Result(Box::new(lty)))
            }
            StageRBinaryOp::TotalGet => {
                let (lty, _) =
                    self.check_buf_target(left, op.keyword(), state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&rty, &StageLType::U64),
                    "'get' index at {} must be 'u64', found '{rty}'",
                    right.span()
                );
                match lty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::List(el) | StageLType::Buf(el) => {
                        Ok(StageLType::Option(el))
                    }
                    StageLType::Dcs => {
                        Ok(StageLType::Option(Box::new(StageLType::Dc)))
                    }
                    other => bail!(
                        "'get' sequence operand must be 'l/T', 'buf/T', or 'dcs', found '{other}' at {span}"
                    ),
                }
            }
            StageRBinaryOp::UnwrapOr => {
                let cty = self.check_subexpr(left, state, ctx)?;
                let fty = self.check_subexpr(right, state, ctx)?;
                match cty {
                    StageLType::Never => Ok(fty),
                    StageLType::Option(inner) | StageLType::Result(inner) => {
                        ensure!(
                            types_compatible(&fty, &inner),
                            "'unwrapOr' fallback type mismatch at {}: expected '{inner}', found '{fty}'",
                            right.span()
                        );
                        Ok(*inner)
                    }
                    other => bail!(
                        "'unwrapOr' requires 'option/T' or 'result/T' as first operand, found '{other}' at {span}"
                    ),
                }
            }
            StageRBinaryOp::OkOr => {
                let oty = self.check_subexpr(left, state, ctx)?;
                let ety = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&ety, &StageLType::Error),
                    "'okOr' error operand at {} must have type 'error', found '{ety}'",
                    right.span()
                );
                match oty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Option(inner) => Ok(StageLType::Result(inner)),
                    other => bail!(
                        "'okOr' first operand must have type 'option/T', found '{other}' at {span}"
                    ),
                }
            }
            StageRBinaryOp::MakeRat => {
                let nty = self.check_subexpr(left, state, ctx)?;
                let dty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&nty, &StageLType::Int),
                    "'makeRat' numerator at {} must have type 'int', found '{nty}'",
                    left.span()
                );
                ensure!(
                    types_compatible(&dty, &StageLType::Int),
                    "'makeRat' denominator at {} must have type 'int', found '{dty}'",
                    right.span()
                );
                if nty == StageLType::Never || dty == StageLType::Never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Result(Box::new(StageLType::Rat)))
                }
            }
            StageRBinaryOp::Explode => {
                let sep_ty = self.check_subexpr(left, state, ctx)?;
                let inp_ty = self.check_subexpr(right, state, ctx)?;
                let seq_ty = match (&sep_ty, &inp_ty) {
                    (StageLType::Never, _) | (_, StageLType::Never) => {
                        return Ok(StageLType::Never);
                    }
                    (StageLType::S, StageLType::S) => StageLType::S,
                    (StageLType::Dcs, StageLType::Dcs) => StageLType::Dcs,
                    (StageLType::List(a), StageLType::List(b)) => {
                        ensure!(
                            a == b,
                            "'explode' list types must match at {span}: '{sep_ty}' vs '{inp_ty}'"
                        );
                        ensure!(
                            self.type_supports_eq(a),
                            "'explode' element type '{a}' must support 'eq' at {span}"
                        );
                        StageLType::List(a.clone())
                    }
                    (l, r) => bail!(
                        "'explode' requires matching 's', 'dcs', or 'l/T' operands at {span}, found '{l}' and '{r}'"
                    ),
                };
                Ok(StageLType::List(Box::new(seq_ty)))
            }
            StageRBinaryOp::Implode => {
                let sep_ty = self.check_subexpr(left, state, ctx)?;
                let parts_ty = self.check_subexpr(right, state, ctx)?;
                match (&sep_ty, &parts_ty) {
                    (StageLType::Never, _) | (_, StageLType::Never) => {
                        Ok(StageLType::Never)
                    }
                    (seq, StageLType::List(part_elem)) => {
                        ensure!(
                            matches!(
                                seq,
                                StageLType::S
                                    | StageLType::Dcs
                                    | StageLType::List(_)
                            ),
                            "'implode' separator must be 's', 'dcs', or 'l/T', found '{seq}' at {span}"
                        );
                        ensure!(
                            seq == part_elem.as_ref(),
                            "'implode' parts list element '{}' must match separator '{seq}' at {span}",
                            part_elem
                        );
                        Ok(seq.clone())
                    }
                    (sep, other) => bail!(
                        "'implode' expects 'l/({sep})' parts list as second operand, found '{other}' at {span}"
                    ),
                }
            }
        }
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Validates StageR total ternary ops slice and bufSet"
    )]
    fn check_stager_ternary(
        &mut self,
        op: StageRTernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        match op {
            StageRTernaryOp::TotalSlice => {
                let seq_ty = self.check_subexpr(first, state, ctx)?;
                let start_ty = self.check_subexpr(second, state, ctx)?;
                let end_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    types_compatible(&start_ty, &StageLType::U64),
                    "'slice' start index at {} must have type 'u64', found '{start_ty}'",
                    second.span()
                );
                ensure!(
                    types_compatible(&end_ty, &StageLType::U64),
                    "'slice' end index at {} must have type 'u64', found '{end_ty}'",
                    third.span()
                );
                match seq_ty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::List(_) | StageLType::Dcs => {
                        Ok(StageLType::Option(Box::new(seq_ty)))
                    }
                    other => bail!(
                        "'slice' requires 'l/T' or 'dcs' sequence in StageR 3, found '{other}' at {span}"
                    ),
                }
            }
            StageRTernaryOp::TotalBufSet => {
                let (bty, _) =
                    self.check_buf_target(first, op.keyword(), state, ctx)?;
                let idx_ty = self.check_subexpr(second, state, ctx)?;
                let val_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    types_compatible(&idx_ty, &StageLType::U64),
                    "'bufSet' index at {} must have type 'u64', found '{idx_ty}'",
                    second.span()
                );
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        ensure!(
                            types_compatible(&val_ty, &elem),
                            "'bufSet' value at {} has type '{val_ty}', expected '{elem}'",
                            third.span()
                        );
                        Ok(StageLType::Result(Box::new(StageLType::Void)))
                    }
                    other => bail!(
                        "'bufSet' target at {} must be 'buf/T', found '{other}'",
                        first.span()
                    ),
                }
            }
        }
    }

    fn check_total_conv(
        &mut self,
        expr: &Expr,
        target_ty: &StageLType,
        span: Span,
        state: &mut FlowState,
        ctx: &mut StageRCtx,
    ) -> Result<StageLType> {
        let canon_target =
            self.check_and_canonicalize_type(target_ty, false, span)?;
        ensure!(
            canon_target.is_stager_numeric(),
            "'conv' target type must be numeric in StageR 3, found '{canon_target}' at {span}"
        );
        let src_ty = self.check_subexpr(expr, state, ctx)?;
        if src_ty == StageLType::Never {
            return Ok(StageLType::Never);
        }
        ensure!(
            src_ty.is_stager_numeric(),
            "'conv' source operand must be numeric in StageR 3, found '{src_ty}' at {span}"
        );
        if is_stager_widening_conv(&src_ty, &canon_target) {
            Ok(canon_target)
        } else {
            Ok(StageLType::Result(Box::new(canon_target)))
        }
    }
}

pub(crate) fn is_stager_widening_conv(
    src: &StageLType,
    dst: &StageLType,
) -> bool {
    if src == dst {
        return true;
    }
    match (src, dst) {
        (
            StageLType::U8,
            StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat,
        ) => true,
        (
            StageLType::I32,
            StageLType::I64
            | StageLType::I128
            | StageLType::Int
            | StageLType::Rat,
        ) => true,
        (
            StageLType::U32,
            StageLType::I64
            | StageLType::U64
            | StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat,
        ) => true,
        (
            StageLType::I64,
            StageLType::I128 | StageLType::Int | StageLType::Rat,
        ) => true,
        (
            StageLType::U64,
            StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat,
        ) => true,
        (
            StageLType::I128 | StageLType::U128,
            StageLType::Int | StageLType::Rat,
        ) => true,
        (StageLType::Int, StageLType::Rat) => true,
        _ => false,
    }
}

fn unify_matching_pair<F>(
    op_name: &str,
    lty: &StageLType,
    rty: &StageLType,
    pred: F,
    expected_desc: &str,
    span: Span,
) -> Result<StageLType>
where
    F: Fn(&StageLType) -> bool,
{
    match (lty == &StageLType::Never, rty == &StageLType::Never) {
        (true, true) => Ok(StageLType::Never),
        (true, false) => {
            ensure!(
                pred(rty),
                "'{op_name}' requires {expected_desc} operands, found '{rty}' at {span}"
            );
            Ok(rty.clone())
        }
        (false, true) => {
            ensure!(
                pred(lty),
                "'{op_name}' requires {expected_desc} operands, found '{lty}' at {span}"
            );
            Ok(lty.clone())
        }
        (false, false) => {
            ensure!(
                lty == rty && pred(lty),
                "'{op_name}' requires matching {expected_desc} types at {span}, found '{lty}' and '{rty}'"
            );
            Ok(lty.clone())
        }
    }
}
