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

//! Static validator for StageL 3 (`f656` `.sl3`) modules and multi-module
//! namespaces.
//!
//! Enforces PSR-4 namespace path matching and acyclic `use` DAG resolution,
//! strict non-coercive static typing with `never` bottom unification, local-only
//! `buf/T`, non-shadowing & no-redeclaration rules for `let` / `set` / `const`,
//! control-flow-sensitive definite assignment across branches and nested verb
//! captures, and tail-position verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr, ImportTarget,
    Module, RoutineDecl, Span, StageLType, SymbolTarget, TestExpected, TopDecl,
    TopValueDecl, format_dc_shorthand,
};
use super::parser::{parse_module_with_imports, scan_module_header_paths};
use ctb_formats_dcdata::{DcChar, DcShorthand};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Source module paired with an optional repository-relative PSR-4 file path.
#[derive(Debug, Clone, Copy)]
pub struct ModuleInput<'a> {
    /// Optional path (e.g. `"std/dctf/codec.sl3"`) checked against `ns`.
    pub source_path: Option<&'a str>,
    /// Raw `.sl3` source text.
    pub source: &'a str,
}

/// Signature of a validated routine (`v/RET/name PARAM...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineSig {
    /// Return type (`void` allowed; `never` allowed).
    pub return_type: StageLType,
    /// Ordered parameter types.
    pub params: Vec<StageLType>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional Dc anchor.
    pub dc_id: Option<DcChar>,
}

/// Exported items from one compiled namespace module.
#[derive(Debug, Clone, Default)]
pub struct NamespaceExports {
    /// Exported `pub` immutable values by name.
    pub values: HashMap<String, (StageLType, Option<DcChar>)>,
    /// Exported `pub` routines by name.
    pub routines: HashMap<String, RoutineSig>,
    /// Assigned Dc shorthand mappings (`dc -> SymbolKind`).
    pub by_dc: HashMap<DcChar, ImportedSymbol>,
}

