// SPDX-License-Identifier: AGPL-3.0-or-later AND GPL-3.0-or-later AND MIT AND BSD-3-Clause
// SPDX-License-Identifier for parts derived from kaitai_struct_compiler: GPL-3.0-or-later AND MIT AND BSD-3-Clause
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

/*

Kaitai Struct compiler itself is copyright (C) 2015-2026 Kaitai Project.

Portions of Kaitai Struct compiler are loosely based on pythonparse from FastParse and are copyright (c) 2014 Li Haoyi (haoyi.sg@gmail.com).

Portions of Kaitai Struct compiler are based on scala/xml/Utility.scala from Scala XML.

Copyright (c) 2002-2017 EPFL
Copyright (c) 2011-2017 Lightbend, Inc.

See full license information at the end of this file.
*/

//! Type and schema resolver converting parsed YAML specs into resolved class trees.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use indexmap::IndexMap;

use super::hierarchy::{
    ClassSpec, EndianSwitch, ResolvedAttr, ResolvedEnum, ResolvedInstance, ResolvedParam,
    ResolvedValidation, ValidationRule,
};
use super::imports::SpecRegistry;
use super::types::{BitEndianness, DataType, Endianness, RepeatMode};
use crate::expr::{parse_expr, Expr, UnaryOp};
use crate::spec::{
    AttrSpec, ContentsSpec, EndianSpec, EnumValueSpec, InstanceSpec, KsyFile, TypeSpec,
    ValidationSpec, ValueOrExpr,
};

fn java_string_hash(s: &str) -> u32 {
    let mut h: u32 = 0;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    h
}

fn value_or_expr_to_expr(ve: &ValueOrExpr) -> Result<Expr> {
    match ve {
        ValueOrExpr::Expr(s) => parse_expr(s),
        ValueOrExpr::Int(i) => Ok(Expr::IntNum(*i as i128)),
        ValueOrExpr::Float(f) => Ok(Expr::FloatNum(*f)),
        ValueOrExpr::Bool(b) => Ok(Expr::Bool(*b)),
    }
}

fn resolve_validation(
    attr_id: &str,
    attr: &AttrSpec,
    src_path: &str,
) -> Result<Option<ResolvedValidation>> {
    if let Some(contents) = &attr.contents {
        let bytes: Vec<Expr> = match contents {
            ContentsSpec::Text(s) => s.bytes().map(|b| Expr::IntNum(i128::from(b))).collect(),
            ContentsSpec::Sequence(seq) => {
                let mut v = Vec::new();
                for item in seq {
                    if let Some(n) = item.as_i64() {
                        v.push(Expr::IntNum(i128::from(n)));
                    } else if let Some(s) = item.as_str() {
                        for b in s.bytes() {
                            v.push(Expr::IntNum(i128::from(b)));
                        }
                    }
                }
                v
            }
        };
        return Ok(Some(ResolvedValidation {
            rule: ValidationRule::Eq(Expr::List(bytes)),
            src_path: src_path.to_string(),
        }));
    }

    if let Some(valid_spec) = &attr.valid {
        let rule = match valid_spec {
            ValidationSpec::Simple(val_or_expr) => {
                let expr = value_or_expr_to_expr(val_or_expr)?;
                ValidationRule::Eq(expr)
            }
            ValidationSpec::Detailed(d) => {
                if let Some(eq_val) = &d.eq {
                    let expr = value_or_expr_to_expr(eq_val)?;
                    ValidationRule::Eq(expr)
                } else if let (Some(min_val), Some(max_val)) = (&d.min, &d.max) {
                    let min_expr = value_or_expr_to_expr(min_val)?;
                    let max_expr = value_or_expr_to_expr(max_val)?;
                    ValidationRule::Range(min_expr, max_expr)
                } else if let Some(min_val) = &d.min {
                    let min_expr = value_or_expr_to_expr(min_val)?;
                    ValidationRule::Min(min_expr)
                } else if let Some(max_val) = &d.max {
                    let max_expr = value_or_expr_to_expr(max_val)?;
                    ValidationRule::Max(max_expr)
                } else if let Some(any_of) = &d.any_of {
                    let exprs = any_of
                        .iter()
                        .map(value_or_expr_to_expr)
                        .collect::<Result<Vec<_>>>()?;
                    ValidationRule::AnyOf(exprs)
                } else if d.in_enum == Some(true) {
                    ValidationRule::InEnum
                } else if let Some(expr_str) = &d.expr {
                    let expr = parse_expr(expr_str)?;
                    ValidationRule::Expr(expr)
                } else {
                    bail!("Unsupported detailed validation spec on {attr_id}");
                }
            }
        };
        return Ok(Some(ResolvedValidation {
            rule,
            src_path: src_path.to_string(),
        }));
    }

    Ok(None)
}

/// Resolves a parsed `.ksy` file into a `ClassSpec` tree.
///
/// # Errors
/// Returns an error if types or expressions in the specification cannot be resolved.
pub fn resolve_ksy(
    top_id: &str,
    ksy: &KsyFile,
    registry: Option<&SpecRegistry>,
) -> Result<ClassSpec> {
    let top_name = vec![top_id.to_string()];
    let top_name_slice: &[String] = &top_name;
    let scopes = [(top_name_slice, ksy)];
    let mut root_spec = resolve_class_spec(&top_name, None, &top_name, ksy, registry, None, &scopes)?;
    markup_parent_types(&mut root_spec);
    update_instance_types(&mut root_spec);
    root_spec.external_types.sort_by(|a, b| {
        let ha = java_string_hash(&a.join("::"));
        let hb = java_string_hash(&b.join("::"));
        ha.cmp(&hb).then_with(|| a.cmp(b))
    });
    Ok(root_spec)
}

fn collect_used_user_types(dt: &DataType, out: &mut Vec<Vec<String>>) {
    match dt {
        DataType::UserType { names, is_external, .. } => {
            if !*is_external {
                out.push(names.clone());
            }
        }
        DataType::ArrayType { element, .. } => {
            collect_used_user_types(element, out);
        }
        DataType::SwitchType { cases, .. } => {
            for case_dt in cases.values() {
                collect_used_user_types(case_dt, out);
            }
        }
        _ => {}
    }
}

fn find_class_spec_mut<'a>(root: &'a mut ClassSpec, path: &[String]) -> Option<&'a mut ClassSpec> {
    if path.is_empty() {
        return None;
    }
    let parts = if path[0] == root.name[0] {
        &path[1..]
    } else {
        path
    };
    let mut cur = root;
    for part in parts {
        cur = cur.subclasses.get_mut(part)?;
    }
    Some(cur)
}

fn find_class_spec<'a>(root: &'a ClassSpec, path: &[String]) -> Option<&'a ClassSpec> {
    if path.is_empty() {
        return None;
    }
    let parts = if path[0] == root.name[0] {
        &path[1..]
    } else {
        path
    };
    let mut cur = root;
    for part in parts {
        cur = cur.subclasses.get(part)?;
    }
    Some(cur)
}

fn infer_attr_type_in_class(root: &ClassSpec, class_name: &[String], attr_name: &str) -> Option<DataType> {
    let class = find_class_spec(root, class_name)?;
    if let Some(seq_attr) = class.seq.iter().find(|a| a.id == attr_name) {
        return Some(seq_attr.data_type.clone());
    }
    if let Some(inst) = class.instances.get(attr_name) {
        return Some(inst.data_type.clone());
    }
    if let Some(p) = class.params.iter().find(|p| p.id == attr_name) {
        return Some(p.data_type.clone());
    }
    None
}

fn infer_expr_type_with_root(
    expr: &Expr,
    curr_class_name: &[String],
    root: &ClassSpec,
) -> Option<DataType> {
    match expr {
        Expr::Attribute { value, attr } => {
            if attr == "to_i" {
                return Some(DataType::CalcIntType);
            }
            if let Expr::Name(name) = &**value {
                let target_class_name: Option<Vec<String>> = if name == "_root" {
                    let curr = find_class_spec(root, curr_class_name)?;
                    Some(curr.root_name.clone())
                } else if name == "_parent" {
                    let curr = find_class_spec(root, curr_class_name)?;
                    curr.parent_name.clone()
                } else {
                    None
                };
                if let Some(target_class) = target_class_name {
                    return infer_attr_type_in_class(root, &target_class, attr);
                }
            }
            if let Some(DataType::UserType { names, .. }) = infer_expr_type_with_root(value, curr_class_name, root) {
                return infer_attr_type_in_class(root, &names, attr);
            }
            None
        }
        _ => None,
    }
}

fn collect_all_class_names(current: &ClassSpec, out: &mut Vec<Vec<String>>) {
    out.push(current.name.clone());
    for sub in current.subclasses.values() {
        collect_all_class_names(sub, out);
    }
}

fn update_instance_types(root: &mut ClassSpec) {
    let mut all_names = Vec::new();
    collect_all_class_names(root, &mut all_names);

    let mut updates = Vec::new();
    for class_name in &all_names {
        let Some(curr) = find_class_spec(root, class_name) else { continue };
        for (inst_id, inst) in &curr.instances {
            if matches!(inst.data_type, DataType::CalcIntType) {
                if let Some(val_ex) = &inst.value_expr {
                    if let Some(inferred) = infer_expr_type_with_root(val_ex, class_name, root) {
                        updates.push((class_name.clone(), inst_id.clone(), inferred));
                    }
                }
            }
        }
    }

    for (class_name, inst_id, new_dt) in updates {
        if let Some(curr_mut) = find_class_spec_mut(root, &class_name) {
            if let Some(inst_mut) = curr_mut.instances.get_mut(&inst_id) {
                inst_mut.data_type = new_dt;
            }
        }
    }
}

