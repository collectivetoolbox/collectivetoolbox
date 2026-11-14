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
    Module, PositionalStructDecl, RoutineDecl, Span, StageLType, StructTypeRef,
    SymbolTarget, TestExpected, TopDecl, TopValueDecl, format_dc_shorthand,
};
use super::parser::{parse_module_with_imports, scan_module_header_paths};
use crate::{DcChar, DcShorthand};
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

/// Unique signature key for a top-level routine within a module (`name` + exact parameter types).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutineKey {
    /// Routine programmatic identifier in its declaring module.
    pub name: String,
    /// Ordered formal parameter types.
    pub params: Vec<StageLType>,
}

impl RoutineKey {
    /// Builds a [`RoutineKey`] from a [`RoutineDecl`].
    #[must_use]
    pub fn from_decl(r: &RoutineDecl) -> Self {
        Self {
            name: r.name.clone(),
            params: r.params.iter().map(|p| p.ty.clone()).collect(),
        }
    }
}

/// Unique key for a top-level constant value within a module (`name` + `StageLType`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueKey {
    /// Constant programmatic identifier in its declaring module.
    pub name: String,
    /// Declared [`StageLType`].
    pub ty: StageLType,
}

impl ValueKey {
    /// Builds a [`ValueKey`] from a [`TopValueDecl`].
    #[must_use]
    pub fn from_decl(v: &TopValueDecl) -> Self {
        Self {
            name: v.name.clone(),
            ty: v.ty.clone(),
        }
    }
}

/// Statically resolved top-level or imported routine target for an `Expr::Call` site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallTarget {
    /// `None` if defined in the current module; `Some(ns)` if imported from `ns`.
    pub namespace: Option<String>,
    /// Original routine identifier in the declaring module.
    pub routine_name: String,
    /// Concrete signature of the selected routine overload.
    pub sig: RoutineSig,
}

impl ResolvedCallTarget {
    /// Returns the [`RoutineKey`] in the declaring module.
    #[must_use]
    pub fn routine_key(&self) -> RoutineKey {
        RoutineKey {
            name: self.routine_name.clone(),
            params: self.sig.params.clone(),
        }
    }
}

/// Statically resolved top-level or imported constant target for an `Expr::ValueRef` site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedValueTarget {
    /// `None` if defined in the current module; `Some(ns)` if imported from `ns`.
    pub namespace: Option<String>,
    /// Original constant identifier in the declaring module.
    pub original_name: String,
    /// Static constant type.
    pub ty: StageLType,
    /// Optional Dc identity on the target constant.
    pub dc_id: Option<DcChar>,
}

impl ResolvedValueTarget {
    /// Returns the [`ValueKey`] in the declaring module.
    #[must_use]
    pub fn value_key(&self) -> ValueKey {
        ValueKey {
            name: self.original_name.clone(),
            ty: self.ty.clone(),
        }
    }
}

/// Validated positional `struct` definition (`Dc 2217`..`Dc 2218`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDef {
    /// Defining namespace path (`None` if declared in an anonymous module).
    pub namespace: Option<String>,
    /// Original struct identifier in its declaring module.
    pub original_name: String,
    /// Canonicalized ordered slot types (`>= 1` slot).
    pub fields: Vec<StageLType>,
    /// Export visibility.
    pub is_pub: bool,
    /// Optional assigned graph Dc identity.
    pub dc_id: Option<DcChar>,
    /// Source declaration span.
    pub span: Span,
}

/// Exported items from one compiled namespace module.
#[derive(Debug, Clone, Default)]
pub struct NamespaceExports {
    /// Exported `pub` immutable values by unique constant name.
    pub values: HashMap<String, (StageLType, Option<DcChar>)>,
    /// Exported `pub` routines by name (same arity, pairwise-distinct parameter type tuples).
    pub routines: HashMap<String, Vec<RoutineSig>>,
    /// Exported `pub` positional structs by exported name.
    pub structs: HashMap<String, StructDef>,
    /// Transitive closure of canonical [`StructDef`]s reachable via this namespace's exports.
    pub all_structs: BTreeMap<(Option<String>, String), StructDef>,
    /// Assigned Dc shorthand mappings (`dc -> ImportedSymbol`).
    pub by_dc: HashMap<DcChar, ImportedSymbol>,
}

/// Resolved imported symbol entry inside a dependent module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportedSymbol {
    /// Imported `pub` constant value `(namespace_key, original_name, ty, dc)`.
    Value {
        /// Defining namespace path key (`"std/math"`).
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
    /// Imported `pub` positional `struct` type.
    Struct {
        /// Defining namespace path key.
        namespace: String,
        /// Original exported struct name.
        original_name: String,
        /// Canonicalized ordered slot types.
        fields: Vec<StageLType>,
        /// Optional Dc identity.
        dc_id: Option<DcChar>,
    },
}

/// Metadata collected for a validated single module.
#[derive(Debug, Clone)]
pub struct ValidatedModule {
    /// Parsed AST.
    pub module: Module,
    /// Namespace key if `ns` was declared.
    pub namespace_key: Option<String>,
    /// Resolved `use` bindings visible inside the module (`local_name -> candidates`).
    pub imports: HashMap<String, Vec<ImportedSymbol>>,
    /// Direct Dc call/value/struct targets resolved from `use dc/<sh>` or System-graph linking.
    pub dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Local or imported struct names mapped to canonical `(namespace, original_name)`.
    pub local_structs: HashMap<String, (Option<String>, String)>,
    /// Local or imported struct Dcs mapped to canonical `(namespace, original_name)`.
    pub dc_structs: HashMap<DcChar, (Option<String>, String)>,
    /// All canonical [`StructDef`]s declared or transitively reachable in this module.
    pub struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    /// Tail-position annotations indexed by `Span` (`line`, `col`) of `Call`.
    pub tail_calls: HashSet<(usize, usize)>,
    /// Statically resolved top-level/imported routine call targets indexed by `Span` (`line`, `col`).
    pub resolved_calls: HashMap<(usize, usize), ResolvedCallTarget>,
    /// Statically resolved top-level/imported constant targets indexed by `Span` (`line`, `col`).
    pub resolved_values: HashMap<(usize, usize), ResolvedValueTarget>,
    /// Statically resolved positional `struct` targets for [`Expr::StructNew`] indexed by `Span` (`line`, `col`).
    pub resolved_structs: HashMap<(usize, usize), StructDef>,
    /// Statically resolved [`StageLType`] for every [`Expr::ValueRef`] site indexed by `Span` (`line`, `col`).
    pub value_ref_types: HashMap<(usize, usize), StageLType>,
    /// Canonicalized static [`StageLType`] for every validated [`Expr`] indexed by `Span` (`line`, `col`).
    pub expr_types: HashMap<(usize, usize), StageLType>,
    /// Module-level exported items.
    pub exports: NamespaceExports,
}

impl ValidatedModule {
    /// Resolves any syntactic struct type references inside `ty` to canonical
    /// [`StructTypeRef::Canonical`] identities using this module's symbol tables.
    #[must_use]
    pub fn canonicalize_type(&self, ty: &StageLType) -> StageLType {
        match ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64
            | StageLType::Void
            | StageLType::Never => ty.clone(),
            StageLType::List(inner) => {
                StageLType::List(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Buf(inner) => {
                StageLType::Buf(Box::new(self.canonicalize_type(inner)))
            }
            StageLType::Tuple(elems) => StageLType::Tuple(
                elems.iter().map(|e| self.canonicalize_type(e)).collect(),
            ),
            StageLType::Struct(StructTypeRef::Ident(name)) => {
                if let Some((ns, orig)) = self.local_structs.get(name) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Struct(StructTypeRef::Dc(dc)) => {
                if let Some((ns, orig)) = self.dc_structs.get(dc) {
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace: ns.clone(),
                        name: orig.clone(),
                    })
                } else {
                    ty.clone()
                }
            }
            StageLType::Struct(StructTypeRef::Canonical { .. }) => ty.clone(),
        }
    }

    /// Builds a canonicalized [`RoutineKey`] for `r` declared in this module.
    #[must_use]
    pub fn routine_key(&self, r: &RoutineDecl) -> RoutineKey {
        RoutineKey {
            name: r.name.clone(),
            params: r
                .params
                .iter()
                .map(|p| self.canonicalize_type(&p.ty))
                .collect(),
        }
    }