/// Resolved imported symbol entry inside a dependent module.
#[derive(Debug, Clone)]
pub enum ImportedSymbol {
    /// Imported `pub` constant value `(namespace_key, original_name, ty, dc)`.
    Value {
        /// Defining namespace path key (`"std/bytes/slice"`).
        namespace: String,
        /// Original exported constant name.
        original_name: String,
        /// Static value type.
        ty: StageLType,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
    /// Imported `pub` routine `(namespace_key, original_name, sig)`.
    Routine {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported routine name.
        original_name: String,
        /// Routine signature.
        sig: RoutineSig,
    },
}

/// Metadata collected for a validated single module.
#[derive(Debug, Clone)]
pub struct ValidatedModule {
    /// Parsed AST.
    pub module: Module,
    /// Namespace key if `ns` was declared.
    pub namespace_key: Option<String>,
    /// Resolved `use` bindings visible inside the module.
    pub imports: HashMap<String, ImportedSymbol>,
    /// Direct Dc call/value targets resolved from `use dc/<sh>`.
    pub dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Tail-position annotations indexed by `Span` (`line`, `col`) of `Call`.
    pub tail_calls: HashSet<(usize, usize)>,
    /// Module-level exported items.
    pub exports: NamespaceExports,
}

/// Validated multi-module workspace ordered topologically (`leaves` first).
#[derive(Debug, Clone)]
pub struct ValidatedWorkspace {
    /// Modules in dependency-safe topological order.
    pub modules: Vec<ValidatedModule>,
}

/// Validates a single standalone StageL 3 module (no external `use` imports).
pub fn validate_module(module: &Module) -> Result<ValidatedModule> {
    validate_module_with_registry(module, &HashMap::new())
}

/// Validates a collection of `.sl3` source files (resolving PSR-4 `ns` paths
/// and acyclic `use` DAG imports).
pub fn parse_and_validate_workspace(
    inputs: &[ModuleInput<'_>],
) -> Result<ValidatedWorkspace> {
    let mut headers = Vec::with_capacity(inputs.len());
    let mut ns_to_idx: HashMap<String, usize> = HashMap::new();

    for (idx, inp) in inputs.iter().enumerate() {
        let (ns_opt, uses) = scan_module_header_paths(inp.source)?;
        let ns_key = ns_opt.as_ref().map(|segs| segs.join("/"));
        if let (Some(src_path), Some(key)) = (inp.source_path, &ns_key) {
            verify_psr4_path(src_path, key)?;
        }
        if let Some(ref key) = ns_key {
            ensure!(
                !ns_to_idx.contains_key(key),
                "Duplicate namespace declaration '{key}'"
            );
            ns_to_idx.insert(key.clone(), idx);
        }
        headers.push((ns_key, uses));
    }

    // Build dependency edges (module -> prerequisite module indices) and
    // topologically sort with cycle reporting.
    let mut deps: Vec<Vec<usize>> = Vec::with_capacity(inputs.len());
    for (idx, (ns_key, uses)) in headers.iter().enumerate() {
        let mut d = Vec::new();
        for u in uses {
            let target_key = u.join("/");
            let Some(&dep_idx) = ns_to_idx.get(&target_key) else {
                // Reason for fallback: Anonymous modules format as '<anonymous>' in diagnostics.
                let caller_ns = ns_key.as_deref().unwrap_or("<anonymous>");
                bail!(
                    "Unresolved 'use {target_key}:' import in module {caller_ns}"
                );
            };
            ensure!(
                dep_idx != idx,
                "Module '{target_key}' cannot import itself"
            );
            if !d.contains(&dep_idx) {
                d.push(dep_idx);
            }
        }
        deps.push(d);
    }

    let order = topo_sort_modules(&headers, &deps)?;
    let mut ns_exports: HashMap<String, NamespaceExports> = HashMap::new();
    let mut validated_modules = Vec::with_capacity(inputs.len());

    for idx in order {
        let inp = inputs.get(idx).context("Invalid topological index")?;
        let (_, uses) = headers.get(idx).context("Invalid header index")?;
        let _ = uses;
        // First collect imported verb arities directly from the source `use`
        // items (resolving against already-compiled dependency namespaces).
        let (parsed, _) = parse_with_resolved_use_arities(
            inp.source,
            &ns_exports,
        )?;
        let vm = validate_module_with_registry(&parsed, &ns_exports)?;
        if let Some(ref key) = vm.namespace_key {
            ns_exports.insert(key.clone(), vm.exports.clone());
        }
        validated_modules.push(vm);
    }

    Ok(ValidatedWorkspace {
        modules: validated_modules,
    })
}

fn verify_psr4_path(source_path: &str, ns_key: &str) -> Result<()> {
    let norm = source_path.replace('\\', "/");
    let expected_suffix = format!("{ns_key}.sl3");
    ensure!(
        norm == expected_suffix
            || norm.ends_with(&format!("/{expected_suffix}")),
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
                bail!("Cyclic 'use' dependency detected: {}", cycle_names.join(" -> "));
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

fn parse_with_resolved_use_arities(
    source: &str,
    ns_exports: &HashMap<String, NamespaceExports>,
) -> Result<(Module, HashMap<String, usize>)> {
    // Parse with empty arities if no `use` statements exist; otherwise extract
    // exact arities from dependency `use` headers before full parse.
    let imported_arities =
        collect_use_verb_arities_from_source(source, ns_exports)?;
    let module = parse_module_with_imports(source, &imported_arities)?;
    Ok((module, imported_arities))
}

fn collect_use_verb_arities_from_source(
    source: &str,
    ns_exports: &HashMap<String, NamespaceExports>,
) -> Result<HashMap<String, usize>> {
    // Minimal header token scan for `use path: item [as alias] ... ::.`
    // leveraging `ns_exports` so calling imported verbs parses cleanly.
    let mut arities = HashMap::new();
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
                let sym = if let Some(sh) = item_w.strip_prefix("dc/") {
                    let dc = super::ast::parse_dc_shorthand(sh)?;
                    exports.by_dc.get(&dc).with_context(|| {
                        format!("Namespace '{path}' does not export 'dc/{sh}'")
                    })?
                } else {
                    if let Some(sig) = exports.routines.get(item_w) {
                        let local_name = if words.get(wi) == Some(&"as") {
                            wi = wi.saturating_add(1);
                            let a = *words.get(wi).context("Expected alias after 'as'")?;
                            wi = wi.saturating_add(1);
                            a
                        } else {
                            item_w
                        };
                        ensure!(
                            !arities.contains_key(local_name),
                            "Conflicting imported routine name '{local_name}'"
                        );
                        arities.insert(local_name.to_owned(), sig.params.len());
                        continue;
                    }
                    exports.values.get(item_w).with_context(|| {
                        format!(
                            "Namespace '{path}' has no exported item '{item_w}'"
                        )
                    })?;
                    if words.get(wi) == Some(&"as") {
                        wi = wi.saturating_add(2);
                    }
                    continue;
                };
                let local_name = if words.get(wi) == Some(&"as") {
                    wi = wi.saturating_add(1);
                    let a = *words.get(wi).context("Expected alias after 'as'")?;
                    wi = wi.saturating_add(1);
                    a
                } else {
                    item_w
                };
                if let ImportedSymbol::Routine { sig, .. } = sym {
                    arities.insert(local_name.to_owned(), sig.params.len());
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
    Ok(arities)
}

#[derive(Debug, Clone)]
enum BindingKind {
    MutableLet,
    Const,
    Param,
    ForEachElem,
    ImportedValue { namespace: String, original_name: String },
}

#[derive(Debug, Clone)]
struct BindingEntry {
    id: usize,
    ty: StageLType,
    kind: BindingKind,
    span: Span,
}

#[derive(Debug, Clone)]
struct NestedVerbSummary {
    sig: RoutineSig,
    /// Outer variable IDs read inside this nested verb without being
    /// locally reassigned inside the verb before the read.
    captured_reads: HashSet<usize>,
    /// Outer variable IDs guaranteed to be written on every normal return path.
    always_written: HashSet<usize>,
    captures_resolved: bool,
    span: Span,
}

#[derive(Debug, Clone)]
enum VerbTarget {
    TopLevel(RoutineSig),
    Nested(usize),
    Imported(RoutineSig),
}

struct Validator<'a> {
    namespace_key: Option<String>,
    /// Top-level immutable values (`name -> (ty, span)`).
    top_values: HashMap<String, (StageLType, Span)>,
    /// Top-level routines (`name -> sig`).
    top_routines: HashMap<String, RoutineSig>,
    /// Top-level routine parameter declarations (`name -> Vec<(ty, param_name)>`).
    top_routine_decls: HashMap<String, &'a RoutineDecl>,
    /// Imported values (`local_name -> (ty, ns, orig)`).
    imported_values: HashMap<String, (StageLType, String, String)>,
    /// Imported routines (`local_name -> sig`).
    imported_routines: HashMap<String, RoutineSig>,
    /// Imported items keyed directly by `DcChar` (`dc/<shorthand>`).
    dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Assigned Dcs seen in this module (for duplicate Dc detection).
    used_dcs: HashMap<DcChar, String>,
    /// Next unique lexical variable ID.
    next_binding_id: usize,
    /// Active lexical variable scopes (outermost local to innermost).
    local_scopes: Vec<HashMap<String, BindingEntry>>,
    /// Active routine scopes (`name -> VerbTarget`).
    routine_scopes: Vec<HashMap<String, VerbTarget>>,
    /// Summaries of validated nested verbs keyed by numeric ID.
    nested_summaries: HashMap<usize, NestedVerbSummary>,
    /// Recorded calls to nested verbs in the current block `(nid, span, state_snapshot)`.
    pending_nested_calls: Vec<(usize, Span, AssignState)>,
    /// Recorded tail call positions `(line, col)`.
    tail_calls: HashSet<(usize, usize)>,
}

/// Evaluates whether a type is valid everywhere in a StageL 3 routine or
/// expression, rejecting invalid element types like `l/void` or `l/buf/u8`.
fn check_type_well_formed(
    ty: &StageLType,
    is_pub_api: bool,
    span: Span,
) -> Result<()> {
    match ty {
        StageLType::B
        | StageLType::U8
        | StageLType::I32
        | StageLType::U32
        | StageLType::I64
        | StageLType::U64
        | StageLType::Void
        | StageLType::Never => Ok(()),
        StageLType::List(inner) => {
            ensure!(
                inner.can_be_element(),
                "Invalid list element type '{}' in 'l/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(inner, is_pub_api, span)
        }
        StageLType::Buf(inner) => {
            ensure!(
                !is_pub_api,
                "Local buffer type 'buf/{}' is disallowed in public constants/signatures at {span}",
                inner.display_name()
            );
            ensure!(
                inner.can_be_element(),
                "Invalid buffer element type '{}' in 'buf/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(inner, is_pub_api, span)
        }
    }
}

fn check_pub_dc_shorthand(dc: DcChar, span: Span) -> Result<()> {
    let sh = DcShorthand::from_global_id(dc.to_u128());
    match sh {
        DcShorthand::Short(_) | DcShorthand::Format(_) => Ok(()),
        other => bail!(
            "'pub dc/{other}' at {span} must reference a Core Short Dc or Format Dc (use `: @dc {other}` comment for System-block IDs)"
        ),
    }
}

fn types_compatible(actual: &StageLType, expected: &StageLType) -> bool {
    actual == expected || *actual == StageLType::Never
}

/// Validates a parsed `Module` against already-validated dependency exports.
pub fn validate_module_with_registry(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
) -> Result<ValidatedModule> {
    let namespace_key = module.namespace.as_ref().map(|segs| segs.join("/"));
    let mut exports = NamespaceExports::default();
    let mut imports = HashMap::new();
    let mut dc_imports = HashMap::new();
    let mut imported_values = HashMap::new();
    let mut imported_routines = HashMap::new();
    let mut used_top_names: HashSet<String> = HashSet::new();

    // 1. Resolve `use` imports.
    for u in &module.uses {
        let target_ns = u.path.join("/");
        let dep_exp = ns_exports.get(&target_ns).with_context(|| {
            format!("Unresolved namespace '{target_ns}' in 'use' at {}", u.span)
        })?;
        for item in &u.items {
            let resolved = match &item.target {
                ImportTarget::Name(name) => {
                    if let Some((ty, dc_id)) = dep_exp.values.get(name) {
                        ImportedSymbol::Value {
                            namespace: target_ns.clone(),
                            original_name: name.clone(),
                            ty: ty.clone(),
                            dc_id: *dc_id,
                        }
                    } else if let Some(sig) = dep_exp.routines.get(name) {
                        ImportedSymbol::Routine {
                            namespace: target_ns.clone(),
                            original_name: name.clone(),
                            sig: sig.clone(),
                        }
                    } else {
                        bail!(
                            "Namespace '{target_ns}' has no exported 'pub' symbol named '{name}' (at {})",
                            item.span
                        );
                    }
                }
                ImportTarget::Dc(dc) => dep_exp
                    .by_dc
                    .get(dc)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "Namespace '{target_ns}' has no exported 'pub dc/{}' (at {})",
                            format_dc_shorthand(*dc),
                            item.span
                        )
                    })?,
            };
            let local_name = match (&item.alias, &item.target) {
                (Some(a), _) => a.clone(),
                (None, ImportTarget::Name(n)) => n.clone(),
                (None, ImportTarget::Dc(dc)) => {
                    dc_imports.insert(*dc, resolved.clone());
                    match &resolved {
                        ImportedSymbol::Value { original_name, .. }
                        | ImportedSymbol::Routine { original_name, .. } => {
                            original_name.clone()
                        }
                    }
                }
            };
            ensure!(
                !used_top_names.contains(&local_name),
                "Duplicate or conflicting imported name '{local_name}' at {}",
                item.span
            );
            used_top_names.insert(local_name.clone());
            match &resolved {
                ImportedSymbol::Value {
                    namespace,
                    original_name,
                    ty,
                    ..
                } => {
                    imported_values.insert(
                        local_name.clone(),
                        (ty.clone(), namespace.clone(), original_name.clone()),
                    );
                }
                ImportedSymbol::Routine { sig, .. } => {
                    imported_routines.insert(local_name.clone(), sig.clone());
                }
            }
            imports.insert(local_name, resolved);
        }
    }

    // 2. Pass-1 top-level signature and constant header collection.
    let mut top_values = HashMap::new();
    let mut top_routines = HashMap::new();
    let mut top_routine_decls = HashMap::new();
    let mut used_dcs = HashMap::new();
    let mut top_value_order = Vec::new();
    let mut top_routine_order = Vec::new();

    for decl in &module.items {
        match decl {
            TopDecl::Comment(_) => {}
            TopDecl::Value(v) => {
                ensure!(
                    !used_top_names.contains(&v.name),
                    "Top-level constant '{}' at {} duplicates/shadows existing binding or import",
                    v.name,
                    v.span
                );
                used_top_names.insert(v.name.clone());
                ensure!(
                    v.ty != StageLType::Void && v.ty != StageLType::Never,
                    "Constant '{}' at {} cannot have type '{}'",
                    v.name,
                    v.span,
                    v.ty
                );
                check_type_well_formed(&v.ty, true, v.span)?;
                ensure!(
                    !v.ty.is_or_contains_buf(),
                    "Module-level constant '{}' at {} cannot hold local buffer 'buf/T'",
                    v.name,
                    v.span
                );
                if v.pub_dc_syntax {
                    if let Some(dc) = v.dc_id {
                        check_pub_dc_shorthand(dc, v.span)?;
                    }
                }
                if let Some(dc) = v.dc_id {
                    ensure!(
                        !used_dcs.contains_key(&dc),
                        "Duplicate Dc binding '{}' on '{}' at {}",
                        format_dc_shorthand(dc),
                        v.name,
                        v.span
                    );
                    used_dcs.insert(dc, v.name.clone());
                }
                top_values.insert(v.name.clone(), (v.ty.clone(), v.span));
                if v.is_pub {
                    exports
                        .values
                        .insert(v.name.clone(), (v.ty.clone(), v.dc_id));
                    if let Some(dc) = v.dc_id {
                        exports.by_dc.insert(
                            dc,
                            ImportedSymbol::Value {
                                // Reason for fallback: Anonymous modules without an 'ns' header use an empty namespace string.
                                namespace: namespace_key
                                    .clone()
                                    .unwrap_or_default(),
                                original_name: v.name.clone(),
                                ty: v.ty.clone(),
                                dc_id: Some(dc),
                            },
                        );
                    }
                }
                top_value_order.push(v);
            }
            TopDecl::Routine(r) => {
                ensure!(
                    !used_top_names.contains(&r.name),
                    "Routine '{}' at {} duplicates/shadows an existing routine, constant, or import (overloading is disallowed)",
                    r.name,
                    r.span
                );
                used_top_names.insert(r.name.clone());
                check_type_well_formed(&r.return_type, r.is_pub, r.span)?;
                let mut seen_params = HashSet::new();
                let mut param_tys = Vec::with_capacity(r.params.len());
                for p in &r.params {
                    ensure!(
                        p.ty != StageLType::Void && p.ty != StageLType::Never,
                        "Parameter '{}' of routine '{}' at {} cannot have type '{}'",
                        p.name,
                        r.name,
                        p.span,
                        p.ty
                    );
                    check_type_well_formed(&p.ty, r.is_pub, p.span)?;
                    ensure!(
                        seen_params.insert(p.name.clone()),
                        "Duplicate parameter '{}' in routine '{}' at {}",
                        p.name,
                        r.name,
                        p.span
                    );
                    param_tys.push(p.ty.clone());
                }
                if r.pub_dc_syntax {
                    if let Some(dc) = r.dc_id {
                        check_pub_dc_shorthand(dc, r.span)?;
                    }
                }
                if let Some(dc) = r.dc_id {
                    ensure!(
                        !used_dcs.contains_key(&dc),
                        "Duplicate Dc binding '{}' on '{}' at {}",
                        format_dc_shorthand(dc),
                        r.name,
                        r.span
                    );
                    used_dcs.insert(dc, r.name.clone());
                }
                let sig = RoutineSig {
                    return_type: r.return_type.clone(),
                    params: param_tys,
                    is_pub: r.is_pub,
                    dc_id: r.dc_id,
                };
                top_routines.insert(r.name.clone(), sig.clone());
                top_routine_decls.insert(r.name.clone(), r);
                if r.is_pub {
                    exports.routines.insert(r.name.clone(), sig.clone());
                    if let Some(dc) = r.dc_id {
                        exports.by_dc.insert(
                            dc,
                            ImportedSymbol::Routine {
                                // Reason for fallback: Anonymous modules without an 'ns' header use an empty namespace string.
                                namespace: namespace_key
                                    .clone()
                                    .unwrap_or_default(),
                                original_name: r.name.clone(),
                                sig,
                            },
                        );
                    }
                }
                top_routine_order.push(r);
            }
        }
    }

    let mut validator = Validator {
        namespace_key: namespace_key.clone(),
        top_values,
        top_routines,
        top_routine_decls,
        imported_values,
        imported_routines,
        dc_imports: dc_imports.clone(),
        used_dcs,
        next_binding_id: 1,
        local_scopes: Vec::new(),
        routine_scopes: Vec::new(),
        nested_summaries: HashMap::new(),
        pending_nested_calls: Vec::new(),
        tail_calls: HashSet::new(),
    };

    // 3. Validate top-level constant initializers (acyclic pure dependencies).
    validator.validate_top_values_acyclic(&top_value_order)?;

    // 4. Validate all top-level routine tests and bodies.
    for r in top_routine_order {
        validator.validate_top_routine(r)?;
    }

    Ok(ValidatedModule {
        module: module.clone(),
        namespace_key,
        imports,
        dc_imports,
        tail_calls: validator.tail_calls,
        exports,
    })
}

#[derive(Debug, Clone)]
struct AssignState {
    assigned: BTreeMap<usize, bool>,
}

impl AssignState {
    fn new() -> Self {
        Self {
            assigned: BTreeMap::new(),
        }
    }

    fn set_state(&mut self, id: usize, is_init: bool) {
        self.assigned.insert(id, is_init);
    }

    fn is_assigned(&self, id: usize) -> bool {
        // Reason for fallback: Variables not yet inserted into the assignment map are unassigned.
        self.assigned.get(&id).copied().unwrap_or(false)
    }

    fn intersect_with(&mut self, other: &Self) {
        for (id, flag) in &mut self.assigned {
            if !other.is_assigned(*id) {
                *flag = false;
            }
        }
    }
}

struct CheckCtx {
    return_type: StageLType,
    loop_depth: usize,
    in_tail: bool,
    allow_buf: bool,
    /// Outer binding IDs visible at entry to the current nested routine.
    outer_ids: HashSet<usize>,
    /// Outer variable reads observed on a path where that variable was not yet
    /// assigned locally.
    captured_reads: HashSet<usize>,
    /// Binding IDs written unconditionally along normal completion paths.
    always_written: HashSet<usize>,
}

impl<'a> Validator<'a> {
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

    fn name_exists_in_local_or_top(&self, name: &str) -> bool {
        if self.lookup_local(name).is_some() {
            return true;
        }
        if self.top_values.contains_key(name)
            || self.top_routines.contains_key(name)
            || self.imported_values.contains_key(name)
            || self.imported_routines.contains_key(name)
        {
            return true;
        }
        for rs in &self.routine_scopes {
            if rs.contains_key(name) {
                return true;
            }
        }
        false
    }

    fn lookup_routine(&self, name: &str) -> Option<VerbTarget> {
        for rs in self.routine_scopes.iter().rev() {
            if let Some(vt) = rs.get(name) {
                return Some(vt.clone());
            }
        }
        if let Some(sig) = self.top_routines.get(name) {
            return Some(VerbTarget::TopLevel(sig.clone()));
        }
        if let Some(sig) = self.imported_routines.get(name) {
            return Some(VerbTarget::Imported(sig.clone()));
        }
        None
    }

    fn validate_top_values_acyclic(
        &mut self,
        values: &[&'a TopValueDecl],
    ) -> Result<()> {
        let mut by_name: HashMap<&str, &'a TopValueDecl> = HashMap::new();
        for &v in values {
            by_name.insert(v.name.as_str(), v);
        }

        // Check initialization order / acyclicity via DFS over value references.
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Visiting,
            Done,
        }
        let mut visit: HashMap<&str, State> = HashMap::new();
        let mut stack: Vec<String> = Vec::new();

        fn visit_const<'a>(
            v: &'a TopValueDecl,
            by_name: &HashMap<&'a str, &'a TopValueDecl>,
            validator: &mut Validator<'a>,
            visit: &mut HashMap<&'a str, State>,
            stack: &mut Vec<String>,
        ) -> Result<()> {
            if let Some(st) = visit.get(v.name.as_str()) {
                match st {
                    State::Done => return Ok(()),
                    State::Visiting => {
                        stack.push(v.name.clone());
                        bail!(
                            "Cyclic constant initializer dependency: {}",
                            stack.join(" -> ")
                        );
                    }
                }
            }
            visit.insert(v.name.as_str(), State::Visiting);
            stack.push(v.name.clone());

            let mut deps = Vec::new();
            collect_value_deps(&v.value, &mut deps);
            for dep in deps {
                if let Some(&dep_decl) = by_name.get(dep.as_str()) {
                    visit_const(dep_decl, by_name, validator, visit, stack)?;
                }
            }

            let _ = stack.pop();
            let mut state = AssignState::new();
            let mut ctx = CheckCtx {
                return_type: v.ty.clone(),
                loop_depth: 0,
                in_tail: false,
                allow_buf: false,
                outer_ids: HashSet::new(),
                captured_reads: HashSet::new(),
                always_written: HashSet::new(),
            };
            validator.check_const_purity(&v.value)?;
            let actual =
                validator.check_expr(&v.value, &mut state, &mut ctx)?;
            ensure!(
                types_compatible(&actual, &v.ty),
                "Constant '{}' declared type '{}' does not match initializer type '{}' at {}",
                v.name,
                v.ty,
                actual,
                v.span
            );
            visit.insert(v.name.as_str(), State::Done);
            Ok(())
        }

        for &v in values {
            visit_const(v, &by_name, self, &mut visit, &mut stack)?;
        }
        Ok(())
    }

    fn check_const_purity(&self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Bool { .. }
            | Expr::Int { .. }
            | Expr::Bytes { .. }
            | Expr::ValueRef { .. }
            | Expr::Fault { .. } => Ok(()),
            Expr::List { elements, .. } => {
                for e in elements {
                    self.check_const_purity(e)?;
                }
                Ok(())
            }
            Expr::Unary { op, operand, span } => {
                ensure!(
                    *op != CoreUnaryOp::BufFreeze,
                    "'bufFreeze' is disallowed in module-level constant initializer at {span}"
                );
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
            Expr::Conv { expr, .. } | Expr::Bitcast { expr, .. } => {
                self.check_const_purity(expr)
            }
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
            other => bail!(
                "Module-level constant initializer must be a pure constant expression (disallowed construct at {})",
                other.span()
            ),
        }
    }

    fn validate_top_routine(&mut self, r: &'a RoutineDecl) -> Result<()> {
        // Validate inline tests in an isolated top-level value scope (no locals).
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
                for (inp_expr, param) in tc.inputs.iter().zip(&r.params) {
                    let mut st = AssignState::new();
                    let mut ctx = CheckCtx {
                        return_type: param.ty.clone(),
                        loop_depth: 0,
                        in_tail: false,
                        allow_buf: true,
                        outer_ids: HashSet::new(),
                        captured_reads: HashSet::new(),
                        always_written: HashSet::new(),
                    };
                    let ty = self.check_expr(inp_expr, &mut st, &mut ctx)?;
                    ensure!(
                        types_compatible(&ty, &param.ty),
                        "Test input type mismatch at {} for parameter '{}' (expected '{}', got '{}')",
                        inp_expr.span(),
                        param.name,
                        param.ty,
                        ty
                    );
                }
                match &tc.expected {
                    TestExpected::Expr(exp_expr) => {
                        let mut st = AssignState::new();
                        let mut ctx = CheckCtx {
                            return_type: r.return_type.clone(),
                            loop_depth: 0,
                            in_tail: false,
                            allow_buf: true,
                            outer_ids: HashSet::new(),
                            captured_reads: HashSet::new(),
                            always_written: HashSet::new(),
                        };
                        let ty =
                            self.check_expr(exp_expr, &mut st, &mut ctx)?;
                        ensure!(
                            types_compatible(&ty, &r.return_type),
                            "Test expected output type mismatch for '{}' at {} (expected '{}', got '{}')",
                            r.name,
                            exp_expr.span(),
                            r.return_type,
                            ty
                        );
                    }
                    TestExpected::Fault(_) => {}
                    TestExpected::Void(sp) => {
                        ensure!(
                            r.return_type == StageLType::Void
                                || r.return_type == StageLType::Never,
                            "Omitted/void test output at {sp} is only valid for 'void' routines ('{}' returns '{}')",
                            r.name,
                            r.return_type
                        );
                    }
                }
            }
        }

        // Validate routine body.
        self.validate_routine_body(r, false)
    }

