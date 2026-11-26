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

//! Unified module validator entrypoints, transitive `dc_imports` seeding,
//! cross-dialect `use` resolution, top-level type/constant/routine
//! registration, and `NamespaceExports` construction.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, BrandTypeDecl, EnumDecl, Expr, ImportTarget, Module,
    NamedStructDecl, PositionalStructDecl, RoutineDecl, Span, StageDialect,
    StageLType, StructTypeRef, SymbolTarget, TestExpected, TopDecl,
    TopValueDecl, UseImportItem, format_dc_shorthand,
    is_enclosing_routine_token_dc, overloadable_std_synonym_counterpart,
};
use super::consts_and_flow::{
    AssignState, BindingEntry, NestedVerbSummary, VerbTarget,
};
use super::types::{
    BufPolicy, TypeDefs, TypeEnv, bridge_stage1_type_for_stager,
    canonicalize_type, check_pub_dc_shorthand, check_stager_value_type_shape,
    check_type_well_formed, check_unguarded_type_cycles,
};
use super::workspace::is_stagel_internal_name;
use super::{
    BrandTypeDef, EnumDef, EnumVariantDef, ImportedSymbol, NamedStructDef,
    NamespaceExports, ResolvedCallTarget, ResolvedValueTarget, RoutineKey,
    RoutineSig, StructDef, ValidatedModule, expr_syntactically_contains_fault,
};
use crate::DcChar;
use crate::syntax::{SyntaxRuleResolver, default_syntax_resolver};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Mutable validator state shared across passes and expression checking within
/// one [`Module`].
pub(super) struct ModuleValidator<'a> {
    pub(super) dialect: StageDialect,
    pub(super) namespace_key: Option<String>,
    /// Module-scope constants (`local_name -> ResolvedValueTarget`), combining
    /// imported and top-level constants (at most one definition per name).
    pub(super) value_candidates: HashMap<String, ResolvedValueTarget>,
    /// Module-scope routine candidates (`local_name -> Vec<ResolvedCallTarget>`),
    /// combining imported and top-level routines with identical arity and
    /// pairwise-distinct parameter type tuples.
    pub(super) routine_candidates: HashMap<String, Vec<ResolvedCallTarget>>,
    /// Local or imported positional/named struct names mapped to canonical
    /// `(namespace, original_name)`.
    pub(super) local_structs: HashMap<String, (Option<String>, String)>,
    /// Local or imported struct Dcs mapped to canonical
    /// `(namespace, original_name)`.
    pub(super) dc_structs: HashMap<DcChar, (Option<String>, String)>,
    /// Local or imported StageR nominal names (`brand`/`type`/`enum`) mapped to
    /// canonical keys.
    pub(super) local_nominals: HashMap<String, (Option<String>, String)>,
    /// Local or imported StageR nominal Dcs (`brand`/`type`/`enum`) mapped to
    /// canonical keys.
    pub(super) dc_nominals: HashMap<DcChar, (Option<String>, String)>,
    /// All canonical [`StructDef`]s declared or transitively reachable here.
    pub(super) struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    /// All canonical [`BrandTypeDef`]s declared or transitively reachable.
    pub(super) brand_types: BTreeMap<(Option<String>, String), BrandTypeDef>,
    /// All canonical [`NamedStructDef`]s declared or transitively reachable.
    pub(super) named_structs:
        BTreeMap<(Option<String>, String), NamedStructDef>,
    /// All canonical [`EnumDef`]s declared or transitively reachable.
    pub(super) enums: BTreeMap<(Option<String>, String), EnumDef>,
    /// Canonical keys for `type` (module-opaque) definitions declared inside
    /// this module (`wrap`/`unwrap` encapsulation checks).
    pub(super) local_declared_opaque: HashSet<(Option<String>, String)>,
    /// Top-level constant types keyed directly by assigned `DcChar`.
    pub(super) top_dc_values: HashMap<DcChar, StageLType>,
    /// Top-level routine signatures keyed directly by assigned `DcChar`.
    pub(super) top_dc_routines: HashMap<DcChar, RoutineSig>,
    /// Top-level declaration names keyed by assigned `DcChar`.
    pub(super) top_dc_names: HashMap<DcChar, String>,
    /// Imported items keyed directly by `DcChar` (`dc/<shorthand>`).
    pub(super) dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Shared Dc syntax/role resolver (for `DcRole::EnclosingRoutine` etc.).
    pub(super) syntax_resolver: &'static dyn SyntaxRuleResolver,
    /// Whether `*StageLInternal` symbols are prohibited in this module.
    pub(super) forbid_stagel_internal: bool,
    /// Next unique lexical variable ID.
    pub(super) next_binding_id: usize,
    /// Active lexical variable scopes (outermost local to innermost).
    pub(super) local_scopes: Vec<HashMap<String, BindingEntry>>,
    /// Active nested routine scopes (`name -> VerbTarget`).
    pub(super) routine_scopes: Vec<HashMap<String, VerbTarget>>,
    /// Summaries of validated nested verbs keyed by numeric ID.
    pub(super) nested_summaries: HashMap<usize, NestedVerbSummary>,
    /// Recorded calls to nested verbs in current block `(nid, span, state)`.
    pub(super) pending_nested_calls: Vec<(usize, Span, AssignState)>,
    /// Recorded tail call positions `(line, col)`.
    pub(super) tail_calls: HashSet<(usize, usize)>,
    /// Statically resolved top-level/imported call targets by `(line, col)`.
    pub(super) resolved_calls: HashMap<(usize, usize), ResolvedCallTarget>,
    /// Local candidate identifier selected for StageL 3 overloaded std operator
    /// or synonym calls, indexed by `(line, col)`.
    pub(super) overload_call_idents: HashMap<(usize, usize), String>,
    /// Statically resolved top-level/imported constant targets by `(line, col)`.
    pub(super) resolved_values: HashMap<(usize, usize), ResolvedValueTarget>,
    /// Statically resolved positional `struct` targets for `Expr::StructNew`.
    pub(super) resolved_structs: HashMap<(usize, usize), StructDef>,
    /// Statically resolved named `struct` targets for `Expr::NamedStructNew`.
    pub(super) resolved_named_structs:
        HashMap<(usize, usize), NamedStructDef>,
    /// Statically resolved `(enum_key, variant_name)` for `Expr::EnumVariant`.
    pub(super) resolved_enum_variants:
        HashMap<(usize, usize), ((Option<String>, String), String)>,
    /// Statically resolved [`StageLType`] for every `Expr::ValueRef` site.
    pub(super) value_ref_types: HashMap<(usize, usize), StageLType>,
    /// Canonicalized static [`StageLType`] for every validated `Expr`.
    pub(super) expr_types: HashMap<(usize, usize), StageLType>,
    /// Phantom marker tying lifetime `'a` to AST borrowed references.
    pub(super) _marker: std::marker::PhantomData<&'a ()>,
}

/// Validates a single standalone StageL 3 or StageR 3 module (no external
/// `use` imports).
pub fn validate_module(module: &Module) -> Result<ValidatedModule> {
    validate_module_with_registry(module, &HashMap::new())
}

/// Validates a parsed [`Module`] against already-validated dependency exports.
pub fn validate_module_with_registry(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
) -> Result<ValidatedModule> {
    validate_module_with_registry_and_dc_imports(
        module,
        ns_exports,
        &HashMap::new(),
    )
}

/// Validates a parsed [`Module`] against already-validated dependency exports
/// and pre-resolved direct `DcChar` symbol targets (such as transitive helpers
/// linked from [`crate::SeedGraphIndex`]).
pub fn validate_module_with_registry_and_dc_imports(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
    extra_dc_imports: &HashMap<DcChar, ImportedSymbol>,
) -> Result<ValidatedModule> {
    validate_module_with_registry_and_dc_imports_and_options(
        module,
        ns_exports,
        extra_dc_imports,
        false,
    )
}