    /// Builds a canonicalized [`ValueKey`] for `v` declared in this module.
    #[must_use]
    pub fn value_key(&self, v: &TopValueDecl) -> ValueKey {
        ValueKey {
            name: v.name.clone(),
            ty: self.canonicalize_type(&v.ty),
        }
    }
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
        let forbid_stagel_internal =
            inp.source_path.is_some_and(is_stage2_or_sr3_path);
        let vm = validate_module_with_registry_and_dc_imports_and_options(
            &parsed,
            &ns_exports,
            &HashMap::new(),
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

pub(crate) fn is_stage2_or_sr3_path(source_path: &str) -> bool {
    let norm = source_path.replace('\\', "/");
    norm.split('/').any(|seg| seg == "impls")
        || norm.ends_with(".sr3")
        || norm.ends_with(".sri3")
}

pub(crate) fn is_stagel_internal_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with("stagelinternal")
}

pub(crate) fn verify_psr4_path(source_path: &str, ns_key: &str) -> Result<()> {
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

pub(crate) fn collect_use_verb_arities_from_source(
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
                    if let Some(sigs) = exports.routines.get(item_w) {
                        let first_sig = sigs
                            .first()
                            .context("Empty routine export overload list")?;
                        let local_name = if words.get(wi) == Some(&"as") {
                            wi = wi.saturating_add(1);
                            let a = *words.get(wi).context("Expected alias after 'as'")?;
                            wi = wi.saturating_add(1);
                            a
                        } else {
                            item_w
                        };
                        if let Some(&prev_arity) = arities.get(local_name) {
                            ensure!(
                                prev_arity == first_sig.params.len(),
                                "Conflicting parameter count for imported routine '{local_name}': expected {prev_arity}, found {}",
                                first_sig.params.len()
                            );
                        } else {
                            arities.insert(local_name.to_owned(), first_sig.params.len());
                        }
                        continue;
                    }
                    if !exports.values.contains_key(item_w)
                        && !exports.structs.contains_key(item_w)
                    {
                        bail!(
                            "Namespace '{path}' has no exported item '{item_w}'"
                        );
                    }
                    if words.get(wi) == Some(&"as") {
                        wi = wi.saturating_add(2);
                    }
                    continue;
                };
                let has_alias = words.get(wi) == Some(&"as");
                let local_name = if has_alias {
                    wi = wi.saturating_add(1);
                    let a = *words.get(wi).context("Expected alias after 'as'")?;
                    wi = wi.saturating_add(1);
                    a
                } else {
                    item_w
                };
                if let ImportedSymbol::Routine {
                    original_name, sig, ..
                } = sym
                {
                    let new_arity = sig.params.len();
                    if let Some(&prev_arity) = arities.get(local_name) {
                        ensure!(
                            prev_arity == new_arity,
                            "Conflicting parameter count for imported routine '{local_name}': expected {prev_arity}, found {new_arity}"
                        );
                    } else {
                        arities.insert(local_name.to_owned(), new_arity);
                    }
                    if !has_alias {
                        if let Some(&prev_arity) = arities.get(original_name.as_str()) {
                            ensure!(
                                prev_arity == new_arity,
                                "Conflicting parameter count for imported routine '{original_name}': expected {prev_arity}, found {new_arity}"
                            );
                        } else {
                            arities.insert(original_name.clone(), new_arity);
                        }
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
    Ok(arities)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BindingKind {
    MutableLet,
    Const,
    Param,
    ForEachElem,
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
    Nested(usize),
}

struct Validator<'a> {
    /// Module-scope constants (`local_name -> ResolvedValueTarget`), combining
    /// imported and top-level constants (at most one definition per name).
    value_candidates: HashMap<String, ResolvedValueTarget>,
    /// Module-scope routine candidates (`local_name -> Vec<ResolvedCallTarget>`),
    /// combining imported and top-level routines with identical arity and
    /// pairwise-distinct parameter type tuples.
    routine_candidates: HashMap<String, Vec<ResolvedCallTarget>>,
    /// Local or imported struct names mapped to canonical `(namespace, original_name)`.
    local_structs: HashMap<String, (Option<String>, String)>,
    /// Local or imported struct Dcs mapped to canonical `(namespace, original_name)`.
    dc_structs: HashMap<DcChar, (Option<String>, String)>,
    /// All canonical [`StructDef`]s declared or transitively reachable in this module.
    struct_defs: BTreeMap<(Option<String>, String), StructDef>,
    /// Top-level constant types keyed directly by assigned `DcChar`.
    top_dc_values: HashMap<DcChar, StageLType>,
    /// Top-level routine signatures keyed directly by assigned `DcChar`.
    top_dc_routines: HashMap<DcChar, RoutineSig>,
    /// Top-level declaration names keyed by assigned `DcChar`.
    top_dc_names: HashMap<DcChar, String>,
    /// Imported items keyed directly by `DcChar` (`dc/<shorthand>`).
    dc_imports: HashMap<DcChar, ImportedSymbol>,
    /// Whether `*StageLInternal` symbols are prohibited in this module.
    forbid_stagel_internal: bool,
    /// Next unique lexical variable ID.
    next_binding_id: usize,
    /// Active lexical variable scopes (outermost local to innermost).
    local_scopes: Vec<HashMap<String, BindingEntry>>,
    /// Active nested routine scopes (`name -> VerbTarget`).
    routine_scopes: Vec<HashMap<String, VerbTarget>>,
    /// Summaries of validated nested verbs keyed by numeric ID.
    nested_summaries: HashMap<usize, NestedVerbSummary>,
    /// Recorded calls to nested verbs in the current block `(nid, span, state_snapshot)`.
    pending_nested_calls: Vec<(usize, Span, AssignState)>,
    /// Recorded tail call positions `(line, col)`.
    tail_calls: HashSet<(usize, usize)>,
    /// Statically resolved top-level/imported call targets indexed by `(line, col)`.
    resolved_calls: HashMap<(usize, usize), ResolvedCallTarget>,
    /// Statically resolved top-level/imported constant targets indexed by `(line, col)`.
    resolved_values: HashMap<(usize, usize), ResolvedValueTarget>,
    /// Statically resolved positional `struct` targets for [`Expr::StructNew`] indexed by `(line, col)`.
    resolved_structs: HashMap<(usize, usize), StructDef>,
    /// Statically resolved [`StageLType`] for every [`Expr::ValueRef`] site indexed by `(line, col)`.
    value_ref_types: HashMap<(usize, usize), StageLType>,
    /// Canonicalized static [`StageLType`] for every validated [`Expr`] indexed by `(line, col)`.
    expr_types: HashMap<(usize, usize), StageLType>,
    /// Phantom marker tying lifetime `'a` when needed by helper methods.
    _marker: std::marker::PhantomData<&'a ()>,
}

fn canonicalize_type_with_maps(
    ty: &StageLType,
    local_structs: &HashMap<String, (Option<String>, String)>,
    dc_structs: &HashMap<DcChar, (Option<String>, String)>,
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
    forbid_stagel_internal: bool,
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
        | StageLType::Void
        | StageLType::Never => Ok(ty.clone()),
        StageLType::List(inner) => Ok(StageLType::List(Box::new(
            canonicalize_type_with_maps(
                inner,
                local_structs,
                dc_structs,
                struct_defs,
                forbid_stagel_internal,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Buf(inner) => Ok(StageLType::Buf(Box::new(
            canonicalize_type_with_maps(
                inner,
                local_structs,
                dc_structs,
                struct_defs,
                forbid_stagel_internal,
                top_dc_names,
                span,
            )?,
        ))),
        StageLType::Tuple(elems) => {
            let mut out = Vec::with_capacity(elems.len());
            for el in elems {
                out.push(canonicalize_type_with_maps(
                    el,
                    local_structs,
                    dc_structs,
                    struct_defs,
                    forbid_stagel_internal,
                    top_dc_names,
                    span,
                )?);
            }
            Ok(StageLType::Tuple(out))
        }
        StageLType::Struct(StructTypeRef::Ident(name)) => {
            if forbid_stagel_internal && is_stagel_internal_name(name) {
                bail!(
                    "StageL-internal struct '{name}' cannot be referenced in Stage 2/StageR module at {span}"
                );
            }
            let Some((ns, orig)) = local_structs.get(name) else {
                bail!("Unknown StageL struct type '{name}' at {span}");
            };
            if forbid_stagel_internal && is_stagel_internal_name(orig) {
                bail!(
                    "StageL-internal struct '{orig}' cannot be referenced in Stage 2/StageR module at {span}"
                );
            }
            Ok(StageLType::Struct(StructTypeRef::Canonical {
                namespace: ns.clone(),
                name: orig.clone(),
            }))
        }
        StageLType::Struct(StructTypeRef::Dc(dc)) => {
            let sh = format_dc_shorthand(*dc);
            if forbid_stagel_internal
                && let Some(orig) = top_dc_names.get(dc)
                && is_stagel_internal_name(orig)
            {
                bail!(
                    "StageL-internal struct '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
                );
            }
            let Some((ns, orig)) = dc_structs.get(dc) else {
                bail!("Unknown StageL struct Dc reference 'dc/{sh}' at {span}");
            };
            if forbid_stagel_internal && is_stagel_internal_name(orig) {
                bail!(
                    "StageL-internal struct '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
                );
            }
            Ok(StageLType::Struct(StructTypeRef::Canonical {
                namespace: ns.clone(),
                name: orig.clone(),
            }))
        }
        StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
            let key = (namespace.clone(), name.clone());
            ensure!(
                struct_defs.contains_key(&key),
                "Unresolved canonical struct '{name}' at {span}"
            );
            Ok(ty.clone())
        }
    }
}

fn type_contains_buf_with_defs(
    ty: &StageLType,
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
) -> bool {
    fn visit(
        ty: &StageLType,
        struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
        visited: &mut HashSet<(Option<String>, String)>,
    ) -> bool {
        match ty {
            StageLType::Buf(_) => true,
            StageLType::List(inner) => visit(inner, struct_defs, visited),
            StageLType::Tuple(elems) => {
                elems.iter().any(|e| visit(e, struct_defs, visited))
            }
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                let key = (namespace.clone(), name.clone());
                if !visited.insert(key.clone()) {
                    return false;
                }
                struct_defs.get(&key).is_some_and(|sdef| {
                    sdef.fields.iter().any(|f| visit(f, struct_defs, visited))
                })
            }
            _ => false,
        }
    }
    let mut visited = HashSet::new();
    visit(ty, struct_defs, &mut visited)
}

fn type_can_be_element_with_defs(
    ty: &StageLType,
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
) -> bool {
    ty.can_be_element() && !type_contains_buf_with_defs(ty, struct_defs)
}

/// Evaluates whether a canonicalized type is valid in a StageL 3 routine or
/// expression, rejecting invalid element types like `l/void`, `l/buf/u8`, or
/// `tup: void ::`.
fn check_type_well_formed(
    ty: &StageLType,
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
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
                type_can_be_element_with_defs(inner, struct_defs),
                "Invalid list element type '{}' in 'l/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(inner, struct_defs, is_pub_api, span)
        }
        StageLType::Buf(inner) => {
            ensure!(
                !is_pub_api,
                "Local buffer type 'buf/{}' is disallowed in public constants/signatures at {span}",
                inner.display_name()
            );
            ensure!(
                type_can_be_element_with_defs(inner, struct_defs),
                "Invalid buffer element type '{}' in 'buf/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(inner, struct_defs, is_pub_api, span)
        }
        StageLType::Tuple(elems) => {
            for el in elems {
                ensure!(
                    *el != StageLType::Void && *el != StageLType::Never,
                    "Invalid tuple slot type '{}' in '{}' at {span}",
                    el.display_name(),
                    ty.display_name()
                );
                check_type_well_formed(el, struct_defs, is_pub_api, span)?;
            }
            Ok(())
        }
        StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
            let key = (namespace.clone(), name.clone());
            ensure!(
                struct_defs.contains_key(&key),
                "Unknown struct type '{name}' at {span}"
            );
            if is_pub_api {
                ensure!(
                    !type_contains_buf_with_defs(ty, struct_defs),
                    "Struct '{name}' containing local buffer 'buf/T' is disallowed in public constants/signatures at {span}"
                );
            }
            Ok(())
        }
        StageLType::Struct(_) => {
            bail!("Uncanonicalized struct type '{}' at {span}", ty.display_name())
        }
    }
}

fn check_unguarded_struct_cycles(
    struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
) -> Result<()> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Visiting,
        Visited,
    }