    fn validate_routine_body(
        &mut self,
        r: &RoutineDecl,
        is_nested: bool,
    ) -> Result<()> {
        let mut outer_ids = HashSet::new();
        if is_nested {
            for scope in &self.local_scopes {
                for entry in scope.values() {
                    outer_ids.insert(entry.id);
                }
            }
        }

        let mut param_scope = HashMap::new();
        let mut state = AssignState::new();
        if is_nested {
            // Preserve current assignment knowledge for outer variables at
            // entry? Note: nested verbs may read an outer variable initialized
            // before any call, so track which outer variables remain uninit vs
            // init across nested execution carefully via `captured_reads`.
            for scope in &self.local_scopes {
                for entry in scope.values() {
                    state.set_state(entry.id, true);
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
            let id = self.alloc_binding_id();
            state.set_state(id, true);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id,
                    ty: p.ty.clone(),
                    kind: BindingKind::Param,
                    span: p.span,
                },
            );
        }

        self.local_scopes.push(param_scope);
        let mut ctx = CheckCtx {
            return_type: r.return_type.clone(),
            loop_depth: 0,
            in_tail: true,
            allow_buf: true,
            outer_ids,
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        };
        let actual_ty = self.check_expr(&r.body, &mut state, &mut ctx)?;
        let _ = self.local_scopes.pop();

        ensure!(
            types_compatible(&actual_ty, &r.return_type),
            "Routine '{}' body returns '{}' which is incompatible with declared return type '{}' at {}",
            r.name,
            actual_ty,
            r.return_type,
            r.span
        );
        Ok(())
    }