/// Formats parameter type lists for routine signature diagnostics.
pub(super) fn format_param_types(params: &[StageLType]) -> String {
    params
        .iter()
        .map(StageLType::display_name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Validates a parsed [`Module`] with explicit Stage 2 / StageR
/// `*StageLInternal` gating enabled when `forbid_stagel_internal` is `true`
/// (always enabled for `StageDialect::StageR3`).
#[allow(
    clippy::too_many_lines,
    reason = "Unified 4-pass module validation pipeline cleanly sequenced in one entrypoint"
)]
pub fn validate_module_with_registry_and_dc_imports_and_options(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
    extra_dc_imports: &HashMap<DcChar, ImportedSymbol>,
    forbid_stagel_internal: bool,
) -> Result<ValidatedModule> {
    let orig_module = module;
    let syntax_resolver: &'static dyn SyntaxRuleResolver =
        default_syntax_resolver().as_ref();
    let lowered_module = lower_module_inline_calls_and_nested_self_refs(
        orig_module,
        syntax_resolver,
    );
    let module = &lowered_module;
    let dialect = module.dialect;
    let forbid_stagel_internal =
        forbid_stagel_internal || dialect == StageDialect::StageR3;
    let namespace_key = module.namespace.as_ref().map(|segs| segs.join("/"));
    let mut exports = NamespaceExports::default();
    let mut imports: HashMap<String, Vec<ImportedSymbol>> = HashMap::new();
    let mut dc_imports = extra_dc_imports.clone();

    let mut reg = ModuleSymbolReg::new(
        dialect,
        forbid_stagel_internal,
        namespace_key.clone(),
    );

    // Pass 0: Seed transitive type definitions from `ns_exports` and direct `dc_imports`.
    reg.seed_from_exports_and_dc_imports(ns_exports, &mut dc_imports)?;

    // Pass 1: Resolve explicit `use` imports with cross-dialect & `*StageLInternal` checks.
    for u in &module.uses {
        let target_ns = u.path.join("/");
        let dep_exp = ns_exports.get(&target_ns).with_context(|| {
            format!("Unresolved namespace '{target_ns}' in 'use' at {}", u.span)
        })?;
        for item in &u.items {
            let resolved_list = reg.resolve_use_item_symbols(
                &target_ns,
                dep_exp,
                item,
                &mut dc_imports,
            )?;
            let first_sym = resolved_list
                .first()
                .context("Empty exported symbol list")?;
            let local_name = match (&item.alias, &item.target) {
                (Some(a), _) => a.clone(),
                (None, ImportTarget::Name(n)) => n.clone(),
                (None, ImportTarget::Dc(_)) => {
                    first_sym.original_name().to_owned()
                }
            };
            if forbid_stagel_internal && is_stagel_internal_name(&local_name) {
                bail!(
                    "StageL-internal symbol '{local_name}' cannot be imported or declared in Stage 2/StageR module at {}",
                    item.span
                );
            }

            for resolved in resolved_list {
                reg.register_imported_symbol(
                    &resolved,
                    &local_name,
                    item.span,
                )?;
                imports
                    .entry(local_name.clone())
                    .or_default()
                    .push(resolved);
            }
        }
    }

    // Pass 2: Register and validate top-level type declarations (Pass 2a names +
    // Pass 2b bodies + unified cycle check + shape checks & type exports).
    reg.register_and_validate_types(&module.items, &mut exports)?;

    // Pass 3: Collect top-level constants and routine signatures.
    let (top_value_order, top_routine_order) =
        reg.register_top_values_and_routines(&module.items, &mut exports)?;

    if dialect == StageDialect::StageL3 {
        exports.all_structs = reg.struct_defs.clone();
    } else if !exports.brand_types.is_empty()
        || !exports.named_structs.is_empty()
        || !exports.enums.is_empty()
        || !exports.structs.is_empty()
        || !exports.values.is_empty()
        || !exports.routines.is_empty()
    {
        for (k, d) in &reg.struct_defs {
            exports.all_structs.insert(k.clone(), d.clone());
        }
        for (k, d) in &reg.brand_types {
            exports.all_brand_types.insert(k.clone(), d.clone());
        }
        for (k, d) in &reg.named_structs {
            exports.all_named_structs.insert(k.clone(), d.clone());
        }
        for (k, d) in &reg.enums {
            exports.all_enums.insert(k.clone(), d.clone());
        }
    }

    let mut validator = ModuleValidator {
        dialect,
        namespace_key: reg.namespace_key,
        value_candidates: reg.value_candidates,
        routine_candidates: reg.routine_candidates,
        local_structs: reg.local_structs,
        dc_structs: reg.dc_structs,
        local_nominals: reg.local_nominals,
        dc_nominals: reg.dc_nominals,
        struct_defs: reg.struct_defs,
        brand_types: reg.brand_types,
        named_structs: reg.named_structs,
        enums: reg.enums,
        local_declared_opaque: reg.local_declared_opaque,
        top_dc_values: reg.top_dc_values,
        top_dc_routines: reg.top_dc_routines,
        top_dc_names: reg.top_dc_names,
        dc_imports,
        syntax_resolver,
        forbid_stagel_internal,
        next_binding_id: 1,
        local_scopes: Vec::new(),
        routine_scopes: Vec::new(),
        nested_summaries: HashMap::new(),
        pending_nested_calls: Vec::new(),
        tail_calls: HashSet::new(),
        resolved_calls: HashMap::new(),
        overload_call_idents: HashMap::new(),
        resolved_values: HashMap::new(),
        resolved_structs: HashMap::new(),
        resolved_named_structs: HashMap::new(),
        resolved_enum_variants: HashMap::new(),
        value_ref_types: HashMap::new(),
        expr_types: HashMap::new(),
        _marker: std::marker::PhantomData,
    };

    // Pass 4: Validate top-level constant initializers (acyclic pure dependencies).
    validator.validate_top_values_acyclic(&top_value_order)?;

    // Pass 5: Validate inline tests and bodies for all top-level routines.
    for r in &top_routine_order {
        validator.validate_top_routine(r)?;
    }

    if dialect == StageDialect::StageL3 {
        validator.propagate_may_fault(&top_routine_order, &mut exports)?;
    }

    let validated_module = if dialect == StageDialect::StageL3 {
        desugar_overloaded_ops_in_module(
            orig_module,
            &validator.resolved_calls,
            &validator.overload_call_idents,
        )
    } else {
        orig_module.clone()
    };

    Ok(ValidatedModule {
        input_index: 0,
        module: validated_module,
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
        resolved_structs: validator.resolved_structs,
        resolved_named_structs: validator.resolved_named_structs,
        resolved_enum_variants: validator.resolved_enum_variants,
        value_ref_types: validator.value_ref_types,
        expr_types: validator.expr_types,
        exports,
    })
}

/// Intermediate registry accumulator for Passes 0-3 of module validation.
struct ModuleSymbolReg {
    dialect: StageDialect,
    forbid_stagel_internal: bool,
    namespace_key: Option<String>,
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
    used_dcs: HashMap<DcChar, String>,
}

impl ModuleSymbolReg {
    fn new(
        dialect: StageDialect,
        forbid_stagel_internal: bool,
        namespace_key: Option<String>,
    ) -> Self {
        Self {
            dialect,
            forbid_stagel_internal,
            namespace_key,
            value_candidates: HashMap::new(),
            routine_candidates: HashMap::new(),
            local_structs: HashMap::new(),
            dc_structs: HashMap::new(),
            local_nominals: HashMap::new(),
            dc_nominals: HashMap::new(),
            struct_defs: BTreeMap::new(),
            brand_types: BTreeMap::new(),
            named_structs: BTreeMap::new(),
            enums: BTreeMap::new(),
            local_declared_opaque: HashSet::new(),
            top_dc_values: HashMap::new(),
            top_dc_routines: HashMap::new(),
            top_dc_names: HashMap::new(),
            used_dcs: HashMap::new(),
        }
    }