    fn collect_direct_struct_edges(
        ty: &StageLType,
        out: &mut Vec<(Option<String>, String)>,
    ) {
        match ty {
            StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
                out.push((namespace.clone(), name.clone()));
            }
            StageLType::Tuple(elems) => {
                for el in elems {
                    collect_direct_struct_edges(el, out);
                }
            }
            // `List(T)` and `Buf(T)` are heap-indirected sequence headers and break value-size cycles.
            _ => {}
        }
    }

    fn dfs(
        key: &(Option<String>, String),
        struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
        marks: &mut HashMap<(Option<String>, String), Mark>,
        stack: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(&m) = marks.get(key) {
            match m {
                Mark::Visited => return Ok(()),
                Mark::Visiting => {
                    stack.push(key.1.clone());
                    let sdef = struct_defs.get(key);
                    // Reason for fallback: Default 1:1 span only if missing from map.
                    let sp = sdef.map_or(Span { line: 1, col: 1 }, |d| d.span);
                    bail!(
                        "Infinite-size unguarded recursive struct cycle '{}' at {sp} (wrap recursive struct references in 'l/...')",
                        stack.join(" -> ")
                    );
                }
            }
        }
        marks.insert(key.clone(), Mark::Visiting);
        stack.push(key.1.clone());
        if let Some(sdef) = struct_defs.get(key) {
            let mut edges = Vec::new();
            for f in &sdef.fields {
                collect_direct_struct_edges(f, &mut edges);
            }
            for target in edges {
                dfs(&target, struct_defs, marks, stack)?;
            }
        }
        let _ = stack.pop();
        marks.insert(key.clone(), Mark::Visited);
        Ok(())
    }

    let mut marks = HashMap::new();
    let mut stack = Vec::new();
    for key in struct_defs.keys() {
        if !marks.contains_key(key) {
            dfs(key, struct_defs, &mut marks, &mut stack)?;
        }
    }
    Ok(())
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
    validate_module_with_registry_and_dc_imports(
        module,
        ns_exports,
        &HashMap::new(),
    )
}

/// Validates a parsed `Module` against already-validated dependency exports and
/// pre-resolved direct `DcChar` symbol targets (such as transitive helpers
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