    fn validate_nested_routine_at_decl(
        &mut self,
        nid: usize,
        r: &RoutineDecl,
        enclosing_state: &AssignState,
    ) -> Result<()> {
        let mut outer_ids = HashSet::new();
        for sc in &self.local_scopes {
            for ent in sc.values() {
                outer_ids.insert(ent.id);
            }
        }
        let mut nested_state = enclosing_state.clone();
        for &id in &outer_ids {
            nested_state.set_state(id, true);
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
            let pid = self.alloc_binding_id();
            nested_state.set_state(pid, true);
            param_scope.insert(
                p.name.clone(),
                BindingEntry {
                    id: pid,
                    ty: p.ty.clone(),
                    kind: BindingKind::Param,
                    span: p.span,
                },
            );
        }
        self.local_scopes.push(param_scope);
        let mut ctx = CheckCtx {
            return_type: r.return_type.clone(),
            loop_depth: 0,
            in_tail: true,
            allow_buf: true,
            outer_ids,
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        };
        let body_ty = self.check_expr(&r.body, &mut nested_state, &mut ctx)?;
        let _ = self.local_scopes.pop();
        ensure!(
            types_compatible(&body_ty, &r.return_type),
            "Nested routine '{}' body returns '{}' (expected '{}') at {}",
            r.name,
            body_ty,
            r.return_type,
            r.span
        );
        if let Some(sum) = self.nested_summaries.get_mut(&nid) {
            sum.captured_reads = ctx.captured_reads;
            if r.return_type != StageLType::Never {
                sum.always_written = ctx.always_written;
            }
            sum.captures_resolved = true;
        }
        Ok(())
    }