fn reset_parent_types(current: &mut ClassSpec) {
    if !current.is_top_level {
        current.parent_name = None;
    }
    for sub in current.subclasses.values_mut() {
        reset_parent_types(sub);
    }
}

fn finalize_parent_types(current: &mut ClassSpec) {
    if !current.is_top_level && current.parent_name.is_none() {
        current.parent_name = Some(vec!["KStructUnit".to_string()]);
    }
    for sub in current.subclasses.values_mut() {
        finalize_parent_types(sub);
    }
}

fn markup_parent_types(root: &mut ClassSpec) {
    reset_parent_types(root);

    let mut queue = std::collections::VecDeque::new();
    queue.push_back(root.name.clone());
    let mut queued = std::collections::BTreeSet::new();
    queued.insert(root.name.clone());

    while let Some(curr_name) = queue.pop_front() {
        let (_p_name, instantiations) = {
            let Some(curr) = find_class_spec_mut(root, &curr_name) else { continue };
            let p_name = curr.parent_name.clone();
            let mut list = Vec::new();

            for attr in &curr.seq {
                let p_target = match &attr.parent_expr {
                    Some(crate::spec::ValueOrExpr::Bool(false)) => Some(vec!["KStructUnit".to_string()]),
                    Some(crate::spec::ValueOrExpr::Expr(p)) if p == "_parent" => p_name.clone(),
                    _ => Some(curr.name.clone()),
                };
                let mut used_types = Vec::new();
                collect_used_user_types(&attr.data_type, &mut used_types);
                for target in used_types {
                    list.push((target, p_target.clone()));
                }
            }

            for inst in curr.instances.values() {
                if inst.pos_expr.is_some() || inst.io_expr.is_some() || inst.value_expr.is_none() {
                    let p_target = match &inst.parent_expr {
                        Some(crate::spec::ValueOrExpr::Bool(false)) => Some(vec!["KStructUnit".to_string()]),
                        Some(crate::spec::ValueOrExpr::Expr(p)) if p == "_parent" => p_name.clone(),
                        _ => Some(curr.name.clone()),
                    };
                    let mut used_types = Vec::new();
                    collect_used_user_types(&inst.data_type, &mut used_types);
                    for target in used_types {
                        list.push((target, p_target.clone()));
                    }
                }
            }

            (p_name, list)
        };

        for (target, p_target) in instantiations {
            let Some(p_to_set) = p_target else { continue };
            if let Some(target_class) = find_class_spec_mut(root, &target) {
                if !target_class.is_top_level {
                    match &target_class.parent_name {
                        None => {
                            target_class.parent_name = Some(p_to_set);
                            if queued.insert(target.clone()) {
                                queue.push_back(target);
                            }
                        }
                        Some(existing) if existing == &p_to_set => {}
                        Some(_) => {
                            target_class.parent_name = Some(vec!["KStructUnit".to_string()]);
                        }
                    }
                }
            }
        }
    }

    finalize_parent_types(root);
}

fn resolve_class_spec(
    name: &[String],
    parent_name: Option<&[String]>,
    root_name: &[String],
    ksy: &KsyFile,
    registry: Option<&SpecRegistry>,
    parent_endian_info: Option<(Option<Endianness>, bool)>,
    scopes: &[(&[String], &KsyFile)],
) -> Result<ClassSpec> {
    let is_top_level = parent_name.is_none();
    let (meta_endian, endian_switch) = match ksy.meta.as_ref().and_then(|m| m.endian.as_ref()) {
        Some(EndianSpec::Simple(s)) => match s.as_str() {
            "le" => (Some(Endianness::Little), None),
            "be" => (Some(Endianness::Big), None),
            "inherited" => (Some(Endianness::Inherited), None),
            _ => (None, None),
        },
        Some(EndianSpec::Switch(sw)) => {
            let switch_on = parse_expr(&sw.switch_on)?;
            let mut cases = IndexMap::new();
            for (case_key, case_val) in &sw.cases {
                let e = match case_val.as_str() {
                    Some("le") => Endianness::Little,
                    Some("be") => Endianness::Big,
                    _ => Endianness::Inherited,
                };
                cases.insert(case_key.clone(), e);
            }
            (None, Some(EndianSwitch { switch_on, cases }))
        }
        None => {
            if let Some((p_endian, p_dynamic)) = parent_endian_info {
                if p_dynamic {
                    (Some(Endianness::Inherited), None)
                } else if let Some(e) = p_endian {
                    (Some(e), None)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
    };
    let has_dynamic_endian = endian_switch.is_some() || meta_endian == Some(Endianness::Inherited);
    let meta_license = ksy
        .meta
        .as_ref()
        .and_then(|m| m.license.as_ref().map(|l| l.to_string()));
    let meta_imports = ksy
        .meta
        .as_ref()
        .map(|m| m.imports.clone())
        .unwrap_or_default();

    let doc = ksy.doc.clone();
    let doc_refs = ksy.doc_ref.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());

    // Resolve enums
    let mut resolved_enums = IndexMap::new();
    for (enum_name, enum_values) in &ksy.enums {
        let mut values_map = IndexMap::new();
        let mut value_docs_map = IndexMap::new();
        let mut value_doc_refs_map = IndexMap::new();
        for (raw_key, val_spec) in enum_values {
            let num: i64 = if let Some(stripped) = raw_key.strip_prefix("0x") {
                i64::from_str_radix(stripped, 16).unwrap_or(0)
            } else {
                raw_key.parse().unwrap_or(0)
            };
            let (label, doc, doc_ref) = match val_spec {
                EnumValueSpec::Simple(s) => (s.clone(), None, None),
                EnumValueSpec::Bool(b) => (b.to_string(), None, None),
                EnumValueSpec::Detailed(d) => (
                    d.id.clone().unwrap_or_else(|| format!("val_{num}")),
                    d.doc.clone(),
                    d.doc_ref.as_ref().map(|r| r.as_slice().to_vec()),
                ),
            };
            values_map.insert(num, label);
            if let Some(d) = doc {
                value_docs_map.insert(num, d);
            }
            if let Some(refs) = doc_ref {
                value_doc_refs_map.insert(num, refs);
            }
        }
        values_map.sort_by(|k1, _, k2, _| k1.cmp(k2));
        value_docs_map.sort_by(|k1, _, k2, _| k1.cmp(k2));
        value_doc_refs_map.sort_by(|k1, _, k2, _| k1.cmp(k2));
        resolved_enums.insert(
            enum_name.clone(),
            ResolvedEnum {
                name: enum_name.clone(),
                values: values_map,
                value_docs: value_docs_map,
                value_doc_refs: value_doc_refs_map,
                doc: None,
            },
        );
    }

    // Resolve sequential attributes
    let mut resolved_seq = Vec::new();
    let mut external_types = Vec::new();

    for (seq_idx, attr) in ksy.seq.iter().enumerate() {
        let attr_id = attr
            .id
            .clone()
            .unwrap_or_else(|| format!("unnamed{seq_idx}"));
        let orig_id = attr.orig_id.as_ref().and_then(|o| o.as_single().map(str::to_string));
        let attr_doc = attr.doc.clone();

        let if_expr = if let Some(if_str) = &attr.if_expr {
            Some(parse_expr(if_str)?)
        } else {
            None
        };

        // Resolve data type
        let (dt, raw_id, io_id, ext_types) = resolve_attr_data_type(
            &attr_id,
            attr,
            meta_endian,
            scopes,
            registry,
        )?;

        for ext in ext_types {
            if !external_types.contains(&ext) {
                external_types.push(ext);
            }
        }

        let attr_doc_refs = attr.doc_ref.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());

        let src_path = if name.len() <= 1 {
            format!("/seq/{seq_idx}")
        } else {
            let types_prefix = name[1..]
                .iter()
                .map(|sub| format!("/types/{sub}"))
                .collect::<Vec<_>>()
                .join("");
            format!("{types_prefix}/seq/{seq_idx}")
        };

        let valid = resolve_validation(&attr_id, attr, &src_path)?;

        resolved_seq.push(ResolvedAttr {
            id: attr_id,
            orig_id,
            doc: attr_doc,
            doc_refs: attr_doc_refs,
            data_type: dt,
            if_expr,
            raw_id,
            io_id,
            valid,
            parent_expr: attr.parent.clone(),
        });
    }

    // Resolve instances
    let mut resolved_instances = IndexMap::new();
    for (inst_id, inst_spec) in &ksy.instances {
        let (dt, value_expr, pos_expr, size_expr, io_expr, if_expr, is_cached) = resolve_instance(
            inst_id,
            inst_spec,
            meta_endian,
            scopes,
            registry,
        )?;
        let inst_doc_refs = inst_spec.doc_ref.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());
        resolved_instances.insert(
            inst_id.clone(),
            ResolvedInstance {
                id: inst_id.clone(),
                doc: inst_spec.doc.clone(),
                doc_refs: inst_doc_refs,
                data_type: dt,
                value_expr,
                pos_expr,
                size_expr,
                io_expr,
                if_expr,
                is_cached,
                parent_expr: inst_spec.parent.clone(),
                enum_name: inst_spec.enum_name.clone(),
            },
        );
    }

    // Resolve nested types (subclasses)
    let mut resolved_subclasses = IndexMap::new();
    for (sub_name, sub_ksy) in &ksy.types {
        let mut sub_path = name.to_vec();
        sub_path.push(sub_name.clone());
        let mut child_scopes = scopes.to_vec();
        child_scopes.push((&sub_path, sub_ksy));
        let sub_spec = resolve_class_spec(
            &sub_path,
            Some(name),
            root_name,
            sub_ksy,
            registry,
            Some((meta_endian, has_dynamic_endian)),
            &child_scopes,
        )?;
        resolved_subclasses.insert(sub_name.clone(), sub_spec);
    }

    // Resolve params
    let mut resolved_params = Vec::new();
    for p in &ksy.params {
        let (param_type, _) = if let Some(type_name) = &p.type_spec {
            resolve_simple_type(
                type_name,
                None,
                meta_endian,
                scopes,
                registry,
            )?
        } else {
            (
                DataType::IntMulti {
                    signed: true,
                    width: 4,
                    endian: meta_endian,
                },
                None,
            )
        };
        let param_doc_refs = p.doc_ref.as_ref().map_or_else(Vec::new, |d| d.as_slice().to_vec());
        resolved_params.push(ResolvedParam {
            id: p.id.clone(),
            data_type: param_type,
            doc: p.doc.clone(),
            doc_refs: param_doc_refs,
        });
    }

    resolved_instances.sort_keys();
    resolved_enums.sort_keys();
    resolved_subclasses.sort_keys();

    Ok(ClassSpec {
        name: name.to_vec(),
        parent_name: parent_name.map(|p| p.to_vec()),
        root_name: root_name.to_vec(),
        doc,
        doc_refs,
        seq: resolved_seq,
        instances: resolved_instances,
        enums: resolved_enums,
        subclasses: resolved_subclasses,
        params: resolved_params,
        is_top_level,
        meta_endian,
        endian_switch,
        meta_license,
        meta_imports,
        external_types,
    })
}

