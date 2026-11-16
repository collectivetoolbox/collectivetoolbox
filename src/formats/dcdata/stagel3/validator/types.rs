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

//! Dialect-parameterized type canonicalization, linear buffer shape
//! verification, unguarded value-size cycle detection, equality support, and
//! total widening conversion predicates.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    Span, StageDialect, StageLType, StructTypeRef, format_dc_shorthand,
};
use super::workspace::is_stagel_internal_name;
use super::{BrandTypeDef, EnumDef, NamedStructDef, StructDef};
use crate::{DcChar, DcShorthand};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Policy controlling where `buf/T` is permitted when checking type shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BufPolicy {
    /// StageL 3 private local/routine/struct context: `buf/T` allowed
    /// directly, inside tuples, and inside private positional structs, while
    /// prohibited inside `l/...` and `buf/buf/...`.
    LocalAllowed,
    /// Public StageL 3 API / module `const` position, or any non-`let` /
    /// nested position in StageR 3: `buf/T` is disallowed anywhere.
    Disallowed,
    /// StageR 3 local `let buf/T/name` or direct `/name` buffer annotation: a
    /// top-level `buf/T` is allowed provided `T` itself forbids `buf/T`.
    StageRDirectBufOnly,
}

/// Borrowed canonical definition tables for type-shape, cycle, and equality
/// checks.
#[derive(Clone, Copy)]
pub(super) struct TypeDefs<'a> {
    pub(super) struct_defs: &'a BTreeMap<(Option<String>, String), StructDef>,
    pub(super) brand_types:
        &'a BTreeMap<(Option<String>, String), BrandTypeDef>,
    pub(super) named_structs:
        &'a BTreeMap<(Option<String>, String), NamedStructDef>,
    pub(super) enums: &'a BTreeMap<(Option<String>, String), EnumDef>,
}

/// Full symbol-resolution environment used to canonicalize syntactic types.
#[derive(Clone, Copy)]
pub(super) struct TypeEnv<'a> {
    pub(super) dialect: StageDialect,
    pub(super) forbid_stagel_internal: bool,
    pub(super) local_structs: &'a HashMap<String, (Option<String>, String)>,
    pub(super) dc_structs: &'a HashMap<DcChar, (Option<String>, String)>,
    pub(super) local_nominals: &'a HashMap<String, (Option<String>, String)>,
    pub(super) dc_nominals: &'a HashMap<DcChar, (Option<String>, String)>,
    pub(super) defs: TypeDefs<'a>,
    pub(super) top_dc_names: &'a HashMap<DcChar, String>,
}