    fn type_defs(&self) -> TypeDefs<'_> {
        TypeDefs {
            struct_defs: &self.struct_defs,
            brand_types: &self.brand_types,
            named_structs: &self.named_structs,
            enums: &self.enums,
        }
    }

    fn type_env(&self, forbid_internal: bool) -> TypeEnv<'_> {
        TypeEnv {
            dialect: self.dialect,
            forbid_stagel_internal: forbid_internal,
            local_structs: &self.local_structs,
            dc_structs: &self.dc_structs,
            local_nominals: &self.local_nominals,
            dc_nominals: &self.dc_nominals,
            defs: self.type_defs(),
            top_dc_names: &self.top_dc_names,
        }
    }

    fn canonicalize(
        &self,
        ty: &StageLType,
        forbid_internal: bool,
        span: Span,
    ) -> Result<StageLType> {
        canonicalize_type(ty, &self.type_env(forbid_internal), span)
    }

    fn name_conflicts_top(&self, name: &str) -> bool {
        self.value_candidates.contains_key(name)
            || self.routine_candidates.contains_key(name)
            || self.local_structs.contains_key(name)
            || self.local_nominals.contains_key(name)
    }

    fn register_dc_binding(
        &mut self,
        dc: DcChar,
        name: &str,
        pub_dc_syntax: bool,
        span: Span,
    ) -> Result<()> {
        if pub_dc_syntax {
            check_pub_dc_shorthand(dc, span)?;
        }
        ensure!(
            !self.used_dcs.contains_key(&dc),
            "Duplicate Dc binding '{}' on '{}' at {}",
            format_dc_shorthand(dc),
            name,
            span
        );
        self.used_dcs.insert(dc, name.to_owned());
        self.top_dc_names.insert(dc, name.to_owned());
        Ok(())
    }

    fn export_ns_for_pub(
        &self,
        what: &str,
        name: &str,
        span: Span,
    ) -> Result<String> {
        match self.dialect {
            StageDialect::StageL3 => {
                // Reason for fallback: Anonymous StageL 3 modules without an 'ns' header use an empty namespace string.
                Ok(self.namespace_key.clone().unwrap_or_default())
            }
            StageDialect::StageR3 => {
                self.namespace_key.clone().with_context(|| {
                    format!(
                        "'pub {what} {name}' requires a module 'ns' declaration (at {span})"
                    )
                })
            }
        }
    }

    fn record_export_by_dc(
        exports: &mut NamespaceExports,
        dc: DcChar,
        sym: ImportedSymbol,
        span: Span,
    ) -> Result<()> {
        if exports.by_dc.insert(dc, sym).is_some() {
            bail!(
                "Duplicate exported Dc shorthand 'dc/{}' at {span}",
                format_dc_shorthand(dc)
            );
        }
        Ok(())
    }

    fn seed_from_exports_and_dc_imports(
        &mut self,
        ns_exports: &HashMap<String, NamespaceExports>,
        dc_imports: &mut HashMap<DcChar, ImportedSymbol>,
    ) -> Result<()> {
        for dep_exp in ns_exports.values() {
            for (k, sdef) in &dep_exp.all_structs {
                self.struct_defs.insert(k.clone(), sdef.clone());
            }
            if self.dialect == StageDialect::StageR3 {
                for (k, bdef) in &dep_exp.all_brand_types {
                    self.brand_types.insert(k.clone(), bdef.clone());
                }
                for (k, nsdef) in &dep_exp.all_named_structs {
                    self.named_structs.insert(k.clone(), nsdef.clone());
                }
                for (k, edef) in &dep_exp.all_enums {
                    self.enums.insert(k.clone(), edef.clone());
                }
            }
        }

        let synth_span = Span { line: 1, col: 1 };
        let mut dc_import_struct_keys = Vec::new();
        for (&dc, sym) in dc_imports.iter() {
            match sym {
                ImportedSymbol::Struct {
                    namespace,
                    original_name,
                    fields,
                    dc_id,
                } => {
                    let key = (Some(namespace.clone()), original_name.clone());
                    self.dc_structs.insert(dc, key.clone());
                    self.struct_defs.entry(key.clone()).or_insert_with(|| {
                        StructDef {
                            namespace: Some(namespace.clone()),
                            original_name: original_name.clone(),
                            fields: fields.clone(),
                            is_pub: true,
                            dc_id: *dc_id,
                            span: synth_span,
                        }
                    });
                    dc_import_struct_keys.push(key);
                }
                ImportedSymbol::BrandType {
                    namespace,
                    original_name,
                    is_opaque,
                    inner_ty,
                    dc_id,
                } => {
                    ensure!(
                        self.dialect == StageDialect::StageR3,
                        "StageR 3 symbol '{original_name}' cannot be referenced inside a StageL 3 module"
                    );
                    let key = (Some(namespace.clone()), original_name.clone());
                    self.dc_nominals.insert(dc, key.clone());
                    self.brand_types.entry(key).or_insert_with(|| {
                        BrandTypeDef {
                            namespace: Some(namespace.clone()),
                            original_name: original_name.clone(),
                            is_opaque: *is_opaque,
                            inner_ty: inner_ty.clone(),
                            is_pub: true,
                            dc_id: *dc_id,
                            span: synth_span,
                        }
                    });
                }
                ImportedSymbol::NamedStruct {
                    namespace,
                    original_name,
                    fields,
                    dc_id,
                } => {
                    ensure!(
                        self.dialect == StageDialect::StageR3,
                        "StageR 3 symbol '{original_name}' cannot be referenced inside a StageL 3 module"
                    );
                    let key = (Some(namespace.clone()), original_name.clone());
                    self.dc_structs.insert(dc, key.clone());
                    self.named_structs.entry(key).or_insert_with(|| {
                        NamedStructDef {
                            namespace: Some(namespace.clone()),
                            original_name: original_name.clone(),
                            fields: fields.clone(),
                            is_pub: true,
                            dc_id: *dc_id,
                            span: synth_span,
                        }
                    });
                }
                ImportedSymbol::Enum {
                    namespace,
                    original_name,
                    variants,
                    dc_id,
                } => {
                    ensure!(
                        self.dialect == StageDialect::StageR3,
                        "StageR 3 symbol '{original_name}' cannot be referenced inside a StageL 3 module"
                    );
                    let key = (Some(namespace.clone()), original_name.clone());
                    self.dc_nominals.insert(dc, key.clone());
                    self.enums.entry(key).or_insert_with(|| EnumDef {
                        namespace: Some(namespace.clone()),
                        original_name: original_name.clone(),
                        variants: variants.clone(),
                        is_pub: true,
                        dc_id: *dc_id,
                        span: synth_span,
                    });
                }
                ImportedSymbol::Value { .. }
                | ImportedSymbol::Routine { .. } => {}
            }
        }

        // Canonicalize transitive type tables with `forbid_stagel_internal = false`
        // so Stage 1 transitive helpers remain linkable unless referenced directly.
        self.canonicalize_seeded_tables(
            dc_imports,
            &dc_import_struct_keys,
            synth_span,
        )
    }

    fn canonicalize_seeded_tables(
        &mut self,
        dc_imports: &mut HashMap<DcChar, ImportedSymbol>,
        dc_import_struct_keys: &[(Option<String>, String)],
        span: Span,
    ) -> Result<()> {
        let env_struct_keys: Vec<_> = if self.dialect == StageDialect::StageR3 {
            self.struct_defs.keys().cloned().collect()
        } else {
            dc_import_struct_keys.to_vec()
        };
        let empty_local = HashMap::new();
        let empty_top_dc = HashMap::new();

        for k in env_struct_keys {
            if let Some(existing) = self.struct_defs.get(&k).cloned() {
                let mut out = Vec::with_capacity(existing.fields.len());
                for f in &existing.fields {
                    out.push(self.canonicalize_detached(
                        f,
                        &empty_local,
                        &empty_local,
                        &empty_top_dc,
                        span,
                    )?);
                }
                if let Some(slot) = self.struct_defs.get_mut(&k) {
                    slot.fields = out;
                }
            }
        }

        if self.dialect == StageDialect::StageR3 {
            let brand_keys: Vec<_> = self.brand_types.keys().cloned().collect();
            for k in brand_keys {
                if let Some(existing) = self.brand_types.get(&k).cloned() {
                    let ci = self.canonicalize_detached(
                        &existing.inner_ty,
                        &empty_local,
                        &empty_local,
                        &empty_top_dc,
                        span,
                    )?;
                    if let Some(slot) = self.brand_types.get_mut(&k) {
                        slot.inner_ty = ci;
                    }
                }
            }

            let ns_keys: Vec<_> = self.named_structs.keys().cloned().collect();
            for k in ns_keys {
                if let Some(existing) = self.named_structs.get(&k).cloned() {
                    let mut out = Vec::with_capacity(existing.fields.len());
                    for (fname, fty) in &existing.fields {
                        let cf = self.canonicalize_detached(
                            fty,
                            &empty_local,
                            &empty_local,
                            &empty_top_dc,
                            span,
                        )?;
                        out.push((fname.clone(), cf));
                    }
                    if let Some(slot) = self.named_structs.get_mut(&k) {
                        slot.fields = out;
                    }
                }
            }

            let enum_keys: Vec<_> = self.enums.keys().cloned().collect();
            for k in enum_keys {
                if let Some(existing) = self.enums.get(&k).cloned() {
                    let mut out = Vec::with_capacity(existing.variants.len());
                    for v in &existing.variants {
                        let cp = match &v.payload {
                            Some(pty) => Some(self.canonicalize_detached(
                                pty,
                                &empty_local,
                                &empty_local,
                                &empty_top_dc,
                                span,
                            )?),
                            None => None,
                        };
                        out.push(EnumVariantDef {
                            name: v.name.clone(),
                            payload: cp,
                            span: v.span,
                        });
                    }
                    if let Some(slot) = self.enums.get_mut(&k) {
                        slot.variants = out;
                    }
                }
            }
        }

        let dcs: Vec<DcChar> = dc_imports.keys().copied().collect();
        for dc in dcs {
            let Some(mut sym) = dc_imports.get(&dc).cloned() else {
                continue;
            };
            if let Some(original_name) = sym.stager_only_name() {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 symbol '{original_name}' cannot be referenced inside a StageL 3 module"
                );
            }
            sym.try_map_types(|ty| {
                self.canonicalize_detached(
                    ty,
                    &empty_local,
                    &empty_local,
                    &empty_top_dc,
                    span,
                )
            })?;
            bridge_import_symbol_if_stager(self.dialect, &mut sym)?;
            dc_imports.insert(dc, sym);
        }

        Ok(())
    }

    fn canonicalize_detached(
        &self,
        ty: &StageLType,
        local_structs: &HashMap<String, (Option<String>, String)>,
        local_nominals: &HashMap<String, (Option<String>, String)>,
        top_dc_names: &HashMap<DcChar, String>,
        span: Span,
    ) -> Result<StageLType> {
        let env = TypeEnv {
            dialect: self.dialect,
            forbid_stagel_internal: false,
            local_structs,
            dc_structs: &self.dc_structs,
            local_nominals,
            dc_nominals: &self.dc_nominals,
            defs: self.type_defs(),
            top_dc_names,
        };
        canonicalize_type(ty, &env, span)
    }

    fn resolve_use_item_symbols(
        &mut self,
        target_ns: &str,
        dep_exp: &NamespaceExports,
        item: &UseImportItem,
        dc_imports: &mut HashMap<DcChar, ImportedSymbol>,
    ) -> Result<Vec<ImportedSymbol>> {
        let mut syms = match &item.target {
            ImportTarget::Name(name) => {
                if let Some((ty, dc_id)) = dep_exp.values.get(name) {
                    vec![ImportedSymbol::Value {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        ty: ty.clone(),
                        dc_id: *dc_id,
                    }]
                } else if let Some(sig_list) = dep_exp.routines.get(name) {
                    sig_list
                        .iter()
                        .map(|sig| ImportedSymbol::Routine {
                            namespace: target_ns.to_owned(),
                            original_name: name.clone(),
                            sig: sig.clone(),
                        })
                        .collect()
                } else if (target_ns == "std" || target_ns.starts_with("std/"))
                    && let Some(alt_name) =
                        overloadable_std_synonym_counterpart(name)
                    && let Some(sig_list) = dep_exp.routines.get(alt_name)
                {
                    sig_list
                        .iter()
                        .map(|sig| ImportedSymbol::Routine {
                            namespace: target_ns.to_owned(),
                            original_name: alt_name.to_owned(),
                            sig: sig.clone(),
                        })
                        .collect()
                } else if let Some(bdef) = dep_exp.brand_types.get(name) {
                    vec![ImportedSymbol::BrandType {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        is_opaque: bdef.is_opaque,
                        inner_ty: bdef.inner_ty.clone(),
                        dc_id: bdef.dc_id,
                    }]
                } else if let Some(nsdef) = dep_exp.named_structs.get(name) {
                    vec![ImportedSymbol::NamedStruct {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        fields: nsdef.fields.clone(),
                        dc_id: nsdef.dc_id,
                    }]
                } else if let Some(edef) = dep_exp.enums.get(name) {
                    vec![ImportedSymbol::Enum {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        variants: edef.variants.clone(),
                        dc_id: edef.dc_id,
                    }]
                } else if let Some(sdef) = dep_exp.structs.get(name) {
                    vec![ImportedSymbol::Struct {
                        namespace: target_ns.to_owned(),
                        original_name: name.clone(),
                        fields: sdef.fields.clone(),
                        dc_id: sdef.dc_id,
                    }]
                } else {
                    bail!(
                        "Namespace '{target_ns}' has no exported 'pub' symbol named '{name}' (at {})",
                        item.span
                    );
                }
            }
            ImportTarget::Dc(dc) => {
                let mut single =
                    dep_exp.by_dc.get(dc).cloned().with_context(|| {
                        format!(
                            "Namespace '{target_ns}' has no exported 'pub dc/{}' (at {})",
                            format_dc_shorthand(*dc),
                            item.span
                        )
                    })?;
                bridge_import_symbol_if_stager(self.dialect, &mut single)?;
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
                            self.dc_structs.insert(
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
                            self.dc_nominals.insert(
                                *dc,
                                (Some(namespace.clone()), original_name.clone()),
                            );
                        }
                        ImportedSymbol::Value { .. }
                        | ImportedSymbol::Routine { .. } => {}
                    }
                }
                return Ok(vec![single]);
            }
        };
        for sym in &mut syms {
            bridge_import_symbol_if_stager(self.dialect, sym)?;
        }
        Ok(syms)
    }

    fn check_imported_type_for_dialect(
        &self,
        ty: &StageLType,
        sym_desc: &str,
        span: Span,
    ) -> Result<()> {
        if self.forbid_stagel_internal && type_references_stagel_internal(ty) {
            bail!(
                "StageL-internal type referenced inside imported {sym_desc} cannot be used in Stage 2/StageR module at {span}"
            );
        }
        let canon = self.canonicalize(ty, self.forbid_stagel_internal, span)?;
        check_type_well_formed(
            &canon,
            &self.type_defs(),
            self.dialect,
            BufPolicy::Disallowed,
            span,
        )
    }

    fn register_imported_type_key(
        &mut self,
        local_name: &str,
        namespace: &str,
        original_name: &str,
        is_struct_table: bool,
        kind_label: &str,
        span: Span,
    ) -> Result<(Option<String>, String)> {
        ensure!(
            !self.name_conflicts_top(local_name),
            "Duplicate or conflicting imported {kind_label} '{local_name}' at {span}"
        );
        let key = (Some(namespace.to_owned()), original_name.to_owned());
        if is_struct_table {
            self.local_structs.insert(local_name.to_owned(), key.clone());
        } else {
            self.local_nominals
                .insert(local_name.to_owned(), key.clone());
        }
        Ok(key)
    }

    fn register_imported_symbol(
        &mut self,
        resolved: &ImportedSymbol,
        local_name: &str,
        span: Span,
    ) -> Result<()> {
        let orig = resolved.original_name();
        if self.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal symbol '{orig}' cannot be imported into Stage 2/StageR module at {span}"
            );
        }
        if let Some(stager_orig) = resolved.stager_only_name() {
            ensure!(
                self.dialect == StageDialect::StageR3,
                "StageR 3 symbol '{stager_orig}' cannot be imported into a StageL 3 module at {span}"
            );
        }
        match resolved {
            ImportedSymbol::Value {
                namespace,
                original_name,
                ty,
                dc_id,
            } => {
                self.check_imported_type_for_dialect(
                    ty,
                    &format!("constant '{original_name}'"),
                    span,
                )?;
                ensure!(
                    !self.routine_candidates.contains_key(local_name)
                        && !self.local_structs.contains_key(local_name)
                        && !self.local_nominals.contains_key(local_name),
                    "Conflicting imported name '{local_name}' at {span}: cannot share a name with a routine or type"
                );
                ensure!(
                    !self.value_candidates.contains_key(local_name),
                    "Duplicate or conflicting imported constant '{local_name}' at {span} (constant names must have a single definition)"
                );
                self.value_candidates.insert(
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
                self.check_imported_type_for_dialect(
                    &sig.return_type,
                    &format!("routine '{original_name}' return type"),
                    span,
                )?;
                for p in &sig.params {
                    self.check_imported_type_for_dialect(
                        p,
                        &format!("routine '{original_name}' parameter"),
                        span,
                    )?;
                }
                ensure!(
                    !self.value_candidates.contains_key(local_name)
                        && !self.local_structs.contains_key(local_name)
                        && !self.local_nominals.contains_key(local_name),
                    "Conflicting imported name '{local_name}' at {span}: cannot share a name with a constant or type"
                );
                let list = self
                    .routine_candidates
                    .entry(local_name.to_owned())
                    .or_default();
                if let Some(first_existing) = list.first() {
                    ensure!(
                        first_existing.sig.params.len() == sig.params.len(),
                        "Conflicting parameter count for imported routine '{local_name}' at {span}: expected {} parameter(s), found {}",
                        first_existing.sig.params.len(),
                        sig.params.len()
                    );
                }
                ensure!(
                    !list.iter().any(|c| c.sig.params == sig.params),
                    "Duplicate or conflicting imported routine '{local_name}' with parameter types ({}) at {span}",
                    format_param_types(&sig.params)
                );
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
                for f in fields {
                    self.check_imported_type_for_dialect(
                        f,
                        &format!("struct '{original_name}'"),
                        span,
                    )?;
                }
                let key = self.register_imported_type_key(
                    local_name,
                    namespace,
                    original_name,
                    true,
                    "struct",
                    span,
                )?;
                self.struct_defs.entry(key).or_insert_with(|| StructDef {
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
                self.check_imported_type_for_dialect(
                    inner_ty,
                    &format!("nominal type '{original_name}'"),
                    span,
                )?;
                let key = self.register_imported_type_key(
                    local_name,
                    namespace,
                    original_name,
                    false,
                    "nominal type",
                    span,
                )?;
                self.brand_types.entry(key).or_insert_with(|| BrandTypeDef {
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
                for (fname, fty) in fields {
                    ensure!(
                        !self.forbid_stagel_internal
                            || !is_stagel_internal_name(fname),
                        "StageL-internal field '{fname}' in imported struct '{original_name}' cannot be imported into StageR 3 module at {span}"
                    );
                    self.check_imported_type_for_dialect(
                        fty,
                        &format!("struct '{original_name}' field '{fname}'"),
                        span,
                    )?;
                }
                let key = self.register_imported_type_key(
                    local_name,
                    namespace,
                    original_name,
                    true,
                    "struct",
                    span,
                )?;
                self.named_structs
                    .entry(key)
                    .or_insert_with(|| NamedStructDef {
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
                for v in variants {
                    ensure!(
                        !self.forbid_stagel_internal
                            || !is_stagel_internal_name(&v.name),
                        "StageL-internal variant '{}' in imported enum '{original_name}' cannot be imported into StageR 3 module at {span}",
                        v.name
                    );
                    if let Some(ref pty) = v.payload {
                        self.check_imported_type_for_dialect(
                            pty,
                            &format!(
                                "enum '{original_name}' variant '{}'",
                                v.name
                            ),
                            span,
                        )?;
                    }
                }
                let key = self.register_imported_type_key(
                    local_name,
                    namespace,
                    original_name,
                    false,
                    "enum",
                    span,
                )?;
                self.enums.entry(key).or_insert_with(|| EnumDef {
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

    fn register_top_type_header(
        &mut self,
        kind: TopTypeDeclKind,
        name: &str,
        dc: Option<(DcChar, bool)>,
        span: Span,
    ) -> Result<(Option<String>, String)> {
        match kind {
            TopTypeDeclKind::PositionalStruct { has_fields } => {
                ensure!(
                    self.dialect == StageDialect::StageL3,
                    "Positional StageL 'struct {name}' at {span} is forbidden in StageR 3 (use named-field 'struct' instead)"
                );
                if self.forbid_stagel_internal
                    && is_stagel_internal_name(name)
                {
                    bail!(
                        "StageL-internal struct '{name}' cannot be declared in Stage 2/StageR module at {span}"
                    );
                }
                ensure!(
                    has_fields,
                    "Positional struct '{name}' at {span} must declare at least 1 slot"
                );
                ensure!(
                    !self.name_conflicts_top(name),
                    "Struct '{name}' at {span} duplicates/shadows an existing declaration or import"
                );
            }
            TopTypeDeclKind::BrandType
            | TopTypeDeclKind::NamedStruct
            | TopTypeDeclKind::Enum => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "StageR 3 declaration is not supported in StageL 3 validator"
                );
                let internal_kind = match kind {
                    TopTypeDeclKind::BrandType => "type",
                    TopTypeDeclKind::NamedStruct => "struct",
                    TopTypeDeclKind::Enum => "enum",
                    TopTypeDeclKind::PositionalStruct { .. } => "struct",
                };
                ensure!(
                    !self.forbid_stagel_internal
                        || !is_stagel_internal_name(name),
                    "StageL-internal {internal_kind} '{name}' cannot be declared in StageR 3 module at {span}"
                );
                ensure!(
                    !self.name_conflicts_top(name),
                    "Duplicate or conflicting declaration name '{name}' at {span}"
                );
            }
        }
        let is_struct_table = matches!(
            kind,
            TopTypeDeclKind::PositionalStruct { .. }
                | TopTypeDeclKind::NamedStruct
        );
        let key = (self.namespace_key.clone(), name.to_owned());
        if let Some((dc_id, pub_dc_syntax)) = dc {
            self.register_dc_binding(dc_id, name, pub_dc_syntax, span)?;
            if is_struct_table {
                self.dc_structs.insert(dc_id, key.clone());
            } else {
                self.dc_nominals.insert(dc_id, key.clone());
            }
        }
        if is_struct_table {
            self.local_structs.insert(name.to_owned(), key.clone());
        } else {
            self.local_nominals.insert(name.to_owned(), key.clone());
        }
        Ok(key)
    }

    fn record_pub_dc_export(
        &self,
        exports: &mut NamespaceExports,
        kind_kw: &str,
        name: &str,
        dc_id: Option<DcChar>,
        span: Span,
        make_dc_sym: impl FnOnce(String, DcChar) -> ImportedSymbol,
    ) -> Result<()> {
        let export_ns = self.export_ns_for_pub(kind_kw, name, span)?;
        if let Some(dc) = dc_id {
            Self::record_export_by_dc(
                exports,
                dc,
                make_dc_sym(export_ns, dc),
                span,
            )?;
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Registers, cycle-checks, shape-checks, and exports all top-level type declarations"
    )]
    fn register_and_validate_types(
        &mut self,
        items: &[TopDecl],
        exports: &mut NamespaceExports,
    ) -> Result<()> {
        let mut local_struct_decls: Vec<&PositionalStructDecl> = Vec::new();
        let mut brand_decls: Vec<&BrandTypeDecl> = Vec::new();
        let mut named_struct_decls: Vec<&NamedStructDecl> = Vec::new();
        let mut enum_decls: Vec<&EnumDecl> = Vec::new();

        // Pass 2a: Register top-level type identifiers and `@dc` bindings.
        for decl in items {
            match decl {
                TopDecl::Struct(s) => {
                    self.register_top_type_header(
                        TopTypeDeclKind::PositionalStruct {
                            has_fields: !s.fields.is_empty(),
                        },
                        &s.name,
                        s.dc_id.map(|dc| (dc, s.pub_dc_syntax)),
                        s.span,
                    )?;
                    local_struct_decls.push(s);
                }
                TopDecl::BrandType(b) => {
                    let key = self.register_top_type_header(
                        TopTypeDeclKind::BrandType,
                        &b.name,
                        b.dc_id.map(|dc| (dc, b.pub_dc_syntax)),
                        b.span,
                    )?;
                    if b.is_opaque {
                        self.local_declared_opaque.insert(key);
                    }
                    brand_decls.push(b);
                }
                TopDecl::NamedStruct(ns) => {
                    self.register_top_type_header(
                        TopTypeDeclKind::NamedStruct,
                        &ns.name,
                        ns.dc_id.map(|dc| (dc, ns.pub_dc_syntax)),
                        ns.span,
                    )?;
                    named_struct_decls.push(ns);
                }
                TopDecl::Enum(en) => {
                    self.register_top_type_header(
                        TopTypeDeclKind::Enum,
                        &en.name,
                        en.dc_id.map(|dc| (dc, en.pub_dc_syntax)),
                        en.span,
                    )?;
                    enum_decls.push(en);
                }
                TopDecl::Value(_)
                | TopDecl::Routine(_)
                | TopDecl::Comment(_) => {}
            }
        }

        // Pass 2b-1: Canonicalize all field / inner / variant payload types.
        for &s in &local_struct_decls {
            let mut canon_fields = Vec::with_capacity(s.fields.len());
            for f in &s.fields {
                ensure!(
                    *f != StageLType::Void && *f != StageLType::Never,
                    "Field of struct '{}' at {} cannot have type '{}'",
                    s.name,
                    s.span,
                    f
                );
                let cf = self.canonicalize(
                    f,
                    self.forbid_stagel_internal,
                    s.span,
                )?;
                canon_fields.push(cf);
            }
            let canon_key = (self.namespace_key.clone(), s.name.clone());
            self.struct_defs.insert(
                canon_key,
                StructDef {
                    namespace: self.namespace_key.clone(),
                    original_name: s.name.clone(),
                    fields: canon_fields,
                    is_pub: s.is_pub,
                    dc_id: s.dc_id,
                    span: s.span,
                },
            );
        }

        for &b in &brand_decls {
            let canon_inner = self.canonicalize(
                &b.inner_ty,
                self.forbid_stagel_internal,
                b.span,
            )?;
            let key = (self.namespace_key.clone(), b.name.clone());
            self.brand_types.insert(
                key,
                BrandTypeDef {
                    namespace: self.namespace_key.clone(),
                    original_name: b.name.clone(),
                    is_opaque: b.is_opaque,
                    inner_ty: canon_inner,
                    is_pub: b.is_pub,
                    dc_id: b.dc_id,
                    span: b.span,
                },
            );
        }

        for &ns in &named_struct_decls {
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
                    !self.forbid_stagel_internal
                        || !is_stagel_internal_name(&f.name),
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
                let cf = self.canonicalize(
                    &f.ty,
                    self.forbid_stagel_internal,
                    f.span,
                )?;
                canon_fields.push((f.name.clone(), cf));
            }
            let key = (self.namespace_key.clone(), ns.name.clone());
            self.named_structs.insert(
                key,
                NamedStructDef {
                    namespace: self.namespace_key.clone(),
                    original_name: ns.name.clone(),
                    fields: canon_fields,
                    is_pub: ns.is_pub,
                    dc_id: ns.dc_id,
                    span: ns.span,
                },
            );
        }

        for &en in &enum_decls {
            let mut seen_vars = HashSet::new();
            let mut canon_vars = Vec::with_capacity(en.variants.len());
            for v in &en.variants {
                ensure!(
                    !self.forbid_stagel_internal
                        || !is_stagel_internal_name(&v.name),
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
                    Some(pty) => Some(self.canonicalize(
                        pty,
                        self.forbid_stagel_internal,
                        v.span,
                    )?),
                    None => None,
                };
                canon_vars.push(EnumVariantDef {
                    name: v.name.clone(),
                    payload: canon_payload,
                    span: v.span,
                });
            }
            let key = (self.namespace_key.clone(), en.name.clone());
            self.enums.insert(
                key,
                EnumDef {
                    namespace: self.namespace_key.clone(),
                    original_name: en.name.clone(),
                    variants: canon_vars,
                    is_pub: en.is_pub,
                    dc_id: en.dc_id,
                    span: en.span,
                },
            );
        }

        // Pass 2b-2: Reject unguarded infinite-size cycles BEFORE shape/transitivity checks.
        check_unguarded_type_cycles(&self.type_defs())?;

        // Pass 2b-3: Verify well-formedness & buffer rules, then record `exports`.
        let defs = self.type_defs();
        for &s in &local_struct_decls {
            let key = (self.namespace_key.clone(), s.name.clone());
            let sdef = self
                .struct_defs
                .get(&key)
                .cloned()
                .context("Missing registered struct definition")?;
            let policy = if s.is_pub {
                BufPolicy::Disallowed
            } else {
                BufPolicy::LocalAllowed
            };
            for cf in &sdef.fields {
                check_type_well_formed(
                    cf,
                    &defs,
                    StageDialect::StageL3,
                    policy,
                    s.span,
                )?;
            }
            if s.is_pub {
                exports.structs.insert(s.name.clone(), sdef.clone());
                self.record_pub_dc_export(
                    exports,
                    "struct",
                    &s.name,
                    s.dc_id,
                    s.span,
                    |export_ns, dc| ImportedSymbol::Struct {
                        namespace: export_ns,
                        original_name: s.name.clone(),
                        fields: sdef.fields,
                        dc_id: Some(dc),
                    },
                )?;
            }
        }

        for &b in &brand_decls {
            let key = (self.namespace_key.clone(), b.name.clone());
            let def = self
                .brand_types
                .get(&key)
                .cloned()
                .context("Missing registered brand/type definition")?;
            check_stager_value_type_shape(&def.inner_ty, &defs, b.span)?;
            if b.is_pub {
                let kind_kw = if b.is_opaque { "type" } else { "brand" };
                exports.brand_types.insert(b.name.clone(), def.clone());
                exports.all_brand_types.insert(key, def.clone());
                self.record_pub_dc_export(
                    exports,
                    kind_kw,
                    &b.name,
                    b.dc_id,
                    b.span,
                    |export_ns, dc| ImportedSymbol::BrandType {
                        namespace: export_ns,
                        original_name: b.name.clone(),
                        is_opaque: b.is_opaque,
                        inner_ty: def.inner_ty,
                        dc_id: Some(dc),
                    },
                )?;
            }
        }

        for &ns in &named_struct_decls {
            let key = (self.namespace_key.clone(), ns.name.clone());
            let def = self
                .named_structs
                .get(&key)
                .cloned()
                .context("Missing registered named struct definition")?;
            for (field_decl, (_, cf)) in ns.fields.iter().zip(&def.fields) {
                check_stager_value_type_shape(cf, &defs, field_decl.span)?;
            }
            if ns.is_pub {
                exports.named_structs.insert(ns.name.clone(), def.clone());
                exports.all_named_structs.insert(key, def.clone());
                self.record_pub_dc_export(
                    exports,
                    "struct",
                    &ns.name,
                    ns.dc_id,
                    ns.span,
                    |export_ns, dc| ImportedSymbol::NamedStruct {
                        namespace: export_ns,
                        original_name: ns.name.clone(),
                        fields: def.fields,
                        dc_id: Some(dc),
                    },
                )?;
            }
        }

        for &en in &enum_decls {
            let key = (self.namespace_key.clone(), en.name.clone());
            let def = self
                .enums
                .get(&key)
                .cloned()
                .context("Missing registered enum definition")?;
            for v in &def.variants {
                if let Some(ref pty) = v.payload {
                    check_stager_value_type_shape(pty, &defs, v.span)?;
                }
            }
            if en.is_pub {
                exports.enums.insert(en.name.clone(), def.clone());
                exports.all_enums.insert(key, def.clone());
                self.record_pub_dc_export(
                    exports,
                    "enum",
                    &en.name,
                    en.dc_id,
                    en.span,
                    |export_ns, dc| ImportedSymbol::Enum {
                        namespace: export_ns,
                        original_name: en.name.clone(),
                        variants: def.variants,
                        dc_id: Some(dc),
                    },
                )?;
            }
        }

        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "Collects and validates top-level constants and routine overloads"
    )]
    fn register_top_values_and_routines<'a>(
        &mut self,
        items: &'a [TopDecl],
        exports: &mut NamespaceExports,
    ) -> Result<(Vec<&'a TopValueDecl>, Vec<&'a RoutineDecl>)> {
        let mut top_value_order = Vec::new();
        let mut top_routine_order = Vec::new();

        for decl in items {
            match decl {
                TopDecl::Value(v) => {
                    if self.forbid_stagel_internal
                        && is_stagel_internal_name(&v.name)
                    {
                        bail!(
                            "StageL-internal constant '{}' cannot be declared in Stage 2/StageR module at {}",
                            v.name,
                            v.span
                        );
                    }
                    ensure!(
                        !self.routine_candidates.contains_key(&v.name)
                            && !self.local_structs.contains_key(&v.name)
                            && !self.local_nominals.contains_key(&v.name),
                        "Top-level constant '{}' at {} duplicates/shadows an existing routine, type, or import",
                        v.name,
                        v.span
                    );
                    ensure!(
                        v.ty != StageLType::Void && v.ty != StageLType::Never,
                        "Constant '{}' at {} cannot have type '{}'",
                        v.name,
                        v.span,
                        v.ty
                    );
                    let canon_ty = self.canonicalize(
                        &v.ty,
                        self.forbid_stagel_internal,
                        v.span,
                    )?;
                    let defs = self.type_defs();
                    check_type_well_formed(
                        &canon_ty,
                        &defs,
                        self.dialect,
                        BufPolicy::Disallowed,
                        v.span,
                    )?;
                    ensure!(
                        !super::types::type_contains_buf_with_defs(
                            &canon_ty, &defs
                        ),
                        "Module-level constant '{}' at {} cannot hold local buffer 'buf/T'",
                        v.name,
                        v.span
                    );
                    if self.dialect == StageDialect::StageR3 {
                        check_stager_value_type_shape(
                            &canon_ty, &defs, v.span,
                        )?;
                    }
                    ensure!(
                        !self.value_candidates.contains_key(&v.name),
                        "Top-level constant '{}' at {} duplicates/shadows existing binding or import (constant names must have a single definition)",
                        v.name,
                        v.span
                    );
                    if let Some(dc) = v.dc_id {
                        self.register_dc_binding(
                            dc,
                            &v.name,
                            v.pub_dc_syntax,
                            v.span,
                        )?;
                        self.top_dc_values.insert(dc, canon_ty.clone());
                    }
                    self.value_candidates.insert(
                        v.name.clone(),
                        ResolvedValueTarget {
                            namespace: None,
                            original_name: v.name.clone(),
                            ty: canon_ty.clone(),
                            dc_id: v.dc_id,
                        },
                    );
                    if v.is_pub {
                        let export_ns = self.export_ns_for_pub(
                            "constant", &v.name, v.span,
                        )?;
                        exports
                            .values
                            .insert(v.name.clone(), (canon_ty.clone(), v.dc_id));
                        if let Some(dc) = v.dc_id {
                            Self::record_export_by_dc(
                                exports,
                                dc,
                                ImportedSymbol::Value {
                                    namespace: export_ns,
                                    original_name: v.name.clone(),
                                    ty: canon_ty,
                                    dc_id: Some(dc),
                                },
                                v.span,
                            )?;
                        }
                    }
                    top_value_order.push(v);
                }
                TopDecl::Routine(r) => {
                    if self.forbid_stagel_internal
                        && is_stagel_internal_name(&r.name)
                    {
                        bail!(
                            "StageL-internal routine '{}' cannot be declared in Stage 2/StageR module at {}",
                            r.name,
                            r.span
                        );
                    }
                    ensure!(
                        !self.value_candidates.contains_key(&r.name)
                            && !self.local_structs.contains_key(&r.name)
                            && !self.local_nominals.contains_key(&r.name),
                        "Routine '{}' at {} duplicates/shadows an existing constant, struct, or import",
                        r.name,
                        r.span
                    );
                    let canon_ret = self.canonicalize(
                        &r.return_type,
                        self.forbid_stagel_internal,
                        r.span,
                    )?;
                    let ret_policy = match self.dialect {
                        StageDialect::StageL3 if !r.is_pub => {
                            BufPolicy::LocalAllowed
                        }
                        StageDialect::StageL3 | StageDialect::StageR3 => {
                            BufPolicy::Disallowed
                        }
                    };
                    let defs = self.type_defs();
                    check_type_well_formed(
                        &canon_ret,
                        &defs,
                        self.dialect,
                        ret_policy,
                        r.span,
                    )?;

                    let mut seen_params = HashSet::new();
                    let mut param_tys = Vec::with_capacity(r.params.len());
                    for p in &r.params {
                        if self.forbid_stagel_internal
                            && is_stagel_internal_name(&p.name)
                        {
                            bail!(
                                "StageL-internal parameter name '{}' in '{}' at {} is forbidden in Stage 2/StageR module",
                                p.name,
                                r.name,
                                p.span
                            );
                        }
                        ensure!(
                            p.ty != StageLType::Void
                                && p.ty != StageLType::Never,
                            "Parameter '{}' of routine '{}' at {} cannot have type '{}'",
                            p.name,
                            r.name,
                            p.span,
                            p.ty
                        );
                        let canon_pty = self.canonicalize(
                            &p.ty,
                            self.forbid_stagel_internal,
                            p.span,
                        )?;
                        check_type_well_formed(
                            &canon_pty,
                            &defs,
                            self.dialect,
                            ret_policy,
                            p.span,
                        )?;
                        if self.dialect == StageDialect::StageR3 {
                            check_stager_value_type_shape(
                                &canon_pty, &defs, p.span,
                            )?;
                        }
                        ensure!(
                            seen_params.insert(p.name.clone()),
                            "Duplicate parameter '{}' in routine '{}' at {}",
                            p.name,
                            r.name,
                            p.span
                        );
                        param_tys.push(canon_pty);
                    }

                    if let Some(rlist) = self.routine_candidates.get(&r.name) {
                        if let Some(first_existing) = rlist.first() {
                            ensure!(
                                first_existing.sig.params.len()
                                    == param_tys.len(),
                                "Conflicting parameter count for overloaded routine '{}' at {}: expected {} parameter(s), found {}",
                                r.name,
                                r.span,
                                first_existing.sig.params.len(),
                                param_tys.len()
                            );
                        }
                        ensure!(
                            !rlist.iter().any(|c| c.sig.params == param_tys),
                            "Routine '{}' at {} duplicates/shadows an existing routine or import with parameter types ({}) (return-type-only overloading is disallowed)",
                            r.name,
                            r.span,
                            format_param_types(&param_tys)
                        );
                    }
                    if let Some(dc) = r.dc_id {
                        self.register_dc_binding(
                            dc,
                            &r.name,
                            r.pub_dc_syntax,
                            r.span,
                        )?;
                    }
                    let sig = RoutineSig {
                        return_type: canon_ret,
                        params: param_tys,
                        is_pub: r.is_pub,
                        dc_id: r.dc_id,
                        may_fault: self.dialect == StageDialect::StageL3
                            && expr_syntactically_contains_fault(&r.body),
                    };
                    if let Some(dc) = r.dc_id {
                        self.top_dc_routines.insert(dc, sig.clone());
                    }
                    self.routine_candidates
                        .entry(r.name.clone())
                        .or_default()
                        .push(ResolvedCallTarget {
                            namespace: None,
                            routine_name: r.name.clone(),
                            sig: sig.clone(),
                        });
                    if r.is_pub {
                        let export_ns = self.export_ns_for_pub(
                            "routine", &r.name, r.span,
                        )?;
                        exports
                            .routines
                            .entry(r.name.clone())
                            .or_default()
                            .push(sig.clone());
                        if let Some(dc) = r.dc_id {
                            Self::record_export_by_dc(
                                exports,
                                dc,
                                ImportedSymbol::Routine {
                                    namespace: export_ns,
                                    original_name: r.name.clone(),
                                    sig,
                                },
                                r.span,
                            )?;
                        }
                    }
                    top_routine_order.push(r);
                }
                TopDecl::Comment(_)
                | TopDecl::Struct(_)
                | TopDecl::BrandType(_)
                | TopDecl::NamedStruct(_)
                | TopDecl::Enum(_) => {}
            }
        }

        Ok((top_value_order, top_routine_order))
    }
}

#[derive(Clone, Copy)]
enum TopTypeDeclKind {
    PositionalStruct { has_fields: bool },
    BrandType,
    NamedStruct,
    Enum,
}

fn type_references_stagel_internal(ty: &StageLType) -> bool {
    match ty {
        StageLType::Struct(StructTypeRef::Ident(name))
        | StageLType::Nominal(StructTypeRef::Ident(name))
        | StageLType::Struct(StructTypeRef::Canonical { name, .. })
        | StageLType::Nominal(StructTypeRef::Canonical { name, .. }) => {
            is_stagel_internal_name(name)
        }
        StageLType::List(inner)
        | StageLType::Buf(inner)
        | StageLType::Option(inner)
        | StageLType::Result(inner) => type_references_stagel_internal(inner),
        StageLType::Tuple(elems) => {
            elems.iter().any(type_references_stagel_internal)
        }
        _ => false,
    }
}

fn bridge_import_symbol_if_stager(
    dialect: StageDialect,
    sym: &mut ImportedSymbol,
) -> Result<()> {
    if dialect == StageDialect::StageR3
        && matches!(
            sym,
            ImportedSymbol::Value { .. } | ImportedSymbol::Routine { .. }
        )
    {
        sym.try_map_types(|ty| Ok(bridge_stage1_type_for_stager(ty)))?;
    }
    Ok(())
}

impl<'a> ModuleValidator<'a> {
    /// Propagates Stage 1 `may_fault` transitively across routines in this
    /// module and updates `exports`, `top_dc_routines`, and `resolved_calls`.
    pub(super) fn propagate_may_fault(
        &mut self,
        top_routines: &[&RoutineDecl],
        exports: &mut NamespaceExports,
    ) -> Result<()> {
        let mut faulting: HashSet<RoutineKey> = HashSet::new();
        let mut local_edges: Vec<(RoutineKey, Vec<RoutineKey>)> =
            Vec::with_capacity(top_routines.len());

        for &r in top_routines {
            let mut params = Vec::with_capacity(r.params.len());
            for p in &r.params {
                params.push(self.canonicalize_type(&p.ty, p.span)?);
            }
            let key = RoutineKey {
                name: r.name.clone(),
                params,
            };
            let mut spans = Vec::new();
            collect_expr_call_spans(&r.body, &mut spans);
            let mut direct_fault = expr_syntactically_contains_fault(&r.body);
            let mut callees = Vec::new();
            for sp in spans {
                if let Some(rc) = self.resolved_calls.get(&sp) {
                    if rc.sig.may_fault {
                        direct_fault = true;
                    }
                    if rc.namespace.is_none() {
                        callees.push(rc.routine_key());
                    }
                }
            }
            if direct_fault {
                faulting.insert(key.clone());
            }
            local_edges.push((key, callees));
        }

        loop {
            let mut changed = false;
            for (caller_key, callees) in &local_edges {
                if !faulting.contains(caller_key)
                    && callees.iter().any(|callee| faulting.contains(callee))
                {
                    faulting.insert(caller_key.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        for (name, sigs) in &mut exports.routines {
            for sig in sigs {
                let key = RoutineKey {
                    name: name.clone(),
                    params: sig.params.clone(),
                };
                if faulting.contains(&key) {
                    sig.may_fault = true;
                }
            }
        }
        for sym in exports.by_dc.values_mut() {
            if let ImportedSymbol::Routine {
                original_name, sig, ..
            } = sym
            {
                let key = RoutineKey {
                    name: original_name.clone(),
                    params: sig.params.clone(),
                };
                if faulting.contains(&key) {
                    sig.may_fault = true;
                }
            }
        }
        for (&dc, sig) in &mut self.top_dc_routines {
            if let Some(orig_name) = self.top_dc_names.get(&dc) {
                let key = RoutineKey {
                    name: orig_name.clone(),
                    params: sig.params.clone(),
                };
                if faulting.contains(&key) {
                    sig.may_fault = true;
                }
            }
        }
        for rc in self.resolved_calls.values_mut() {
            if rc.namespace.is_none() && faulting.contains(&rc.routine_key()) {
                rc.sig.may_fault = true;
            }
        }
        Ok(())
    }
}

fn collect_expr_call_spans(expr: &Expr, out: &mut Vec<(usize, usize)>) {
    match expr {
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::ValueRef { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. }
        | Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::OptionNone { .. }
        | Expr::ErrorTag { .. } => {}
        Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
            for e in elements {
                collect_expr_call_spans(e, out);
            }
        }
        Expr::StructNew { fields, .. } => {
            for f in fields {
                collect_expr_call_spans(f, out);
            }
        }
        Expr::NamedStructNew { fields, .. } => {
            for (_, v, _) in fields {
                collect_expr_call_spans(v, out);
            }
        }
        Expr::Field { base, .. }
        | Expr::NamedField { base, .. }
        | Expr::BufNew { capacity: base, .. }
        | Expr::Conv { expr: base, .. }
        | Expr::Bitcast { expr: base, .. }
        | Expr::WrapNominal { expr: base, .. }
        | Expr::UnwrapNominal { expr: base, .. }
        | Expr::Propagate { expr: base, .. }
        | Expr::Const { value: base, .. }
        | Expr::Set { value: base, .. }
        | Expr::OptionSome { value: base, .. }
        | Expr::ResultOk { value: base, .. }
        | Expr::ResultErr { error: base, .. } => {
            collect_expr_call_spans(base, out);
        }
        Expr::Let { init, .. } => {
            if let Some(e) = init {
                collect_expr_call_spans(e, out);
            }
        }
        Expr::Return { value, .. } => {
            if let Some(e) = value {
                collect_expr_call_spans(e, out);
            }
        }
        Expr::EnumVariant { payload, .. } => {
            if let Some(e) = payload {
                collect_expr_call_spans(e, out);
            }
        }
        Expr::With { base, value, .. }
        | Expr::NamedWith { base, value, .. }
        | Expr::While {
            cond: base,
            body: value,
            ..
        }
        | Expr::ForEach {
            list: base,
            body: value,
            ..
        } => {
            collect_expr_call_spans(base, out);
            collect_expr_call_spans(value, out);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_expr_call_spans(cond, out);
            collect_expr_call_spans(then_branch, out);
            collect_expr_call_spans(else_branch, out);
        }
        Expr::Unary { operand, span, .. } => {
            out.push((span.line, span.col));
            collect_expr_call_spans(operand, out);
        }
        Expr::Binary {
            left, right, span, ..
        } => {
            out.push((span.line, span.col));
            collect_expr_call_spans(left, out);
            collect_expr_call_spans(right, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            span,
            ..
        } => {
            out.push((span.line, span.col));
            collect_expr_call_spans(first, out);
            collect_expr_call_spans(second, out);
            collect_expr_call_spans(third, out);
        }
        Expr::Call { args, span, .. } => {
            out.push((span.line, span.col));
            for a in args {
                collect_expr_call_spans(a, out);
            }
        }
        Expr::RoutineDecl(r) => collect_expr_call_spans(&r.body, out),
        Expr::AnonRoutine(r) => collect_expr_call_spans(&r.body, out),
        Expr::InlineCall {
            routine,
            args,
            span,
        } => {
            out.push((span.line, span.col));
            collect_expr_call_spans(&routine.body, out);
            for a in args {
                collect_expr_call_spans(a, out);
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(sub) = item {
                    collect_expr_call_spans(sub, out);
                }
            }
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            collect_expr_call_spans(scrutinee, out);
            for a in arms {
                collect_expr_call_spans(&a.body, out);
            }
            if let Some(fb) = fallback {
                collect_expr_call_spans(fb, out);
            }
        }
    }
}

fn desugar_expr_in_place(
    expr: &mut Expr,
    resolved_calls: &HashMap<(usize, usize), ResolvedCallTarget>,
    overload_call_idents: &HashMap<(usize, usize), String>,
) {
    match expr {
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::ValueRef { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. }
        | Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::OptionNone { .. }
        | Expr::ErrorTag { .. } => {}
        Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
            for e in elements {
                desugar_expr_in_place(e, resolved_calls, overload_call_idents);
            }
        }
        Expr::StructNew { fields, .. } => {
            for f in fields {
                desugar_expr_in_place(f, resolved_calls, overload_call_idents);
            }
        }
        Expr::NamedStructNew { fields, .. } => {
            for (_, v, _) in fields {
                desugar_expr_in_place(v, resolved_calls, overload_call_idents);
            }
        }
        Expr::Field { base, .. }
        | Expr::NamedField { base, .. }
        | Expr::BufNew { capacity: base, .. }
        | Expr::Conv { expr: base, .. }
        | Expr::Bitcast { expr: base, .. }
        | Expr::WrapNominal { expr: base, .. }
        | Expr::UnwrapNominal { expr: base, .. }
        | Expr::Propagate { expr: base, .. }
        | Expr::Const { value: base, .. }
        | Expr::Set { value: base, .. }
        | Expr::OptionSome { value: base, .. }
        | Expr::ResultOk { value: base, .. }
        | Expr::ResultErr { error: base, .. } => {
            desugar_expr_in_place(base, resolved_calls, overload_call_idents);
        }
        Expr::Let { init, .. } => {
            if let Some(e) = init {
                desugar_expr_in_place(e, resolved_calls, overload_call_idents);
            }
        }
        Expr::Return { value, .. } => {
            if let Some(e) = value {
                desugar_expr_in_place(e, resolved_calls, overload_call_idents);
            }
        }
        Expr::EnumVariant { payload, .. } => {
            if let Some(e) = payload {
                desugar_expr_in_place(e, resolved_calls, overload_call_idents);
            }
        }
        Expr::With { base, value, .. }
        | Expr::NamedWith { base, value, .. }
        | Expr::While {
            cond: base,
            body: value,
            ..
        }
        | Expr::ForEach {
            list: base,
            body: value,
            ..
        } => {
            desugar_expr_in_place(base, resolved_calls, overload_call_idents);
            desugar_expr_in_place(value, resolved_calls, overload_call_idents);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            desugar_expr_in_place(cond, resolved_calls, overload_call_idents);
            desugar_expr_in_place(
                then_branch,
                resolved_calls,
                overload_call_idents,
            );
            desugar_expr_in_place(
                else_branch,
                resolved_calls,
                overload_call_idents,
            );
        }
        Expr::Unary { operand, .. } => {
            desugar_expr_in_place(
                operand,
                resolved_calls,
                overload_call_idents,
            );
        }
        Expr::Binary { left, right, .. } => {
            desugar_expr_in_place(left, resolved_calls, overload_call_idents);
            desugar_expr_in_place(right, resolved_calls, overload_call_idents);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            desugar_expr_in_place(first, resolved_calls, overload_call_idents);
            desugar_expr_in_place(second, resolved_calls, overload_call_idents);
            desugar_expr_in_place(third, resolved_calls, overload_call_idents);
        }
        Expr::Call {
            target, args, span, ..
        } => {
            for a in args {
                desugar_expr_in_place(a, resolved_calls, overload_call_idents);
            }
            if let SymbolTarget::Ident(ident) = target
                && let Some(mapped) =
                    overload_call_idents.get(&(span.line, span.col))
            {
                ident.clone_from(mapped);
            }
        }
        Expr::RoutineDecl(r) => {
            desugar_routine_in_place(r, resolved_calls, overload_call_idents);
        }
        Expr::AnonRoutine(r) => {
            desugar_expr_in_place(
                &mut r.body,
                resolved_calls,
                overload_call_idents,
            );
        }
        Expr::InlineCall { routine, args, .. } => {
            desugar_expr_in_place(
                &mut routine.body,
                resolved_calls,
                overload_call_idents,
            );
            for a in args {
                desugar_expr_in_place(a, resolved_calls, overload_call_idents);
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(sub) = item {
                    desugar_expr_in_place(
                        sub,
                        resolved_calls,
                        overload_call_idents,
                    );
                }
            }
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            desugar_expr_in_place(
                scrutinee,
                resolved_calls,
                overload_call_idents,
            );
            for a in arms {
                desugar_expr_in_place(
                    &mut a.body,
                    resolved_calls,
                    overload_call_idents,
                );
            }
            if let Some(fb) = fallback {
                desugar_expr_in_place(fb, resolved_calls, overload_call_idents);
            }
        }
    }

    let op_span = expr.span();
    if !resolved_calls.contains_key(&(op_span.line, op_span.col)) {
        return;
    }
    if !matches!(
        expr,
        Expr::Unary { .. } | Expr::Binary { .. } | Expr::Ternary { .. }
    ) {
        return;
    }
    let matched_ident = overload_call_idents
        .get(&(op_span.line, op_span.col))
        .cloned();
    let taken = std::mem::replace(expr, Expr::Nop { span: op_span });
    *expr = match taken {
        Expr::Unary { op, operand, span } => Expr::Call {
            // Reason for fallback: defaults to canonical op.keyword() when the local candidate key matches op.keyword().
            target: SymbolTarget::Ident(
                matched_ident.unwrap_or_else(|| op.keyword().to_owned()),
            ),
            args: vec![*operand],
            span,
        },
        Expr::Binary {
            op,
            left,
            right,
            span,
        } => Expr::Call {
            // Reason for fallback: defaults to canonical op.keyword() when the local candidate key matches op.keyword().
            target: SymbolTarget::Ident(
                matched_ident.unwrap_or_else(|| op.keyword().to_owned()),
            ),
            args: vec![*left, *right],
            span,
        },
        Expr::Ternary {
            op,
            first,
            second,
            third,
            span,
        } => Expr::Call {
            // Reason for fallback: defaults to canonical op.keyword() when the local candidate key matches op.keyword().
            target: SymbolTarget::Ident(
                matched_ident.unwrap_or_else(|| op.keyword().to_owned()),
            ),
            args: vec![*first, *second, *third],
            span,
        },
        other => other,
    };
}

fn desugar_routine_in_place(
    r: &mut RoutineDecl,
    resolved_calls: &HashMap<(usize, usize), ResolvedCallTarget>,
    overload_call_idents: &HashMap<(usize, usize), String>,
) {
    for tb in &mut r.tests {
        for tc in &mut tb.cases {
            for inp in &mut tc.inputs {
                desugar_expr_in_place(
                    inp,
                    resolved_calls,
                    overload_call_idents,
                );
            }
            if let TestExpected::Expr(exp) = &mut tc.expected {
                desugar_expr_in_place(
                    exp,
                    resolved_calls,
                    overload_call_idents,
                );
            }
        }
    }
    desugar_expr_in_place(&mut r.body, resolved_calls, overload_call_idents);
}

fn desugar_overloaded_ops_in_module(
    module: &Module,
    resolved_calls: &HashMap<(usize, usize), ResolvedCallTarget>,
    overload_call_idents: &HashMap<(usize, usize), String>,
) -> Module {
    let mut out = module.clone();
    if resolved_calls.is_empty() && overload_call_idents.is_empty() {
        return out;
    }
    for item in &mut out.items {
        match item {
            TopDecl::Value(v) => {
                desugar_expr_in_place(
                    &mut v.value,
                    resolved_calls,
                    overload_call_idents,
                );
            }
            TopDecl::Routine(r) => {
                desugar_routine_in_place(
                    r,
                    resolved_calls,
                    overload_call_idents,
                );
            }
            TopDecl::Struct(_)
            | TopDecl::BrandType(_)
            | TopDecl::NamedStruct(_)
            | TopDecl::Enum(_)
            | TopDecl::Comment(_) => {}
        }
    }
    out
}

pub(super) fn lower_module_for_execution(module: &Module) -> Module {
    let resolver = default_syntax_resolver();
    lower_module_inline_calls_and_nested_self_refs(module, resolver.as_ref())
}

fn lower_module_inline_calls_and_nested_self_refs(
    module: &Module,
    resolver: &dyn SyntaxRuleResolver,
) -> Module {
    let mut out = module.clone();
    let mut anon_counter = 0usize;
    for item in &mut out.items {
        if let TopDecl::Routine(r) = item {
            lower_routine_body_expr(
                &mut r.body,
                None,
                &mut anon_counter,
                resolver,
            );
        }
    }
    out
}

fn lower_routine_body_expr(
    expr: &mut Expr,
    enclosing_nested_name: Option<&str>,
    anon_counter: &mut usize,
    resolver: &dyn SyntaxRuleResolver,
) {
    match expr {
        Expr::RoutineDecl(nr) => {
            let name = nr.name.clone();
            lower_routine_body_expr(
                &mut nr.body,
                Some(&name),
                anon_counter,
                resolver,
            );
        }
        Expr::AnonRoutine(_) => {}
        Expr::InlineCall { span, .. } => {
            let call_span = *span;
            let taken = std::mem::replace(expr, Expr::Nop { span: call_span });
            let Expr::InlineCall {
                mut routine,
                mut args,
                span,
            } = taken
            else {
                return;
            };
            for arg in &mut args {
                lower_routine_body_expr(
                    arg,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
            let id = *anon_counter;
            *anon_counter = anon_counter.saturating_add(1);
            let anon_name = format!("$anon_{id}");
            lower_routine_body_expr(
                &mut routine.body,
                Some(&anon_name),
                anon_counter,
                resolver,
            );
            let decl = RoutineDecl {
                is_pub: false,
                pub_dc_syntax: false,
                dc_id: None,
                return_type: routine.return_type,
                name: anon_name.clone(),
                params: routine.params,
                tests: Vec::new(),
                body: routine.body,
                inline_body: true,
                comments_before: Vec::new(),
                span: routine.span,
            };
            *expr = Expr::Block {
                stmts: vec![
                    BlockItem::Expr(Expr::RoutineDecl(Box::new(decl))),
                    BlockItem::Expr(Expr::Call {
                        target: SymbolTarget::Ident(anon_name),
                        args,
                        span,
                    }),
                ],
                span,
            };
        }
        Expr::Call { target, args, .. } => {
            if let Some(nested_name) = enclosing_nested_name
                && let SymbolTarget::Dc(dc) = target
                && is_enclosing_routine_token_dc(resolver, *dc)
            {
                *target = SymbolTarget::Ident(nested_name.to_owned());
            }
            for arg in args {
                lower_routine_body_expr(
                    arg,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(sub) = item {
                    lower_routine_body_expr(
                        sub,
                        enclosing_nested_name,
                        anon_counter,
                        resolver,
                    );
                }
            }
        }
        Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
            for el in elements {
                lower_routine_body_expr(
                    el,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::StructNew { fields, .. } => {
            for f in fields {
                lower_routine_body_expr(
                    f,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::NamedStructNew { fields, .. } => {
            for (_, v, _) in fields {
                lower_routine_body_expr(
                    v,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::Field { base, .. }
        | Expr::NamedField { base, .. }
        | Expr::BufNew { capacity: base, .. }
        | Expr::Conv { expr: base, .. }
        | Expr::Bitcast { expr: base, .. }
        | Expr::WrapNominal { expr: base, .. }
        | Expr::UnwrapNominal { expr: base, .. }
        | Expr::Propagate { expr: base, .. }
        | Expr::Const { value: base, .. }
        | Expr::Set { value: base, .. }
        | Expr::Unary { operand: base, .. }
        | Expr::OptionSome { value: base, .. }
        | Expr::ResultOk { value: base, .. }
        | Expr::ResultErr { error: base, .. } => {
            lower_routine_body_expr(
                base,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
        }
        Expr::Let { init, .. } => {
            if let Some(e) = init {
                lower_routine_body_expr(
                    e,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::Return { value, .. } => {
            if let Some(e) = value {
                lower_routine_body_expr(
                    e,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::EnumVariant { payload, .. } => {
            if let Some(e) = payload {
                lower_routine_body_expr(
                    e,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::With { base, value, .. }
        | Expr::NamedWith { base, value, .. }
        | Expr::While {
            cond: base,
            body: value,
            ..
        }
        | Expr::ForEach {
            list: base,
            body: value,
            ..
        }
        | Expr::Binary {
            left: base,
            right: value,
            ..
        } => {
            lower_routine_body_expr(
                base,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
            lower_routine_body_expr(
                value,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        }
        | Expr::Ternary {
            first: cond,
            second: then_branch,
            third: else_branch,
            ..
        } => {
            lower_routine_body_expr(
                cond,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
            lower_routine_body_expr(
                then_branch,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
            lower_routine_body_expr(
                else_branch,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            lower_routine_body_expr(
                scrutinee,
                enclosing_nested_name,
                anon_counter,
                resolver,
            );
            for arm in arms {
                lower_routine_body_expr(
                    &mut arm.body,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
            if let Some(fb) = fallback {
                lower_routine_body_expr(
                    fb,
                    enclosing_nested_name,
                    anon_counter,
                    resolver,
                );
            }
        }
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::ValueRef { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. }
        | Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::OptionNone { .. }
        | Expr::ErrorTag { .. } => {}
    }
}