fn resolve_attr_data_type(
    attr_id: &str,
    attr: &AttrSpec,
    default_endian: Option<Endianness>,
    scopes: &[(&[String], &KsyFile)],
    registry: Option<&SpecRegistry>,
) -> Result<(DataType, Option<String>, Option<String>, Vec<Vec<String>>)> {
    let mut raw_id = None;
    let mut io_id = None;
    let mut external_types = Vec::new();

    // Parse repeat
    let repeat_mode = if let Some(rep) = &attr.repeat {
        match rep.as_str() {
            "eos" => RepeatMode::Eos,
            "expr" => {
                if let Some(r_expr) = &attr.repeat_expr {
                    let parsed = match r_expr {
                        ValueOrExpr::Expr(s) => parse_expr(s)?,
                        ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                        ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                        ValueOrExpr::Bool(b) => Expr::Bool(*b),
                    };
                    RepeatMode::Expr(parsed)
                } else {
                    RepeatMode::None
                }
            }
            "until" => {
                if let Some(u_str) = &attr.repeat_until {
                    RepeatMode::Until(parse_expr(u_str)?)
                } else {
                    RepeatMode::None
                }
            }
            _ => RepeatMode::None,
        }
    } else {
        RepeatMode::None
    };

    // Base type
    let base_type = match &attr.type_spec {
        Some(TypeSpec::Simple(t_name)) => {
            if t_name == "str" || t_name == "strz" {
                let size_expr = if let Some(s) = &attr.size {
                    Some(match s {
                        ValueOrExpr::Expr(e) => parse_expr(e)?,
                        ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                        ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                        ValueOrExpr::Bool(b) => Expr::Bool(*b),
                    })
                } else {
                    None
                };
                DataType::Str {
                    size: size_expr,
                    size_eos: attr.size_eos.unwrap_or(false),
                    encoding: attr.encoding.clone(),
                    terminator: if t_name == "strz" {
                        attr.terminator.as_ref().and_then(|t| match t {
                            ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                            _ => None,
                        }).or(Some(0))
                    } else {
                        attr.terminator.as_ref().and_then(|t| match t {
                            ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                            _ => None,
                        })
                    },
                    consume: attr.consume.unwrap_or(true),
                    include: attr.include.unwrap_or(false),
                    pad_right: attr.pad_right.as_ref().and_then(|p| match p {
                        ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                        _ => None,
                    }),
                }
            } else {
                let (dt, ext) = resolve_simple_type(
                    t_name,
                    attr.enum_name.as_deref(),
                    default_endian,
                    scopes,
                    registry,
                )?;
                if let Some(ext_t) = ext {
                    external_types.push(ext_t);
                }
                dt
            }
        }
        Some(TypeSpec::Switch(switch_spec)) => {
            let switch_expr = parse_expr(&switch_spec.switch_on)?;
            let mut cases = IndexMap::new();
            for (case_key, val) in &switch_spec.cases {
                let case_type_str = match val {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{other:?}"),
                };
                let (case_dt, ext) = resolve_simple_type(
                    &case_type_str,
                    None,
                    default_endian,
                    scopes,
                    registry,
                )?;
                if let Some(ext_t) = ext {
                    if !external_types.contains(&ext_t) {
                        external_types.push(ext_t);
                    }
                }
                cases.insert(case_key.clone(), case_dt);
            }
            // Substream reading for switch
            if attr.size.is_some() || attr.size_eos == Some(true) || attr.process.is_some() {
                if !cases.contains_key("_") {
                    cases.insert(
                        "_".to_string(),
                        DataType::Bytes {
                            size: None,
                            size_eos: false,
                            terminator: None,
                            include: false,
                            consume: false,
                            pad_right: None,
                            process: attr.process.clone(),
                        },
                    );
                }
                raw_id = Some(format!("{attr_id}_raw"));
                io_id = Some(format!("_t_{attr_id}_raw_io"));
            }
            DataType::SwitchType {
                switch_on: switch_expr,
                cases: sort_switch_cases(cases),
            }
        }
        None => {
            // Raw bytes or string
            if let Some(enc) = &attr.encoding {
                let size_expr = if let Some(s) = &attr.size {
                    Some(match s {
                        ValueOrExpr::Expr(e) => parse_expr(e)?,
                        ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                        ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                        ValueOrExpr::Bool(b) => Expr::Bool(*b),
                    })
                } else {
                    None
                };
                DataType::Str {
                    size: size_expr,
                    size_eos: attr.size_eos.unwrap_or(false),
                    encoding: Some(enc.clone()),
                    terminator: attr.terminator.as_ref().and_then(|t| match t {
                        ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                        _ => None,
                    }),
                    consume: attr.consume.unwrap_or(true),
                    include: attr.include.unwrap_or(false),
                    pad_right: attr.pad_right.as_ref().and_then(|p| match p {
                        ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                        _ => None,
                    }),
                }
            } else {
                let size_expr = if let Some(s) = &attr.size {
                    Some(match s {
                        ValueOrExpr::Expr(e) => parse_expr(e)?,
                        ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                        ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                        ValueOrExpr::Bool(b) => Expr::Bool(*b),
                    })
                } else if let Some(c) = &attr.contents {
                    let len = match c {
                        ContentsSpec::Text(s) => s.len(),
                        ContentsSpec::Sequence(seq) => {
                            let mut count = 0usize;
                            for item in seq {
                                if item.as_i64().is_some() {
                                    count = count.saturating_add(1);
                                } else if let Some(s) = item.as_str() {
                                    count = count.saturating_add(s.len());
                                }
                            }
                            count
                        }
                    };
                    Some(Expr::IntNum(i128::try_from(len).unwrap_or(0)))
                } else {
                    None
                };
                DataType::Bytes {
                    size: size_expr,
                    size_eos: attr.size_eos.unwrap_or(false),
                    terminator: attr.terminator.as_ref().and_then(|t| match t {
                        ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                        _ => None,
                    }),
                    consume: attr.consume.unwrap_or(true),
                    include: attr.include.unwrap_or(false),
                    pad_right: attr.pad_right.as_ref().and_then(|p| match p {
                        ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                        _ => None,
                    }),
                    process: attr.process.clone(),
                }
            }
        }
    };

    let final_type = if repeat_mode != RepeatMode::None {
        DataType::ArrayType {
            element: Box::new(base_type),
            repeat: repeat_mode,
        }
    } else {
        base_type
    };

    Ok((final_type, raw_id, io_id, external_types))
}