fn canonicalize_struct_ident(
    name: &str,
    env: &TypeEnv<'_>,
    span: Span,
) -> Result<StageLType> {
    if env.forbid_stagel_internal && is_stagel_internal_name(name) {
        bail!(
            "StageL-internal type '{name}' cannot be referenced in Stage 2/StageR module at {span}"
        );
    }
    if let Some((ns, orig)) = env.local_structs.get(name) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal struct '{orig}' cannot be referenced in Stage 2/StageR module at {span}"
            );
        }
        return Ok(StageLType::Struct(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    if env.dialect == StageDialect::StageR3
        && let Some((ns, orig)) = env.local_nominals.get(name)
    {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal type '{orig}' cannot be referenced in Stage 2/StageR module at {span}"
            );
        }
        return Ok(StageLType::Nominal(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    match env.dialect {
        StageDialect::StageL3 => {
            bail!("Unknown StageL struct type '{name}' at {span}")
        }
        StageDialect::StageR3 => {
            bail!("Unknown StageR struct/nominal type '{name}' at {span}")
        }
    }
}

fn canonicalize_struct_dc(
    dc: DcChar,
    env: &TypeEnv<'_>,
    span: Span,
) -> Result<StageLType> {
    let sh = format_dc_shorthand(dc);
    if env.forbid_stagel_internal
        && let Some(orig) = env.top_dc_names.get(&dc)
        && is_stagel_internal_name(orig)
    {
        bail!(
            "StageL-internal struct '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
        );
    }
    if let Some((ns, orig)) = env.dc_structs.get(&dc) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal struct '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
            );
        }
        return Ok(StageLType::Struct(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    if env.dialect == StageDialect::StageR3
        && let Some((ns, orig)) = env.dc_nominals.get(&dc)
    {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal type '{orig}' ('dc/{sh}') cannot be referenced in Stage 2/StageR module at {span}"
            );
        }
        return Ok(StageLType::Nominal(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    match env.dialect {
        StageDialect::StageL3 => {
            bail!("Unknown StageL struct Dc reference 'dc/{sh}' at {span}")
        }
        StageDialect::StageR3 => bail!(
            "Unknown StageR struct/nominal Dc reference 'dc/{sh}' at {span}"
        ),
    }
}

fn canonicalize_nominal_ident(
    name: &str,
    env: &TypeEnv<'_>,
    span: Span,
) -> Result<StageLType> {
    if env.forbid_stagel_internal && is_stagel_internal_name(name) {
        bail!(
            "StageL-internal type '{name}' is forbidden in StageR 3 at {span}"
        );
    }
    if let Some((ns, orig)) = env.local_nominals.get(name) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal type '{orig}' is forbidden in StageR 3 at {span}"
            );
        }
        return Ok(StageLType::Nominal(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    if let Some((ns, orig)) = env.local_structs.get(name) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal struct '{orig}' is forbidden in StageR 3 at {span}"
            );
        }
        return Ok(StageLType::Struct(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    bail!("Unknown StageR nominal type '{name}' at {span}")
}

fn canonicalize_nominal_dc(
    dc: DcChar,
    env: &TypeEnv<'_>,
    span: Span,
) -> Result<StageLType> {
    let sh = format_dc_shorthand(dc);
    if env.forbid_stagel_internal
        && let Some(orig) = env.top_dc_names.get(&dc)
        && is_stagel_internal_name(orig)
    {
        bail!(
            "StageL-internal type '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
        );
    }
    if let Some((ns, orig)) = env.dc_nominals.get(&dc) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal type '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
            );
        }
        return Ok(StageLType::Nominal(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    if let Some((ns, orig)) = env.dc_structs.get(&dc) {
        if env.forbid_stagel_internal && is_stagel_internal_name(orig) {
            bail!(
                "StageL-internal struct '{orig}' ('dc/{sh}') is forbidden in StageR 3 at {span}"
            );
        }
        return Ok(StageLType::Struct(StructTypeRef::Canonical {
            namespace: ns.clone(),
            name: orig.clone(),
        }));
    }
    bail!("Unknown StageR nominal Dc reference 'dc/{sh}' at {span}")
}

/// Canonicalizes syntactic struct and nominal references inside `ty` according
/// to `env.dialect`.
pub(super) fn canonicalize_type(
    ty: &StageLType,
    env: &TypeEnv<'_>,
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
        StageLType::I128
        | StageLType::U128
        | StageLType::Int
        | StageLType::Rat
        | StageLType::Dc
        | StageLType::Dcs
        | StageLType::S
        | StageLType::Error
        | StageLType::View
        | StageLType::Cursor => {
            ensure!(
                env.dialect == StageDialect::StageR3,
                "StageR 3 type '{ty}' is not valid in StageL 3 validator at {span}"
            );
            Ok(ty.clone())
        }
        StageLType::List(inner) => Ok(StageLType::List(Box::new(
            canonicalize_type(inner, env, span)?,
        ))),
        StageLType::Buf(inner) => Ok(StageLType::Buf(Box::new(
            canonicalize_type(inner, env, span)?,
        ))),
        StageLType::Option(inner) => {
            ensure!(
                env.dialect == StageDialect::StageR3,
                "StageR 3 type '{ty}' is not valid in StageL 3 validator at {span}"
            );
            Ok(StageLType::Option(Box::new(canonicalize_type(
                inner, env, span,
            )?)))
        }
        StageLType::Result(inner) => {
            ensure!(
                env.dialect == StageDialect::StageR3,
                "StageR 3 type '{ty}' is not valid in StageL 3 validator at {span}"
            );
            Ok(StageLType::Result(Box::new(canonicalize_type(
                inner, env, span,
            )?)))
        }
        StageLType::Tuple(elems) => {
            let mut out = Vec::with_capacity(elems.len());
            for el in elems {
                out.push(canonicalize_type(el, env, span)?);
            }
            Ok(StageLType::Tuple(out))
        }
        StageLType::Struct(StructTypeRef::Ident(name)) => {
            canonicalize_struct_ident(name, env, span)
        }
        StageLType::Struct(StructTypeRef::Dc(dc)) => {
            canonicalize_struct_dc(*dc, env, span)
        }
        StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
            let key = (namespace.clone(), name.clone());
            match env.dialect {
                StageDialect::StageL3 => {
                    ensure!(
                        env.defs.struct_defs.contains_key(&key),
                        "Unresolved canonical struct '{name}' at {span}"
                    );
                    Ok(ty.clone())
                }
                StageDialect::StageR3 => {
                    if env.defs.named_structs.contains_key(&key)
                        || env.defs.struct_defs.contains_key(&key)
                    {
                        Ok(StageLType::Struct(StructTypeRef::Canonical {
                            namespace: namespace.clone(),
                            name: name.clone(),
                        }))
                    } else if env.defs.brand_types.contains_key(&key)
                        || env.defs.enums.contains_key(&key)
                    {
                        Ok(StageLType::Nominal(StructTypeRef::Canonical {
                            namespace: namespace.clone(),
                            name: name.clone(),
                        }))
                    } else {
                        bail!(
                            "Unknown canonical struct/nominal '{name}' at {span}"
                        );
                    }
                }
            }
        }
        StageLType::Nominal(nref) => {
            ensure!(
                env.dialect == StageDialect::StageR3,
                "StageR 3 type '{ty}' is not valid in StageL 3 validator at {span}"
            );
            match nref {
                StructTypeRef::Ident(name) => {
                    canonicalize_nominal_ident(name, env, span)
                }
                StructTypeRef::Dc(dc) => {
                    canonicalize_nominal_dc(*dc, env, span)
                }
                StructTypeRef::Canonical { namespace, name } => {
                    let key = (namespace.clone(), name.clone());
                    if env.defs.brand_types.contains_key(&key)
                        || env.defs.enums.contains_key(&key)
                    {
                        Ok(StageLType::Nominal(StructTypeRef::Canonical {
                            namespace: namespace.clone(),
                            name: name.clone(),
                        }))
                    } else if env.defs.named_structs.contains_key(&key)
                        || env.defs.struct_defs.contains_key(&key)
                    {
                        Ok(StageLType::Struct(StructTypeRef::Canonical {
                            namespace: namespace.clone(),
                            name: name.clone(),
                        }))
                    } else {
                        bail!("Unknown canonical nominal '{name}' at {span}");
                    }
                }
            }
        }
    }
}

/// Returns `true` if canonical type `ty` is `buf/T` or transitively holds one
/// inside tuples, lists, options, results, or struct/nominal fields.
pub(super) fn type_contains_buf_with_defs(
    ty: &StageLType,
    defs: &TypeDefs<'_>,
) -> bool {
    fn visit(
        ty: &StageLType,
        defs: &TypeDefs<'_>,
        visited: &mut HashSet<(Option<String>, String)>,
    ) -> bool {
        match ty {
            StageLType::Buf(_) => true,
            StageLType::List(inner)
            | StageLType::Option(inner)
            | StageLType::Result(inner) => visit(inner, defs, visited),
            StageLType::Tuple(elems) => {
                elems.iter().any(|e| visit(e, defs, visited))
            }
            StageLType::Struct(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if !visited.insert(key.clone()) {
                    return false;
                }
                let has_buf = if let Some(sdef) = defs.struct_defs.get(&key) {
                    sdef.fields.iter().any(|f| visit(f, defs, visited))
                } else if let Some(nsdef) = defs.named_structs.get(&key) {
                    nsdef.fields.iter().any(|(_, f)| visit(f, defs, visited))
                } else {
                    false
                };
                visited.remove(&key);
                has_buf
            }
            StageLType::Nominal(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if !visited.insert(key.clone()) {
                    return false;
                }
                let has_buf = if let Some(bdef) = defs.brand_types.get(&key) {
                    visit(&bdef.inner_ty, defs, visited)
                } else if let Some(edef) = defs.enums.get(&key) {
                    edef.variants.iter().any(|v| {
                        v.payload
                            .as_ref()
                            .is_some_and(|pty| visit(pty, defs, visited))
                    })
                } else {
                    false
                };
                visited.remove(&key);
                has_buf
            }
            _ => false,
        }
    }
    let mut visited = HashSet::new();
    visit(ty, defs, &mut visited)
}

/// Returns `true` if `ty` may appear inside `l/T`, `option/T`, `result/T`, or
/// `buf/T` without violating element-shape or buffer-transitivity rules.
pub(super) fn type_can_be_element_with_defs(
    ty: &StageLType,
    defs: &TypeDefs<'_>,
) -> bool {
    ty.can_be_element() && !type_contains_buf_with_defs(ty, defs)
}

/// Validates element/field/parameter shape in positions that forbid `void`,
/// `never`, and `buf/T` (either directly or inside compound types).
pub(super) fn check_stager_value_type_shape(
    ty: &StageLType,
    defs: &TypeDefs<'_>,
    span: Span,
) -> Result<()> {
    ensure!(
        type_can_be_element_with_defs(ty, defs),
        "Type '{ty}' at {span} is not valid as a stored value, field, parameter, or element type"
    );
    Ok(())
}

/// Validates whether a canonicalized type `ty` is well-formed under `dialect`
/// and `buf_policy`.
pub(super) fn check_type_well_formed(
    ty: &StageLType,
    defs: &TypeDefs<'_>,
    dialect: StageDialect,
    buf_policy: BufPolicy,
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
        StageLType::I128
        | StageLType::U128
        | StageLType::Int
        | StageLType::Rat
        | StageLType::Dc
        | StageLType::Dcs
        | StageLType::S
        | StageLType::Error
        | StageLType::View
        | StageLType::Cursor => {
            ensure!(
                dialect == StageDialect::StageR3,
                "StageR 3 type '{}' is not supported in StageL 3 validator at {span}",
                ty.display_name()
            );
            Ok(())
        }
        StageLType::List(inner) => {
            ensure!(
                type_can_be_element_with_defs(inner, defs),
                "Invalid list element type '{}' in 'l/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            let inner_policy = if buf_policy == BufPolicy::LocalAllowed {
                BufPolicy::LocalAllowed
            } else {
                BufPolicy::Disallowed
            };
            check_type_well_formed(inner, defs, dialect, inner_policy, span)
        }
        StageLType::Buf(inner) => {
            match dialect {
                StageDialect::StageL3 => {
                    ensure!(
                        buf_policy == BufPolicy::LocalAllowed,
                        "Local buffer type 'buf/{}' is disallowed in public constants/signatures at {span}",
                        inner.display_name()
                    );
                }
                StageDialect::StageR3 => {
                    ensure!(
                        buf_policy == BufPolicy::StageRDirectBufOnly,
                        "'buf/T' type '{ty}' is not permitted in this position at {span}"
                    );
                }
            }
            ensure!(
                type_can_be_element_with_defs(inner, defs),
                "Invalid buffer element type '{}' in 'buf/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(
                inner,
                defs,
                dialect,
                if dialect == StageDialect::StageL3 {
                    BufPolicy::LocalAllowed
                } else {
                    BufPolicy::Disallowed
                },
                span,
            )
        }
        StageLType::Option(inner) => {
            ensure!(
                dialect == StageDialect::StageR3,
                "StageR 3 type '{}' is not supported in StageL 3 validator at {span}",
                ty.display_name()
            );
            ensure!(
                type_can_be_element_with_defs(inner, defs),
                "Invalid option element type '{}' in 'option/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(
                inner,
                defs,
                dialect,
                BufPolicy::Disallowed,
                span,
            )
        }
        StageLType::Result(inner) => {
            ensure!(
                dialect == StageDialect::StageR3,
                "StageR 3 type '{}' is not supported in StageL 3 validator at {span}",
                ty.display_name()
            );
            ensure!(
                *inner.as_ref() == StageLType::Void
                    || type_can_be_element_with_defs(inner, defs),
                "Invalid result ok type '{}' in 'result/{}' at {span}",
                inner.display_name(),
                inner.display_name()
            );
            check_type_well_formed(
                inner,
                defs,
                dialect,
                BufPolicy::Disallowed,
                span,
            )
        }
        StageLType::Tuple(elems) => {
            let slot_policy = if dialect == StageDialect::StageR3
                || buf_policy == BufPolicy::Disallowed
            {
                BufPolicy::Disallowed
            } else {
                BufPolicy::LocalAllowed
            };
            for el in elems {
                ensure!(
                    *el != StageLType::Void && *el != StageLType::Never,
                    "Invalid tuple slot type '{}' in '{}' at {span}",
                    el.display_name(),
                    ty.display_name()
                );
                if dialect == StageDialect::StageR3 {
                    ensure!(
                        !el.is_or_contains_buf(),
                        "'buf/T' cannot be stored inside compound type '{ty}' at {span}"
                    );
                }
                check_type_well_formed(el, defs, dialect, slot_policy, span)?;
            }
            Ok(())
        }
        StageLType::Struct(StructTypeRef::Canonical { namespace, name }) => {
            let key = (namespace.clone(), name.clone());
            let known = match dialect {
                StageDialect::StageL3 => defs.struct_defs.contains_key(&key),
                StageDialect::StageR3 => {
                    defs.named_structs.contains_key(&key)
                        || defs.struct_defs.contains_key(&key)
                }
            };
            ensure!(known, "Unknown struct type '{name}' at {span}");
            if dialect == StageDialect::StageR3
                || buf_policy == BufPolicy::Disallowed
            {
                ensure!(
                    !type_contains_buf_with_defs(ty, defs),
                    "Struct '{name}' containing local buffer 'buf/T' is disallowed in this position at {span}"
                );
            }
            Ok(())
        }
        StageLType::Struct(_) => {
            bail!(
                "Uncanonicalized struct type '{}' at {span}",
                ty.display_name()
            )
        }
        StageLType::Nominal(StructTypeRef::Canonical { namespace, name }) => {
            ensure!(
                dialect == StageDialect::StageR3,
                "StageR 3 type '{}' is not supported in StageL 3 validator at {span}",
                ty.display_name()
            );
            let key = (namespace.clone(), name.clone());
            ensure!(
                defs.brand_types.contains_key(&key)
                    || defs.enums.contains_key(&key),
                "Unknown nominal type '{name}' at {span}"
            );
            ensure!(
                !type_contains_buf_with_defs(ty, defs),
                "Nominal type '{name}' cannot contain 'buf/T' at {span}"
            );
            Ok(())
        }
        StageLType::Nominal(_) => {
            bail!(
                "StageR 3 or uncanonicalized nominal type '{}' at {span}",
                ty.display_name()
            )
        }
    }
}