    fn check_expr(
        &mut self,
        expr: &Expr,
        state: &mut AssignState,
        ctx: &mut CheckCtx,
    ) -> Result<StageLType> {
        match expr {
            Expr::Bool { .. } => Ok(StageLType::B),
            Expr::Int { value, .. } => Ok(value.stage_l_type()),
            Expr::Bytes { .. } => Ok(StageLType::List(Box::new(StageLType::U8))),
            Expr::List {
                elem_ty,
                elements,
                span,
            } => {
                check_type_well_formed(
                    &StageLType::List(Box::new(elem_ty.clone())),
                    false,
                    *span,
                )?;
                let mut child_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth,
                    in_tail: false,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
                for el in elements {
                    let ety = self.check_expr(el, state, &mut child_ctx)?;
                    ensure!(
                        types_compatible(&ety, elem_ty),
                        "List element at {} has type '{}', expected '{}'",
                        el.span(),
                        ety,
                        elem_ty
                    );
                }
                ctx.captured_reads.extend(child_ctx.captured_reads);
                Ok(StageLType::List(Box::new(elem_ty.clone())))
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
                check_type_well_formed(
                    &StageLType::Buf(Box::new(elem_ty.clone())),
                    false,
                    *span,
                )?;
                let cap_ty = self.check_subexpr(capacity, state, ctx)?;
                ensure!(
                    types_compatible(&cap_ty, &StageLType::U64),
                    "'bufNew' capacity at {} must have type 'u64', found '{}'",
                    capacity.span(),
                    cap_ty
                );
                Ok(StageLType::Buf(Box::new(elem_ty.clone())))
            }
            Expr::ValueRef { ty, target, span } => {
                check_type_well_formed(ty, false, *span)?;
                match target {
                    SymbolTarget::Ident(name) => {
                        if let Some(entry) = self.lookup_local(name).cloned() {
                            ensure!(
                                &entry.ty == ty,
                                "Type annotation mismatch on '{name}' at {span}: declared '{}', referenced as '{}'",
                                entry.ty,
                                ty
                            );
                            if !state.is_assigned(entry.id) {
                                bail!(
                                    "Use of potentially uninitialized variable '{name}' at {span}"
                                );
                            }
                            if ctx.outer_ids.contains(&entry.id)
                                && !ctx.always_written.contains(&entry.id)
                            {
                                ctx.captured_reads.insert(entry.id);
                            }
                            return Ok(entry.ty);
                        }
                        if let Some((top_ty, _)) = self.top_values.get(name) {
                            ensure!(
                                top_ty == ty,
                                "Type annotation mismatch on constant '{name}' at {span}: declared '{top_ty}', referenced as '{ty}'"
                            );
                            return Ok(top_ty.clone());
                        }
                        if let Some((imp_ty, _, _)) =
                            self.imported_values.get(name)
                        {
                            ensure!(
                                imp_ty == ty,
                                "Type annotation mismatch on imported constant '{name}' at {span}: declared '{imp_ty}', referenced as '{ty}'"
                            );
                            return Ok(imp_ty.clone());
                        }
                        bail!("Undeclared identifier '{name}' at {span}");
                    }
                    SymbolTarget::Dc(dc) => {
                        let sh = format_dc_shorthand(*dc);
                        // Allow referencing a top-level value or imported value matching dc.
                        for (name, (v_ty, _)) in &self.top_values {
                            if self.used_dcs.get(dc) == Some(name) {
                                ensure!(
                                    v_ty == ty,
                                    "Type mismatch on Dc reference '{sh}' at {span}"
                                );
                                return Ok(v_ty.clone());
                            }
                        }
                        if let Some(ImportedSymbol::Value { ty: imp_ty, .. }) =
                            self.dc_imports.get(dc)
                        {
                            ensure!(
                                imp_ty == ty,
                                "Type mismatch on imported Dc reference '{sh}' at {span}"
                            );
                            return Ok(imp_ty.clone());
                        }
                        bail!("Unresolved value Dc reference '{sh}' at {span}");
                    }
                }
            }
            Expr::Let {
                ty,
                name,
                init,
                span,
            } => {
                ensure!(
                    *ty != StageLType::Void && *ty != StageLType::Never,
                    "'let' variable '{name}' at {span} cannot have type '{ty}'"
                );
                check_type_well_formed(ty, false, *span)?;
                ensure!(
                    !self.name_exists_in_local_or_top(name),
                    "Cannot re-declare or shadow '{name}' with 'let' at {span} (use 'set' to mutate)"
                );
                let is_init = if let Some(init_expr) = init {
                    let rhs_ty = self.check_subexpr(init_expr, state, ctx)?;
                    ensure!(
                        types_compatible(&rhs_ty, ty),
                        "Initializer type mismatch for 'let {ty}/{name}' at {span}: expected '{ty}', found '{rhs_ty}'"
                    );
                    true
                } else {
                    false
                };
                let id = self.alloc_binding_id();
                state.set_state(id, is_init);
                let scope = self.local_scopes.last_mut().context(
                    "'let' is only allowed inside routine/code blocks",
                )?;
                scope.insert(
                    name.clone(),
                    BindingEntry {
                        id,
                        ty: ty.clone(),
                        kind: BindingKind::MutableLet,
                        span: *span,
                    },
                );
                Ok(StageLType::Void)
            }
            Expr::Const {
                ty,
                name,
                value,
                span,
                ..
            } => {
                ensure!(
                    *ty != StageLType::Void && *ty != StageLType::Never,
                    "'const' binding '{name}' at {span} cannot have type '{ty}'"
                );
                check_type_well_formed(ty, false, *span)?;
                ensure!(
                    !self.name_exists_in_local_or_top(name),
                    "Cannot re-declare or shadow '{name}' with 'const' at {span}"
                );
                let rhs_ty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    types_compatible(&rhs_ty, ty),
                    "Initializer type mismatch for 'const {ty}/{name}' at {span}: expected '{ty}', found '{rhs_ty}'"
                );
                let id = self.alloc_binding_id();
                state.set_state(id, true);
                let scope = self.local_scopes.last_mut().context(
                    "Local 'const' requires an enclosing block",
                )?;
                scope.insert(
                    name.clone(),
                    BindingEntry {
                        id,
                        ty: ty.clone(),
                        kind: BindingKind::Const,
                        span: *span,
                    },
                );
                Ok(StageLType::Void)
            }
            Expr::Set {
                ty,
                name,
                value,
                span,
            } => {
                check_type_well_formed(ty, false, *span)?;
                let entry = self
                    .lookup_local(name)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "Cannot 'set' undeclared variable '{name}' at {span} (must declare with 'let' first)"
                        )
                    })?;
                ensure!(
                    matches!(entry.kind, BindingKind::MutableLet),
                    "Cannot 'set' immutable binding '{name}' ({:?}) at {span}",
                    entry.kind
                );
                ensure!(
                    &entry.ty == ty,
                    "Type mismatch in 'set {ty}/{name}' at {span}: declared type is '{}'",
                    entry.ty
                );
                let rhs_ty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    types_compatible(&rhs_ty, ty),
                    "Assigned value type mismatch in 'set {ty}/{name}' at {span}: expected '{ty}', found '{rhs_ty}'"
                );
                state.set_state(entry.id, true);
                ctx.always_written.insert(entry.id);
                Ok(StageLType::Void)
            }
            Expr::RoutineDecl(r) => {
                ensure!(
                    !r.is_pub && !r.pub_dc_syntax && r.dc_id.is_none(),
                    "Nested routine '{}' at {} cannot carry 'pub' or Dc annotations",
                    r.name,
                    r.span
                );
                let nested_id = match self.lookup_routine(&r.name) {
                    Some(VerbTarget::Nested(nid)) => nid,
                    _ => bail!(
                        "Nested routine '{}' at {} missing from enclosing block signature table",
                        r.name,
                        r.span
                    ),
                };
                self.validate_nested_routine_at_decl(nested_id, r, state)?;
                Ok(StageLType::Void)
            }
            Expr::Call { target, args, span } => {
                let sig = match target {
                    SymbolTarget::Ident(name) => match self.lookup_routine(name)
                    {
                        Some(VerbTarget::TopLevel(s))
                        | Some(VerbTarget::Imported(s)) => s,
                        Some(VerbTarget::Nested(nid)) => {
                            let summary = self
                                .nested_summaries
                                .get(&nid)
                                .cloned()
                                .with_context(|| {
                                    format!("Missing nested verb summary for '{name}' at {span}")
                                })?;
                            self.pending_nested_calls.push((
                                nid,
                                *span,
                                state.clone(),
                            ));
                            if summary.captures_resolved {
                                for cap_id in &summary.captured_reads {
                                    if !state.is_assigned(*cap_id) {
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
                                    state.set_state(*written_id, true);
                                    ctx.always_written.insert(*written_id);
                                }
                            }
                            summary.sig
                        }
                        None => {
                            bail!("Unknown routine '{name}' at {span}");
                        }
                    },
                    SymbolTarget::Dc(dc) => {
                        let sh = format_dc_shorthand(*dc);
                        let mut found = None;
                        if let Some(name) = self.used_dcs.get(dc) {
                            if let Some(s) = self.top_routines.get(name) {
                                found = Some(s.clone());
                            }
                        }
                        if found.is_none() {
                            if let Some(ImportedSymbol::Routine {
                                sig,
                                ..
                            }) = self.dc_imports.get(dc)
                            {
                                found = Some(sig.clone());
                            }
                        }
                        found.with_context(|| {
                            format!(
                                "Unknown routine Dc reference 'dc/{sh}' at {span}"
                            )
                        })?
                    }
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
            Expr::Block { stmts, .. } => {
                // Collect nested routine signatures first inside this block
                // (enabling forward and mutually recursive nested calls!).
                let mut block_routines: HashMap<String, VerbTarget> =
                    HashMap::new();
                let mut nested_decls: Vec<(usize, &RoutineDecl)> =
                    Vec::new();

                for item in stmts {
                    if let BlockItem::Expr(Expr::RoutineDecl(r)) = item {
                        ensure!(
                            !self.name_exists_in_local_or_top(&r.name)
                                && !block_routines.contains_key(&r.name),
                            "Nested routine '{}' at {} duplicates or shadows an existing identifier",
                            r.name,
                            r.span
                        );
                        check_type_well_formed(&r.return_type, false, r.span)?;
                        let mut seen_p = HashSet::new();
                        let mut ptys = Vec::with_capacity(r.params.len());
                        for p in &r.params {
                            ensure!(
                                p.ty != StageLType::Void
                                    && p.ty != StageLType::Never,
                                "Parameter '{}' of '{}' at {} cannot have type '{}'",
                                p.name,
                                r.name,
                                p.span,
                                p.ty
                            );
                            check_type_well_formed(&p.ty, false, p.span)?;
                            ensure!(
                                seen_p.insert(p.name.clone()),
                                "Duplicate parameter '{}' in nested routine '{}' at {}",
                                p.name,
                                r.name,
                                p.span
                            );
                            ptys.push(p.ty.clone());
                        }
                        let nid = self.alloc_binding_id();
                        let sig = RoutineSig {
                            return_type: r.return_type.clone(),
                            params: ptys,
                            is_pub: false,
                            dc_id: None,
                        };
                        self.nested_summaries.insert(
                            nid,
                            NestedVerbSummary {
                                sig: sig.clone(),
                                captured_reads: HashSet::new(),
                                always_written: HashSet::new(),
                                captures_resolved: false,
                                span: r.span,
                            },
                        );
                        block_routines.insert(
                            r.name.clone(),
                            VerbTarget::Nested(nid),
                        );
                        nested_decls.push((nid, r));
                    }
                }

                self.routine_scopes.push(block_routines);
                self.local_scopes.push(HashMap::new());
                let saved_pending =
                    std::mem::take(&mut self.pending_nested_calls);

                // Validate statements sequentially; each nested `RoutineDecl`
                // validates when reached so locals declared before it are in
                // scope, and forward calls are verified against call snapshots
                // once the block's nested routines are resolved.
                let last_expr_idx = stmts
                    .iter()
                    .rposition(|it| matches!(it, BlockItem::Expr(_)));

                let mut result_ty = StageLType::Void;
                let mut terminated = false;
                for (idx, item) in stmts.iter().enumerate() {
                    let BlockItem::Expr(e) = item else {
                        continue;
                    };
                    let is_last = Some(idx) == last_expr_idx;
                    let mut item_ctx = CheckCtx {
                        return_type: ctx.return_type.clone(),
                        loop_depth: ctx.loop_depth,
                        in_tail: ctx.in_tail && is_last,
                        allow_buf: ctx.allow_buf,
                        outer_ids: ctx.outer_ids.clone(),
                        captured_reads: HashSet::new(),
                        always_written: HashSet::new(),
                    };
                    let ety = self.check_expr(e, state, &mut item_ctx)?;
                    ctx.captured_reads.extend(item_ctx.captured_reads);
                    ctx.always_written.extend(item_ctx.always_written);
                    if ety == StageLType::Never {
                        terminated = true;
                    }
                    if is_last {
                        result_ty = if terminated {
                            StageLType::Never
                        } else {
                            ety
                        };
                    }
                }

                let block_calls = std::mem::replace(
                    &mut self.pending_nested_calls,
                    saved_pending,
                );
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
                    if let Some(sum) = self.nested_summaries.get(&nid) {
                        if !called_nids.contains(&nid) {
                            for cap_id in &sum.captured_reads {
                                if self.lookup_is_declared_in_scope(
                                    self.local_scopes.last(),
                                    *cap_id,
                                ) && !state.is_assigned(*cap_id)
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
                }

                let _ = self.local_scopes.pop();
                let _ = self.routine_scopes.pop();
                Ok(result_ty)
            }
            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                let cty = self.check_subexpr(cond, state, ctx)?;
                ensure!(
                    types_compatible(&cty, &StageLType::B),
                    "'if' condition at {} must have type 'b', found '{cty}'",
                    cond.span()
                );

                let mut then_state = state.clone();
                let mut then_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth,
                    in_tail: ctx.in_tail,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
                let tty =
                    self.check_expr(then_branch, &mut then_state, &mut then_ctx)?;

                let mut else_state = state.clone();
                let mut else_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth,
                    in_tail: ctx.in_tail,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
                let ety =
                    self.check_expr(else_branch, &mut else_state, &mut else_ctx)?;

                ctx.captured_reads.extend(then_ctx.captured_reads);
                ctx.captured_reads.extend(else_ctx.captured_reads);

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
            Expr::While { cond, body, span } => {
                let cty = self.check_subexpr(cond, state, ctx)?;
                ensure!(
                    types_compatible(&cty, &StageLType::B),
                    "'while' condition at {} must have type 'b', found '{cty}'",
                    cond.span()
                );
                let mut body_state = state.clone();
                let mut body_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth.saturating_add(1),
                    in_tail: false,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
                let bty =
                    self.check_expr(body, &mut body_state, &mut body_ctx)?;
                ctx.captured_reads.extend(body_ctx.captured_reads);
                ensure!(
                    bty == StageLType::Void || bty == StageLType::Never,
                    "'while' body at {span} must evaluate to 'void' (or 'never'), found '{bty}'"
                );
                Ok(StageLType::Void)
            }
            Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                span,
            } => {
                check_type_well_formed(elem_ty, false, *span)?;
                ensure!(
                    elem_ty.can_be_element(),
                    "'foreach' element type '{}' at {span} is invalid",
                    elem_ty
                );
                ensure!(
                    !self.name_exists_in_local_or_top(elem_name),
                    "'foreach' element binding '{elem_name}' at {span} shadows an existing declaration"
                );
                let lty = self.check_subexpr(list, state, ctx)?;
                let expected_list = StageLType::List(Box::new(elem_ty.clone()));
                ensure!(
                    types_compatible(&lty, &expected_list),
                    "'foreach' iterable at {} must have type '{}', found '{lty}'",
                    list.span(),
                    expected_list
                );
                let elem_id = self.alloc_binding_id();
                let mut body_state = state.clone();
                body_state.set_state(elem_id, true);
                let mut elem_scope = HashMap::new();
                elem_scope.insert(
                    elem_name.clone(),
                    BindingEntry {
                        id: elem_id,
                        ty: elem_ty.clone(),
                        kind: BindingKind::ForEachElem,
                        span: *span,
                    },
                );
                self.local_scopes.push(elem_scope);
                let mut body_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth.saturating_add(1),
                    in_tail: false,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
                let bty =
                    self.check_expr(body, &mut body_state, &mut body_ctx)?;
                let _ = self.local_scopes.pop();
                ctx.captured_reads.extend(body_ctx.captured_reads);
                ensure!(
                    bty == StageLType::Void || bty == StageLType::Never,
                    "'foreach' body at {span} must evaluate to 'void' (or 'never'), found '{bty}'"
                );
                Ok(StageLType::Void)
            }
            Expr::Break { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'break' at {span} is outside any enclosing loop in the current routine"
                );
                Ok(StageLType::Never)
            }
            Expr::Continue { span } => {
                ensure!(
                    ctx.loop_depth > 0,
                    "'continue' at {span} is outside any enclosing loop in the current routine"
                );
                Ok(StageLType::Never)
            }
            Expr::Return { value, span } => {
                match (&ctx.return_type, value) {
                    (StageLType::Void, None) => Ok(StageLType::Never),
                    (StageLType::Void, Some(v)) => bail!(
                        "Cannot return a value from 'void' routine at {}",
                        v.span()
                    ),
                    (expected, None) => bail!(
                        "Missing return value expression at {span} (routine returns '{expected}')"
                    ),
                    (expected, Some(v)) => {
                        let mut ret_ctx = CheckCtx {
                            return_type: ctx.return_type.clone(),
                            loop_depth: ctx.loop_depth,
                            in_tail: true,
                            allow_buf: ctx.allow_buf,
                            outer_ids: ctx.outer_ids.clone(),
                            captured_reads: HashSet::new(),
                            always_written: HashSet::new(),
                        };
                        let vty = self.check_expr(v, state, &mut ret_ctx)?;
                        ctx.captured_reads.extend(ret_ctx.captured_reads);
                        ensure!(
                            types_compatible(&vty, expected),
                            "Return value type mismatch at {span}: expected '{expected}', found '{vty}'"
                        );
                        Ok(StageLType::Never)
                    }
                }
            }
            Expr::Fault { .. } => Ok(StageLType::Never),
            Expr::Unary { op, operand, span } => {
                let ty = self.check_subexpr(operand, state, ctx)?;
                match op {
                    CoreUnaryOp::Neg => {
                        if ty == StageLType::Never {
                            return Ok(StageLType::Never);
                        }
                        ensure!(
                            ty.is_signed_int(),
                            "'neg' requires signed integer operand ('i32' or 'i64'), found '{ty}' at {span}"
                        );
                        Ok(ty)
                    }
                    CoreUnaryOp::BitNot => {
                        if ty == StageLType::Never {
                            return Ok(StageLType::Never);
                        }
                        ensure!(
                            ty.is_integer(),
                            "'bitNot' requires integer operand, found '{ty}' at {span}"
                        );
                        Ok(ty)
                    }
                    CoreUnaryOp::Not => {
                        ensure!(
                            types_compatible(&ty, &StageLType::B),
                            "'not' requires 'b' operand, found '{ty}' at {span}"
                        );
                        Ok(StageLType::B)
                    }
                    CoreUnaryOp::Clz | CoreUnaryOp::Ctz | CoreUnaryOp::Popcnt => {
                        ensure!(
                            ty == StageLType::Never || ty.is_integer(),
                            "'{}' requires integer operand, found '{ty}' at {span}",
                            op.keyword()
                        );
                        Ok(StageLType::U32)
                    }
                    CoreUnaryOp::Len => {
                        ensure!(
                            ty == StageLType::Never
                                || matches!(
                                    ty,
                                    StageLType::List(_) | StageLType::Buf(_)
                                ),
                            "'len' requires 'l/T' or 'buf/T', found '{ty}' at {span}"
                        );
                        Ok(StageLType::U64)
                    }
                    CoreUnaryOp::BufFreeze => {
                        if ty == StageLType::Never {
                            return Ok(StageLType::Never);
                        }
                        match ty {
                            StageLType::Buf(elem) => Ok(StageLType::List(elem)),
                            other => bail!(
                                "'bufFreeze' requires 'buf/T', found '{other}' at {span}"
                            ),
                        }
                    }
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => self.check_binary_op(*op, left, right, *span, state, ctx),
            Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            } => self.check_ternary_op(*op, first, second, third, *span, state, ctx),
            Expr::Conv {
                expr,
                target_ty,
                span,
            } => {
                check_type_well_formed(target_ty, false, *span)?;
                ensure!(
                    target_ty.is_integer(),
                    "'conv' target type must be integer, found '{target_ty}' at {span}"
                );
                let src_ty = self.check_subexpr(expr, state, ctx)?;
                ensure!(
                    src_ty == StageLType::Never || src_ty.is_integer(),
                    "'conv' source operand must be integer, found '{src_ty}' at {span}"
                );
                Ok(target_ty.clone())
            }
            Expr::Bitcast {
                expr,
                target_ty,
                span,
            } => {
                check_type_well_formed(target_ty, false, *span)?;
                let dst_width = target_ty.bit_width().with_context(|| {
                    format!(
                        "'bitcast' target type must be integer, found '{target_ty}' at {span}"
                    )
                })?;
                let src_ty = self.check_subexpr(expr, state, ctx)?;
                if src_ty != StageLType::Never {
                    let src_width = src_ty.bit_width().with_context(|| {
                        format!(
                            "'bitcast' source operand must be integer, found '{src_ty}' at {span}"
                        )
                    })?;
                    ensure!(
                        src_width == dst_width,
                        "'bitcast' requires matching bit widths ('{src_ty}' is {src_width}-bit, '{target_ty}' is {dst_width}-bit) at {span}"
                    );
                }
                Ok(target_ty.clone())
            }
        }
    }

    fn lookup_is_declared_in_scope(
        &self,
        scope: Option<&HashMap<String, BindingEntry>>,
        id: usize,
    ) -> bool {
        let Some(sc) = scope else {
            return false;
        };
        sc.values().any(|e| e.id == id)
    }

    fn check_subexpr(
        &mut self,
        expr: &Expr,
        state: &mut AssignState,
        ctx: &mut CheckCtx,
    ) -> Result<StageLType> {
        let mut sub_ctx = CheckCtx {
            return_type: ctx.return_type.clone(),
            loop_depth: ctx.loop_depth,
            in_tail: false,
            allow_buf: ctx.allow_buf,
            outer_ids: ctx.outer_ids.clone(),
            captured_reads: HashSet::new(),
            always_written: HashSet::new(),
        };
        let ty = self.check_expr(expr, state, &mut sub_ctx)?;
        ctx.captured_reads.extend(sub_ctx.captured_reads);
        ctx.always_written.extend(sub_ctx.always_written);
        Ok(ty)
    }

    fn check_binary_op(
        &mut self,
        op: CoreBinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
        state: &mut AssignState,
        ctx: &mut CheckCtx,
    ) -> Result<StageLType> {
        match op {
            CoreBinaryOp::And | CoreBinaryOp::Or => {
                let lty = self.check_subexpr(left, state, ctx)?;
                ensure!(
                    types_compatible(&lty, &StageLType::B),
                    "'{}' left operand at {} must be 'b', found '{lty}'",
                    op.keyword(),
                    left.span()
                );
                // Right operand only evaluates conditionally (short-circuit),
                // so definite writes in `right` do not apply post-op.
                let mut right_state = state.clone();
                let mut right_ctx = CheckCtx {
                    return_type: ctx.return_type.clone(),
                    loop_depth: ctx.loop_depth,
                    in_tail: false,
                    allow_buf: ctx.allow_buf,
                    outer_ids: ctx.outer_ids.clone(),
                    captured_reads: HashSet::new(),
                    always_written: HashSet::new(),
                };
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
            CoreBinaryOp::Add
            | CoreBinaryOp::Sub
            | CoreBinaryOp::Mul
            | CoreBinaryOp::Div
            | CoreBinaryOp::Rem
            | CoreBinaryOp::WrapAdd
            | CoreBinaryOp::WrapSub
            | CoreBinaryOp::WrapMul
            | CoreBinaryOp::BitAnd
            | CoreBinaryOp::BitOr
            | CoreBinaryOp::BitXor => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                unify_int_pair(op.keyword(), &lty, &rty, span)
            }
            CoreBinaryOp::Lt
            | CoreBinaryOp::Le
            | CoreBinaryOp::Gt
            | CoreBinaryOp::Ge => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                let _ = unify_int_pair(op.keyword(), &lty, &rty, span)?;
                Ok(StageLType::B)
            }
            CoreBinaryOp::Eq | CoreBinaryOp::Ne => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                match (lty == StageLType::Never, rty == StageLType::Never) {
                    (true, true) => {}
                    (true, false) => ensure!(
                        rty.supports_eq(),
                        "'{}' does not support type '{rty}' at {span}",
                        op.keyword()
                    ),
                    (false, true) => ensure!(
                        lty.supports_eq(),
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
                            lty.supports_eq(),
                            "'{}' does not support type '{lty}' at {span}",
                            op.keyword()
                        );
                    }
                }
                Ok(StageLType::B)
            }
            CoreBinaryOp::Shl | CoreBinaryOp::Shr => {
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
                    lty.is_integer(),
                    "'{}' value operand at {span} must be integer, found '{lty}'",
                    op.keyword()
                );
                Ok(lty)
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
                    lty.is_unsigned_int(),
                    "'{}' requires unsigned integer operand ('u8', 'u32', 'u64'), found '{lty}' at {span}",
                    op.keyword()
                );
                Ok(lty)
            }
            CoreBinaryOp::Get => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                ensure!(
                    types_compatible(&rty, &StageLType::U64),
                    "'get' index at {} must be 'u64', found '{rty}'",
                    right.span()
                );
                match lty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::List(el) | StageLType::Buf(el) => Ok(*el),
                    other => bail!(
                        "'get' sequence operand must be 'l/T' or 'buf/T', found '{other}' at {span}"
                    ),
                }
            }
            CoreBinaryOp::Concat => {
                let lty = self.check_subexpr(left, state, ctx)?;
                let rty = self.check_subexpr(right, state, ctx)?;
                match (lty, rty) {
                    (StageLType::Never, StageLType::Never) => {
                        Ok(StageLType::Never)
                    }
                    (StageLType::Never, r @ StageLType::List(_)) => Ok(r),
                    (l @ StageLType::List(_), StageLType::Never) => Ok(l),
                    (
                        l @ StageLType::List(_),
                        r @ StageLType::List(_),
                    ) => {
                        ensure!(
                            l == r,
                            "'concat' list types must match at {span}: '{l}' vs '{r}'"
                        );
                        Ok(l)
                    }
                    (l, r) => bail!(
                        "'concat' requires matching 'l/T' operands at {span}, found '{l}' and '{r}'"
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
                ensure!(
                    vty.can_be_element(),
                    "'bufFill' element type '{vty}' is invalid at {span}"
                );
                Ok(StageLType::Buf(Box::new(vty)))
            }
            CoreBinaryOp::BufPush => {
                let bty = self.check_subexpr(left, state, ctx)?;
                let vty = self.check_subexpr(right, state, ctx)?;
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        ensure!(
                            types_compatible(&vty, &elem),
                            "'bufPush' element type mismatch at {span}: expected '{elem}', found '{vty}'"
                        );
                        Ok(StageLType::Void)
                    }
                    other => bail!(
                        "'bufPush' target must be 'buf/T', found '{other}' at {span}"
                    ),
                }
            }
            CoreBinaryOp::BufExtend => {
                let bty = self.check_subexpr(left, state, ctx)?;
                let lty = self.check_subexpr(right, state, ctx)?;
                match bty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        let expected_list = StageLType::List(elem);
                        ensure!(
                            types_compatible(&lty, &expected_list),
                            "'bufExtend' list type mismatch at {span}: expected '{expected_list}', found '{lty}'"
                        );
                        Ok(StageLType::Void)
                    }
                    other => bail!(
                        "'bufExtend' target must be 'buf/T', found '{other}' at {span}"
                    ),
                }
            }
        }
    }

    fn check_ternary_op(
        &mut self,
        op: CoreTernaryOp,
        first: &Expr,
        second: &Expr,
        third: &Expr,
        span: Span,
        state: &mut AssignState,
        ctx: &mut CheckCtx,
    ) -> Result<StageLType> {
        match op {
            CoreTernaryOp::Slice => {
                let seq_ty = self.check_subexpr(first, state, ctx)?;
                let start_ty = self.check_subexpr(second, state, ctx)?;
                let end_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    types_compatible(&start_ty, &StageLType::U64),
                    "'slice' start index at {} must be 'u64', found '{start_ty}'",
                    second.span()
                );
                ensure!(
                    types_compatible(&end_ty, &StageLType::U64),
                    "'slice' end index at {} must be 'u64', found '{end_ty}'",
                    third.span()
                );
                match seq_ty {
                    StageLType::Never => Ok(StageLType::Never),
                    l @ StageLType::List(_) => Ok(l),
                    other => bail!(
                        "'slice' sequence must be 'l/T', found '{other}' at {span}"
                    ),
                }
            }
            CoreTernaryOp::BufSet => {
                let buf_ty = self.check_subexpr(first, state, ctx)?;
                let idx_ty = self.check_subexpr(second, state, ctx)?;
                let val_ty = self.check_subexpr(third, state, ctx)?;
                ensure!(
                    types_compatible(&idx_ty, &StageLType::U64),
                    "'bufSet' index at {} must be 'u64', found '{idx_ty}'",
                    second.span()
                );
                match buf_ty {
                    StageLType::Never => Ok(StageLType::Never),
                    StageLType::Buf(elem) => {
                        ensure!(
                            types_compatible(&val_ty, &elem),
                            "'bufSet' value type mismatch at {span}: expected '{elem}', found '{val_ty}'"
                        );
                        Ok(StageLType::Void)
                    }
                    other => bail!(
                        "'bufSet' target must be 'buf/T', found '{other}' at {span}"
                    ),
                }
            }
        }
    }
}