/// Validates a parsed `Module` with explicit Stage 2 / StageR `*StageLInternal`
/// gating enabled when `forbid_stagel_internal` is `true`.
pub fn validate_module_with_registry_and_dc_imports_and_options(
    module: &Module,
    ns_exports: &HashMap<String, NamespaceExports>,
    extra_dc_imports: &HashMap<DcChar, ImportedSymbol>,
    forbid_stagel_internal: bool,
) -> Result<ValidatedModule> {
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
    let mut struct_defs: BTreeMap<(Option<String>, String), StructDef> =
        BTreeMap::new();

    // Seed transitive struct definitions from all dependency namespaces and direct `dc_imports`.
    for dep_exp in ns_exports.values() {
        for (k, sdef) in &dep_exp.all_structs {
            struct_defs.insert(k.clone(), sdef.clone());
        }
    }
    let mut dc_import_struct_keys = Vec::new();
    for (&dc, sym) in &dc_imports {
        if let ImportedSymbol::Struct {
            namespace,
            original_name,
            fields,
            dc_id,
        } = sym
        {
            let key = (Some(namespace.clone()), original_name.clone());
            dc_structs.insert(dc, key.clone());
            struct_defs.entry(key.clone()).or_insert_with(|| StructDef {
                namespace: Some(namespace.clone()),
                original_name: original_name.clone(),
                fields: fields.clone(),
                is_pub: true,
                dc_id: *dc_id,
                span: Span { line: 1, col: 1 },
            });
            dc_import_struct_keys.push(key);
        }
    }
    let empty_local_structs = HashMap::new();
    let empty_top_dc_names = HashMap::new();
    let synth_span = Span { line: 1, col: 1 };
    for key in dc_import_struct_keys {
        if let Some(existing) = struct_defs.get(&key).cloned() {
            let mut canon_fields = Vec::with_capacity(existing.fields.len());
            for f in &existing.fields {
                canon_fields.push(canonicalize_type_with_maps(
                    f,
                    &empty_local_structs,
                    &dc_structs,
                    &struct_defs,
                    false,
                    &empty_top_dc_names,
                    synth_span,
                )?);
            }
            if let Some(slot) = struct_defs.get_mut(&key) {
                slot.fields = canon_fields;
            }
        }
    }
    for sym in dc_imports.values_mut() {
        match sym {
            ImportedSymbol::Value { ty, .. } => {
                *ty = canonicalize_type_with_maps(
                    ty,
                    &empty_local_structs,
                    &dc_structs,
                    &struct_defs,
                    false,
                    &empty_top_dc_names,
                    synth_span,
                )?;
            }
            ImportedSymbol::Routine { sig, .. } => {
                sig.return_type = canonicalize_type_with_maps(
                    &sig.return_type,
                    &empty_local_structs,
                    &dc_structs,
                    &struct_defs,
                    false,
                    &empty_top_dc_names,
                    synth_span,
                )?;
                let mut canon_params = Vec::with_capacity(sig.params.len());
                for p in &sig.params {
                    canon_params.push(canonicalize_type_with_maps(
                        p,
                        &empty_local_structs,
                        &dc_structs,
                        &struct_defs,
                        false,
                        &empty_top_dc_names,
                        synth_span,
                    )?);
                }
                sig.params = canon_params;
            }
            ImportedSymbol::Struct { fields, .. } => {
                let mut canon_fields = Vec::with_capacity(fields.len());
                for f in fields.iter() {
                    canon_fields.push(canonicalize_type_with_maps(
                        f,
                        &empty_local_structs,
                        &dc_structs,
                        &struct_defs,
                        false,
                        &empty_top_dc_names,
                        synth_span,
                    )?);
                }
                *fields = canon_fields;
            }
        }
    }

    // 1. Resolve `use` imports.
    for u in &module.uses {
        let target_ns = u.path.join("/");
        let dep_exp = ns_exports.get(&target_ns).with_context(|| {
            format!("Unresolved namespace '{target_ns}' in 'use' at {}", u.span)
        })?;
        for item in &u.items {
            let resolved_list: Vec<ImportedSymbol> = match &item.target {
                ImportTarget::Name(name) => {
                    if let Some((ty, dc_id)) = dep_exp.values.get(name) {
                        vec![ImportedSymbol::Value {
                            namespace: target_ns.clone(),
                            original_name: name.clone(),
                            ty: ty.clone(),
                            dc_id: *dc_id,
                        }]
                    } else if let Some(sig_list) = dep_exp.routines.get(name) {
                        sig_list
                            .iter()
                            .map(|sig| ImportedSymbol::Routine {
                                namespace: target_ns.clone(),
                                original_name: name.clone(),
                                sig: sig.clone(),
                            })
                            .collect()
                    } else if let Some(sdef) = dep_exp.structs.get(name) {
                        vec![ImportedSymbol::Struct {
                            namespace: target_ns.clone(),
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
                    let single = dep_exp
                        .by_dc
                        .get(dc)
                        .cloned()
                        .with_context(|| {
                            format!(
                                "Namespace '{target_ns}' has no exported 'pub dc/{}' (at {})",
                                format_dc_shorthand(*dc),
                                item.span
                            )
                        })?;
                    if item.alias.is_none() {
                        dc_imports.insert(*dc, single.clone());
                        if let ImportedSymbol::Struct {
                            ref namespace,
                            ref original_name,
                            ..
                        } = single
                        {
                            dc_structs.insert(
                                *dc,
                                (Some(namespace.clone()), original_name.clone()),
                            );
                        }
                    }
                    vec![single]
                }
            };

            let first_sym = resolved_list
                .first()
                .context("Empty exported symbol list")?;
            let local_name = match (&item.alias, &item.target) {
                (Some(a), _) => a.clone(),
                (None, ImportTarget::Name(n)) => n.clone(),
                (None, ImportTarget::Dc(_)) => match first_sym {
                    ImportedSymbol::Value { original_name, .. }
                    | ImportedSymbol::Routine { original_name, .. }
                    | ImportedSymbol::Struct { original_name, .. } => {
                        original_name.clone()
                    }
                },
            };

            if forbid_stagel_internal && is_stagel_internal_name(&local_name) {
                bail!(
                    "StageL-internal symbol '{local_name}' cannot be imported or declared in Stage 2/StageR module at {}",
                    item.span
                );
            }

            for resolved in resolved_list {
                match &resolved {
                    ImportedSymbol::Value {
                        namespace,
                        original_name,
                        ty,
                        dc_id,
                    } => {
                        if forbid_stagel_internal
                            && is_stagel_internal_name(original_name)
                        {
                            bail!(
                                "StageL-internal symbol '{original_name}' cannot be imported into Stage 2/StageR module at {}",
                                item.span
                            );
                        }
                        ensure!(
                            !routine_candidates.contains_key(&local_name)
                                && !local_structs.contains_key(&local_name),
                            "Conflicting imported name '{local_name}' at {}: cannot share a name with a routine or struct",
                            item.span
                        );
                        ensure!(
                            !value_candidates.contains_key(&local_name),
                            "Duplicate or conflicting imported constant '{local_name}' at {} (constant names must have a single definition)",
                            item.span
                        );
                        value_candidates.insert(
                            local_name.clone(),
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
                        if forbid_stagel_internal
                            && is_stagel_internal_name(original_name)
                        {
                            bail!(
                                "StageL-internal routine '{original_name}' cannot be imported into Stage 2/StageR module at {}",
                                item.span
                            );
                        }
                        ensure!(
                            !value_candidates.contains_key(&local_name)
                                && !local_structs.contains_key(&local_name),
                            "Conflicting imported name '{local_name}' at {}: cannot share a name with a constant or struct",
                            item.span
                        );
                        let list = routine_candidates
                            .entry(local_name.clone())
                            .or_default();
                        if let Some(first_existing) = list.first() {
                            ensure!(
                                first_existing.sig.params.len()
                                    == sig.params.len(),
                                "Conflicting parameter count for imported routine '{local_name}' at {}: expected {} parameter(s), found {}",
                                item.span,
                                first_existing.sig.params.len(),
                                sig.params.len()
                            );
                        }
                        ensure!(
                            !list.iter().any(|c| c.sig.params == sig.params),
                            "Duplicate or conflicting imported routine '{local_name}' with parameter types ({}) at {}",
                            format_param_types(&sig.params),
                            item.span
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
                        if forbid_stagel_internal
                            && is_stagel_internal_name(original_name)
                        {
                            bail!(
                                "StageL-internal struct '{original_name}' cannot be imported into Stage 2/StageR module at {}",
                                item.span
                            );
                        }
                        ensure!(
                            !value_candidates.contains_key(&local_name)
                                && !routine_candidates.contains_key(&local_name)
                                && !local_structs.contains_key(&local_name),
                            "Duplicate or conflicting imported struct '{local_name}' at {}",
                            item.span
                        );
                        let key =
                            (Some(namespace.clone()), original_name.clone());
                        local_structs.insert(local_name.clone(), key.clone());
                        struct_defs.entry(key).or_insert_with(|| StructDef {
                            namespace: Some(namespace.clone()),
                            original_name: original_name.clone(),
                            fields: fields.clone(),
                            is_pub: true,
                            dc_id: *dc_id,
                            span: item.span,
                        });
                    }
                }
                imports
                    .entry(local_name.clone())
                    .or_default()
                    .push(resolved);
            }
        }
    }

    // 2a. Pass-1a: collect top-level positional `struct` identities first so forward
    // references from earlier structs, constants, and routines resolve cleanly.
    let mut top_dc_values = HashMap::new();
    let mut top_dc_routines = HashMap::new();
    let mut top_dc_names = HashMap::new();
    let mut used_dcs = HashMap::new();
    let mut local_struct_decls: Vec<&PositionalStructDecl> = Vec::new();
    let mut top_value_order = Vec::new();
    let mut top_routine_order = Vec::new();

    for decl in &module.items {
        if let TopDecl::Struct(s) = decl {
            if forbid_stagel_internal && is_stagel_internal_name(&s.name) {
                bail!(
                    "StageL-internal struct '{}' cannot be declared in Stage 2/StageR module at {}",
                    s.name,
                    s.span
                );
            }
            ensure!(
                !s.fields.is_empty(),
                "Positional struct '{}' at {} must declare at least 1 slot",
                s.name,
                s.span
            );
            ensure!(
                !local_structs.contains_key(&s.name)
                    && !value_candidates.contains_key(&s.name)
                    && !routine_candidates.contains_key(&s.name),
                "Struct '{}' at {} duplicates/shadows an existing declaration or import",
                s.name,
                s.span
            );
            if s.pub_dc_syntax
                && let Some(dc) = s.dc_id
            {
                check_pub_dc_shorthand(dc, s.span)?;
            }
            let canon_key = (namespace_key.clone(), s.name.clone());
            if let Some(dc) = s.dc_id {
                ensure!(
                    !used_dcs.contains_key(&dc),
                    "Duplicate Dc binding '{}' on '{}' at {}",
                    format_dc_shorthand(dc),
                    s.name,
                    s.span
                );
                used_dcs.insert(dc, s.name.clone());
                top_dc_names.insert(dc, s.name.clone());
                dc_structs.insert(dc, canon_key.clone());
            }
            local_structs.insert(s.name.clone(), canon_key);
            local_struct_decls.push(s);
        }
    }

    // 2b. Pass-1b: canonicalize struct slot types, reject unguarded size cycles, and
    // verify well-formedness (including transitive `buf/T` prohibitions).
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
            let cf = canonicalize_type_with_maps(
                f,
                &local_structs,
                &dc_structs,
                &struct_defs,
                forbid_stagel_internal,
                &top_dc_names,
                s.span,
            )?;
            canon_fields.push(cf);
        }
        let canon_key = (namespace_key.clone(), s.name.clone());
        struct_defs.insert(
            canon_key,
            StructDef {
                namespace: namespace_key.clone(),
                original_name: s.name.clone(),
                fields: canon_fields,
                is_pub: s.is_pub,
                dc_id: s.dc_id,
                span: s.span,
            },
        );
    }

    check_unguarded_struct_cycles(&struct_defs)?;

    for &s in &local_struct_decls {
        let canon_key = (namespace_key.clone(), s.name.clone());
        let sdef = struct_defs
            .get(&canon_key)
            .cloned()
            .context("Missing registered struct definition")?;
        for cf in &sdef.fields {
            check_type_well_formed(cf, &struct_defs, s.is_pub, s.span)?;
        }
        if s.is_pub {
            exports.structs.insert(s.name.clone(), sdef.clone());
            if let Some(dc) = s.dc_id {
                exports.by_dc.insert(
                    dc,
                    ImportedSymbol::Struct {
                        // Reason for fallback: Anonymous modules without an 'ns' header use an empty namespace string.
                        namespace: namespace_key.clone().unwrap_or_default(),
                        original_name: s.name.clone(),
                        fields: sdef.fields,
                        dc_id: Some(dc),
                    },
                );
            }
        }
    }

    // 2c. Pass-1c: top-level constant and routine signature collection.
    for decl in &module.items {
        match decl {
            TopDecl::Comment(_) | TopDecl::Struct(_) => {}
            TopDecl::Value(v) => {
                if forbid_stagel_internal && is_stagel_internal_name(&v.name) {
                    bail!(
                        "StageL-internal constant '{}' cannot be declared in Stage 2/StageR module at {}",
                        v.name,
                        v.span
                    );
                }
                ensure!(
                    !routine_candidates.contains_key(&v.name)
                        && !local_structs.contains_key(&v.name),
                    "Top-level constant '{}' at {} duplicates/shadows an existing routine, struct, or import",
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
                let canon_ty = canonicalize_type_with_maps(
                    &v.ty,
                    &local_structs,
                    &dc_structs,
                    &struct_defs,
                    forbid_stagel_internal,
                    &top_dc_names,
                    v.span,
                )?;
                check_type_well_formed(&canon_ty, &struct_defs, true, v.span)?;
                ensure!(
                    !type_contains_buf_with_defs(&canon_ty, &struct_defs),
                    "Module-level constant '{}' at {} cannot hold local buffer 'buf/T'",
                    v.name,
                    v.span
                );
                ensure!(
                    !value_candidates.contains_key(&v.name),
                    "Top-level constant '{}' at {} duplicates/shadows existing binding or import (constant names must have a single definition)",
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
                    top_dc_values.insert(dc, canon_ty.clone());
                    top_dc_names.insert(dc, v.name.clone());
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
                    exports
                        .values
                        .insert(v.name.clone(), (canon_ty.clone(), v.dc_id));
                    if let Some(dc) = v.dc_id {
                        exports.by_dc.insert(
                            dc,
                            ImportedSymbol::Value {
                                // Reason for fallback: Anonymous modules without an 'ns' header use an empty namespace string.
                                namespace: namespace_key
                                    .clone()
                                    .unwrap_or_default(),
                                original_name: v.name.clone(),
                                ty: canon_ty,
                                dc_id: Some(dc),
                            },
                        );
                    }
                }
                top_value_order.push(v);
            }
            TopDecl::Routine(r) => {
                if forbid_stagel_internal && is_stagel_internal_name(&r.name) {
                    bail!(
                        "StageL-internal routine '{}' cannot be declared in Stage 2/StageR module at {}",
                        r.name,
                        r.span
                    );
                }
                ensure!(
                    !value_candidates.contains_key(&r.name)
                        && !local_structs.contains_key(&r.name),
                    "Routine '{}' at {} duplicates/shadows an existing constant, struct, or import",
                    r.name,
                    r.span
                );
                let canon_ret = canonicalize_type_with_maps(
                    &r.return_type,
                    &local_structs,
                    &dc_structs,
                    &struct_defs,
                    forbid_stagel_internal,
                    &top_dc_names,
                    r.span,
                )?;
                check_type_well_formed(
                    &canon_ret,
                    &struct_defs,
                    r.is_pub,
                    r.span,
                )?;
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
                    let canon_pty = canonicalize_type_with_maps(
                        &p.ty,
                        &local_structs,
                        &dc_structs,
                        &struct_defs,
                        forbid_stagel_internal,
                        &top_dc_names,
                        p.span,
                    )?;
                    check_type_well_formed(
                        &canon_pty,
                        &struct_defs,
                        r.is_pub,
                        p.span,
                    )?;
                    ensure!(
                        seen_params.insert(p.name.clone()),
                        "Duplicate parameter '{}' in routine '{}' at {}",
                        p.name,
                        r.name,
                        p.span
                    );
                    param_tys.push(canon_pty);
                }
                let rlist =
                    routine_candidates.entry(r.name.clone()).or_default();
                if let Some(first_existing) = rlist.first() {
                    ensure!(
                        first_existing.sig.params.len() == param_tys.len(),
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
                if r.pub_dc_syntax {
                    if let Some(dc) = r.dc_id {
                        check_pub_dc_shorthand(dc, r.span)?;
                    }
                }
                let sig = RoutineSig {
                    return_type: canon_ret,
                    params: param_tys,
                    is_pub: r.is_pub,
                    dc_id: r.dc_id,
                };
                if let Some(dc) = r.dc_id {
                    ensure!(
                        !used_dcs.contains_key(&dc),
                        "Duplicate Dc binding '{}' on '{}' at {}",
                        format_dc_shorthand(dc),
                        r.name,
                        r.span
                    );
                    used_dcs.insert(dc, r.name.clone());
                    top_dc_routines.insert(dc, sig.clone());
                    top_dc_names.insert(dc, r.name.clone());
                }
                rlist.push(ResolvedCallTarget {
                    namespace: None,
                    routine_name: r.name.clone(),
                    sig: sig.clone(),
                });
                if r.is_pub {
                    exports
                        .routines
                        .entry(r.name.clone())
                        .or_default()
                        .push(sig.clone());
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

    exports.all_structs = struct_defs.clone();

    let mut validator = Validator {
        value_candidates,
        routine_candidates,
        local_structs: local_structs.clone(),
        dc_structs: dc_structs.clone(),
        struct_defs: struct_defs.clone(),
        top_dc_values,
        top_dc_routines,
        top_dc_names,
        dc_imports: dc_imports.clone(),
        forbid_stagel_internal,
        next_binding_id: 1,
        local_scopes: Vec::new(),
        routine_scopes: Vec::new(),
        nested_summaries: HashMap::new(),
        pending_nested_calls: Vec::new(),
        tail_calls: HashSet::new(),
        resolved_calls: HashMap::new(),
        resolved_values: HashMap::new(),
        resolved_structs: HashMap::new(),
        value_ref_types: HashMap::new(),
        expr_types: HashMap::new(),
        _marker: std::marker::PhantomData,
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
        local_structs,
        dc_structs,
        struct_defs,
        tail_calls: validator.tail_calls,
        resolved_calls: validator.resolved_calls,
        resolved_values: validator.resolved_values,
        resolved_structs: validator.resolved_structs,
        value_ref_types: validator.value_ref_types,
        expr_types: validator.expr_types,
        exports,
    })
}

fn format_param_types(params: &[StageLType]) -> String {
    params
        .iter()
        .map(StageLType::display_name)
        .collect::<Vec<_>>()
        .join(", ")
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
        if self.value_candidates.contains_key(name)
            || self.routine_candidates.contains_key(name)
            || self.local_structs.contains_key(name)
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

    fn canonicalize_type(&self, ty: &StageLType, span: Span) -> Result<StageLType> {
        canonicalize_type_with_maps(
            ty,
            &self.local_structs,
            &self.dc_structs,
            &self.struct_defs,
            self.forbid_stagel_internal,
            &self.top_dc_names,
            span,
        )
    }

    fn check_and_canonicalize_type(
        &self,
        ty: &StageLType,
        is_pub_api: bool,
        span: Span,
    ) -> Result<StageLType> {
        let canon = self.canonicalize_type(ty, span)?;
        check_type_well_formed(&canon, &self.struct_defs, is_pub_api, span)?;
        Ok(canon)
    }

    fn type_supports_eq(&self, ty: &StageLType) -> bool {
        fn visit(
            ty: &StageLType,
            struct_defs: &BTreeMap<(Option<String>, String), StructDef>,
            visited: &mut HashSet<(Option<String>, String)>,
        ) -> bool {
            match ty {
                StageLType::B
                | StageLType::U8
                | StageLType::I32
                | StageLType::U32
                | StageLType::I64
                | StageLType::U64 => true,
                StageLType::Void | StageLType::Never | StageLType::Buf(_) => false,
                StageLType::List(inner) => visit(inner, struct_defs, visited),
                StageLType::Tuple(elems) => {
                    elems.iter().all(|e| visit(e, struct_defs, visited))
                }
                StageLType::Struct(StructTypeRef::Canonical {
                    namespace,
                    name,
                }) => {
                    let key = (namespace.clone(), name.clone());
                    if !visited.insert(key.clone()) {
                        return true;
                    }
                    struct_defs.get(&key).is_some_and(|sdef| {
                        sdef.fields.iter().all(|f| visit(f, struct_defs, visited))
                    })
                }
                StageLType::Struct(_) => false,
            }
        }
        let mut visited = HashSet::new();
        visit(ty, &self.struct_defs, &mut visited)
    }

    fn lookup_nested_routine(&self, name: &str) -> Option<usize> {
        for rs in self.routine_scopes.iter().rev() {
            if let Some(VerbTarget::Nested(nid)) = rs.get(name) {
                return Some(*nid);
            }
        }
        None
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

        // Check initialization order / acyclicity via DFS over value references.
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
            validator: &mut Validator<'a>,
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
            let mut state = AssignState::new();
            let mut ctx = CheckCtx {
                return_type: expected_ty.clone(),
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
                types_compatible(&actual, &expected_ty),
                "Constant '{}' declared type '{}' does not match initializer type '{}' at {}",
                v.name,
                expected_ty,
                actual,
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
            | Expr::ValueRef { .. }
            | Expr::Fault { .. }
            | Expr::Nop { .. } => Ok(()),
            Expr::List { elements, .. }
            | Expr::TupleNew { elements, .. }
            | Expr::StructNew {
                fields: elements, ..
            } => {
                for e in elements {
                    self.check_const_purity(e)?;
                }
                Ok(())
            }
            Expr::Field { base, .. } => self.check_const_purity(base),
            Expr::With { base, value, .. } => {
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
        let canon_ret = self.canonicalize_type(&r.return_type, r.span)?;
        let mut canon_params = Vec::with_capacity(r.params.len());
        for p in &r.params {
            canon_params.push(self.canonicalize_type(&p.ty, p.span)?);
        }

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
                for ((inp_expr, param), pty) in
                    tc.inputs.iter().zip(&r.params).zip(&canon_params)
                {
                    let mut st = AssignState::new();
                    let mut ctx = CheckCtx {
                        return_type: pty.clone(),
                        loop_depth: 0,
                        in_tail: false,
                        allow_buf: true,
                        outer_ids: HashSet::new(),
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
                        let mut st = AssignState::new();
                        let mut ctx = CheckCtx {
                            return_type: canon_ret.clone(),
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
                            types_compatible(&ty, &canon_ret),
                            "Test expected output type mismatch for '{}' at {} (expected '{}', got '{}')",
                            r.name,
                            exp_expr.span(),
                            canon_ret,
                            ty
                        );
                    }
                    TestExpected::Fault(_) => {}
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

        // Validate routine body.
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
            let pty = self.canonicalize_type(&p.ty, p.span)?;
            let id = self.alloc_binding_id();
            state.set_state(id, true);
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
        let mut ctx = CheckCtx {
            return_type: canon_ret.clone(),
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
        enclosing_state: &AssignState,
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
            let pty = self.canonicalize_type(&p.ty, p.span)?;
            let pid = self.alloc_binding_id();
            nested_state.set_state(pid, true);
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
        let mut ctx = CheckCtx {
            return_type: canon_ret.clone(),
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
        state: &mut AssignState,
        ctx: &mut CheckCtx,
    ) -> Result<StageLType> {
        let ty = self.check_expr_inner(expr, state, ctx)?;
        let sp = expr.span();
        self.expr_types.insert((sp.line, sp.col), ty.clone());
        Ok(ty)
    }

    fn check_expr_inner(
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
                let canon_elem = self.canonicalize_type(elem_ty, *span)?;
                let list_ty = StageLType::List(Box::new(canon_elem.clone()));
                check_type_well_formed(&list_ty, &self.struct_defs, false, *span)?;
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
                        types_compatible(&ety, &canon_elem),
                        "List element at {} has type '{}', expected '{}'",
                        el.span(),
                        ety,
                        canon_elem
                    );
                }
                ctx.captured_reads.extend(child_ctx.captured_reads);
                Ok(list_ty)
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
                let canon_elem = self.canonicalize_type(elem_ty, *span)?;
                let buf_ty = StageLType::Buf(Box::new(canon_elem));
                check_type_well_formed(&buf_ty, &self.struct_defs, false, *span)?;
                let cap_ty = self.check_subexpr(capacity, state, ctx)?;
                ensure!(
                    types_compatible(&cap_ty, &StageLType::U64),
                    "'bufNew' capacity at {} must have type 'u64', found '{}'",
                    capacity.span(),
                    cap_ty
                );
                Ok(buf_ty)
            }
            Expr::ValueRef { ty, target, span } => {
                let canon_expected = match ty {
                    Some(t) => {
                        Some(self.check_and_canonicalize_type(t, false, *span)?)
                    }
                    None => None,
                };
                match target {
                    SymbolTarget::Ident(name) => {
                        if let Some(entry) = self.lookup_local(name).cloned() {
                            if let Some(ref expected_ty) = canon_expected {
                                ensure!(
                                    &entry.ty == expected_ty,
                                    "Type annotation mismatch on '{name}' at {span}: declared '{}', referenced as '{expected_ty}'",
                                    entry.ty
                                );
                            }
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
                            self.value_ref_types
                                .insert((span.line, span.col), entry.ty.clone());
                            return Ok(entry.ty);
                        }
                        if let Some(cand) =
                            self.value_candidates.get(name).cloned()
                        {
                            if let Some(ref expected_ty) = canon_expected {
                                ensure!(
                                    &cand.ty == expected_ty,
                                    "Type annotation mismatch on constant '{name}' at {span}: declared '{}', referenced as '{expected_ty}'",
                                    cand.ty
                                );
                            }
                            let cty = cand.ty.clone();
                            self.resolved_values
                                .insert((span.line, span.col), cand);
                            self.value_ref_types
                                .insert((span.line, span.col), cty.clone());
                            return Ok(cty);
                        }
                        bail!("Undeclared identifier '{name}' at {span}");
                    }
                    SymbolTarget::Dc(dc) => {
                        let sh = format_dc_shorthand(*dc);
                        if self.forbid_stagel_internal
                            && let Some(orig) = self.top_dc_names.get(dc)
                            && is_stagel_internal_name(orig)
                        {
                            bail!(
                                "StageL-internal symbol '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
                            );
                        }
                        if let Some(v_ty) = self.top_dc_values.get(dc) {
                            if let Some(ref expected_ty) = canon_expected {
                                ensure!(
                                    v_ty == expected_ty,
                                    "Type mismatch on Dc reference '{sh}' at {span}"
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
                            if self.forbid_stagel_internal
                                && is_stagel_internal_name(original_name)
                            {
                                bail!(
                                    "StageL-internal symbol '{original_name}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
                                );
                            }
                            let canon_imp =
                                self.canonicalize_type(imp_ty, *span)?;
                            if let Some(ref expected_ty) = canon_expected {
                                ensure!(
                                    &canon_imp == expected_ty,
                                    "Type mismatch on imported Dc reference '{sh}' at {span}"
                                );
                            }
                            self.value_ref_types
                                .insert((span.line, span.col), canon_imp.clone());
                            return Ok(canon_imp);
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
                if self.forbid_stagel_internal && is_stagel_internal_name(name)
                {
                    bail!(
                        "StageL-internal variable '{name}' cannot be declared in Stage 2/StageR module at {span}"
                    );
                }
                ensure!(
                    *ty != StageLType::Void && *ty != StageLType::Never,
                    "'let' variable '{name}' at {span} cannot have type '{ty}'"
                );
                let canon_ty = self.check_and_canonicalize_type(ty, false, *span)?;
                ensure!(
                    !self.name_exists_in_local_or_top(name),
                    "Cannot re-declare or shadow '{name}' with 'let' at {span} (use 'set' to mutate)"
                );
                let is_init = if let Some(init_expr) = init {
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
                state.set_state(id, is_init);
                let scope = self.local_scopes.last_mut().context(
                    "'let' is only allowed inside routine/code blocks",
                )?;
                scope.insert(
                    name.clone(),
                    BindingEntry {
                        id,
                        ty: canon_ty,
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
                if self.forbid_stagel_internal && is_stagel_internal_name(name)
                {
                    bail!(
                        "StageL-internal constant '{name}' cannot be declared in Stage 2/StageR module at {span}"
                    );
                }
                ensure!(
                    *ty != StageLType::Void && *ty != StageLType::Never,
                    "'const' binding '{name}' at {span} cannot have type '{ty}'"
                );
                let canon_ty = self.check_and_canonicalize_type(ty, false, *span)?;
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
                state.set_state(id, true);
                let scope = self.local_scopes.last_mut().context(
                    "Local 'const' requires an enclosing block",
                )?;
                scope.insert(
                    name.clone(),
                    BindingEntry {
                        id,
                        ty: canon_ty,
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
                let canon_expected = match ty {
                    Some(t) => {
                        Some(self.check_and_canonicalize_type(t, false, *span)?)
                    }
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
                    matches!(entry.kind, BindingKind::MutableLet),
                    "Cannot 'set' immutable binding '{name}' ({:?}) at {span}",
                    entry.kind
                );
                if let Some(ref expected_ty) = canon_expected {
                    ensure!(
                        &entry.ty == expected_ty,
                        "Type mismatch in 'set {expected_ty}/{name}' at {span}: declared type is '{}' ('set' cannot change an existing variable's type)",
                        entry.ty
                    );
                }
                let rhs_ty = self.check_subexpr(value, state, ctx)?;
                ensure!(
                    types_compatible(&rhs_ty, &entry.ty),
                    "Assigned value type mismatch in 'set' on '{name}' at {span}: expected '{}', found '{rhs_ty}'",
                    entry.ty
                );
                state.set_state(entry.id, true);
                ctx.always_written.insert(entry.id);
                Ok(StageLType::Void)
            }
            Expr::RoutineDecl(r) => {
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
                    !r.is_pub && !r.pub_dc_syntax && r.dc_id.is_none(),
                    "Nested routine '{}' at {} cannot carry 'pub' or Dc annotations",
                    r.name,
                    r.span
                );
                let nested_id = self.lookup_nested_routine(&r.name).with_context(|| {
                    format!(
                        "Nested routine '{}' at {} missing from enclosing block signature table",
                        r.name,
                        r.span
                    )
                })?;
                self.validate_nested_routine_at_decl(nested_id, r, state)?;
                Ok(StageLType::Void)
            }
            Expr::Call { target, args, span } => match target {
                SymbolTarget::Ident(name) => {
                    if let Some(nid) = self.lookup_nested_routine(name) {
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
                        let sig = summary.sig;
                        ensure!(
                            args.len() == sig.params.len(),
                            "Wrong argument count in call at {span}: expected {}, got {}",
                            sig.params.len(),
                            args.len()
                        );
                        for (arg_expr, param_ty) in args.iter().zip(&sig.params)
                        {
                            let aty =
                                self.check_subexpr(arg_expr, state, ctx)?;
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
                        .with_context(|| {
                            format!("Unknown routine '{name}' at {span}")
                        })?;
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
                            let aty =
                                self.check_subexpr(arg_expr, state, ctx)?;
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
                            .map(|c| {
                                format!("({})", format_param_types(&c.sig.params))
                            })
                            .collect();
                        bail!(
                            "No matching overload for routine '{name}' at {span} with argument types ({}) (Call argument type mismatch; available signatures: {})",
                            format_param_types(&arg_tys),
                            avail.join(", ")
                        );
                    }
                    ensure!(
                        matches.len() == 1,
                        "Ambiguous call to overloaded routine '{name}' at {span} with argument types ({}): matches {} overloads",
                        format_param_types(&arg_tys),
                        matches.len()
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
                    if self.forbid_stagel_internal
                        && let Some(orig) = self.top_dc_names.get(dc)
                        && is_stagel_internal_name(orig)
                    {
                        bail!(
                            "StageL-internal routine '{orig}' ('dc/{sh}') cannot be called in Stage 2/StageR module at {span}"
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
                        if self.forbid_stagel_internal
                            && is_stagel_internal_name(original_name)
                        {
                            bail!(
                                "StageL-internal routine '{original_name}' ('dc/{sh}') cannot be called in Stage 2/StageR module at {span}"
                            );
                        }
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
            },
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
                        let canon_ret =
                            self.check_and_canonicalize_type(&r.return_type, false, r.span)?;
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
                            let canon_pty =
                                self.check_and_canonicalize_type(&p.ty, false, p.span)?;
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
                let canon_elem =
                    self.check_and_canonicalize_type(elem_ty, false, *span)?;
                ensure!(
                    type_can_be_element_with_defs(&canon_elem, &self.struct_defs),
                    "'foreach' element type '{}' at {span} is invalid",
                    canon_elem
                );
                ensure!(
                    !self.name_exists_in_local_or_top(elem_name),
                    "'foreach' element binding '{elem_name}' at {span} shadows an existing declaration"
                );
                let lty = self.check_subexpr(list, state, ctx)?;
                let expected_list =
                    StageLType::List(Box::new(canon_elem.clone()));
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
                        ty: canon_elem,
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
            Expr::Nop { .. } => Ok(StageLType::Void),
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
                let canon_target =
                    self.check_and_canonicalize_type(target_ty, false, *span)?;
                ensure!(
                    canon_target.is_integer(),
                    "'conv' target type must be integer, found '{canon_target}' at {span}"
                );
                let src_ty = self.check_subexpr(expr, state, ctx)?;
                ensure!(
                    src_ty == StageLType::Never || src_ty.is_integer(),
                    "'conv' source operand must be integer, found '{src_ty}' at {span}"
                );
                Ok(canon_target)
            }
            Expr::Bitcast {
                expr,
                target_ty,
                span,
            } => {
                let canon_target =
                    self.check_and_canonicalize_type(target_ty, false, *span)?;
                let dst_width = canon_target.bit_width().with_context(|| {
                    format!(
                        "'bitcast' target type must be integer, found '{canon_target}' at {span}"
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
                        "'bitcast' requires matching bit widths ('{src_ty}' is {src_width}-bit, '{canon_target}' is {dst_width}-bit) at {span}"
                    );
                }
                Ok(canon_target)
            }
            Expr::TupleNew { elements, .. } => {
                let mut slot_tys = Vec::with_capacity(elements.len());
                let mut any_never = false;
                for el in elements {
                    let ety = self.check_subexpr(el, state, ctx)?;
                    ensure!(
                        ety != StageLType::Void,
                        "Tuple element at {} cannot have type 'void'",
                        el.span()
                    );
                    if ety == StageLType::Never {
                        any_never = true;
                    }
                    slot_tys.push(ety);
                }
                if any_never {
                    Ok(StageLType::Never)
                } else {
                    Ok(StageLType::Tuple(slot_tys))
                }
            }
            Expr::StructNew {
                target,
                fields,
                span,
            } => {
                let canon_key = match target {
                    SymbolTarget::Ident(name) => {
                        if self.forbid_stagel_internal
                            && is_stagel_internal_name(name)
                        {
                            bail!(
                                "StageL-internal struct '{name}' cannot be constructed in Stage 2/StageR module at {span}"
                            );
                        }
                        let Some(k) = self.local_structs.get(name).cloned()
                        else {
                            bail!(
                                "Unknown positional struct '{name}' in 'new' at {span}"
                            );
                        };
                        if self.forbid_stagel_internal
                            && is_stagel_internal_name(&k.1)
                        {
                            bail!(
                                "StageL-internal struct '{}' cannot be constructed in Stage 2/StageR module at {span}",
                                k.1
                            );
                        }
                        k
                    }
                    SymbolTarget::Dc(dc) => {
                        let sh = format_dc_shorthand(*dc);
                        if self.forbid_stagel_internal
                            && let Some(orig) = self.top_dc_names.get(dc)
                            && is_stagel_internal_name(orig)
                        {
                            bail!(
                                "StageL-internal struct '{orig}' ('dc/{sh}') cannot be constructed in Stage 2/StageR module at {span}"
                            );
                        }
                        let Some(k) = self.dc_structs.get(dc).cloned() else {
                            bail!(
                                "Unknown positional struct Dc reference 'dc/{sh}' in 'new' at {span}"
                            );
                        };
                        if self.forbid_stagel_internal
                            && is_stagel_internal_name(&k.1)
                        {
                            bail!(
                                "StageL-internal struct '{}' ('dc/{sh}') cannot be constructed in Stage 2/StageR module at {span}",
                                k.1
                            );
                        }
                        k
                    }
                };
                let sdef = self
                    .struct_defs
                    .get(&canon_key)
                    .cloned()
                    .with_context(|| {
                        format!(
                            "Unresolved struct definition '{}' at {span}",
                            canon_key.1
                        )
                    })?;
                let struct_ty = StageLType::Struct(StructTypeRef::Canonical {
                    namespace: sdef.namespace.clone(),
                    name: sdef.original_name.clone(),
                });
                if !ctx.allow_buf
                    && type_contains_buf_with_defs(&struct_ty, &self.struct_defs)
                {
                    bail!(
                        "Struct '{}' containing 'buf/T' cannot be constructed in constant context at {span}",
                        sdef.original_name
                    );
                }
                ensure!(
                    fields.len() == sdef.fields.len(),
                    "Wrong field count in 'new {}' at {span}: expected {}, got {}",
                    sdef.original_name,
                    sdef.fields.len(),
                    fields.len()
                );
                for (f_expr, expected_ty) in fields.iter().zip(&sdef.fields) {
                    let fty = self.check_subexpr(f_expr, state, ctx)?;
                    ensure!(
                        types_compatible(&fty, expected_ty),
                        "Struct field type mismatch in 'new {}' at {}: expected '{expected_ty}', found '{fty}'",
                        sdef.original_name,
                        f_expr.span()
                    );
                }
                self.resolved_structs.insert((span.line, span.col), sdef);
                Ok(struct_ty)
            }
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
                    StageLType::Struct(StructTypeRef::Canonical {
                        ref namespace,
                        ref name,
                    }) => {
                        let key = (namespace.clone(), name.clone());
                        let sdef =
                            self.struct_defs.get(&key).with_context(|| {
                                format!(
                                    "Unresolved struct '{name}' in 'field' at {span}"
                                )
                            })?;
                        let slot_ty = sdef
                            .fields
                            .get(idx_usize)
                            .cloned()
                            .with_context(|| {
                                format!(
                                    "Struct field index {index} out of bounds for '{name}' ({} slot(s)) at {span}",
                                    sdef.fields.len()
                                )
                            })?;
                        Ok(slot_ty)
                    }
                    other => bail!(
                        "'field' requires tuple or positional struct operand, found '{other}' at {span}"
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
                    StageLType::Struct(StructTypeRef::Canonical {
                        namespace,
                        name,
                    }) => {
                        let key = (namespace.clone(), name.clone());
                        let sdef =
                            self.struct_defs.get(&key).with_context(|| {
                                format!(
                                    "Unresolved struct '{name}' in 'with' at {span}"
                                )
                            })?;
                        let slot_ty =
                            sdef.fields.get(idx_usize).with_context(|| {
                                format!(
                                    "Struct field index {index} out of bounds in 'with' for '{name}' ({} slot(s)) at {span}",
                                    sdef.fields.len()
                                )
                            })?;
                        ensure!(
                            types_compatible(&vty, slot_ty),
                            "'with' field {index} type mismatch on '{name}' at {}: expected '{slot_ty}', found '{vty}'",
                            value.span()
                        );
                        Ok(bty)
                    }
                    other => bail!(
                        "'with' requires tuple or positional struct operand, found '{other}' at {span}"
                    ),
                }
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
                    type_can_be_element_with_defs(&vty, &self.struct_defs),
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

fn collect_value_deps(
    expr: &Expr,
    dc_to_name: &HashMap<DcChar, String>,
    out: &mut Vec<String>,
) {
    match expr {
        Expr::ValueRef {
            target: SymbolTarget::Ident(n),
            ..
        } => out.push(n.clone()),
        Expr::ValueRef {
            target: SymbolTarget::Dc(dc),
            ..
        } => {
            if let Some(n) = dc_to_name.get(dc) {
                out.push(n.clone());
            }
        }
        Expr::List { elements, .. }
        | Expr::TupleNew { elements, .. }
        | Expr::StructNew {
            fields: elements, ..
        } => {
            for e in elements {
                collect_value_deps(e, dc_to_name, out);
            }
        }
        Expr::Field { base, .. } => {
            collect_value_deps(base, dc_to_name, out);
        }
        Expr::With { base, value, .. } => {
            collect_value_deps(base, dc_to_name, out);
            collect_value_deps(value, dc_to_name, out);
        }
        Expr::Unary { operand, .. } => {
            collect_value_deps(operand, dc_to_name, out);
        }
        Expr::Binary { left, right, .. } => {
            collect_value_deps(left, dc_to_name, out);
            collect_value_deps(right, dc_to_name, out);
        }
        Expr::Ternary {
            first,
            second,
            third,
            ..
        } => {
            collect_value_deps(first, dc_to_name, out);
            collect_value_deps(second, dc_to_name, out);
            collect_value_deps(third, dc_to_name, out);
        }
        Expr::Conv { expr, .. } | Expr::Bitcast { expr, .. } => {
            collect_value_deps(expr, dc_to_name, out);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            collect_value_deps(cond, dc_to_name, out);
            collect_value_deps(then_branch, dc_to_name, out);
            collect_value_deps(else_branch, dc_to_name, out);
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
    const v/u32/readX :: u32/x.
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
    const v/u32/readX :: u32/x.
    u32:0.
"#;
        validate_module(&parse_module(capture_never_init).unwrap())
            .expect_err("Capturing uninitialized outer variable fails");

        // Nested verb capture: called AFTER set succeeds!
        let capture_after_set = r#"
v/u32/f
    let u32/x.
    const v/u32/readX :: u32/x.
    set u32/x u32:9.
    readX.
"#;
        validate_module(&parse_module(capture_after_set).unwrap())
            .expect("Calling nested verb after set should validate");

        // Nested verb initializes outer variable via set, then outer reads after calling it -> OK!
        let capture_init_inside_nested = r#"
v/u32/f
    let u32/x.
    const v/void/initX :: set u32/x u32:12.
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
        const v/void/stopper :: break.
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
        assert!(vm.tail_calls.contains(&(4, 5)));
    }

    #[crate::ctb_test]
    fn test_parameter_type_dispatch_and_typed_constants_validation() {
        // 1. Positive: intra-module + cross-module `use` of overloaded routines,
        // single-definition camelCase constants, optional types on `/name` and
        // `set /name`, `nop`, and unambiguous single-definition `never` arg.
        let lib_src = r#"
ns demo/math.

pub u8/u8Unit u8:1.
pub i32/i32Unit i32:1.
pub i64/i64Unit i64:1.

pub v/i32/stepAdd i32/x i32/y :: add /x /y.
pub v/i64/stepAdd i64/x i64/y :: add /x /y.
pub v/l/u8/stepAdd l/u8/prefix u64/take :: slice /prefix u64:0 /take.
pub v/i32/solo i32/x :: /x.
pub v/i32/modStageLInternal i32/x i32/y :: rem /x /y.
"#;
        let caller_src = r#"
ns demo/caller.
use demo/math: u8Unit i32Unit i64Unit stepAdd solo modStageLInternal ::.

v/u8/stepAdd u8/x u8/y :: add /x /y.

pub v/b/checkAll b/abortEarly
    if /abortEarly
        eq solo fault i32:0
        do
            let i32/acc i32:0.
            set /acc modStageLInternal i32:41 i32:42.
            if eq /acc i32:41 nop fault.
            const u8/res8 stepAdd u8:2 /u8Unit.
            const i32/res32 stepAdd i32:40 /i32Unit.
            const i64/res64 stepAdd i64:99 /i64Unit.
            const l/u8/resBytes stepAdd b"abcdef" u64:3.
            and eq /res8 u8:3 and eq /res32 i32:41 and eq /res64 i64:100 eq /resBytes b"abc".
        ::.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("stage_1_impls/demo/caller.sl3"),
                source: caller_src,
            },
        ])
        .expect("valid overloaded workspace should pass");
        assert_eq!(ws.modules.len(), 2);

        // 1b. `*StageLInternal` import or declaration is rejected in `impls/` or `.sr3`.
        let stage2_bad_import = r#"
ns app/user.
use demo/math: modStageLInternal as safeMod ::.

pub v/i32/run i32/x :: safeMod /x i32:2.
"#;
        let err = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("impls/app/user.sl3"),
                source: stage2_bad_import,
            },
        ])
        .expect_err("*StageLInternal import in impls/ must fail");
        assert!(
            err.to_string().contains("StageL-internal"),
            "Unexpected error: {err}"
        );

        let sr3_bad_decl = r#"
v/i32/helperStageLInternal i32/x :: /x.
"#;
        let err = parse_and_validate_workspace(&[ModuleInput {
            source_path: Some("foo.sr3"),
            source: sr3_bad_decl,
        }])
        .expect_err("*StageLInternal declaration in .sr3 must fail");
        assert!(
            err.to_string().contains("StageL-internal"),
            "Unexpected error: {err}"
        );

        // 2. Return-type-only routine overload is rejected.
        let ret_only_src = r#"
v/i32/foo i32/x :: /x.
v/i64/foo i32/x :: conv /x i64.
"#;
        let err = validate_module(&parse_module(ret_only_src).unwrap())
            .expect_err("return-type-only overload must fail");
        assert!(
            err.to_string()
                .contains("return-type-only overloading is disallowed"),
            "Unexpected error: {err}"
        );

        // 3. Mismatched-arity intra-module routine overload is rejected.
        let arity_mismatch_src = r#"
v/i32/foo i32/x :: /x.
v/i32/foo i32/x i32/y :: add /x /y.
"#;
        let err = parse_module(arity_mismatch_src)
            .and_then(|m| validate_module(&m))
            .expect_err("mismatched-arity overload must fail");
        assert!(
            err.to_string().contains("parameter count")
                || err.to_string().contains("arity"),
            "Unexpected error: {err}"
        );

        // 4. Mismatched-arity across `use` import + local routine is rejected.
        let bad_import_arity = r#"
ns demo/bad_arity.
use demo/math: solo ::.

v/i64/solo i64/x i64/y :: add /x /y.
"#;
        let err = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("stage_1_impls/demo/math.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("stage_1_impls/demo/bad_arity.sl3"),
                source: bad_import_arity,
            },
        ])
        .expect_err("imported vs local arity mismatch must fail");
        assert!(
            err.to_string().contains("parameter count")
                || err.to_string().contains("arity"),
            "Unexpected error: {err}"
        );

        // 5. Ambiguous `never`-argument call across multiple overloads is rejected.
        let ambiguous_never_src = r#"
v/i32/pick i32/x i32/y :: add /x /y.
v/i32/pick u32/x i32/y :: /y.
v/i32/caller :: pick fault i32:1.
"#;
        let err = validate_module(&parse_module(ambiguous_never_src).unwrap())
            .expect_err("ambiguous never argument must fail");
        assert!(
            err.to_string().contains("Ambiguous"),
            "Unexpected error: {err}"
        );

        // 6. Unmatched concrete parameter tuple is rejected.
        let no_match_src = r#"
v/i32/pair i32/x i32/y :: add /x /y.
v/i64/pair i64/x i64/y :: add /x /y.
v/i64/caller :: pair i32:1 i64:2.
"#;
        let err = validate_module(&parse_module(no_match_src).unwrap())
            .expect_err("unmatched overload parameter tuple must fail");
        assert!(
            err.to_string().contains("No matching overload"),
            "Unexpected error: {err}"
        );

        // 7. Duplicate constant name is rejected even when declared with distinct types.
        let dup_const_src = r#"
const i32/K i32:1.
const i64/K i64:2.
"#;
        let err = validate_module(&parse_module(dup_const_src).unwrap())
            .expect_err("duplicate constant name must fail even with distinct types");
        assert!(
            err.to_string().contains("duplicates/shadows")
                || err.to_string().contains("Duplicate"),
            "Unexpected error: {err}"
        );

        // 8. Same identifier for both a constant and a routine is rejected.
        let const_vs_routine_src = r#"
const i32/clash i32:7.
v/i32/clash i32/x :: i32/x.
"#;
        let err = parse_module(const_vs_routine_src)
            .and_then(|m| validate_module(&m))
            .expect_err("constant vs routine name collision must fail");
        assert!(
            err.to_string().contains("constant")
                || err.to_string().contains("duplicates/shadows")
                || err.to_string().contains("Duplicate")
                || err.to_string().contains("conflicts"),
            "Unexpected error: {err}"
        );

        // 9. Nested lexical routine overloading remains prohibited.
        let nested_overload_src = r#"
v/i32/outer i32/x
    const v/i32/inner i32/a :: i32/a.
    const v/i64/inner i64/a :: i64/a.
    inner i32/x.
"#;
        let err = parse_module(nested_overload_src)
            .and_then(|m| validate_module(&m))
            .expect_err("nested lexical routine overloading must fail");
        assert!(
            err.to_string().contains("Duplicate")
                || err.to_string().contains("re-declare"),
            "Unexpected error: {err}"
        );
    }

    #[crate::ctb_test]
    fn test_tuples_and_positional_structs_validation() {
        // 1. Valid: 0/1/3-ary tuples, recursive `l/node`, coinductive `eq`/`ne`, cross-module alias + overloading.
        let lib_src = r#"ns geo/core.
pub struct point :: i32 i32.
pub struct tree :: point l/tree.
pub point/ORIGIN structNew point: i32:0 i32:0 ::.
pub v/i32/norm point/p :: add field /p u32:0 field /p u32:1.
pub v/i32/norm i32/x :: /x.
"#;
        let app_src = r#"ns geo/app.
use geo/core: point as pt tree ORIGIN norm ::.

pub v/b/check pt/a
    const tup: ::/u tupNew: ::.
    const tup: pt b ::/pair tupNew: with /a u32:0 i32:9 true ::.
    const tree/t1 structNew tree: field /pair u32:0 l/tree: :: ::.
    const tree/t2 structNew tree: structNew pt: i32:9 field /a u32:1 :: l/tree: :: ::.
    and eq /u tupNew: :: and eq /t1 /t2 eq norm /ORIGIN norm i32:0.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("geo/core.sl3"),
                source: lib_src,
            },
            ModuleInput {
                source_path: Some("geo/app.sl3"),
                source: app_src,
            },
        ])
        .expect("valid tuple + recursive struct workspace should validate");
        assert_eq!(ws.modules.len(), 2);

        // 2. Unguarded direct and tuple-mediated cycles must fail.
        for bad_cycle in [
            "struct bad :: bad.",
            "struct a :: tup: i32 bStruct ::.\nstruct bStruct :: a.",
        ] {
            let err = validate_module(&parse_module(bad_cycle).unwrap())
                .expect_err("unguarded struct cycle must fail");
            assert!(
                err.to_string().contains("cycle"),
                "Unexpected error: {err}"
            );
        }

        // 3. Nominal distinction: two structs with identical slots are distinct types.
        let nominal_mismatch = r#"
struct p1 :: i32 i32.
struct p2 :: i32 i32.
v/b/cmp :: eq structNew p1: i32:1 i32:2 :: structNew p2: i32:1 i32:2 ::.
"#;
        assert!(
            validate_module(&parse_module(nominal_mismatch).unwrap()).is_err()
        );

        // 4. Transitive `buf/T` prohibitions (`pub struct`, `l/holder`, `eq`, `const`).
        for bad_buf in [
            "pub struct holder :: buf/u8.",
            "struct holder :: buf/u8.\nv/u64/f :: len l/holder: ::.",
            "struct holder :: buf/u8.\nv/b/f :: eq structNew holder: bufNew u8 u64:0 :: structNew holder: bufNew u8 u64:0 ::.",
            "struct holder :: buf/u8.\nconst holder/H structNew holder: bufNew u8 u64:0 ::.",
        ] {
            assert!(
                validate_module(&parse_module(bad_buf).unwrap()).is_err(),
                "Expected error for: {bad_buf}"
            );
        }

        // 5. Out-of-bounds field index and `with` type mismatch must fail.
        for bad_field in [
            "v/i32/f :: field tupNew: i32:1 i32:2 :: u32:2.",
            "struct pt :: i32 i32.\nv/pt/f :: with structNew pt: i32:1 i32:2 :: u32:1 true.",
        ] {
            assert!(
                validate_module(&parse_module(bad_field).unwrap()).is_err(),
                "Expected error for: {bad_field}"
            );
        }
    }
}