fn resolve_simple_type(
    type_str: &str,
    enum_name: Option<&str>,
    default_endian: Option<Endianness>,
    scopes: &[(&[String], &KsyFile)],
    registry: Option<&SpecRegistry>,
) -> Result<(DataType, Option<Vec<String>>)> {
    let (clean_type_str, type_args) = if let Ok(parsed_ref) = crate::expr::parser::parse_type_ref(type_str) {
        (parsed_ref.type_name.names.join("::"), parsed_ref.arguments)
    } else {
        (type_str.to_string(), Vec::new())
    };
    let clean_str = clean_type_str.as_str();

    // 1. Bit types
    if let Some(bit_str) = clean_str.strip_prefix('b') {
        if let Ok(count) = bit_str.parse::<usize>() {
            let underlying = if count == 1 {
                DataType::Bits1 {
                    bit_endian: BitEndianness::Big,
                }
            } else {
                DataType::Bits {
                    count,
                    bit_endian: BitEndianness::Big,
                }
            };
            if let Some(ename) = enum_name {
                let mut owner = scopes.last().map_or_else(Vec::new, |s| s.0.to_vec());
                for (scope_name, scope_ksy) in scopes.iter().rev() {
                    if scope_ksy.enums.contains_key(ename) {
                        owner = scope_name.to_vec();
                        break;
                    }
                }
                return Ok((
                    DataType::EnumType {
                        owner,
                        name: ename.to_string(),
                        underlying: Some(Box::new(underlying)),
                    },
                    None,
                ));
            }
            return Ok((underlying, None));
        }
    }
    if clean_str == "bool" {
        return Ok((DataType::CalcBoolType, None));
    }
    if clean_str == "str" {
        return Ok((
            DataType::Str {
                size: None,
                size_eos: false,
                encoding: None,
                terminator: None,
                consume: true,
                include: false,
                pad_right: None,
            },
            None,
        ));
    } else if clean_str == "strz" {
        return Ok((
            DataType::Str {
                size: None,
                size_eos: false,
                encoding: None,
                terminator: Some(0),
                consume: true,
                include: false,
                pad_right: None,
            },
            None,
        ));
    }

    // 2. 1-byte integer
    if clean_str == "u1" {
        if let Some(ename) = enum_name {
            let mut owner = scopes.last().map_or_else(Vec::new, |s| s.0.to_vec());
            for (scope_name, scope_ksy) in scopes.iter().rev() {
                if scope_ksy.enums.contains_key(ename) {
                    owner = scope_name.to_vec();
                    break;
                }
            }
            return Ok((
                DataType::EnumType {
                    owner,
                    name: ename.to_string(),
                    underlying: Some(Box::new(DataType::Int1 { signed: false })),
                },
                None,
            ));
        }
        return Ok((DataType::Int1 { signed: false }, None));
    } else if clean_str == "s1" {
        if let Some(ename) = enum_name {
            let mut owner = scopes.last().map_or_else(Vec::new, |s| s.0.to_vec());
            for (scope_name, scope_ksy) in scopes.iter().rev() {
                if scope_ksy.enums.contains_key(ename) {
                    owner = scope_name.to_vec();
                    break;
                }
            }
            return Ok((
                DataType::EnumType {
                    owner,
                    name: ename.to_string(),
                    underlying: Some(Box::new(DataType::Int1 { signed: true })),
                },
                None,
            ));
        }
        return Ok((DataType::Int1 { signed: true }, None));
    }

    // 3. Multi-byte integers and floats
    let (signed, width, endian) = match clean_str {
        "u2" => (false, 2, default_endian),
        "u2le" => (false, 2, Some(Endianness::Little)),
        "u2be" => (false, 2, Some(Endianness::Big)),
        "u4" => (false, 4, default_endian),
        "u4le" => (false, 4, Some(Endianness::Little)),
        "u4be" => (false, 4, Some(Endianness::Big)),
        "u8" => (false, 8, default_endian),
        "u8le" => (false, 8, Some(Endianness::Little)),
        "u8be" => (false, 8, Some(Endianness::Big)),
        "s2" => (true, 2, default_endian),
        "s2le" => (true, 2, Some(Endianness::Little)),
        "s2be" => (true, 2, Some(Endianness::Big)),
        "s4" => (true, 4, default_endian),
        "s4le" => (true, 4, Some(Endianness::Little)),
        "s4be" => (true, 4, Some(Endianness::Big)),
        "s8" => (true, 8, default_endian),
        "s8le" => (true, 8, Some(Endianness::Little)),
        "s8be" => (true, 8, Some(Endianness::Big)),
        "f4" => return Ok((DataType::Float { width: 4, endian: default_endian }, None)),
        "f4le" => return Ok((DataType::Float { width: 4, endian: Some(Endianness::Little) }, None)),
        "f4be" => return Ok((DataType::Float { width: 4, endian: Some(Endianness::Big) }, None)),
        "f8" => return Ok((DataType::Float { width: 8, endian: default_endian }, None)),
        "f8le" => return Ok((DataType::Float { width: 8, endian: Some(Endianness::Little) }, None)),
        "f8be" => return Ok((DataType::Float { width: 8, endian: Some(Endianness::Big) }, None)),
        _ => (false, 0, None),
    };

    if width > 0 {
        if let Some(ename) = enum_name {
            let mut owner = scopes.last().map_or_else(Vec::new, |s| s.0.to_vec());
            for (scope_name, scope_ksy) in scopes.iter().rev() {
                if scope_ksy.enums.contains_key(ename) {
                    owner = scope_name.to_vec();
                    break;
                }
            }
            return Ok((
                DataType::EnumType {
                    owner,
                    name: ename.to_string(),
                    underlying: Some(Box::new(DataType::IntMulti { signed, width, endian })),
                },
                None,
            ));
        }
        return Ok((DataType::IntMulti { signed, width, endian }, None));
    }

    // 4. Check if it's an enum name defined in any scope (innermost to outermost)
    let parts: Vec<&str> = clean_str.split("::").collect();
    if parts.len() == 1 {
        for (scope_name, scope_ksy) in scopes.iter().rev() {
            if scope_ksy.enums.contains_key(clean_str) {
                return Ok((
                    DataType::EnumType {
                        owner: scope_name.to_vec(),
                        name: clean_str.to_string(),
                        underlying: None,
                    },
                    None,
                ));
            }
        }
    } else {
        let type_parts = &parts[..parts.len().saturating_sub(1)];
        let enum_last = parts.last().unwrap();
        for (scope_name, scope_ksy) in scopes.iter().rev() {
            let mut curr_ksy = *scope_ksy;
            let mut found = true;
            for part in type_parts {
                if let Some(child_ksy) = curr_ksy.types.get(*part) {
                    curr_ksy = child_ksy;
                } else {
                    found = false;
                    break;
                }
            }
            if found && curr_ksy.enums.contains_key(*enum_last) {
                let mut owner = scope_name.to_vec();
                for part in type_parts {
                    owner.push((*part).to_string());
                }
                return Ok((
                    DataType::EnumType {
                        owner,
                        name: (*enum_last).to_string(),
                        underlying: None,
                    },
                    None,
                ));
            }
        }
    }

    // 5. Check if it's a nested type defined in current or parent classes
    for (scope_name, scope_ksy) in scopes.iter().rev() {
        let mut curr_ksy = *scope_ksy;
        let mut found = true;
        for part in &parts {
            if let Some(child_ksy) = curr_ksy.types.get(*part) {
                curr_ksy = child_ksy;
            } else {
                found = false;
                break;
            }
        }
        if found {
            let mut full_name = scope_name.to_vec();
            for part in &parts {
                full_name.push((*part).to_string());
            }
            return Ok((
                DataType::UserType {
                    names: full_name,
                    is_external: false,
                    args: type_args,
                },
                None,
            ));
        }
    }

    // 6. External user type from imports or registry
    let ext_name = parts.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if let Some(reg) = registry {
        if reg.specs.contains_key(parts[0]) {
            return Ok((
                DataType::UserType {
                    names: ext_name.clone(),
                    is_external: true,
                    args: type_args,
                },
                Some(ext_name),
            ));
        }
    }

    // Default to UserType with is_external = true
    Ok((
        DataType::UserType {
            names: ext_name.clone(),
            is_external: true,
            args: type_args,
        },
        Some(ext_name),
    ))
}