fn unify_int_pair(
    op_name: &str,
    lty: &StageLType,
    rty: &StageLType,
    span: Span,
) -> Result<StageLType> {
    match (lty == &StageLType::Never, rty == &StageLType::Never) {
        (true, true) => Ok(StageLType::Never),
        (true, false) => {
            ensure!(
                rty.is_integer(),
                "'{op_name}' requires integer operands, found '{rty}' at {span}"
            );
            Ok(rty.clone())
        }
        (false, true) => {
            ensure!(
                lty.is_integer(),
                "'{op_name}' requires integer operands, found '{lty}' at {span}"
            );
            Ok(lty.clone())
        }
        (false, false) => {
            ensure!(
                lty == rty && lty.is_integer(),
                "'{op_name}' requires matching integer types at {span}, found '{lty}' and '{rty}'"
            );
            Ok(lty.clone())
        }
    }
}

fn collect_value_deps(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::ValueRef {
            target: SymbolTarget::Ident(n),
            ..
        } => out.push(n.clone()),
        Expr::List { elements, .. } => {
            for e in elements {
                collect_value_deps(e, out);
            }
        }
        Expr::Unary { operand, .. } => collect_value_deps(operand, out),
        Expr::Binary { left, right, .. } => {
            collect_value_deps(left, out);
            collect_value_deps(right, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_value_deps(first, out);
            collect_value_deps(second, out);
            collect_value_deps(third, out);
        }
        Expr::Conv { expr, .. } | Expr::Bitcast { expr, .. } => {
            collect_value_deps(expr, out);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_value_deps(cond, out);
            collect_value_deps(then_branch, out);
            collect_value_deps(else_branch, out);
        }
        _ => {}
    }
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
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Unit tests"
)]
mod tests {
    use super::*;
    use super::super::parser::parse_module;