/// Rejects unguarded (infinite-size) recursive value-type cycles across
/// `struct_defs`, `brand_types`, `named_structs`, and `enums`.
pub(super) fn check_unguarded_type_cycles(defs: &TypeDefs<'_>) -> Result<()> {
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
            StageLType::Struct(StructTypeRef::Canonical {
                namespace,
                name,
            })
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
            // `List(T)` and `Buf(T)` are heap-indirected sequence headers and break value-size cycles.
            _ => {}
        }
    }

    fn dfs(
        key: &(Option<String>, String),
        defs: &TypeDefs<'_>,
        marks: &mut HashMap<(Option<String>, String), Mark>,
        stack: &mut Vec<String>,
    ) -> Result<()> {
        if let Some(&m) = marks.get(key) {
            match m {
                Mark::Visited => return Ok(()),
                Mark::Visiting => {
                    stack.push(key.1.clone());
                    let sp = defs
                        .struct_defs
                        .get(key)
                        .map(|d| d.span)
                        .or_else(|| defs.named_structs.get(key).map(|d| d.span))
                        .or_else(|| defs.brand_types.get(key).map(|d| d.span))
                        .or_else(|| defs.enums.get(key).map(|d| d.span))
                        // Reason for fallback: Default 1:1 span only if definition table is missing.
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
        if let Some(sdef) = defs.struct_defs.get(key) {
            for f in &sdef.fields {
                collect_direct_edges(f, &mut edges);
            }
        }
        if let Some(nsdef) = defs.named_structs.get(key) {
            for (_, fty) in &nsdef.fields {
                collect_direct_edges(fty, &mut edges);
            }
        }
        if let Some(bdef) = defs.brand_types.get(key) {
            collect_direct_edges(&bdef.inner_ty, &mut edges);
        }
        if let Some(edef) = defs.enums.get(key) {
            for v in &edef.variants {
                if let Some(ref pty) = v.payload {
                    collect_direct_edges(pty, &mut edges);
                }
            }
        }
        for target in edges {
            dfs(&target, defs, marks, stack)?;
        }
        let _ = stack.pop();
        marks.insert(key.clone(), Mark::Visited);
        Ok(())
    }

    let mut marks = HashMap::new();
    let mut stack = Vec::new();
    for key in defs
        .struct_defs
        .keys()
        .chain(defs.named_structs.keys())
        .chain(defs.brand_types.keys())
        .chain(defs.enums.keys())
    {
        if !marks.contains_key(key) {
            dfs(key, defs, &mut marks, &mut stack)?;
        }
    }
    Ok(())
}

/// Validates that `pub dc/<shorthand>` uses only Short or Format Dc IDs.
pub(super) fn check_pub_dc_shorthand(dc: DcChar, span: Span) -> Result<()> {
    let sh = DcShorthand::from_global_id(dc.to_u128());
    match sh {
        DcShorthand::Short(_) | DcShorthand::Format(_) => Ok(()),
        other => bail!(
            "'pub dc/{other}' at {span} must reference a Core Short Dc or Format Dc (use `: @dc {other}` comment for System-block IDs)"
        ),
    }
}

/// Returns `true` when `actual` satisfies `expected` (exact match or `never`).
pub(super) fn types_compatible(
    actual: &StageLType,
    expected: &StageLType,
) -> bool {
    actual == expected || *actual == StageLType::Never
}

/// Returns `true` if `ty` supports structural `eq` / `ne` under `dialect`.
pub(super) fn type_supports_eq(
    ty: &StageLType,
    defs: &TypeDefs<'_>,
    dialect: StageDialect,
) -> bool {
    fn visit(
        ty: &StageLType,
        defs: &TypeDefs<'_>,
        dialect: StageDialect,
        visiting: &mut HashSet<(Option<String>, String)>,
    ) -> bool {
        match ty {
            StageLType::B
            | StageLType::U8
            | StageLType::I32
            | StageLType::U32
            | StageLType::I64
            | StageLType::U64 => true,
            StageLType::I128
            | StageLType::U128
            | StageLType::Int
            | StageLType::Rat
            | StageLType::Dc
            | StageLType::Dcs
            | StageLType::S
            | StageLType::Error => dialect == StageDialect::StageR3,
            StageLType::Void
            | StageLType::Never
            | StageLType::Buf(_)
            | StageLType::View
            | StageLType::Cursor => false,
            StageLType::List(inner) => visit(inner, defs, dialect, visiting),
            StageLType::Option(inner) => {
                dialect == StageDialect::StageR3
                    && visit(inner, defs, dialect, visiting)
            }
            StageLType::Result(inner) => {
                dialect == StageDialect::StageR3
                    && (**inner == StageLType::Void
                        || visit(inner, defs, dialect, visiting))
            }
            StageLType::Tuple(elems) => {
                elems.iter().all(|e| visit(e, defs, dialect, visiting))
            }
            StageLType::Struct(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                let key = (namespace.clone(), name.clone());
                if !visiting.insert(key.clone()) {
                    return true;
                }
                let ok = if let Some(nsdef) = defs.named_structs.get(&key) {
                    dialect == StageDialect::StageR3
                        && nsdef
                            .fields
                            .iter()
                            .all(|(_, fty)| visit(fty, defs, dialect, visiting))
                } else if let Some(sdef) = defs.struct_defs.get(&key) {
                    sdef.fields
                        .iter()
                        .all(|fty| visit(fty, defs, dialect, visiting))
                } else {
                    false
                };
                visiting.remove(&key);
                ok
            }
            StageLType::Nominal(StructTypeRef::Canonical {
                namespace,
                name,
            }) => {
                if dialect != StageDialect::StageR3 {
                    return false;
                }
                let key = (namespace.clone(), name.clone());
                if !visiting.insert(key.clone()) {
                    return true;
                }
                let ok = if let Some(bdef) = defs.brand_types.get(&key) {
                    visit(&bdef.inner_ty, defs, dialect, visiting)
                } else if let Some(edef) = defs.enums.get(&key) {
                    edef.variants.iter().all(|v| {
                        v.payload.as_ref().is_none_or(|pty| {
                            visit(pty, defs, dialect, visiting)
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
    let mut visiting = HashSet::new();
    visit(ty, defs, dialect, &mut visiting)
}

/// Returns `true` when `conv <src> <dst>` in StageR 3 is statically lossless
/// (direct return type `dst` rather than `result/dst`).
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
        (StageLType::List(inner), StageLType::Dcs)
            if **inner == StageLType::Dc =>
        {
            true
        }
        (StageLType::Dcs, StageLType::List(inner))
            if **inner == StageLType::Dc =>
        {
            true
        }
        _ => false,
    }
}

/// Unifies two operand types satisfying `pred` with `never` bottom propagation.
pub(super) fn unify_matching_pair<F>(
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

/// Maps well-known Stage 1 carrier structs (`std/i128::i128Val`, `std/u128::u128Val`,
/// `std/bigint::bigInt`, `std/rat::ratVal`, `std/dctf::dcVal`) to native StageR 3
/// primitive types (`i128`, `u128`, `int`, `rat`, `dc`), recursing in `l/T` and `tup`.
#[must_use]
pub(crate) fn bridge_stage1_type_for_stager(ty: &StageLType) -> StageLType {
    match ty {
        StageLType::Struct(StructTypeRef::Canonical {
            namespace: Some(ns),
            name,
        }) => match (ns.as_str(), name.as_str()) {
            ("std/i128", "i128Val") => StageLType::I128,
            ("std/u128", "u128Val") => StageLType::U128,
            ("std/bigint", "bigInt") => StageLType::Int,
            ("std/rat", "ratVal") => StageLType::Rat,
            ("std/dctf", "dcVal") => StageLType::Dc,
            _ => ty.clone(),
        },
        StageLType::List(inner) => {
            StageLType::List(Box::new(bridge_stage1_type_for_stager(inner)))
        }
        StageLType::Tuple(elems) => StageLType::Tuple(
            elems.iter().map(bridge_stage1_type_for_stager).collect(),
        ),
        _ => ty.clone(),
    }
}

/// Inverse of [`bridge_stage1_type_for_stager`]: maps StageR 3 primitive types
/// backed by Stage 1 carriers (`i128`, `u128`, `int`, `rat`, `dc`) back to
/// canonical Stage 1 struct types, recursing in `l/T` and `tup`.
#[must_use]
pub(crate) fn unbridge_stager_type_for_stage1(ty: &StageLType) -> StageLType {
    let stage1_struct = |ns: &str, name: &str| {
        StageLType::Struct(StructTypeRef::Canonical {
            namespace: Some(ns.to_owned()),
            name: name.to_owned(),
        })
    };
    match ty {
        StageLType::I128 => stage1_struct("std/i128", "i128Val"),
        StageLType::U128 => stage1_struct("std/u128", "u128Val"),
        StageLType::Int => stage1_struct("std/bigint", "bigInt"),
        StageLType::Rat => stage1_struct("std/rat", "ratVal"),
        StageLType::Dc => stage1_struct("std/dctf", "dcVal"),
        StageLType::List(inner) => {
            StageLType::List(Box::new(unbridge_stager_type_for_stage1(inner)))
        }
        StageLType::Tuple(elems) => StageLType::Tuple(
            elems.iter().map(unbridge_stager_type_for_stage1).collect(),
        ),
        _ => ty.clone(),
    }
}