fn resolve_instance(
    _inst_id: &str,
    inst: &InstanceSpec,
    default_endian: Option<Endianness>,
    scopes: &[(&[String], &KsyFile)],
    registry: Option<&SpecRegistry>,
) -> Result<(
    DataType,
    Option<Expr>,
    Option<Expr>,
    Option<Expr>,
    Option<Expr>,
    Option<Expr>,
    bool,
)> {
    let value_expr = if let Some(v) = &inst.value {
        Some(match v {
            ValueOrExpr::Expr(s) => parse_expr(s)?,
            ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
            ValueOrExpr::Float(f) => Expr::FloatNum(*f),
            ValueOrExpr::Bool(b) => Expr::Bool(*b),
        })
    } else {
        None
    };

    let pos_expr = if let Some(p) = &inst.pos {
        Some(match p {
            ValueOrExpr::Expr(s) => parse_expr(s)?,
            ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
            ValueOrExpr::Float(f) => Expr::FloatNum(*f),
            ValueOrExpr::Bool(b) => Expr::Bool(*b),
        })
    } else {
        None
    };

    let size_expr = if let Some(s) = &inst.size {
        Some(match s {
            ValueOrExpr::Expr(s) => parse_expr(s)?,
            ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
            ValueOrExpr::Float(f) => Expr::FloatNum(*f),
            ValueOrExpr::Bool(b) => Expr::Bool(*b),
        })
    } else {
        None
    };

    let io_expr = if let Some(i_str) = &inst.io {
        Some(parse_expr(i_str)?)
    } else {
        None
    };

    let if_expr = if let Some(if_str) = &inst.if_expr {
        Some(parse_expr(if_str)?)
    } else {
        None
    };

    let is_cached = true; // Instances cache their calculation in Kaitai

    let dt = if let Some(t_spec) = &inst.type_spec {
        match t_spec {
            TypeSpec::Simple(s) => {
                if s == "str" || s == "strz" {
                    let size_expr = if let Some(sz) = &inst.size {
                        Some(match sz {
                            ValueOrExpr::Expr(e) => parse_expr(e)?,
                            ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                            ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                            ValueOrExpr::Bool(b) => Expr::Bool(*b),
                        })
                    } else {
                        None
                    };
                    DataType::Str {
                        size: size_expr,
                        size_eos: inst.size_eos.unwrap_or(false),
                        encoding: inst.encoding.clone(),
                        terminator: if s == "strz" {
                            inst.terminator.as_ref().and_then(|t| match t {
                                ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                                _ => None,
                            }).or(Some(0))
                        } else {
                            inst.terminator.as_ref().and_then(|t| match t {
                                ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                                _ => None,
                            })
                        },
                        consume: inst.consume.unwrap_or(true),
                        include: inst.include.unwrap_or(false),
                        pad_right: inst.pad_right.as_ref().and_then(|p| match p {
                            ValueOrExpr::Int(i) => u8::try_from(*i).ok(),
                            _ => None,
                        }),
                    }
                } else {
                    let (dt, _) = resolve_simple_type(
                        s,
                        inst.enum_name.as_deref(),
                        default_endian,
                        scopes,
                        registry,
                    )?;
                    dt
                }
            }
            TypeSpec::Switch(sw) => {
                let switch_expr = parse_expr(&sw.switch_on)?;
                let mut cases = IndexMap::new();
                for (case_key, val) in &sw.cases {
                    let case_type_str = match val {
                        serde_yaml::Value::String(s) => s.clone(),
                        other => format!("{other:?}"),
                    };
                    let (case_dt, _) = resolve_simple_type(
                        &case_type_str,
                        None,
                        default_endian,
                        scopes,
                        registry,
                    )?;
                    cases.insert(case_key.clone(), case_dt);
                }
                if (inst.size.is_some() || inst.size_eos == Some(true) || inst.process.is_some())
                    && !cases.contains_key("_")
                {
                    cases.insert(
                        "_".to_string(),
                        DataType::Bytes {
                            size: None,
                            size_eos: false,
                            terminator: None,
                            include: false,
                            consume: false,
                            pad_right: None,
                            process: inst.process.clone(),
                        },
                    );
                }
                DataType::SwitchType {
                    switch_on: switch_expr,
                    cases: sort_switch_cases(cases),
                }
            }
        }
    } else if let Some(enum_name) = &inst.enum_name {
        let mut owner = scopes.last().map_or_else(Vec::new, |s| s.0.to_vec());
        for (scope_name, scope_ksy) in scopes.iter().rev() {
            if scope_ksy.enums.contains_key(enum_name) {
                owner = scope_name.to_vec();
                break;
            }
        }
        DataType::EnumType {
            owner,
            name: enum_name.clone(),
            underlying: None,
        }
    } else if let Some(val_ex) = &value_expr {
        infer_expr_type(val_ex, scopes, registry).unwrap_or(DataType::CalcIntType)
    } else {
        DataType::CalcIntType
    };

    let dt = if let Some(rep) = &inst.repeat {
        match rep.as_str() {
            "eos" => DataType::ArrayType {
                element: Box::new(dt),
                repeat: RepeatMode::Eos,
            },
            "expr" => {
                if let Some(r_expr) = &inst.repeat_expr {
                    let parsed = match r_expr {
                        ValueOrExpr::Expr(s) => parse_expr(s)?,
                        ValueOrExpr::Int(i) => Expr::IntNum(i128::from(*i)),
                        ValueOrExpr::Float(f) => Expr::FloatNum(*f),
                        ValueOrExpr::Bool(b) => Expr::Bool(*b),
                    };
                    DataType::ArrayType {
                        element: Box::new(dt),
                        repeat: RepeatMode::Expr(parsed),
                    }
                } else {
                    dt
                }
            }
            "until" => {
                if let Some(u_str) = &inst.repeat_until {
                    DataType::ArrayType {
                        element: Box::new(dt),
                        repeat: RepeatMode::Until(parse_expr(u_str)?),
                    }
                } else {
                    dt
                }
            }
            _ => dt,
        }
    } else {
        dt
    };

    Ok((dt, value_expr, pos_expr, size_expr, io_expr, if_expr, is_cached))
}

fn combine_types(t1: &DataType, t2: &DataType) -> DataType {
    if t1 == t2 {
        return t1.clone();
    }
    match (t1, t2) {
        (DataType::Int1 { signed: s1 }, DataType::Int1 { signed: s2 }) => {
            if s1 != s2 {
                DataType::Int1 { signed: false }
            } else {
                t1.clone()
            }
        }
        (DataType::Int1 { .. }, DataType::IntMulti { signed, width, endian })
        | (DataType::IntMulti { signed, width, endian }, DataType::Int1 { .. }) => {
            DataType::IntMulti {
                signed: *signed,
                width: *width,
                endian: *endian,
            }
        }
        (
            DataType::IntMulti { signed: s1, width: w1, endian: e1 },
            DataType::IntMulti { signed: s2, width: w2, endian: e2 },
        ) => {
            if s1 == s2 {
                DataType::IntMulti {
                    signed: *s1,
                    width: (*w1).max(*w2),
                    endian: e1.or(*e2),
                }
            } else {
                DataType::CalcIntType
            }
        }
        (DataType::Int1 { .. } | DataType::IntMulti { .. } | DataType::CalcIntType,
         DataType::Int1 { .. } | DataType::IntMulti { .. } | DataType::CalcIntType) => {
            DataType::CalcIntType
        }
        (DataType::Float { width: w1, endian: e1 }, DataType::Float { width: w2, endian: e2 }) => {
            DataType::Float {
                width: (*w1).max(*w2),
                endian: e1.or(*e2),
            }
        }
        (DataType::CalcFloatType, DataType::Float { .. })
        | (DataType::Float { .. }, DataType::CalcFloatType) => DataType::CalcFloatType,
        _ => t1.clone(),
    }
}