    #[crate::ctb_test]
    fn test_psr4_paths_and_use_import_validation() {
        let files = [ModuleInput {
            source_path: Some("app/math.sl3"),
            source: "ns demo/wrong.\npub v/u32/id u32/x :: u32/x.\n",
        }];
        let err = parse_and_validate_workspace(&files)
            .expect_err("PSR-4 mismatch should fail");
        assert!(
            err.to_string().contains("PSR-4"),
            "Unexpected error: {err}"
        );

        // Cycle detection A -> B -> A
        let cyclic = [
            ModuleInput {
                source_path: Some("a/one.sl3"),
                source: "ns a/one.\nuse b/two: fnB ::.\npub v/u32/fnA :: fnB.\n",
            },
            ModuleInput {
                source_path: Some("b/two.sl3"),
                source: "ns b/two.\nuse a/one: fnA ::.\npub v/u32/fnB :: fnA.\n",
            },
        ];
        let err = parse_and_validate_workspace(&cyclic)
            .expect_err("Cyclic 'use' graph should fail");
        assert!(
            err.to_string().contains("Cyclic"),
            "Unexpected cycle error: {err}"
        );

        // Importing private symbol fails
        let priv_imp = [
            ModuleInput {
                source_path: Some("lib/secret.sl3"),
                source: "ns lib/secret.\nv/u32/hidden :: u32:1.\n",
            },
            ModuleInput {
                source_path: Some("app/main.sl3"),
                source: "ns app/main.\nuse lib/secret: hidden ::.\npub v/u32/f :: hidden.\n",
            },
        ];
        let err = parse_and_validate_workspace(&priv_imp)
            .expect_err("Importing private symbol should fail");
        assert!(
            err.to_string().contains("hidden"),
            "Unexpected error: {err}"
        );

        // Wildcard import rejected in parser
        let wildcard_src = "ns app/main.\nuse lib/mod: * ::.\n";
        let err = parse_module(wildcard_src)
            .expect_err("Wildcard use import must fail");
        assert!(err.to_string().contains("Wildcard"));

        // scan_module_header_paths verifies raw ns/use extraction
        let (ns, uses) = scan_module_header_paths(
            "ns pkg/sub.\nuse a/b: x y as z ::.\nuse c/d: k ::.\n",
        )
        .expect("scan_module_header_paths");
        assert_eq!(ns, Some(vec!["pkg".to_string(), "sub".to_string()]));
        assert_eq!(
            uses,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string(), "d".to_string()],
            ]
        );
    }

    #[crate::ctb_test]
    fn test_const_purity_cycles_and_buf_local_only_rules() {
        // Const cycle
        let cycle_src = r#"
const u32/firstConst add u32/secondConst u32:1.
const u32/secondConst add u32/firstConst u32:2.
"#;
        let err = validate_module(&parse_module(cycle_src).unwrap())
            .expect_err("Const cycle must fail");
        assert!(
            err.to_string().contains("Cycle") || err.to_string().contains("Cyclic"),
            "Unexpected error: {err}"
        );

        // Const referencing impure/runtime routine fails
        let impure_const = r#"
v/u32/runtimeFn :: u32:5.
const u32/bad runtimeFn.
"#;
        let err = validate_module(&parse_module(impure_const).unwrap())
            .expect_err("Const calling routine must fail");
        assert!(
            err.to_string().contains("constant")
                || err.to_string().contains("const")
                || err.to_string().contains("pure")
                || err.to_string().contains("Routine"),
            "Unexpected error: {err}"
        );

        // Module-level constant holding buf/u8 fails
        let pub_buf_src = r#"
pub buf/u8/badBuf bufNew u8 u64:4.
"#;
        let err = validate_module(&parse_module(pub_buf_src).unwrap())
            .expect_err("buf in pub const must fail");
        assert!(err.to_string().contains("buf"));

        // Exporting pub routine returning buf/u8 fails
        let ret_buf_src = r#"
pub v/buf/u8/badRet :: bufNew u8 u64:4.
"#;
        let err = validate_module(&parse_module(ret_buf_src).unwrap())
            .expect_err("Exporting pub buf/T return must fail");
        assert!(err.to_string().contains("buf"));

        // Nested list of buf (l/buf/u8) fails
        let nested_buf_src = r#"
v/u32/badNested
  let l/buf/u8/xs l/buf/u8: bufNew u8 u64:1 ::.
  u32:0.
"#;
        let err = validate_module(&parse_module(nested_buf_src).unwrap())
            .expect_err("l/buf/T must fail");
        assert!(err.to_string().contains("buf"));
    }

    #[crate::ctb_test]
    fn test_lexical_non_shadowing_and_definite_assignment() {
        // Re-declaring let in same block fails
        let redecl_src = r#"
v/u32/f
  let u32/x u32:1.
  let u32/x u32:2.
  u32/x.
"#;
        let err = validate_module(&parse_module(redecl_src).unwrap())
            .expect_err("Re-declaring let must fail");
        assert!(
            err.to_string().contains("re-declare")
                || err.to_string().contains("shadow")
        );

        // Shadowing outer variable in nested block fails
        let shadow_src = r#"
v/u32/f u32/p
  if true do
    let u32/p u32:3.
    u32/p.
  :: u32:0.
"#;
        let err = validate_module(&parse_module(shadow_src).unwrap())
            .expect_err("Shadowing param must fail");
        assert!(
            err.to_string().contains("shadow")
                || err.to_string().contains("re-declare")
        );

        // Setting a constant or parameter fails
        let set_param_src = r#"
v/u32/f u32/p
  set u32/p u32:9.
  u32/p.
"#;
        let err = validate_module(&parse_module(set_param_src).unwrap())
            .expect_err("Setting param must fail");
        assert!(
            err.to_string().contains("immutable")
                || err.to_string().contains("let")
        );

        // Reading uninitialized let fails
        let uninit_read = r#"
v/u32/f
  let u32/x.
  u32/x.
"#;
        let err = validate_module(&parse_module(uninit_read).unwrap())
            .expect_err("Reading uninit let must fail");
        assert!(
            err.to_string().contains("definitely")
                || err.to_string().contains("uninitialized")
                || err.to_string().contains("before"),
            "Unexpected error: {err}"
        );

        // Initialized only in one branch fails after 'if'
        let one_branch_src = r#"
v/u32/f b/cond
  let u32/x.
  if b/cond do
    set u32/x u32:1.
    u32:0.
  :: u32:0.
  u32/x.
"#;
        validate_module(&parse_module(one_branch_src).unwrap())
            .expect_err("Partial branch init must fail");

        // Initialized inside while body is still uninit after while
        let while_init_src = r#"
v/u32/f b/cond
  let u32/x.
  while b/cond do
    set u32/x u32:1.
  ::.
  u32/x.
"#;
        validate_module(&parse_module(while_init_src).unwrap())
            .expect_err("While-body init must fail after loop");

        // One branch initializes and the other is Never (fault) -> OK!
        let both_or_never_ok = r#"
v/u32/f b/cond
  let u32/x.
  if b/cond do
    set u32/x u32:42.
  :: do
    fault.
  ::.
  u32/x.
"#;
        validate_module(&parse_module(both_or_never_ok).unwrap())
            .expect("fault else branch guarantees x assigned");

        // Nested verb capture: called BEFORE set fails
        let capture_before_set = r#"
v/u32/f
  let u32/x.
  v/u32/readX :: u32/x.
  let u32/early readX.
  set u32/x u32:5.
  u32/early.
"#;
        validate_module(&parse_module(capture_before_set).unwrap())
            .expect_err("Calling nested verb before captured let is set must fail");

        // Nested verb capture: uncalled nested verb reading uninitialized var fails
        let capture_never_init = r#"
v/u32/f
  let u32/x.
  v/u32/readX :: u32/x.
  u32:0.
"#;
        validate_module(&parse_module(capture_never_init).unwrap())
            .expect_err("Capturing uninitialized outer variable fails");

        // Nested verb capture: called AFTER set succeeds!
        let capture_after_set = r#"
v/u32/f
  let u32/x.
  v/u32/readX :: u32/x.
  set u32/x u32:9.
  readX.
"#;
        validate_module(&parse_module(capture_after_set).unwrap())
            .expect("Calling nested verb after set should validate");

        // Nested verb initializes outer variable via set, then outer reads after calling it -> OK!
        let capture_init_inside_nested = r#"
v/u32/f
  let u32/x.
  v/void/initX :: set u32/x u32:12.
  initX.
  u32/x.
"#;
        validate_module(&parse_module(capture_init_inside_nested).unwrap())
            .expect("Nested verb initializing captured var should validate");
    }

    #[crate::ctb_test]
    fn test_strict_types_loop_control_and_tail_spans() {
        // No implicit coercions between u8 / u32 / l/u8
        let bad_coerce = r#"
v/u32/f u8/alpha :: add u32:1 u8/alpha.
"#;
        let err = validate_module(&parse_module(bad_coerce).unwrap())
            .expect_err("u8 -> u32 coercion must fail");
        assert!(
            err.to_string().contains("mismatch")
                || err.to_string().contains("matching integer types"),
            "Unexpected error: {err}"
        );

        let bad_list_type = r#"
v/l/u32/f :: l/u8: u8:1 u8:2 ::.
"#;
        let err = validate_module(&parse_module(bad_list_type).unwrap())
            .expect_err("l/u8 vs l/u32 must fail");
        assert!(
            err.to_string().contains("l/u8") && err.to_string().contains("l/u32"),
            "Unexpected error: {err}"
        );

        // break outside loop or inside nested verb fails
        let bad_break_nested = r#"
v/u32/f
  while true do
    v/void/stopper :: break.
    stopper.
  ::.
  u32:0.
"#;
        let err = validate_module(&parse_module(bad_break_nested).unwrap())
            .expect_err("break inside nested verb must fail");
        assert!(
            err.to_string().contains("while") || err.to_string().contains("loop")
        );

        // Tail call recording vs non-tail call
        let tail_src = r#"v/u32/helper u32/n :: u32/n.
v/u32/caller u32/n
  let u32/nonTail helper u32/n.
  helper u32/nonTail.
"#;
        let vm = validate_module(&parse_module(tail_src).unwrap()).unwrap();
        assert_eq!(vm.tail_calls.len(), 1);
        assert!(vm.tail_calls.contains(&(4, 3)));
    }
}