fn infer_expr_type(
    expr: &Expr,
    scopes: &[(&[String], &KsyFile)],
    registry: Option<&SpecRegistry>,
) -> Option<DataType> {
    match expr {
        Expr::Bool(_) | Expr::Compare { .. } | Expr::BoolOp { .. } => {
            Some(DataType::CalcBoolType)
        }
        Expr::UnaryOp {
            op: UnaryOp::Not, ..
        } => Some(DataType::CalcBoolType),
        Expr::IntNum(x) => {
            if *x >= 0 && *x <= 127 {
                Some(DataType::Int1 { signed: true })
            } else if *x >= 0 && *x <= 255 {
                Some(DataType::Int1 { signed: false })
            } else {
                Some(DataType::CalcIntType)
            }
        }
        Expr::FloatNum(_) => Some(DataType::CalcFloatType),
        Expr::Str(_) => Some(DataType::CalcStrType),
        Expr::IfExp { if_true, if_false, .. } => {
            let t1 = infer_expr_type(if_true, scopes, registry);
            let t2 = infer_expr_type(if_false, scopes, registry);
            match (t1, t2) {
                (Some(a), Some(b)) => Some(combine_types(&a, &b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }
        Expr::Name(name) => {
            for (scope_name, scope_ksy) in scopes.iter().rev() {
                if let Some(attr) = scope_ksy.seq.iter().find(|a| a.id.as_deref() == Some(name)) {
                    if let Some(enum_name) = &attr.enum_name {
                        let mut owner = scope_name.to_vec();
                        for (s_name, s_ksy) in scopes.iter().rev() {
                            if s_ksy.enums.contains_key(enum_name) {
                                owner = s_name.to_vec();
                                break;
                            }
                        }
                        return Some(DataType::EnumType {
                            owner,
                            name: enum_name.clone(),
                            underlying: None,
                        });
                    }
                    if let Some(TypeSpec::Simple(s)) = &attr.type_spec {
                        if let Ok((dt, _)) = resolve_simple_type(s, None, None, scopes, registry) {
                            return Some(dt);
                        }
                    }
                }
                if let Some(inst) = scope_ksy.instances.get(name) {
                    if let Some(enum_name) = &inst.enum_name {
                        let mut owner = scope_name.to_vec();
                        for (s_name, s_ksy) in scopes.iter().rev() {
                            if s_ksy.enums.contains_key(enum_name) {
                                owner = s_name.to_vec();
                                break;
                            }
                        }
                        return Some(DataType::EnumType {
                            owner,
                            name: enum_name.clone(),
                            underlying: None,
                        });
                    }
                }
            }
            None
        }
        Expr::Subscript { value, .. } => {
            if let Some(DataType::ArrayType { element, .. }) = infer_expr_type(value, scopes, registry) {
                Some(*element)
            } else {
                None
            }
        }
        Expr::Attribute { value, attr } => {
            if attr == "to_i" {
                return Some(DataType::CalcIntType);
            }
            let target_ksy = if let Expr::Name(name) = &**value {
                if name == "_root" {
                    scopes.first().copied()
                } else if name == "_parent" {
                    if scopes.len() >= 2 {
                        scopes.get(scopes.len().saturating_sub(2)).copied()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let (target_scope_name, target_scope_ksy): (Vec<String>, &KsyFile) = if let Some((s_name, s_ksy)) = target_ksy {
                (s_name.to_vec(), s_ksy)
            } else if let Some(DataType::UserType { names, .. }) = infer_expr_type(value, scopes, registry) {
                let (_, root_ksy) = scopes.first()?;
                let ksy = find_ksy_file_by_path(root_ksy, &names)?;
                (names, ksy)
            } else {
                return None;
            };

            if let Some(seq_attr) = target_scope_ksy.seq.iter().find(|a| a.id.as_deref() == Some(attr)) {
                if let Some(enum_name) = &seq_attr.enum_name {
                    return Some(DataType::EnumType {
                        owner: target_scope_name.clone(),
                        name: enum_name.clone(),
                        underlying: None,
                    });
                }
                if let Some(TypeSpec::Simple(s)) = &seq_attr.type_spec {
                    let mut new_scopes = scopes.to_vec();
                    new_scopes.push((&target_scope_name, target_scope_ksy));
                    if let Ok((dt, _)) = resolve_simple_type(s, None, None, &new_scopes, registry) {
                        let final_dt = if seq_attr.repeat.is_some() {
                            DataType::ArrayType {
                                element: Box::new(dt),
                                repeat: RepeatMode::Eos,
                            }
                        } else {
                            dt
                        };
                        return Some(final_dt);
                    }
                }
            }
            if let Some(inst) = target_scope_ksy.instances.get(attr) {
                if let Some(enum_name) = &inst.enum_name {
                    return Some(DataType::EnumType {
                        owner: target_scope_name.clone(),
                        name: enum_name.clone(),
                        underlying: None,
                    });
                }
                if let Some(TypeSpec::Simple(s)) = &inst.type_spec {
                    let mut new_scopes = scopes.to_vec();
                    new_scopes.push((&target_scope_name, target_scope_ksy));
                    if let Ok((dt, _)) = resolve_simple_type(s, None, None, &new_scopes, registry) {
                        let final_dt = if inst.repeat.is_some() {
                            DataType::ArrayType {
                                element: Box::new(dt),
                                repeat: RepeatMode::Eos,
                            }
                        } else {
                            dt
                        };
                        return Some(final_dt);
                    }
                }
                if let Some(val) = &inst.value {
                    let val_ex = match val {
                        ValueOrExpr::Expr(s) => parse_expr(s).ok(),
                        ValueOrExpr::Int(i) => Some(Expr::IntNum(i128::from(*i))),
                        ValueOrExpr::Float(f) => Some(Expr::FloatNum(*f)),
                        ValueOrExpr::Bool(b) => Some(Expr::Bool(*b)),
                    };
                    if let Some(val_ex) = val_ex {
                        let mut new_scopes = scopes.to_vec();
                        new_scopes.push((&target_scope_name, target_scope_ksy));
                        return infer_expr_type(&val_ex, &new_scopes, registry);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn find_ksy_file_by_path<'a>(root_ksy: &'a KsyFile, path: &[String]) -> Option<&'a KsyFile> {
    if path.is_empty() {
        return None;
    }
    let mut cur = root_ksy;
    for part in &path[1..] {
        cur = cur.types.get(part)?;
    }
    Some(cur)
}

fn sort_switch_cases(cases: IndexMap<String, DataType>) -> IndexMap<String, DataType> {
    let mut entries: Vec<_> = cases.into_iter().collect();
    entries.sort_by(|(k1, _), (k2, _)| {
        if k1 == "_" {
            std::cmp::Ordering::Greater
        } else if k2 == "_" {
            std::cmp::Ordering::Less
        } else {
            k1.cmp(k2)
        }
    });
    let mut sorted = IndexMap::new();
    for (k, v) in entries {
        sorted.insert(k, v);
    }
    sorted
}

/* License information for parts derived from Kaitai Struct:

From https://raw.githubusercontent.com/kaitai-io/kaitai_struct_compiler/fd2594257834241455f27121ae1adf296bcf09b9/README.md:



## Licensing

### Main code

Kaitai Struct compiler itself is copyright (C) 2015-2026 Kaitai
Project.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

### FastParse

Portions of Kaitai Struct compiler are loosely based on
[pythonparse](https://github.com/com-lihaoyi/fastparse/tree/1.0.0/pythonparse/shared/src/main/scala/pythonparse)
from [FastParse](https://com-lihaoyi.github.io/fastparse/) and are copyright
(c) 2014 Li Haoyi (haoyi.sg@gmail.com).

Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

### XMLUtils code

Portions of Kaitai Struct compiler are based on `scala/xml/Utility.scala` from [Scala XML](https://github.com/scala/scala-xml).

Copyright (c) 2002-2017 EPFL\
Copyright (c) 2011-2017 Lightbend, Inc.

All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

* Redistributions of source code must retain the above copyright notice,
  this list of conditions and the following disclaimer.
* Redistributions in binary form must reproduce the above copyright notice,
  this list of conditions and the following disclaimer in the documentation
  and/or other materials provided with the distribution.
* Neither the name of the EPFL nor the names of its contributors may be
  used to endorse or promote products derived from this software without
  specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
THE POSSIBILITY OF SUCH DAMAGE.

### Libraries used

Kaitai Struct compiler depends on the following libraries:

* [scopt](https://github.com/scopt/scopt) — MIT license
* [fastparse](https://com-lihaoyi.github.io/fastparse/) — MIT license
* [snakeyaml](https://bitbucket.org/snakeyaml/snakeyaml) — Apache 2.0 license

---

Note that these clauses only apply only to compiler itself, not `.ksy`
input files that one supplies in normal process of compilation, nor to
compiler's output files — that consitutes normal usage process and you
obviously keep copyright to both.





From https://raw.githubusercontent.com/kaitai-io/kaitai_struct_compiler/fd2594257834241455f27121ae1adf296bcf09b9/LICENSE:


                    GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

  To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights.  Therefore, you have
certain responsibilities if you distribute copies of the software, or if
you modify it: responsibilities to respect the freedom of others.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received.  You must make sure that they, too, receive
or can get the source code.  And you must show them these terms so they
know their rights.

  Developers that use the GNU GPL protect your rights with two steps:
(1) assert copyright on the software, and (2) offer you this License
giving you legal permission to copy, distribute and/or modify it.

  For the developers' and authors' protection, the GPL clearly explains
that there is no warranty for this free software.  For both users' and
authors' sake, the GPL requires that modified versions be marked as
changed, so that their problems will not be attributed erroneously to
authors of previous versions.

  Some devices are designed to deny users access to install or run
modified versions of the software inside them, although the manufacturer
can do so.  This is fundamentally incompatible with the aim of
protecting users' freedom to change the software.  The systematic
pattern of such abuse occurs in the area of products for individuals to
use, which is precisely where it is most unacceptable.  Therefore, we
have designed this version of the GPL to prohibit the practice for those
products.  If such problems arise substantially in other domains, we
stand ready to extend this provision to those domains in future versions
of the GPL, as needed to protect the freedom of users.

  Finally, every program is threatened constantly by software patents.
States should not allow patents to restrict development and use of
software on general-purpose computers, but in those that do, we wish to
avoid the special danger that patents applied to a free program could
make it effectively proprietary.  To prevent this, the GPL assures that
patents cannot be used to render the program non-free.

  The precise terms and conditions for copying, distribution and
modification follow.

                       TERMS AND CONDITIONS

  0. Definitions.

  "This License" refers to version 3 of the GNU General Public License.

  "Copyright" also means copyright-like laws that apply to other kinds of
works, such as semiconductor masks.

  "The Program" refers to any copyrightable work licensed under this
License.  Each licensee is addressed as "you".  "Licensees" and
"recipients" may be individuals or organizations.

  To "modify" a work means to copy from or adapt all or part of the work
in a fashion requiring copyright permission, other than the making of an
exact copy.  The resulting work is called a "modified version" of the
earlier work or a work "based on" the earlier work.

  A "covered work" means either the unmodified Program or a work based
on the Program.

  To "propagate" a work means to do anything with it that, without
permission, would make you directly or secondarily liable for
infringement under applicable copyright law, except executing it on a
computer or modifying a private copy.  Propagation includes copying,
distribution (with or without modification), making available to the
public, and in some countries other activities as well.

  To "convey" a work means any kind of propagation that enables other
parties to make or receive copies.  Mere interaction with a user through
a computer network, with no transfer of a copy, is not conveying.

  An interactive user interface displays "Appropriate Legal Notices"
to the extent that it includes a convenient and prominently visible
feature that (1) displays an appropriate copyright notice, and (2)
tells the user that there is no warranty for the work (except to the
extent that warranties are provided), that licensees may convey the
work under this License, and how to view a copy of this License.  If
the interface presents a list of user commands or options, such as a
menu, a prominent item in the list meets this criterion.

  1. Source Code.

  The "source code" for a work means the preferred form of the work
for making modifications to it.  "Object code" means any non-source
form of a work.

  A "Standard Interface" means an interface that either is an official
standard defined by a recognized standards body, or, in the case of
interfaces specified for a particular programming language, one that
is widely used among developers working in that language.

  The "System Libraries" of an executable work include anything, other
than the work as a whole, that (a) is included in the normal form of
packaging a Major Component, but which is not part of that Major
Component, and (b) serves only to enable use of the work with that
Major Component, or to implement a Standard Interface for which an
implementation is available to the public in source code form.  A
"Major Component", in this context, means a major essential component
(kernel, window system, and so on) of the specific operating system
(if any) on which the executable work runs, or a compiler used to
produce the work, or an object code interpreter used to run it.

  The "Corresponding Source" for a work in object code form means all
the source code needed to generate, install, and (for an executable
work) run the object code and to modify the work, including scripts to
control those activities.  However, it does not include the work's
System Libraries, or general-purpose tools or generally available free
programs which are used unmodified in performing those activities but
which are not part of the work.  For example, Corresponding Source
includes interface definition files associated with source files for
the work, and the source code for shared libraries and dynamically
linked subprograms that the work is specifically designed to require,
such as by intimate data communication or control flow between those
subprograms and other parts of the work.

  The Corresponding Source need not include anything that users
can regenerate automatically from other parts of the Corresponding
Source.

  The Corresponding Source for a work in source code form is that
same work.

  2. Basic Permissions.

  All rights granted under this License are granted for the term of
copyright on the Program, and are irrevocable provided the stated
conditions are met.  This License explicitly affirms your unlimited
permission to run the unmodified Program.  The output from running a
covered work is covered by this License only if the output, given its
content, constitutes a covered work.  This License acknowledges your
rights of fair use or other equivalent, as provided by copyright law.

  You may make, run and propagate covered works that you do not
convey, without conditions so long as your license otherwise remains
in force.  You may convey covered works to others for the sole purpose
of having them make modifications exclusively for you, or provide you
with facilities for running those works, provided that you comply with
the terms of this License in conveying all material for which you do
not control copyright.  Those thus making or running the covered works
for you must do so exclusively on your behalf, under your direction
and control, on terms that prohibit them from making any copies of
your copyrighted material outside their relationship with you.

  Conveying under any other circumstances is permitted solely under
the conditions stated below.  Sublicensing is not allowed; section 10
makes it unnecessary.

  3. Protecting Users' Legal Rights From Anti-Circumvention Law.

  No covered work shall be deemed part of an effective technological
measure under any applicable law fulfilling obligations under article
11 of the WIPO copyright treaty adopted on 20 December 1996, or
similar laws prohibiting or restricting circumvention of such
measures.

  When you convey a covered work, you waive any legal power to forbid
circumvention of technological measures to the extent such circumvention
is effected by exercising rights under this License with respect to
the covered work, and you disclaim any intention to limit operation or
modification of the work as a means of enforcing, against the work's
users, your or third parties' legal rights to forbid circumvention of
technological measures.

  4. Conveying Verbatim Copies.

  You may convey verbatim copies of the Program's source code as you
receive it, in any medium, provided that you conspicuously and
appropriately publish on each copy an appropriate copyright notice;
keep intact all notices stating that this License and any
non-permissive terms added in accord with section 7 apply to the code;
keep intact all notices of the absence of any warranty; and give all
recipients a copy of this License along with the Program.

  You may charge any price or no price for each copy that you convey,
and you may offer support or warranty protection for a fee.

  5. Conveying Modified Source Versions.

  You may convey a work based on the Program, or the modifications to
produce it from the Program, in the form of source code under the
terms of section 4, provided that you also meet all of these conditions:

    a) The work must carry prominent notices stating that you modified
    it, and giving a relevant date.

    b) The work must carry prominent notices stating that it is
    released under this License and any conditions added under section
    7.  This requirement modifies the requirement in section 4 to
    "keep intact all notices".

    c) You must license the entire work, as a whole, under this
    License to anyone who comes into possession of a copy.  This
    License will therefore apply, along with any applicable section 7
    additional terms, to the whole of the work, and all its parts,
    regardless of how they are packaged.  This License gives no
    permission to license the work in any other way, but it does not
    invalidate such permission if you have separately received it.

    d) If the work has interactive user interfaces, each must display
    Appropriate Legal Notices; however, if the Program has interactive
    interfaces that do not display Appropriate Legal Notices, your
    work need not make them do so.

  A compilation of a covered work with other separate and independent
works, which are not by their nature extensions of the covered work,
and which are not combined with it such as to form a larger program,
in or on a volume of a storage or distribution medium, is called an
"aggregate" if the compilation and its resulting copyright are not
used to limit the access or legal rights of the compilation's users
beyond what the individual works permit.  Inclusion of a covered work
in an aggregate does not cause this License to apply to the other
parts of the aggregate.

  6. Conveying Non-Source Forms.

  You may convey a covered work in object code form under the terms
of sections 4 and 5, provided that you also convey the
machine-readable Corresponding Source under the terms of this License,
in one of these ways:

    a) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by the
    Corresponding Source fixed on a durable physical medium
    customarily used for software interchange.

    b) Convey the object code in, or embodied in, a physical product
    (including a physical distribution medium), accompanied by a
    written offer, valid for at least three years and valid for as
    long as you offer spare parts or customer support for that product
    model, to give anyone who possesses the object code either (1) a
    copy of the Corresponding Source for all the software in the
    product that is covered by this License, on a durable physical
    medium customarily used for software interchange, for a price no
    more than your reasonable cost of physically performing this
    conveying of source, or (2) access to copy the
    Corresponding Source from a network server at no charge.

    c) Convey individual copies of the object code with a copy of the
    written offer to provide the Corresponding Source.  This
    alternative is allowed only occasionally and noncommercially, and
    only if you received the object code with such an offer, in accord
    with subsection 6b.

    d) Convey the object code by offering access from a designated
    place (gratis or for a charge), and offer equivalent access to the
    Corresponding Source in the same way through the same place at no
    further charge.  You need not require recipients to copy the
    Corresponding Source along with the object code.  If the place to
    copy the object code is a network server, the Corresponding Source
    may be on a different server (operated by you or a third party)
    that supports equivalent copying facilities, provided you maintain
    clear directions next to the object code saying where to find the
    Corresponding Source.  Regardless of what server hosts the
    Corresponding Source, you remain obligated to ensure that it is
    available for as long as needed to satisfy these requirements.

    e) Convey the object code using peer-to-peer transmission, provided
    you inform other peers where the object code and Corresponding
    Source of the work are being offered to the general public at no
    charge under subsection 6d.

  A separable portion of the object code, whose source code is excluded
from the Corresponding Source as a System Library, need not be
included in conveying the object code work.

  A "User Product" is either (1) a "consumer product", which means any
tangible personal property which is normally used for personal, family,
or household purposes, or (2) anything designed or sold for incorporation
into a dwelling.  In determining whether a product is a consumer product,
doubtful cases shall be resolved in favor of coverage.  For a particular
product received by a particular user, "normally used" refers to a
typical or common use of that class of product, regardless of the status
of the particular user or of the way in which the particular user
actually uses, or expects or is expected to use, the product.  A product
is a consumer product regardless of whether the product has substantial
commercial, industrial or non-consumer uses, unless such uses represent
the only significant mode of use of the product.

  "Installation Information" for a User Product means any methods,
procedures, authorization keys, or other information required to install
and execute modified versions of a covered work in that User Product from
a modified version of its Corresponding Source.  The information must
suffice to ensure that the continued functioning of the modified object
code is in no case prevented or interfered with solely because
modification has been made.

  If you convey an object code work under this section in, or with, or
specifically for use in, a User Product, and the conveying occurs as
part of a transaction in which the right of possession and use of the
User Product is transferred to the recipient in perpetuity or for a
fixed term (regardless of how the transaction is characterized), the
Corresponding Source conveyed under this section must be accompanied
by the Installation Information.  But this requirement does not apply
if neither you nor any third party retains the ability to install
modified object code on the User Product (for example, the work has
been installed in ROM).

  The requirement to provide Installation Information does not include a
requirement to continue to provide support service, warranty, or updates
for a work that has been modified or installed by the recipient, or for
the User Product in which it has been modified or installed.  Access to a
network may be denied when the modification itself materially and
adversely affects the operation of the network or violates the rules and
protocols for communication across the network.

  Corresponding Source conveyed, and Installation Information provided,
in accord with this section must be in a format that is publicly
documented (and with an implementation available to the public in
source code form), and must require no special password or key for
unpacking, reading or copying.

  7. Additional Terms.

  "Additional permissions" are terms that supplement the terms of this
License by making exceptions from one or more of its conditions.
Additional permissions that are applicable to the entire Program shall
be treated as though they were included in this License, to the extent
that they are valid under applicable law.  If additional permissions
apply only to part of the Program, that part may be used separately
under those permissions, but the entire Program remains governed by
this License without regard to the additional permissions.

  When you convey a copy of a covered work, you may at your option
remove any additional permissions from that copy, or from any part of
it.  (Additional permissions may be written to require their own
removal in certain cases when you modify the work.)  You may place
additional permissions on material, added by you to a covered work,
for which you have or can give appropriate copyright permission.

  Notwithstanding any other provision of this License, for material you
add to a covered work, you may (if authorized by the copyright holders of
that material) supplement the terms of this License with terms:

    a) Disclaiming warranty or limiting liability differently from the
    terms of sections 15 and 16 of this License; or

    b) Requiring preservation of specified reasonable legal notices or
    author attributions in that material or in the Appropriate Legal
    Notices displayed by works containing it; or

    c) Prohibiting misrepresentation of the origin of that material, or
    requiring that modified versions of such material be marked in
    reasonable ways as different from the original version; or

    d) Limiting the use for publicity purposes of names of licensors or
    authors of the material; or

    e) Declining to grant rights under trademark law for use of some
    trade names, trademarks, or service marks; or

    f) Requiring indemnification of licensors and authors of that
    material by anyone who conveys the material (or modified versions of
    it) with contractual assumptions of liability to the recipient, for
    any liability that these contractual assumptions directly impose on
    those licensors and authors.

  All other non-permissive additional terms are considered "further
restrictions" within the meaning of section 10.  If the Program as you
received it, or any part of it, contains a notice stating that it is
governed by this License along with a term that is a further
restriction, you may remove that term.  If a license document contains
a further restriction but permits relicensing or conveying under this
License, you may add to a covered work material governed by the terms
of that license document, provided that the further restriction does
not survive such relicensing or conveying.

  If you add terms to a covered work in accord with this section, you
must place, in the relevant source files, a statement of the
additional terms that apply to those files, or a notice indicating
where to find the applicable terms.

  Additional terms, permissive or non-permissive, may be stated in the
form of a separately written license, or stated as exceptions;
the above requirements apply either way.

  8. Termination.

  You may not propagate or modify a covered work except as expressly
provided under this License.  Any attempt otherwise to propagate or
modify it is void, and will automatically terminate your rights under
this License (including any patent licenses granted under the third
paragraph of section 11).

  However, if you cease all violation of this License, then your
license from a particular copyright holder is reinstated (a)
provisionally, unless and until the copyright holder explicitly and
finally terminates your license, and (b) permanently, if the copyright
holder fails to notify you of the violation by some reasonable means
prior to 60 days after the cessation.

  Moreover, your license from a particular copyright holder is
reinstated permanently if the copyright holder notifies you of the
violation by some reasonable means, this is the first time you have
received notice of violation of this License (for any work) from that
copyright holder, and you cure the violation prior to 30 days after
your receipt of the notice.

  Termination of your rights under this section does not terminate the
licenses of parties who have received copies or rights from you under
this License.  If your rights have been terminated and not permanently
reinstated, you do not qualify to receive new licenses for the same
material under section 10.

  9. Acceptance Not Required for Having Copies.

  You are not required to accept this License in order to receive or
run a copy of the Program.  Ancillary propagation of a covered work
occurring solely as a consequence of using peer-to-peer transmission
to receive a copy likewise does not require acceptance.  However,
nothing other than this License grants you permission to propagate or
modify any covered work.  These actions infringe copyright if you do
not accept this License.  Therefore, by modifying or propagating a
covered work, you indicate your acceptance of this License to do so.

  10. Automatic Licensing of Downstream Recipients.

  Each time you convey a covered work, the recipient automatically
receives a license from the original licensors, to run, modify and
propagate that work, subject to this License.  You are not responsible
for enforcing compliance by third parties with this License.

  An "entity transaction" is a transaction transferring control of an
organization, or substantially all assets of one, or subdividing an
organization, or merging organizations.  If propagation of a covered
work results from an entity transaction, each party to that
transaction who receives a copy of the work also receives whatever
licenses to the work the party's predecessor in interest had or could
give under the previous paragraph, plus a right to possession of the
Corresponding Source of the work from the predecessor in interest, if
the predecessor has it or can get it with reasonable efforts.

  You may not impose any further restrictions on the exercise of the
rights granted or affirmed under this License.  For example, you may
not impose a license fee, royalty, or other charge for exercise of
rights granted under this License, and you may not initiate litigation
(including a cross-claim or counterclaim in a lawsuit) alleging that
any patent claim is infringed by making, using, selling, offering for
sale, or importing the Program or any portion of it.

  11. Patents.

  A "contributor" is a copyright holder who authorizes use under this
License of the Program or a work on which the Program is based.  The
work thus licensed is called the contributor's "contributor version".

  A contributor's "essential patent claims" are all patent claims
owned or controlled by the contributor, whether already acquired or
hereafter acquired, that would be infringed by some manner, permitted
by this License, of making, using, or selling its contributor version,
but do not include claims that would be infringed only as a
consequence of further modification of the contributor version.  For
purposes of this definition, "control" includes the right to grant
patent sublicenses in a manner consistent with the requirements of
this License.

  Each contributor grants you a non-exclusive, worldwide, royalty-free
patent license under the contributor's essential patent claims, to
make, use, sell, offer for sale, import and otherwise run, modify and
propagate the contents of its contributor version.

  In the following three paragraphs, a "patent license" is any express
agreement or commitment, however denominated, not to enforce a patent
(such as an express permission to practice a patent or covenant not to
sue for patent infringement).  To "grant" such a patent license to a
party means to make such an agreement or commitment not to enforce a
patent against the party.

  If you convey a covered work, knowingly relying on a patent license,
and the Corresponding Source of the work is not available for anyone
to copy, free of charge and under the terms of this License, through a
publicly available network server or other readily accessible means,
then you must either (1) cause the Corresponding Source to be so
available, or (2) arrange to deprive yourself of the benefit of the
patent license for this particular work, or (3) arrange, in a manner
consistent with the requirements of this License, to extend the patent
license to downstream recipients.  "Knowingly relying" means you have
actual knowledge that, but for the patent license, your conveying the
covered work in a country, or your recipient's use of the covered work
in a country, would infringe one or more identifiable patents in that
country that you have reason to believe are valid.

  If, pursuant to or in connection with a single transaction or
arrangement, you convey, or propagate by procuring conveyance of, a
covered work, and grant a patent license to some of the parties
receiving the covered work authorizing them to use, propagate, modify
or convey a specific copy of the covered work, then the patent license
you grant is automatically extended to all recipients of the covered
work and works based on it.

  A patent license is "discriminatory" if it does not include within
the scope of its coverage, prohibits the exercise of, or is
conditioned on the non-exercise of one or more of the rights that are
specifically granted under this License.  You may not convey a covered
work if you are a party to an arrangement with a third party that is
in the business of distributing software, under which you make payment
to the third party based on the extent of your activity of conveying
the work, and under which the third party grants, to any of the
parties who would receive the covered work from you, a discriminatory
patent license (a) in connection with copies of the covered work
conveyed by you (or copies made from those copies), or (b) primarily
for and in connection with specific products or compilations that
contain the covered work, unless you entered into that arrangement,
or that patent license was granted, prior to 28 March 2007.

  Nothing in this License shall be construed as excluding or limiting
any implied license or other defenses to infringement that may
otherwise be available to you under applicable patent law.

  12. No Surrender of Others' Freedom.

  If conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot convey a
covered work so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you may
not convey it at all.  For example, if you agree to terms that obligate you
to collect a royalty for further conveying from those to whom you convey
the Program, the only way you could satisfy both those terms and this
License would be to refrain entirely from conveying the Program.

  13. Use with the GNU Affero General Public License.

  Notwithstanding any other provision of this License, you have
permission to link or combine any covered work with a work licensed
under version 3 of the GNU Affero General Public License into a single
combined work, and to convey the resulting work.  The terms of this
License will continue to apply to the part which is the covered work,
but the special requirements of the GNU Affero General Public License,
section 13, concerning interaction through a network will apply to the
combination as such.

  14. Revised Versions of this License.

  The Free Software Foundation may publish revised and/or new versions of
the GNU General Public License from time to time.  Such new versions will
be similar in spirit to the present version, but may differ in detail to
address new problems or concerns.

  Each version is given a distinguishing version number.  If the
Program specifies that a certain numbered version of the GNU General
Public License "or any later version" applies to it, you have the
option of following the terms and conditions either of that numbered
version or of any later version published by the Free Software
Foundation.  If the Program does not specify a version number of the
GNU General Public License, you may choose any version ever published
by the Free Software Foundation.

  If the Program specifies that a proxy can decide which future
versions of the GNU General Public License can be used, that proxy's
public statement of acceptance of a version permanently authorizes you
to choose that version for the Program.

  Later license versions may give you additional or different
permissions.  However, no additional obligations are imposed on any
author or copyright holder as a result of your choosing to follow a
later version.

  15. Disclaimer of Warranty.

  THERE IS NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW.  EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT
HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM "AS IS" WITHOUT WARRANTY
OF ANY KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO,
THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE PROGRAM
IS WITH YOU.  SHOULD THE PROGRAM PROVE DEFECTIVE, YOU ASSUME THE COST OF
ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. Limitation of Liability.

  IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN WRITING
WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS
THE PROGRAM AS PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY
GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES ARISING OUT OF THE
USE OR INABILITY TO USE THE PROGRAM (INCLUDING BUT NOT LIMITED TO LOSS OF
DATA OR DATA BEING RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD
PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY OTHER PROGRAMS),
EVEN IF SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF
SUCH DAMAGES.

  17. Interpretation of Sections 15 and 16.

  If the disclaimer of warranty and limitation of liability provided
above cannot be given local legal effect according to their terms,
reviewing courts shall apply local law that most closely approximates
an absolute waiver of all civil liability in connection with the
Program, unless a warranty or assumption of liability accompanies a
copy of the Program in return for a fee.

                     END OF TERMS AND CONDITIONS

            How to Apply These Terms to Your New Programs

  If you develop a new program, and you want it to be of the greatest
possible use to the public, the best way to achieve this is to make it
free software which everyone can redistribute and change under these terms.

  To do so, attach the following notices to the program.  It is safest
to attach them to the start of each source file to most effectively
state the exclusion of warranty; and each file should have at least
the "copyright" line and a pointer to where the full notice is found.

    <one line to give the program's name and a brief idea of what it does.>
    Copyright (C) <year>  <name of author>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper mail.

  If the program does terminal interaction, make it output a short
notice like this when it starts in an interactive mode:

    <program>  Copyright (C) <year>  <name of author>
    This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
    This is free software, and you are welcome to redistribute it
    under certain conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License.  Of course, your program's commands
might be different; for a GUI interface, you would use an "about box".

  You should also get your employer (if you work as a programmer) or school,
if any, to sign a "copyright disclaimer" for the program, if necessary.
For more information on this, and how to apply and follow the GNU GPL, see
<https://www.gnu.org/licenses/>.

  The GNU General Public License does not permit incorporating your program
into proprietary programs.  If your program is a subroutine library, you
may consider it more useful to permit linking proprietary applications with
the library.  If this is what you want to do, use the GNU Lesser General
Public License instead of this License.  But first, please read
<https://www.gnu.org/licenses/why-not-lgpl.html>.

*/