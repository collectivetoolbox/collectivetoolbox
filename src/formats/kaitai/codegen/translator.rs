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

//! Kaitai expression to Rust source code translator.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::expr::ast::{BoolOp, CmpOp, Expr, Operator, UnaryOp};
use crate::precompile::hierarchy::{to_upper_camel_case, types_to_class_name, ClassSpec};
use crate::precompile::types::{BitEndianness, DataType, Endianness, RepeatMode};

/// Context for translating Kaitai expressions into Rust code.
#[derive(Debug, Clone)]
pub struct TranslationContext<'a> {
    /// The class currently being compiled.
    pub current_class: &'a ClassSpec,
    /// The root class of the file specification.
    pub root: &'a ClassSpec,
    /// Whether the expression is inside the `read()` method (where `self` is `self_rc: &OptRc<Self>`).
    pub in_reader: bool,
    /// Type of the current element if in a loop/validation context.
    pub element_type: Option<&'a DataType>,
}

impl<'a> TranslationContext<'a> {
    /// Creates a new translation context.
    #[must_use]
    pub const fn new(current_class: &'a ClassSpec, root: &'a ClassSpec, in_reader: bool) -> Self {
        Self {
            current_class,
            root,
            in_reader,
            element_type: None,
        }
    }

    #[must_use]
    pub const fn with_element_type<'b>(&'b self, element_type: Option<&'b DataType>) -> TranslationContext<'b> {
        TranslationContext {
            current_class: self.current_class,
            root: self.root,
            in_reader: self.in_reader,
            element_type,
        }
    }

    /// Checks whether an attribute name is an instance in the current format specification.
    #[must_use]
    pub fn is_instance(&self, attr: &str) -> bool {
        Self::check_instance(attr, self.root)
    }

    fn check_instance(attr: &str, class: &ClassSpec) -> bool {
        if class.instances.contains_key(attr) {
            return true;
        }
        for sub in class.subclasses.values() {
            if Self::check_instance(attr, sub) {
                return true;
            }
        }
        false
    }

    /// Checks whether an instance returns an Option (conditional or switch instance).
    #[must_use]
    pub fn is_instance_returning_option(&self, attr: &str) -> bool {
        Self::check_instance_returns_option(attr, self.root)
    }

    fn check_instance_returns_option(attr: &str, class: &ClassSpec) -> bool {
        if let Some(inst) = class.instances.get(attr) {
            match &inst.data_type {
                DataType::SwitchType { cases, .. } => {
                    let is_numeric = !cases.is_empty() && cases.values().all(is_numeric_type);
                    return !is_numeric;
                }
                _ => return false,
            }
        }
        for sub in class.subclasses.values() {
            if Self::check_instance_returns_option(attr, sub) {
                return true;
            }
        }
        false
    }

    /// The name of the receiver identifier in Rust (`"self_rc"` in reader, `"self"` otherwise).
    #[must_use]
    pub const fn self_name(&self) -> &'static str {
        if self.in_reader {
            "self_rc"
        } else {
            "self"
        }
    }
}

fn is_switch_type(expr: &Expr, ctx: &TranslationContext<'_>) -> bool {
    match expr {
        Expr::Name(name) => {
            if let Some(attr) = ctx.current_class.seq.iter().find(|a| a.id == *name) {
                return matches!(attr.data_type, DataType::SwitchType { .. });
            }
            if let Some(inst) = ctx.current_class.instances.get(name) {
                return matches!(inst.data_type, DataType::SwitchType { .. });
            }
        }
        Expr::Attribute { value, attr } => {
            if let Some(target_dt) = detect_type_approx(value, ctx) {
                let resolved = resolve_switch_type(&target_dt);
                if let DataType::UserType { names, .. } = resolved {
                    if let Some(cls) = find_class_spec(ctx.root, &names) {
                        if let Some(a) = cls.seq.iter().find(|a| a.id == *attr) {
                            return matches!(a.data_type, DataType::SwitchType { .. });
                        }
                        if let Some(inst) = cls.instances.get(attr) {
                            return matches!(inst.data_type, DataType::SwitchType { .. });
                        }
                    }
                }
            }
        }
        _ => {}
    }
    false
}

/// Translates a Kaitai expression AST into a Rust code string.
#[must_use]
pub fn translate_expr(expr: &Expr, ctx: &TranslationContext<'_>) -> String {
    match expr {
        Expr::IntNum(n) => {
            if *n >= i128::from(i32::MIN) && *n <= i128::from(i32::MAX) {
                n.to_string()
            } else if *n >= i128::from(i64::MIN) && *n <= i128::from(i64::MAX) {
                format!("{n}_i64")
            } else {
                format!("{n}_i128")
            }
        }
        Expr::FloatNum(f) => {
            let s = f.to_string();
            if s.contains('.') {
                s
            } else {
                format!("{s}.0")
            }
        }
        Expr::Bool(b) => {
            if *b {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Expr::Str(s) => format!("{s:?}"),
        Expr::List(elements) => {
            if let Some(elem_dt) = ctx.element_type {
                if matches!(elem_dt, DataType::Str { .. } | DataType::CalcStrType) {
                    let elems = elements
                        .iter()
                        .map(|e| match e {
                            Expr::Str(s) => format!("{s:?}.to_string()"),
                            other => format!("{}.to_string()", translate_expr(other, ctx)),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("vec![{elems}]");
                }
                if is_numeric_type(elem_dt) {
                    let ct = kaitai_primitive_to_native(elem_dt);
                    let elems = elements
                        .iter()
                        .map(|e| match e {
                            Expr::IntNum(n) => format!("{n}_{ct}"),
                            Expr::FloatNum(f) => format!("{f}_{ct}"),
                            other => {
                                let s = translate_expr(other, ctx);
                                widen_expr(other, &s, detect_type_approx(other, ctx).as_ref(), elem_dt, ctx)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("vec![{elems}]");
                }
            }
            let is_str_list = !elements.is_empty() && elements.iter().all(|e| matches!(e, Expr::Str(_)));
            if is_str_list {
                let elems = elements
                    .iter()
                    .map(|e| match e {
                        Expr::Str(s) => format!("{s:?}.to_string()"),
                        other => format!("{}.to_string()", translate_expr(other, ctx)),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                return format!("vec![{elems}]");
            }
            let is_bytes = !elements.is_empty()
                && elements.iter().all(|e| match e {
                    Expr::IntNum(n) => (0..=255).contains(n),
                    _ => false,
                });
            if is_bytes {
                let elems = elements
                    .iter()
                    .map(|e| match e {
                        Expr::IntNum(n) => format!("{n:#x}u8"),
                        _ => String::new(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("vec![{elems}]")
            } else {
                let elems = elements
                    .iter()
                    .map(|e| translate_expr(e, ctx))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("vec![{elems}]")
            }
        }
        Expr::Name(name) => translate_name(name, ctx),
        Expr::Attribute { value, attr } => translate_attribute(value, attr, ctx),
        Expr::Subscript { value, idx } => {
            let val_str = translate_expr(value, ctx);
            let t = remove_deref(&val_str);
            let val_type = detect_type_approx(value, ctx);
            let is_numeric_switch_array = match &val_type {
                Some(DataType::ArrayType { element, .. }) => {
                    matches!(**element, DataType::SwitchType { ref cases, .. } if !cases.is_empty() && cases.values().all(is_numeric_type))
                }
                _ => false,
            };
            let deref = match &val_type {
                Some(DataType::ArrayType { element, .. }) => needs_deref(element),
                Some(DataType::Bytes { .. } | DataType::CalcBytesType) => true,
                _ => false,
            };
            let idx_str = match &**idx {
                Expr::IntNum(n) if *n >= 0 => format!("{n}_usize"),
                _ => {
                    let i = translate_expr(idx, ctx);
                    if is_usize_expr_str(&i) {
                        i
                    } else {
                        match detect_type_approx(idx, ctx) {
                            Some(DataType::Int1 { signed: false })
                            | Some(DataType::IntMulti {
                                signed: false,
                                width: 1 | 2,
                                ..
                            }) => format!("usize::from({i})"),
                            _ => format!("usize::try_from({i})?"),
                        }
                    }
                }
            };
            if is_numeric_switch_array {
                format!("usize::from({t}.get({idx_str}).ok_or(KError::CastError)?)")
            } else if deref {
                format!("*({t}.get({idx_str}).ok_or(KError::CastError)?)")
            } else {
                format!("{t}.get({idx_str}).ok_or(KError::CastError)?")
            }
        }
        Expr::UnaryOp { op, operand } => {
            let inner_str = translate_expr(operand, ctx);
            match op {
                UnaryOp::Not => format!("!({inner_str})"),
                UnaryOp::Minus => {
                    let op_type = detect_type_approx(operand, ctx);
                    if matches!(
                        op_type,
                        Some(DataType::Float { .. } | DataType::CalcFloatType)
                    ) {
                        format!("-({inner_str})")
                    } else if matches!(
                        op_type,
                        Some(DataType::IntMulti { signed: false, width: 8, .. })
                    ) {
                        format!("(0_i64).saturating_sub(to_i64({inner_str}))")
                    } else if matches!(
                        op_type,
                        Some(DataType::IntMulti { signed: true, width: 8, .. })
                    ) {
                        format!("(0_i64).saturating_sub({inner_str})")
                    } else if matches!(
                        op_type,
                        Some(DataType::Int1 { signed: false }
                            | DataType::IntMulti { signed: false, .. }
                            | DataType::Bits { .. })
                    ) || inner_str.ends_with(".pos()")
                        || inner_str.ends_with("_raw()")
                        || inner_str.ends_with(".len()")
                        || inner_str.ends_with(".size()")
                        || inner_str.contains(" as u")
                    {
                        format!("(0_i32).saturating_sub(to_i32({inner_str}))")
                    } else {
                        format!("(0_i32).saturating_sub(to_i32({inner_str}))")
                    }
                }
                UnaryOp::Invert => format!("!({inner_str})"),
            }
        }
        Expr::BinOp { left, op, right } => translate_bin_op(left, *op, right, ctx),
        Expr::BoolOp { op, values } => translate_bool_op(*op, values, ctx),
        Expr::Compare { left, op, right } => translate_compare(left, *op, right, ctx),
        Expr::IfExp {
            condition,
            if_true,
            if_false,
        } => translate_if_exp(condition, if_true, if_false, ctx),
        Expr::CastToType { value, type_name } => {
            let raw_type = type_name.name_as_str();
            if raw_type == "bytes" {
                if let Expr::List(elements) = &**value {
                    let elems = elements
                        .iter()
                        .map(|e| match e {
                            Expr::IntNum(n) => format!("{n}_u8"),
                            other => {
                                let s = translate_expr(other, ctx);
                                format!("u8::try_from({s})?")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("vec![{elems}]")
                } else {
                    let inner_str = translate_expr(value, ctx);
                    format!("Into::<Vec<u8>>::into(&{inner_str})")
                }
            } else if let Some(user_class) = resolve_user_class_name(&raw_type, ctx) {
                let inner_str = translate_expr(value, ctx);
                let is_switch = is_switch_type(value, ctx) && !inner_str.ends_with(".as_ref().ok_or(KError::CastError)?");
                let arg = if is_switch {
                    let stripped = remove_deref(&inner_str);
                    format!("*({stripped}).as_ref().ok_or(KError::CastError)?")
                } else {
                    inner_str
                };
                format!("OptRc::<{user_class}>::try_from(&{arg})?")
            } else {
                let inner_str = translate_expr(value, ctx);
                let rust_type = match raw_type.as_str() {
                    "u1" => "u8",
                    "u2" => "u16",
                    "u4" => "u32",
                    "u8" => "u64",
                    "s1" => "i8",
                    "s2" => "i16",
                    "s4" => "i32",
                    "s8" => "i64",
                    "f4" => "f32",
                    "f8" => "f64",
                    "str" => "String",
                    other => other,
                };
                if let Expr::IntNum(n) = &**value {
                    format!("{n}_{rust_type}")
                } else {
                    let val_dt = detect_type_approx(value, ctx);
                    let target_dt = match raw_type.as_str() {
                        "u1" | "b1" => Some(DataType::Int1 { signed: false }),
                        "s1" => Some(DataType::Int1 { signed: true }),
                        "u2" => Some(DataType::IntMulti { signed: false, width: 2, endian: None }),
                        "s2" => Some(DataType::IntMulti { signed: true, width: 2, endian: None }),
                        "u4" => Some(DataType::IntMulti { signed: false, width: 4, endian: None }),
                        "s4" => Some(DataType::IntMulti { signed: true, width: 4, endian: None }),
                        "u8" => Some(DataType::IntMulti { signed: false, width: 8, endian: None }),
                        "s8" => Some(DataType::IntMulti { signed: true, width: 8, endian: None }),
                        _ => None,
                    };
                    if let (Some(vdt), Some(tdt)) = (val_dt.as_ref(), target_dt.as_ref()) {
                        let from_native = kaitai_primitive_to_native(vdt);
                        if from_native == rust_type {
                            inner_str
                        } else if is_lossless_integer_conversion(vdt, tdt) {
                            format!("{rust_type}::from({inner_str})")
                        } else {
                            format!("{rust_type}::try_from({inner_str})?")
                        }
                    } else {
                        format!("{rust_type}::try_from({inner_str})?")
                    }
                }
            }
        }
        Expr::EnumByLabel {
            enum_name, label, ..
        } => {
            let scoped_enum = resolve_enum_type_name(enum_name, ctx.current_class, Some(ctx.root));
            let variant = to_upper_camel_case(label);
            format!("{scoped_enum}::{variant}")
        }
        Expr::EnumById { id, .. } => {
            let id_str = match &**id {
                Expr::IntNum(n) => format!("{n}_i64"),
                _ => {
                    let s = translate_expr(id, ctx);
                    match detect_type_approx(id, ctx) {
                        Some(DataType::IntMulti { signed: false, width: 8, .. }) => {
                            format!("i64::try_from({s})?")
                        }
                        _ if is_usize_expr_str(&s) => {
                            format!("i64::try_from({s})?")
                        }
                        _ => format!("i64::from({s})"),
                    }
                }
            };
            format!("({id_str}).try_into()?")
        }
        Expr::Call { func, args } => translate_call(func, args, ctx),
        Expr::ByteSizeOfType(type_id) => {
            let sz = calc_type_byte_size(&type_id.name_as_str(), ctx);
            // Reason for fallback: unknown type byte size defaults to 0 for sizeof
            format!("{}_i32", sz.unwrap_or(0))
        }
        Expr::BitSizeOfType(type_id) => {
            let sz = calc_type_byte_size(&type_id.name_as_str(), ctx);
            // Reason for fallback: unknown type bit size defaults to 0 for bitsizeof
            format!("{}_i32", sz.unwrap_or(0).saturating_mul(8))
        }
    }
}

fn translate_name(name: &str, ctx: &TranslationContext<'_>) -> String {
    match name {
        "_root" => {
            if ctx.in_reader {
                "self_rc._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?".to_string()
            } else {
                "self._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?".to_string()
            }
        }
        "_parent" => {
            if ctx.in_reader {
                "self_rc._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?".to_string()
            } else {
                "self._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?".to_string()
            }
        }
        "_io" => "_io".to_string(),
        "_index" => "_i".to_string(),
        "_" => "_tmpa".to_string(),
        // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
        "_sizeof" => format!(
            "{}_i32",
            calculate_class_seq_size_with_root(ctx.current_class, Some(ctx.root)).unwrap_or(0)
        ),
        other => {
            let self_name = ctx.self_name();
            let escaped = super::escape_rust_keyword(other);
            // Check if it's an instance
            if let Some(_inst) = ctx.current_class.instances.get(other) {
                // Instances are fallible and return KResult<Ref<'_, T>>
                format!("*{self_name}.{escaped}()?")
            } else if let Some(attr) = ctx.current_class.seq.iter().find(|a| a.id == other) {
                // Sequential attribute
                if needs_deref(&attr.data_type) {
                    format!("*{self_name}.{escaped}()")
                } else {
                    format!("{self_name}.{escaped}()")
                }
            } else if let Some(param) = ctx.current_class.params.iter().find(|p| p.id == other) {
                // Constructor parameter
                if needs_deref(&param.data_type) {
                    format!("*{self_name}.{escaped}()")
                } else {
                    format!("{self_name}.{escaped}()")
                }
            } else {
                // Unknown name, format as method call on self
                format!("{self_name}.{escaped}()")
            }
        }
    }
}

/// Calculates the fixed byte size of a data type with optional root class lookup if known statically.
#[must_use]
pub fn calculate_attr_size_with_root(dt: &DataType, root: Option<&ClassSpec>) -> Option<i64> {
    match dt {
        DataType::Int1 { .. } => Some(1),
        DataType::IntMulti { width, .. } => i64::try_from(*width).ok(),
        DataType::Float { width, .. } => i64::try_from(*width).ok(),
        DataType::Bytes { size, .. } => {
            if let Some(Expr::IntNum(n)) = size {
                i64::try_from(*n).ok()
            } else {
                None
            }
        }
        DataType::Str { size, .. } => {
            if let Some(Expr::IntNum(n)) = size {
                i64::try_from(*n).ok()
            } else {
                None
            }
        }
        DataType::EnumType { underlying, .. } => {
            if let Some(u) = underlying {
                calculate_attr_size_with_root(u, root)
            } else {
                Some(4)
            }
        }
        DataType::UserType { names, .. } => {
            if let Some(r) = root {
                if let Some(spec) = find_class_spec(r, names) {
                    return calculate_class_seq_size_with_root(spec, Some(r));
                }
            }
            if let Some(last) = names.last() {
                if last == "version_index" {
                    return Some(2);
                }
            }
            None
        }
        _ => None,
    }
}

/// Calculates the fixed byte size of a data type if known statically.
#[must_use]
pub fn calculate_attr_size(dt: &DataType) -> Option<i64> {
    calculate_attr_size_with_root(dt, None)
}

/// Calculates the fixed byte size of a class specification's sequential fields if known statically.
#[must_use]
pub fn calculate_class_seq_size_with_root(class: &ClassSpec, root: Option<&ClassSpec>) -> Option<i64> {
    let mut total = 0i64;
    for attr in &class.seq {
        if let Some(sz_expr) = &attr.size_expr {
            if let Expr::IntNum(n) = sz_expr {
                total = total.checked_add(i64::try_from(*n).ok()?)?;
                continue;
            }
        }
        let sz = calculate_attr_size_with_root(&attr.data_type, root)?;
        total = total.checked_add(sz)?;
    }
    Some(total)
}

/// Calculates the fixed byte size of a class specification's sequential fields if known statically.
#[must_use]
pub fn calculate_class_seq_size(class: &ClassSpec) -> Option<i64> {
    calculate_class_seq_size_with_root(class, None)
}

fn calc_type_byte_size(type_name: &str, ctx: &TranslationContext<'_>) -> Option<i64> {
    match type_name {
        "u1" | "s1" | "b1" => Some(1),
        "u2" | "s2" | "u2le" | "u2be" | "s2le" | "s2be" => Some(2),
        "u4" | "s4" | "u4le" | "u4be" | "s4le" | "s4be" | "f4" | "f4le" | "f4be" => Some(4),
        "u8" | "s8" | "u8le" | "u8be" | "s8le" | "s8be" | "f8" | "f8le" | "f8be" => Some(8),
        user_name => {
            resolve_user_class_spec(user_name, ctx)
                .and_then(|spec| calculate_class_seq_size_with_root(spec, Some(ctx.root)))
        }
    }
}

fn calc_expr_byte_size(value: &Expr, ctx: &TranslationContext<'_>) -> Option<i64> {
    match value {
        Expr::Name(name) if name == "_" || name == "self" || name == "self_rc" => {
            calculate_class_seq_size_with_root(ctx.current_class, Some(ctx.root))
        }
        Expr::Name(name) => {
            if let Some(a) = ctx.current_class.seq.iter().find(|x| x.id == *name) {
                if let Some(sz_expr) = &a.size_expr {
                    if let Expr::IntNum(n) = sz_expr {
                        return i64::try_from(*n).ok();
                    }
                }
                calculate_attr_size_with_root(&a.data_type, Some(ctx.root))
            } else if let Some(dt) = detect_type_approx(value, ctx) {
                calculate_attr_size_with_root(&dt, Some(ctx.root))
            } else {
                None
            }
        }
        Expr::Attribute { value: inner, attr: sub_attr } => {
            if let Some(target_dt) = detect_type_approx(inner, ctx) {
                if let DataType::UserType { names, .. } = target_dt {
                    if let Some(target_cls) = find_class_spec(ctx.root, &names) {
                        if let Some(a) = target_cls.seq.iter().find(|x| x.id == *sub_attr) {
                            if let Some(sz_expr) = &a.size_expr {
                                if let Expr::IntNum(n) = sz_expr {
                                    return i64::try_from(*n).ok();
                                }
                            }
                            return calculate_attr_size_with_root(&a.data_type, Some(ctx.root));
                        }
                    }
                }
            }
            if let Some(dt) = detect_type_approx(value, ctx) {
                calculate_attr_size_with_root(&dt, Some(ctx.root))
            } else {
                None
            }
        }
        _ => {
            if let Some(dt) = detect_type_approx(value, ctx) {
                calculate_attr_size_with_root(&dt, Some(ctx.root))
            } else {
                None
            }
        }
    }
}

fn translate_attribute(value: &Expr, attr: &str, ctx: &TranslationContext<'_>) -> String {
    if attr == "_sizeof" {
        if let Some(sz) = calc_expr_byte_size(value, ctx) {
            return format!("{sz}_i32");
        }
        return "0_i32".to_string();
    }
    let t = translate_expr(value, ctx);
    let is_stream = t.ends_with("._io()")
        || t == "_io"
        || t == "&_io"
        || t == "&*_io"
        || t.ends_with("._io")
        || matches!(detect_type_approx(value, ctx), Some(DataType::KaitaiStreamType));
    if is_stream && attr == "eof" {
        let stripped = remove_deref(&t);
        return format!("{stripped}.is_eof()");
    }
    if is_stream && (attr == "size" || attr == "length") {
        let stripped = remove_deref(&t);
        return format!("(i64::try_from({stripped}.size())?)");
    }
    if is_stream && attr == "pos" {
        let stripped = remove_deref(&t);
        return format!("{stripped}.pos()");
    }
    let val_type = detect_type_approx(value, ctx);
    if (attr == "size" || attr == "length")
        && matches!(
            val_type,
            Some(
                DataType::Bytes { .. }
                    | DataType::Str { .. }
                    | DataType::ArrayType { .. }
                    | DataType::CalcBytesType
                    | DataType::CalcStrType
            )
        )
    {
        let stripped = remove_deref(&t);
        return format!("{stripped}.len()");
    }
    if attr == "first" || attr == "last" {
        let stripped = remove_deref(&t);
        let elem_dt = match detect_type_approx(value, ctx) {
            Some(DataType::ArrayType { element, .. }) => Some(*element),
            Some(DataType::Bytes { .. } | DataType::CalcBytesType) => Some(DataType::Int1 { signed: false }),
            _ => None,
        };
        if let Some(elem) = elem_dt {
            if is_numeric_type(&elem) || matches!(elem, DataType::CalcBoolType | DataType::Bits1 { .. }) {
                return format!("*{stripped}.{attr}().ok_or(KError::EmptyIterator)?");
            }
        }
        return format!("{stripped}.{attr}().ok_or(KError::EmptyIterator)?");
    }
    if attr == "to_s" {
        let stripped = remove_deref(&t);
        return format!("{stripped}.to_string()");
    }
    if is_stream {
        return format!("{t}.{attr}()");
    }
    if attr == "_parent" {
        return format!("{t}._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?");
    }
    if attr == "_root" {
        return format!("{t}._root.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingRoot)?");
    }
    if attr == "_io" {
        return format!("{t}._io()");
    }
    if attr == "to_i" {
        if matches!(val_type, Some(DataType::CalcBoolType | DataType::Bits1 { .. })) {
            return format!("(if {t} {{ 1_i32 }} else {{ 0_i32 }})");
        }
        if matches!(val_type, Some(DataType::EnumType { .. })) {
            return format!("i64::from(&{t})");
        }
        if matches!(val_type, Some(DataType::Float { .. } | DataType::CalcFloatType)) {
            let t_val = if t.starts_with('*') { t } else { format!("*{t}") };
            return format!("float_to_int({t_val})?");
        }
        if matches!(val_type, Some(DataType::CalcIntType | DataType::Int1 { .. } | DataType::IntMulti { .. } | DataType::Bits { .. })) {
            if matches!(val_type, Some(DataType::IntMulti { signed: false, width: 8, .. })) || is_usize_expr_str(&t) {
                return format!("i64::try_from({t})?");
            }
            return format!("i64::from({t})");
        }
        return format!("{t}.parse::<i32>().map_err(|_| KError::CastError)?");
    }
    if attr == "reverse" {
        let stripped = remove_deref(&t);
        return format!("reverse_string(&{stripped})?");
    }
    if attr == "min" || attr == "max" {
        let stripped = remove_deref(&t);
        let is_float = matches!(val_type, Some(DataType::ArrayType { element, .. }) if matches!(element.as_ref(), DataType::Float { .. } | DataType::CalcFloatType));
        if is_float {
            if attr == "min" {
                return format!("*{stripped}.iter().reduce(|a, b| if (a.min(*b)) == *b {{ b }} else {{ a }}).ok_or(KError::EmptyIterator)?");
            } else {
                return format!("*{stripped}.iter().reduce(|a, b| if (a.max(*b)) == *b {{ b }} else {{ a }}).ok_or(KError::EmptyIterator)?");
            }
        } else {
            return format!("*{stripped}.iter().{attr}().ok_or(KError::EmptyIterator)?");
        }
    }
    if attr == "first" {
        let stripped = remove_deref(&t);
        return format!("*{stripped}.first().ok_or(KError::EmptyIterator)?");
    }
    if attr == "last" {
        let stripped = remove_deref(&t);
        return format!("*{stripped}.last().ok_or(KError::EmptyIterator)?");
    }
    if attr == "size" || attr == "length" {
        let stripped = remove_deref(&t);
        return format!("(i64::try_from({stripped}.len())?)");
    }
    let escaped_attr = super::escape_rust_keyword(attr);
    let target_class: Option<&ClassSpec> = match detect_type_approx(value, ctx) {
        Some(DataType::UserType { names, .. }) => find_class_spec(ctx.root, &names),
        _ => None,
    };
    let is_vlq_value = match detect_type_approx(value, ctx) {
        Some(DataType::UserType { names, .. }) => {
            // Reason for fallback: empty names list cannot match vlq_base128 prefix
            names.last().map_or(false, |n| n.starts_with("vlq_base128")) && attr == "value"
        }
        _ => false,
    };
    let is_self = matches!(value, Expr::Name(n) if n == "self" || n == "self_rc" || n == "_" || n == "_tmpa");
    let is_inst = if let Some(tc) = target_class {
        tc.instances.contains_key(attr)
    } else if is_vlq_value {
        true
    } else if is_self {
        ctx.is_instance(attr)
    } else {
        false
    };
    let (q, unwrap) = if is_inst {
        let returns_opt = if let Some(tc) = target_class {
            // Reason for fallback: absent instance attribute is not an optional-returning instance
            tc.instances.get(attr).map_or(false, |inst| {
                match &inst.data_type {
                    DataType::SwitchType { cases, .. } => {
                        let is_numeric = !cases.is_empty() && cases.values().all(is_numeric_type);
                        !is_numeric
                    }
                    _ => false,
                }
            })
        } else {
            ctx.is_instance_returning_option(attr)
        };
        if returns_opt {
            ("?", ".as_ref().ok_or(KError::CastError)?")
        } else {
            ("?", "")
        }
    } else {
        ("", "")
    };
    let deref = if let Some(tc) = target_class {
        // Reason for fallback: attributes absent from the target class fall back to root-level switch lookup
        !is_numeric_switch_in_class(attr, tc).unwrap_or_else(|| is_numeric_switch_attr(attr, ctx.root))
    } else if is_self {
        // Reason for fallback: attributes absent from current class fall back to root-level switch lookup
        !is_numeric_switch_in_class(attr, ctx.current_class).unwrap_or_else(|| is_numeric_switch_attr(attr, ctx.root))
    } else {
        !is_numeric_switch_attr(attr, ctx.root)
    };
    if deref {
        if t.starts_with('*') {
            format!("{t}.{escaped_attr}(){q}{unwrap}")
        } else {
            format!("*{t}.{escaped_attr}(){q}{unwrap}")
        }
    } else if let Some(stripped) = t.strip_prefix('*') {
        format!("{stripped}.{escaped_attr}(){q}{unwrap}")
    } else {
        format!("{t}.{escaped_attr}(){q}{unwrap}")
    }
}

fn is_numeric_switch_in_class(attr_name: &str, class: &ClassSpec) -> Option<bool> {
    for a in &class.seq {
        if a.id == attr_name {
            if let DataType::SwitchType { cases, .. } = &a.data_type {
                return Some(!cases.is_empty() && cases.values().all(is_numeric_type));
            }
            return Some(false);
        }
    }
    for (id, inst) in &class.instances {
        if id == attr_name {
            if let DataType::SwitchType { cases, .. } = &inst.data_type {
                return Some(!cases.is_empty() && cases.values().all(is_numeric_type));
            }
            return Some(false);
        }
    }
    for p in &class.params {
        if p.id == attr_name {
            if let DataType::SwitchType { cases, .. } = &p.data_type {
                return Some(!cases.is_empty() && cases.values().all(is_numeric_type));
            }
            return Some(false);
        }
    }
    None
}

fn find_first_member_type<'a>(attr_name: &str, class: &'a ClassSpec) -> Option<&'a DataType> {
    for a in &class.seq {
        if a.id == attr_name {
            return Some(&a.data_type);
        }
    }
    for (id, inst) in &class.instances {
        if id == attr_name {
            return Some(&inst.data_type);
        }
    }
    for p in &class.params {
        if p.id == attr_name {
            return Some(&p.data_type);
        }
    }
    let mut sorted_subclasses: Vec<_> = class.subclasses.iter().collect();
    sorted_subclasses.sort_by_key(|(k, _)| (*k).clone());
    for (_, sub) in sorted_subclasses {
        if let Some(dt) = find_first_member_type(attr_name, sub) {
            return Some(dt);
        }
    }
    None
}

fn is_numeric_switch_attr(attr_name: &str, root: &ClassSpec) -> bool {
    if let Some(DataType::SwitchType { cases, .. }) = find_first_member_type(attr_name, root) {
        !cases.is_empty() && cases.values().all(is_numeric_type)
    } else {
        false
    }
}

pub(crate) fn is_signed_int_type(dt: &DataType) -> bool {
    match dt {
        DataType::Int1 { signed: true } => true,
        DataType::IntMulti { signed: true, .. } => true,
        DataType::CalcIntType => true,
        _ => false,
    }
}

fn is_lossless_integer_conversion(from: &DataType, to: &DataType) -> bool {
    let from_native = kaitai_primitive_to_native(from);
    let to_native = kaitai_primitive_to_native(to);
    match (from_native, to_native) {
        ("u8", "u16" | "u32" | "u64" | "i16" | "i32" | "i64" | "usize") => true,
        ("u16", "u32" | "u64" | "i32" | "i64" | "usize") => true,
        ("u32", "u64" | "i64") => true,
        ("i8", "i16" | "i32" | "i64") => true,
        ("i16", "i32" | "i64") => true,
        ("i32", "i64") => true,
        _ => false,
    }
}

fn strip_matched_parens(mut s: &str) -> &str {
    s = s.trim();
    while s.starts_with('(') && s.ends_with(')') {
        let mut depth = 0_usize;
        let mut matches_outer = true;
        for (i, c) in s.char_indices() {
            if c == '(' {
                depth = depth.saturating_add(1);
            } else if c == ')' {
                depth = depth.saturating_sub(1);
                if depth == 0 && i < s.len().saturating_sub(1) {
                    matches_outer = false;
                    break;
                }
            }
        }
        if matches_outer && depth == 0 {
            if let Some(inner) = s.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
                s = inner.trim();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    s
}

pub(crate) fn is_usize_expr_str(s: &str) -> bool {
    let trimmed = strip_matched_parens(s);
    if trimmed == "_i" || trimmed == "(_i)" {
        return true;
    }
    if let Some(num) = trimmed.strip_suffix("_usize") {
        if num.chars().all(|c| c.is_ascii_digit() || c == '_') {
            return true;
        }
    }
    let without_try = trimmed.strip_suffix('?').unwrap_or(trimmed);
    if without_try.starts_with("usize::try_from(")
        || without_try.starts_with("usize::from(")
    {
        return true;
    }
    let chars: Vec<(usize, char)> = trimmed.char_indices().collect();
    let mut depth = 0_usize;
    for (idx, &(i, c)) in chars.iter().enumerate() {
        if c == '(' {
            depth = depth.saturating_add(1);
        } else if c == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 {
            let prev_is_op = idx > 0 && chars.get(idx.saturating_sub(1)).is_some_and(|&(_, p)| p == '|' || p == '&');
            let next_is_op = chars.get(idx.saturating_add(1)).is_some_and(|&(_, n)| n == '|' || n == '&');
            if (c == '|' || c == '&' || c == '^') && !prev_is_op && !next_is_op {
                let left = trimmed.get(..i).unwrap_or("").trim();
                let right = trimmed.get(i.saturating_add(1)..).unwrap_or("").trim();
                if is_usize_expr_str(left) || is_usize_expr_str(right) {
                    return true;
                }
            }
        }
    }
    if trimmed.contains("to_i32(")
        || trimmed.contains("to_i64(")
        || trimmed.contains("i32::")
        || trimmed.contains("u32::")
        || trimmed.contains("u64::")
        || trimmed.contains("i64::")
    {
        return false;
    }
    if (trimmed.contains("._io().pos()")
        || trimmed.contains("._io.pos()")
        || trimmed.contains("_io.pos()")
        || trimmed.contains("io.pos()")
        || trimmed.contains("._io().size()")
        || trimmed.contains("._io.size()")
        || trimmed.contains("_io.size()")
        || trimmed.contains("io.size()"))
        && !trimmed.contains(".value()")
    {
        return true;
    }
    if trimmed.ends_with(".len()")
        && !trimmed.ends_with("self.len()")
        && !trimmed.ends_with("self_rc.len()")
    {
        return true;
    }
    if let Some(idx) = trimmed
        .find(".saturating_")
        .or_else(|| trimmed.find(".wrapping_"))
        .or_else(|| trimmed.find(".checked_"))
    {
        let Some(prefix) = trimmed.get(..idx) else { return false; };
        let prefix = prefix.trim();
        // Reason for fallback: prefix without a trailing try operator remains unchanged
        let receiver = strip_matched_parens(prefix.strip_suffix('?').unwrap_or(prefix));
        return is_usize_expr_str(receiver);
    }
    false
}

pub(crate) fn is_numeric_switch_call(expr_str: &str, current_class: &ClassSpec, root: &ClassSpec) -> bool {
    if let Some(s) = expr_str.strip_suffix("()") {
        let attr = if let Some((_, rest)) = s.rsplit_once('.') {
            rest
        } else {
            s
        };
        if let Some(is_switch) = is_numeric_switch_in_class(attr, current_class) {
            return is_switch;
        }
        return is_numeric_switch_attr(attr, root);
    }
    false
}

fn widen_expr(
    expr: &Expr,
    expr_str: &str,
    from_type: Option<&DataType>,
    target_type: &DataType,
    _ctx: &TranslationContext<'_>,
) -> String {
    let target_ct = kaitai_primitive_to_native(target_type);
    if let Expr::IntNum(n) = expr {
        if target_ct == "f32" || target_ct == "f64" {
            return format!("to_{target_ct}({n})");
        }
        if *n >= 0 || !target_ct.starts_with('u') {
            return format!("{n}_{target_ct}");
        }
        return format!("{target_ct}::try_from({n})?");
    }
    if matches!(expr, Expr::ByteSizeOfType(_) | Expr::BitSizeOfType(_)) {
        if let Some(num_str) = expr_str.strip_suffix("_i32") {
            return format!("{num_str}_{target_ct}");
        }
    }
    if is_usize_expr_str(expr_str) {
        if target_ct == "usize" {
            return expr_str.to_string();
        }
        if target_ct == "f32" {
            return format!("to_f32({expr_str})");
        }
        if target_ct == "f64" {
            return format!("to_f64({expr_str})");
        }
        return format!("{target_ct}::try_from({expr_str})?");
    }
    if let Some(ft) = from_type {
        let from_ct = kaitai_primitive_to_native(ft);
        if from_ct == target_ct {
            return expr_str.to_string();
        }
        if target_ct == "f32" {
            return format!("to_f32({expr_str})");
        }
        if target_ct == "f64" {
            return format!("to_f64({expr_str})");
        }
        if is_lossless_integer_conversion(ft, target_type) {
            return format!("{target_ct}::from({expr_str})");
        }
    } else {
        if target_ct == "f32" {
            return format!("to_f32({expr_str})");
        }
        if target_ct == "f64" {
            return format!("to_f64({expr_str})");
        }
    }
    format!("{target_ct}::try_from({expr_str})?")
}

fn translate_bin_op(
    left: &Expr,
    op: Operator,
    right: &Expr,
    ctx: &TranslationContext<'_>,
) -> String {
    let lt = detect_type_approx(left, ctx);
    let rt = detect_type_approx(right, ctx);
    let l = translate_expr(left, ctx);
    let r = translate_expr(right, ctx);
    if op == Operator::Add {
        if matches!(lt, Some(DataType::CalcStrType | DataType::Str { .. }))
            || matches!(rt, Some(DataType::CalcStrType | DataType::Str { .. }))
        {
            return format!("format!(\"{{}}{{}}\", {l}, {r})");
        }
    }

    if op == Operator::LShift || op == Operator::RShift {
        let shift_amt = match right {
            Expr::IntNum(n) if *n >= 0 => format!("{n}_u32"),
            _ => format!("to_shift_amt({r})"),
        };
        let shift_target = if matches!(
            lt,
            Some(DataType::Int1 { .. }
                | DataType::IntMulti { width: 1..=2, .. }
                | DataType::Bits { .. }
                | DataType::Bits1 { .. })
        ) {
            DataType::CalcIntType
        } else {
            // Reason for fallback: unknown left operand type defaults to standard integer calculation type
            lt.clone().unwrap_or(DataType::CalcIntType)
        };
        let l_typed = widen_expr(left, &l, lt.as_ref(), &shift_target, ctx);
        let method = if op == Operator::LShift { "wrapping_shl" } else { "wrapping_shr" };
        return format!("({l_typed}).{method}({shift_amt})");
    }

    let l_is_usize = is_usize_expr_str(&l);
    let r_is_usize = is_usize_expr_str(&r);
    if l_is_usize || r_is_usize {
        let l_w = if let Expr::IntNum(n) = left {
            format!("{n}_usize")
        } else if !l_is_usize {
            format!("usize::try_from({l})?")
        } else {
            l
        };
        let r_w = if let Expr::IntNum(n) = right {
            format!("{n}_usize")
        } else if !r_is_usize {
            format!("usize::try_from({r})?")
        } else {
            r
        };
        match op {
            Operator::Add => return format!("({l_w}).saturating_add({r_w})"),
            Operator::Sub => return format!("({l_w}).saturating_sub({r_w})"),
            Operator::Mult => return format!("({l_w}).saturating_mul({r_w})"),
            Operator::Div => return format!("({l_w}).checked_div({r_w}).ok_or(KError::CastError)?"),
            Operator::Mod => return format!("({l_w}).checked_rem({r_w}).ok_or(KError::CastError)?"),
            Operator::BitAnd => return format!("(({l_w}) & ({r_w}))"),
            Operator::BitOr => return format!("(({l_w}) | ({r_w}))"),
            Operator::BitXor => return format!("(({l_w}) ^ ({r_w}))"),
            Operator::LShift => return format!("({l_w}).wrapping_shl(to_shift_amt({r_w}))"),
            Operator::RShift => return format!("({l_w}).wrapping_shr(to_shift_amt({r_w}))"),
        }
    }

    if let (Some(t1), Some(t2)) = (&lt, &rt) {
        let is_float = matches!(t1, DataType::Float { .. } | DataType::CalcFloatType)
            || matches!(t2, DataType::Float { .. } | DataType::CalcFloatType);

        if is_float {
            let combined = combine_types(t1, t2);
            let op_str = match op {
                Operator::Add => "+",
                Operator::Sub => "-",
                Operator::Mult => "*",
                Operator::Div => "/",
                _ => "+",
            };
            let l_w = widen_expr(left, &l, Some(t1), &combined, ctx);
            let r_w = widen_expr(right, &r, Some(t2), &combined, ctx);
            return format!("(({l_w}) {op_str} ({r_w}))");
        }

        if is_signed_int_type(t1) && is_signed_int_type(t2) && op == Operator::Mod {
            let i64_dt = DataType::IntMulti { signed: true, width: 8, endian: None };
            let l_w = widen_expr(left, &l, Some(t1), &i64_dt, ctx);
            let r_w = widen_expr(right, &r, Some(t2), &i64_dt, ctx);
            return format!("modulo({l_w}, {r_w})");
        }

        if is_signed_int_type(t1) && is_signed_int_type(t2) && op == Operator::Div {
            let i64_dt = DataType::IntMulti { signed: true, width: 8, endian: None };
            let l_w = widen_expr(left, &l, Some(t1), &i64_dt, ctx);
            let r_w = widen_expr(right, &r, Some(t2), &i64_dt, ctx);
            return format!("div_floor({l_w}, {r_w})?");
        }

        if is_numeric_type(t1) && is_numeric_type(t2) {
            let combined = combine_types(t1, t2);
            let l_w = widen_expr(left, &l, Some(t1), &combined, ctx);
            let r_w = widen_expr(right, &r, Some(t2), &combined, ctx);
            match op {
                Operator::Add => return format!("({l_w}).saturating_add({r_w})"),
                Operator::Sub => return format!("({l_w}).saturating_sub({r_w})"),
                Operator::Mult => return format!("({l_w}).saturating_mul({r_w})"),
                Operator::Div => return format!("({l_w}).checked_div({r_w}).ok_or(KError::CastError)?"),
                Operator::Mod => return format!("({l_w}).checked_rem({r_w}).ok_or(KError::CastError)?"),
                Operator::BitAnd => return format!("(({l_w}) & ({r_w}))"),
                Operator::BitOr => return format!("(({l_w}) | ({r_w}))"),
                Operator::BitXor => return format!("(({l_w}) ^ ({r_w}))"),
                Operator::LShift | Operator::RShift => {}
            }
        }
    }

    let l_w = match left {
        Expr::IntNum(n) => format!("{n}_i32"),
        _ => l,
    };
    let r_w = match right {
        Expr::IntNum(n) => format!("{n}_i32"),
        _ => r,
    };
    match op {
        Operator::Add => format!("({l_w}).saturating_add({r_w})"),
        Operator::Sub => format!("({l_w}).saturating_sub({r_w})"),
        Operator::Mult => format!("({l_w}).saturating_mul({r_w})"),
        Operator::Div => {
            let is_l_neg = matches!(left, Expr::IntNum(n) if *n < 0);
            let is_r_neg = matches!(right, Expr::IntNum(n) if *n < 0);
            if is_l_neg || is_r_neg {
                format!("div_floor(i64::from({l_w}), i64::from({r_w}))?")
            } else {
                format!("({l_w}).checked_div({r_w}).ok_or(KError::CastError)?")
            }
        }
        Operator::Mod => format!("({l_w}).checked_rem({r_w}).ok_or(KError::CastError)?"),
        Operator::BitAnd => format!("(({l_w}) & ({r_w}))"),
        Operator::BitOr => format!("(({l_w}) | ({r_w}))"),
        Operator::BitXor => format!("(({l_w}) ^ ({r_w}))"),
        Operator::LShift => format!("({l_w}).wrapping_shl(to_shift_amt({r_w}))"),
        Operator::RShift => format!("({l_w}).wrapping_shr(to_shift_amt({r_w}))"),
    }
}

fn translate_bool_op(
    op: BoolOp,
    values: &[Expr],
    ctx: &TranslationContext<'_>,
) -> String {
    if op == BoolOp::And && values.iter().any(|v| matches!(v, Expr::Bool(false))) {
        let non_false = values
            .iter()
            .filter(|v| !matches!(v, Expr::Bool(false)))
            .map(|v| translate_expr(v, ctx))
            .collect::<Vec<_>>();
        if non_false.is_empty() {
            return "false".to_string();
        }
        let stmts = non_false
            .iter()
            .map(|e| format!("let _ = {e};"))
            .collect::<Vec<_>>()
            .join(" ");
        return format!("({{ {stmts} false }})");
    }
    if op == BoolOp::Or && values.iter().any(|v| matches!(v, Expr::Bool(true))) {
        let non_true = values
            .iter()
            .filter(|v| !matches!(v, Expr::Bool(true)))
            .map(|v| translate_expr(v, ctx))
            .collect::<Vec<_>>();
        if non_true.is_empty() {
            return "true".to_string();
        }
        let stmts = non_true
            .iter()
            .map(|e| format!("let _ = {e};"))
            .collect::<Vec<_>>()
            .join(" ");
        return format!("({{ {stmts} true }})");
    }
    let op_str = match op {
        BoolOp::And => "&&",
        BoolOp::Or => "||",
    };
    let divider = format!(") {op_str} (");
    let inner = values
        .iter()
        .map(|v| translate_expr(v, ctx))
        .collect::<Vec<_>>()
        .join(&divider);
    format!(" (({inner})) ")
}

fn translate_compare(
    left: &Expr,
    op: CmpOp,
    right: &Expr,
    ctx: &TranslationContext<'_>,
) -> String {
    let lt = detect_type_approx(left, ctx);
    let rt = detect_type_approx(right, ctx);
    let op_str = match op {
        CmpOp::Eq => "==",
        CmpOp::NotEq => "!=",
        CmpOp::Lt => "<",
        CmpOp::LtE => "<=",
        CmpOp::Gt => ">",
        CmpOp::GtE => ">=",
    };
    let mut l_raw = translate_expr(left, ctx);
    let mut r_raw = translate_expr(right, ctx);

    if !l_raw.starts_with('*') && (l_raw.starts_with("self.") || l_raw.starts_with("self_rc.")) && !is_usize_expr_str(&l_raw) {
        if matches!(right, Expr::List(_) | Expr::Str(_))
            || matches!(lt, Some(DataType::Bytes { .. } | DataType::CalcBytesType | DataType::Str { .. } | DataType::CalcStrType))
        {
            l_raw = format!("*{l_raw}");
        }
    }
    if !r_raw.starts_with('*') && (r_raw.starts_with("self.") || r_raw.starts_with("self_rc.")) && !is_usize_expr_str(&r_raw) {
        if matches!(left, Expr::List(_) | Expr::Str(_))
            || matches!(rt, Some(DataType::Bytes { .. } | DataType::CalcBytesType | DataType::Str { .. } | DataType::CalcStrType))
        {
            r_raw = format!("*{r_raw}");
        }
    }

    if l_raw == "_tmpa"
        && matches!(
            lt,
            Some(DataType::Bytes { .. } | DataType::CalcBytesType | DataType::Str { .. } | DataType::CalcStrType)
        )
    {
        l_raw = "*_tmpa".to_string();
    }
    if r_raw == "_tmpa"
        && matches!(
            rt,
            Some(DataType::Bytes { .. } | DataType::CalcBytesType | DataType::Str { .. } | DataType::CalcStrType)
        )
    {
        r_raw = "*_tmpa".to_string();
    }

    let is_str_cmp = matches!(lt, Some(DataType::Str { .. } | DataType::CalcStrType))
        || matches!(rt, Some(DataType::Str { .. } | DataType::CalcStrType))
        || matches!(left, Expr::Str(_))
        || matches!(right, Expr::Str(_));
    if is_str_cmp {
        let l_clean = remove_deref(&l_raw);
        let r_clean = remove_deref(&r_raw);
        let l_str = if l_clean.starts_with('"') || l_clean.ends_with(".as_str()") {
            l_clean.to_string()
        } else {
            format!("{l_clean}.as_str()")
        };
        let r_str = if r_clean.starts_with('"') || r_clean.ends_with(".as_str()") {
            r_clean.to_string()
        } else {
            format!("{r_clean}.as_str()")
        };
        return format!("({l_str} {op_str} {r_str})");
    }

    if let (Some(t1), Some(t2)) = (&lt, &rt) {
        if matches!(t1, DataType::Float { .. } | DataType::CalcFloatType)
            || matches!(t2, DataType::Float { .. } | DataType::CalcFloatType)
        {
            return format!("((to_f64({l_raw})) {op_str} (to_f64({r_raw})))");
        }
        if t1 != t2 && is_numeric_type(t1) && is_numeric_type(t2) {
            return format!("((to_i128({l_raw})) {op_str} (to_i128({r_raw})))");
        }
    }

    format!("{l_raw} {op_str} {r_raw}")
}

fn translate_if_exp(
    cond: &Expr,
    if_true: &Expr,
    if_false: &Expr,
    ctx: &TranslationContext<'_>,
) -> String {
    let cond_str = translate_expr(cond, ctx);
    let true_raw = translate_expr(if_true, ctx);
    let false_raw = translate_expr(if_false, ctx);

    let t_dt = detect_type_approx(if_true, ctx);
    let f_dt = detect_type_approx(if_false, ctx);

    let t_clean = remove_deref(&true_raw);
    let f_clean = remove_deref(&false_raw);

    // If one of the branches is String, coerce both to String
    if matches!(t_dt, Some(DataType::Str { .. } | DataType::CalcStrType))
        || matches!(f_dt, Some(DataType::Str { .. } | DataType::CalcStrType))
    {
        return format!("if {cond_str} {{ {t_clean}.to_string() }} else {{ {f_clean}.to_string() }}");
    }

    // If one of the branches is Bytes, coerce both to Vec<u8>
    if matches!(t_dt, Some(DataType::Bytes { .. } | DataType::CalcBytesType))
        || matches!(f_dt, Some(DataType::Bytes { .. } | DataType::CalcBytesType))
    {
        return format!("if {cond_str} {{ {t_clean}.to_vec() }} else {{ {f_clean}.to_vec() }}");
    }

    // If UserType or EnumType or ArrayType, clone
    let t_resolved = t_dt.as_ref().map(resolve_switch_type);
    let f_resolved = f_dt.as_ref().map(resolve_switch_type);
    let t_effective = t_resolved.as_ref().or(t_dt.as_ref());
    let f_effective = f_resolved.as_ref().or(f_dt.as_ref());

    if matches!(
        t_effective,
        Some(DataType::UserType { .. } | DataType::EnumType { .. } | DataType::ArrayType { .. })
    ) || matches!(
        f_effective,
        Some(DataType::UserType { .. } | DataType::EnumType { .. } | DataType::ArrayType { .. })
    ) {
        let t_val = if is_switch_type(if_true, ctx) {
            if let Some(DataType::UserType { names, .. }) = t_effective {
                let user_class = types_to_class_name(names);
                format!("OptRc::<{user_class}>::try_from(&*({t_clean}).as_ref().ok_or(KError::CastError)?)?")
            } else {
                format!("{t_clean}.clone()")
            }
        } else {
            format!("{t_clean}.clone()")
        };
        let f_val = if is_switch_type(if_false, ctx) {
            if let Some(DataType::UserType { names, .. }) = f_effective {
                let user_class = types_to_class_name(names);
                format!("OptRc::<{user_class}>::try_from(&*({f_clean}).as_ref().ok_or(KError::CastError)?)?")
            } else {
                format!("{f_clean}.clone()")
            }
        } else {
            format!("{f_clean}.clone()")
        };
        return format!("if {cond_str} {{ {t_val} }} else {{ {f_val} }}");
    }

    // Numeric branches coercion
    if let (Some(t_type), Some(f_type)) = (&t_dt, &f_dt) {
        if is_numeric_type(t_type) && is_numeric_type(f_type) {
            let combined = combine_types(t_type, f_type);
            let t_val = widen_expr(if_true, &true_raw, Some(t_type), &combined, ctx);
            let f_val = widen_expr(if_false, &false_raw, Some(f_type), &combined, ctx);
            return format!("if {cond_str} {{ {t_val} }} else {{ {f_val} }}");
        }
    }

    let t_is_bool = matches!(t_dt, Some(DataType::CalcBoolType | DataType::Bits1 { .. })) || true_raw == "true" || true_raw == "false";
    let f_is_bool = matches!(f_dt, Some(DataType::CalcBoolType | DataType::Bits1 { .. })) || false_raw == "true" || false_raw == "false";
    if !t_is_bool && !f_is_bool {
        let t_is_usize = is_usize_expr_str(&true_raw);
        let f_is_usize = is_usize_expr_str(&false_raw);
        if t_is_usize != f_is_usize {
            let t_val = if t_is_usize && !f_is_usize {
                true_raw
            } else if !t_is_usize && f_is_usize {
                format!("usize::try_from({true_raw})?")
            } else {
                true_raw
            };
            let f_val = if f_is_usize && !t_is_usize {
                false_raw
            } else if !f_is_usize && t_is_usize {
                format!("usize::try_from({false_raw})?")
            } else {
                false_raw
            };
            return format!("if {cond_str} {{ {t_val} }} else {{ {f_val} }}");
        }
    }

    let is_stream = |t: &str| t.ends_with("._io()") || t == "_io" || t == "&_io" || t == "&*_io" || t.ends_with("._io");
    if is_stream(&true_raw) || is_stream(&false_raw) {
        return format!("if {cond_str} {{ KStream::clone(&*{t_clean}) }} else {{ KStream::clone(&*{f_clean}) }}");
    }

    format!("if {cond_str} {{ {true_raw} }} else {{ {false_raw} }}")
}

fn translate_call(func: &Expr, args: &[Expr], ctx: &TranslationContext<'_>) -> String {
    if let Expr::Attribute { value, attr } = func {
        let t = translate_expr(value, ctx);
        match attr.as_str() {
            "to_i" => {
                let val_type = detect_type_approx(value, ctx);
                if matches!(val_type, Some(DataType::EnumType { .. })) {
                    return format!("i64::from(&{t})");
                }
                if let Some(arg) = args.first() {
                    let base = translate_expr(arg, ctx);
                    if base == "10" {
                        format!("{t}.parse::<i32>().map_err(|_| KError::CastError)?")
                    } else {
                        format!("i32::from_str_radix({t}, {base}).map_err(|_| KError::CastError)?")
                    }
                } else {
                    format!("{t}.parse::<i32>().map_err(|_| KError::CastError)?")
                }
            }
            "length" | "size" => {
                let stripped = remove_deref(&t);
                format!("{stripped}.len()")
            }
            "substring" => {
                let stripped = remove_deref(&t);
                if let (Some(arg0), Some(arg1)) = (args.first(), args.get(1)) {
                    let from = translate_expr(arg0, ctx);
                    let to = translate_expr(arg1, ctx);
                    format!("substring(&{stripped}, {from}, {to})")
                } else {
                    format!("{stripped}.to_string()")
                }
            }
            "to_s" => {
                if let Some(arg) = args.first() {
                    let enc = translate_expr(arg, ctx);
                    format!("bytes_to_str(&{t}, {enc})?")
                } else {
                    let stripped = remove_deref(&t);
                    format!("{stripped}.to_string()")
                }
            }
            "reverse" => {
                format!("reverse_string(&{t})?")
            }
            other => {
                let args_str = args
                    .iter()
                    .map(|a| translate_expr(a, ctx))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{t}.{other}({args_str})")
            }
        }
    } else {
        let func_str = translate_expr(func, ctx);
        let args_str = args
            .iter()
            .map(|a| translate_expr(a, ctx))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{func_str}({args_str})")
    }
}

/// Strips leading `*` if present.
#[must_use]
pub fn remove_deref(s: &str) -> &str {
    // Reason for fallback: string without leading asterisk remains unchanged
    s.strip_prefix('*').unwrap_or(s)
}

/// Determines whether accessing this data type as an attribute needs `*` dereference.
#[must_use]
pub const fn needs_deref(dt: &DataType) -> bool {
    match dt {
        DataType::EnumType { .. }
        | DataType::Int1 { .. }
        | DataType::IntMulti { .. }
        | DataType::Bits1 { .. }
        | DataType::Bits { .. }
        | DataType::Float { .. }
        | DataType::CalcIntType
        | DataType::CalcFloatType
        | DataType::CalcBoolType => true,
        DataType::Bytes { .. }
        | DataType::Str { .. }
        | DataType::ArrayType { .. }
        | DataType::UserType { .. }
        | DataType::SwitchType { .. }
        | DataType::KaitaiStreamType
        | DataType::CalcStrType
        | DataType::CalcBytesType => false,
    }
}

pub(crate) fn find_class_spec<'a>(root: &'a ClassSpec, path: &[String]) -> Option<&'a ClassSpec> {
    if path.is_empty() {
        return None;
    }
    let parts = if path.first() == root.name.first() {
        // Reason for fallback: path matches root class prefix and falls back to empty remainder if exactly matching
        path.split_first().map_or(&[][..], |(_, rest)| rest)
    } else {
        path
    };
    let mut cur = root;
    for part in parts {
        cur = cur.subclasses.get(part)?;
    }
    Some(cur)
}

fn resolve_user_class_spec<'a>(type_name: &str, ctx: &'a TranslationContext<'_>) -> Option<&'a ClassSpec> {
    let primitives = ["u1", "u2", "u4", "u8", "s1", "s2", "s4", "s8", "f4", "f8", "b1", "bool", "str", "strz"];
    if primitives.contains(&type_name) {
        return None;
    }
    let root = ctx.root;
    let parts: Vec<&str> = type_name.split("::").collect();
    let mut scope = ctx.current_class.name.clone();
    loop {
        let mut candidate = scope.clone();
        for p in &parts {
            candidate.push((*p).to_string());
        }
        if let Some(spec) = find_class_spec(root, &candidate) {
            return Some(spec);
        }
        if scope.is_empty() {
            break;
        }
        scope.pop();
    }
    if parts.len() == 1 {
        if let Some(spec) = root.subclasses.get(type_name) {
            return Some(spec);
        }
    } else {
        let mut candidate = root.name.clone();
        for p in &parts {
            candidate.push((*p).to_string());
        }
        if let Some(spec) = find_class_spec(root, &candidate) {
            return Some(spec);
        }
    }
    None
}

fn resolve_user_class_name(type_name: &str, ctx: &TranslationContext<'_>) -> Option<String> {
    resolve_user_class_spec(type_name, ctx).map(|spec| types_to_class_name(&spec.name))
}

fn find_enum_owner<'a>(class: &'a ClassSpec, enum_name: &str) -> Option<&'a ClassSpec> {
    if class.enums.contains_key(enum_name) {
        return Some(class);
    }
    for sub in class.subclasses.values() {
        if let Some(found) = find_enum_owner(sub, enum_name) {
            return Some(found);
        }
    }
    None
}

/// Resolves an enum type name to its scoped Rust enum name (e.g. `EthernetFrame_EtherTypeEnum`).
#[must_use]
pub fn resolve_enum_type_name(
    enum_name: &str,
    current_class: &ClassSpec,
    root: Option<&ClassSpec>,
) -> String {
    if let Some((class_path, simple_enum_name)) = enum_name.rsplit_once("::") {
        let parts: Vec<&str> = class_path.split("::").collect();
        let search_bases = [Some(current_class), root];
        for base_opt in search_bases {
            if let Some(base) = base_opt {
                let mut curr = base;
                let mut found = true;
                for part in &parts {
                    if let Some(sub) = curr.subclasses.get(*part) {
                        curr = sub;
                    } else {
                        found = false;
                        break;
                    }
                }
                if found && curr.enums.contains_key(simple_enum_name) {
                    let mut full_parts = curr.name.clone();
                    full_parts.push(simple_enum_name.to_string());
                    return types_to_class_name(&full_parts);
                }
            }
        }
    }
    if current_class.enums.contains_key(enum_name) {
        let mut parts = current_class.name.clone();
        parts.push(enum_name.to_string());
        return types_to_class_name(&parts);
    }
    if let Some(root_spec) = root {
        if let Some(owner) = find_enum_owner(root_spec, enum_name) {
            let mut parts = owner.name.clone();
            parts.push(enum_name.to_string());
            return types_to_class_name(&parts);
        }
    }
    // If not found in current class, check root class
    let mut parts = current_class.root_name.clone();
    parts.push(enum_name.to_string());
    types_to_class_name(&parts)
}

fn find_attr_type(class: &ClassSpec, attr: &str) -> Option<DataType> {
    if let Some(inst) = class.instances.get(attr) {
        return Some(inst.data_type.clone());
    }
    if let Some(a) = class.seq.iter().find(|x| x.id == attr) {
        return Some(a.data_type.clone());
    }
    if let Some(p) = class.params.iter().find(|x| x.id == attr) {
        return Some(p.data_type.clone());
    }
    None
}

/// Simple heuristic to detect data type of an expression for clone and cast formatting.
pub(crate) fn detect_type_approx(expr: &Expr, ctx: &TranslationContext<'_>) -> Option<DataType> {
    let current_class = ctx.current_class;
    match expr {
        Expr::IntNum(x) => {
            if *x >= i128::from(i32::MIN) && *x <= i128::from(i32::MAX) {
                Some(DataType::CalcIntType)
            } else if *x >= 0 && *x <= i128::from(u32::MAX) {
                Some(DataType::IntMulti { signed: false, width: 4, endian: None })
            } else if *x >= i128::from(i64::MIN) && *x <= i128::from(i64::MAX) {
                Some(DataType::IntMulti { signed: true, width: 8, endian: None })
            } else {
                Some(DataType::IntMulti { signed: false, width: 8, endian: None })
            }
        }
        Expr::FloatNum(_) => Some(DataType::CalcFloatType),
        Expr::Bool(_) | Expr::Compare { .. } | Expr::BoolOp { .. } => Some(DataType::CalcBoolType),
        Expr::UnaryOp { op: UnaryOp::Not, .. } => Some(DataType::CalcBoolType),
        Expr::UnaryOp { op: UnaryOp::Minus, operand } => {
            let t = detect_type_approx(operand, ctx);
            match t {
                Some(DataType::IntMulti { width, .. }) if width > 4 => t,
                Some(DataType::Int1 { .. } | DataType::IntMulti { .. } | DataType::CalcIntType) => {
                    Some(DataType::CalcIntType)
                }
                Some(DataType::Float { .. } | DataType::CalcFloatType) => t,
                _ => Some(DataType::CalcIntType),
            }
        }
        Expr::UnaryOp { operand, .. } => detect_type_approx(operand, ctx),
        Expr::List(elements) => {
            // Reason for fallback: empty list or uninferrable element type defaults to integer calculation type
            let mut elem_type = DataType::CalcIntType;
            if let Some(first) = elements.first().and_then(|e| detect_type_approx(e, ctx)) {
                elem_type = first;
                for e in elements.iter().skip(1) {
                    if let Some(t) = detect_type_approx(e, ctx) {
                        elem_type = combine_types(&elem_type, &t);
                    }
                }
            }
            Some(DataType::ArrayType {
                element: Box::new(elem_type),
                // Reason for fallback: list length integer conversion overflow defaults to 0
                repeat: RepeatMode::Expr(Expr::IntNum(i128::try_from(elements.len()).unwrap_or(0))),
            })
        }
        Expr::Str(_) => Some(DataType::CalcStrType),
        Expr::EnumByLabel { enum_name, .. } => {
            let mut curr_opt = Some(current_class.name.as_slice());
            while let Some(cls_name) = curr_opt {
                if let Some(cls) = find_class_spec(ctx.root, cls_name) {
                    if cls.enums.contains_key(enum_name) {
                        return Some(DataType::EnumType {
                            owner: cls_name.to_vec(),
                            name: enum_name.clone(),
                            underlying: None,
                        });
                    }
                }
                curr_opt = if cls_name.len() > 1 {
                    cls_name.get(..cls_name.len().saturating_sub(1))
                } else {
                    None
                };
            }
            if ctx.root.enums.contains_key(enum_name) {
                return Some(DataType::EnumType {
                    owner: ctx.root.name.clone(),
                    name: enum_name.clone(),
                    underlying: None,
                });
            }
            None
        }
        Expr::ByteSizeOfType(_) | Expr::BitSizeOfType(_) => Some(DataType::CalcIntType),
        Expr::Name(name) => {
            if name == "_index" {
                return Some(DataType::IntMulti { signed: false, width: 8, endian: None });
            }
            if name == "_sizeof" {
                return Some(DataType::CalcIntType);
            }
            if name == "_" || name == "_tmpa" {
                if let Some(et) = ctx.element_type {
                    return Some(resolve_switch_type(et));
                }
            }
            if name == "_root" {
                return Some(DataType::UserType {
                    names: ctx.root.name.clone(),
                    is_external: false,
                    args: Vec::new(),
                });
            }
            if name == "self" || name == "self_rc" {
                return Some(DataType::UserType {
                    names: current_class.name.clone(),
                    is_external: false,
                    args: Vec::new(),
                });
            }
            if name == "_parent" {
                if let Some(pname) = &current_class.parent_name {
                    return Some(DataType::UserType {
                        names: pname.clone(),
                        is_external: false,
                        args: Vec::new(),
                    });
                }
            }
            if let Some(param) = current_class.params.iter().find(|p| p.id == *name) {
                return Some(resolve_switch_type(&param.data_type));
            }
            if let Some(inst) = current_class.instances.get(name) {
                return Some(resolve_switch_type(&inst.data_type));
            }
            if let Some(attr) = current_class.seq.iter().find(|a| a.id == *name) {
                return Some(resolve_switch_type(&attr.data_type));
            }
            None
        }
        Expr::IfExp { if_true, if_false, .. } => {
            let t1 = detect_type_approx(if_true, ctx);
            let t2 = detect_type_approx(if_false, ctx);
            match (t1, t2) {
                (Some(a), Some(b)) => Some(combine_types(&a, &b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            }
        }
        Expr::Subscript { value, .. } => {
            if let Some(target_dt) = detect_type_approx(value, ctx) {
                let resolved_dt = resolve_switch_type(&target_dt);
                if let DataType::ArrayType { element: elem, .. } = resolved_dt {
                    return Some(*elem);
                }
                if matches!(resolved_dt, DataType::Bytes { .. } | DataType::CalcBytesType) {
                    return Some(DataType::Int1 { signed: false });
                }
            }
            None
        }
        Expr::BinOp { left, op, right } => {
            let lt = detect_type_approx(left, ctx);
            let rt = detect_type_approx(right, ctx);
            if *op == Operator::Add {
                if matches!(lt, Some(DataType::CalcStrType | DataType::Str { .. }))
                    || matches!(rt, Some(DataType::CalcStrType | DataType::Str { .. }))
                {
                    return Some(DataType::CalcStrType);
                }
            }
            if *op == Operator::LShift || *op == Operator::RShift {
                return if matches!(
                    lt,
                    Some(DataType::Int1 { .. }
                        | DataType::IntMulti { width: 1..=2, .. }
                        | DataType::Bits { .. }
                        | DataType::Bits1 { .. })
                ) {
                    Some(DataType::CalcIntType)
                } else {
                    lt.or(Some(DataType::CalcIntType))
                };
            }
            if matches!(lt, Some(DataType::CalcFloatType | DataType::Float { .. }))
                || matches!(rt, Some(DataType::CalcFloatType | DataType::Float { .. }))
            {
                return match (lt, rt) {
                    (Some(t1), Some(t2)) => Some(combine_types(&t1, &t2)),
                    (Some(t), None) | (None, Some(t)) => Some(t),
                    _ => Some(DataType::CalcFloatType),
                };
            }
            if let (Some(t1), Some(t2)) = (&lt, &rt) {
                if is_numeric_type(t1) && is_numeric_type(t2) {
                    return Some(combine_types(t1, t2));
                }
            }
            Some(DataType::CalcIntType)
        }
        Expr::Attribute { value, attr } => {
            if let Some(target_dt) = detect_type_approx(value, ctx) {
                let resolved_dt = resolve_switch_type(&target_dt);
                if let DataType::UserType { names, .. } = resolved_dt {
                    if let Some(target_cls) = find_class_spec(ctx.root, &names) {
                        if attr == "_parent" {
                            if let Some(pname) = &target_cls.parent_name {
                                return Some(DataType::UserType {
                                    names: pname.clone(),
                                    is_external: false,
                                    args: Vec::new(),
                                });
                            }
                        }
                        if let Some(dt) = find_attr_type(target_cls, attr) {
                            return Some(resolve_switch_type(&dt));
                        }
                    }
                }
                if matches!(target_dt, DataType::ArrayType { .. }) && (attr == "first" || attr == "last") {
                    if let DataType::ArrayType { element, .. } = target_dt {
                        return Some(resolve_switch_type(&element));
                    }
                }
            }
            if attr == "_sizeof" {
                return Some(DataType::CalcIntType);
            }
            if attr == "to_i" {
                if let Some(target_dt) = detect_type_approx(value, ctx) {
                    if matches!(target_dt, DataType::EnumType { .. } | DataType::Float { .. } | DataType::CalcFloatType) {
                        return Some(DataType::IntMulti { signed: true, width: 8, endian: None });
                    }
                }
                return Some(DataType::CalcIntType);
            }
            if attr == "length" || attr == "size" || attr == "pos" {
                return Some(DataType::CalcIntType);
            }
            if attr == "to_s" || attr == "substring" || attr == "reverse" {
                return Some(DataType::CalcStrType);
            }
            if attr == "eof" {
                return Some(DataType::CalcBoolType);
            }
            None
        }
        Expr::Call { func, .. } => {
            if let Expr::Attribute { value, attr } = &**func {
                if let Some(target_dt) = detect_type_approx(value, ctx) {
                    if matches!(target_dt, DataType::ArrayType { .. }) && (attr == "first" || attr == "last") {
                        if let DataType::ArrayType { element, .. } = target_dt {
                            return Some(resolve_switch_type(&element));
                        }
                    }
                }
                if attr == "_sizeof" {
                    return Some(DataType::CalcIntType);
                }
                if attr == "to_i" {
                    if let Some(target_dt) = detect_type_approx(value, ctx) {
                        if matches!(target_dt, DataType::EnumType { .. } | DataType::Float { .. } | DataType::CalcFloatType) {
                            return Some(DataType::IntMulti { signed: true, width: 8, endian: None });
                        }
                    }
                    return Some(DataType::CalcIntType);
                }
                if attr == "length" || attr == "size" || attr == "pos" {
                    return Some(DataType::CalcIntType);
                }
                if attr == "to_s" || attr == "substring" || attr == "reverse" {
                    return Some(DataType::CalcStrType);
                }
                if attr == "eof" {
                    return Some(DataType::CalcBoolType);
                }
            }
            None
        }
        Expr::CastToType { type_name, .. } => {
            let s = type_name.name_as_str();
            if s == "bytes" {
                Some(DataType::CalcBytesType)
            } else if s == "str" {
                Some(DataType::CalcStrType)
            } else if s == "u1" {
                Some(DataType::Int1 { signed: false })
            } else if s == "s1" {
                Some(DataType::Int1 { signed: true })
            } else if s == "u2" {
                Some(DataType::IntMulti { signed: false, width: 2, endian: None })
            } else if s == "s2" {
                Some(DataType::IntMulti { signed: true, width: 2, endian: None })
            } else if s == "u4" {
                Some(DataType::IntMulti { signed: false, width: 4, endian: None })
            } else if s == "s4" {
                Some(DataType::IntMulti { signed: true, width: 4, endian: None })
            } else if s == "u8" {
                Some(DataType::IntMulti { signed: false, width: 8, endian: None })
            } else if s == "s8" {
                Some(DataType::IntMulti { signed: true, width: 8, endian: None })
            } else if s == "b1" {
                Some(DataType::Bits1 { bit_endian: BitEndianness::Big })
            } else if matches!(s.as_str(), "f4" | "f8") {
                Some(DataType::CalcFloatType)
            } else if let Some(spec) = resolve_user_class_spec(&s, ctx) {
                Some(DataType::UserType {
                    names: spec.name.clone(),
                    is_external: false,
                    args: Vec::new(),
                })
            } else {
                None
            }
        }
        Expr::EnumById { enum_name, .. } => {
            let mut curr_opt = Some(current_class.name.as_slice());
            while let Some(cls_name) = curr_opt {
                if let Some(cls) = find_class_spec(ctx.root, cls_name) {
                    if cls.enums.contains_key(enum_name) {
                        return Some(DataType::EnumType {
                            owner: cls_name.to_vec(),
                            name: enum_name.clone(),
                            underlying: None,
                        });
                    }
                }
                curr_opt = if cls_name.len() > 1 {
                    cls_name.get(..cls_name.len().saturating_sub(1))
                } else {
                    None
                };
            }
            if ctx.root.enums.contains_key(enum_name) {
                return Some(DataType::EnumType {
                    owner: ctx.root.name.clone(),
                    name: enum_name.clone(),
                    underlying: None,
                });
            }
            None
        }
    }
}

pub(crate) fn is_copy_type(dt: &DataType) -> bool {
    matches!(
        dt,
        DataType::Int1 { .. }
            | DataType::IntMulti { .. }
            | DataType::Float { .. }
            | DataType::Bits1 { .. }
            | DataType::Bits { .. }
            | DataType::CalcIntType
            | DataType::CalcFloatType
            | DataType::CalcBoolType
            | DataType::EnumType { .. }
    )
}

pub(crate) fn resolve_switch_type(dt: &DataType) -> DataType {
    if let DataType::SwitchType { cases, .. } = dt {
        let mut combined: Option<DataType> = None;
        for c in cases.values() {
            combined = match combined {
                None => Some(c.clone()),
                Some(prev) => Some(combine_types(&prev, c)),
            };
        }
        // Reason for fallback: switch type with empty cases retains the original switch data type
        combined.unwrap_or_else(|| dt.clone())
    } else {
        dt.clone()
    }
}

pub(crate) fn is_numeric_type(dt: &DataType) -> bool {
    match dt {
        DataType::Int1 { .. }
        | DataType::IntMulti { .. }
        | DataType::Float { .. }
        | DataType::CalcIntType
        | DataType::CalcFloatType
        | DataType::Bits { .. } => true,
        DataType::SwitchType { cases, .. } => cases.values().all(is_numeric_type),
        _ => false,
    }
}

pub(crate) fn combine_types(t1: &DataType, t2: &DataType) -> DataType {
    let t1_eff = resolve_switch_type(t1);
    let t2_eff = resolve_switch_type(t2);
    if t1_eff == t2_eff {
        return t1_eff;
    }
    match (&t1_eff, &t2_eff) {
        (DataType::Bits { .. }, _) | (_, DataType::Bits { .. }) => {
            DataType::IntMulti {
                signed: false,
                width: 8,
                endian: Some(Endianness::Big),
            }
        }
        (DataType::Int1 { signed: false }, DataType::Int1 { signed: true })
        | (DataType::Int1 { signed: true }, DataType::Int1 { signed: false }) => {
            DataType::Int1 { signed: false }
        }
        (DataType::Int1 { .. }, DataType::IntMulti { .. }) => t2_eff.clone(),
        (DataType::IntMulti { .. }, DataType::Int1 { .. }) => t1_eff.clone(),
        (
            DataType::IntMulti {
                signed: s1,
                width: w1,
                endian: e1,
            },
            DataType::IntMulti {
                signed: s2,
                width: w2,
                endian: _,
            },
        ) => {
            if s1 == s2 {
                DataType::IntMulti {
                    signed: *s1,
                    width: (*w1).max(*w2),
                    endian: *e1,
                }
            } else if *w1 >= 4 || *w2 >= 4 {
                DataType::IntMulti {
                    signed: false,
                    width: (*w1).max(*w2).max(8),
                    endian: *e1,
                }
            } else {
                DataType::IntMulti {
                    signed: true,
                    width: 4,
                    endian: *e1,
                }
            }
        }
        (DataType::CalcBytesType, DataType::Bytes { .. })
        | (DataType::Bytes { .. }, DataType::CalcBytesType)
        | (DataType::Bytes { .. }, DataType::Bytes { .. })
        | (DataType::CalcBytesType, DataType::CalcBytesType) => {
            DataType::CalcBytesType
        }
        (DataType::CalcStrType, DataType::Str { .. })
        | (DataType::Str { .. }, DataType::CalcStrType)
        | (DataType::Str { .. }, DataType::Str { .. })
        | (DataType::CalcStrType, DataType::CalcStrType) => {
            DataType::CalcStrType
        }
        (DataType::CalcBoolType | DataType::Bits1 { .. }, DataType::CalcBoolType | DataType::Bits1 { .. }) => {
            DataType::CalcBoolType
        }
        (DataType::UserType { names: n1, .. }, DataType::UserType { names: n2, .. }) if n1 == n2 => {
            t1_eff.clone()
        }
        (DataType::UserType { .. }, DataType::UserType { .. }) => {
            DataType::UserType {
                names: vec!["struct".to_string()],
                is_external: true,
                args: Vec::new(),
            }
        }
        (DataType::CalcFloatType, DataType::Float { width, endian })
        | (DataType::Float { width, endian }, DataType::CalcFloatType) => {
            if *width == 4 {
                DataType::Float { width: 4, endian: *endian }
            } else {
                DataType::CalcFloatType
            }
        }
        (DataType::Float { width: w1, endian: e1 }, DataType::Float { width: w2, endian: e2 }) => {
            DataType::Float {
                width: (*w1).max(*w2),
                endian: e1.or(*e2),
            }
        }
        (DataType::CalcFloatType, other) if is_numeric_type(other) => DataType::CalcFloatType,
        (other, DataType::CalcFloatType) if is_numeric_type(other) => DataType::CalcFloatType,
        (DataType::Float { width, endian }, other) if is_numeric_type(other) => {
            DataType::Float { width: *width, endian: *endian }
        }
        (other, DataType::Float { width, endian }) if is_numeric_type(other) => {
            DataType::Float { width: *width, endian: *endian }
        }
        (DataType::CalcIntType, DataType::Int1 { .. } | DataType::IntMulti { width: 2, .. }) => {
            DataType::CalcIntType
        }
        (DataType::Int1 { .. } | DataType::IntMulti { width: 2, .. }, DataType::CalcIntType) => {
            DataType::CalcIntType
        }
        (DataType::CalcIntType, other) if is_numeric_type(other) => other.clone(),
        (other, DataType::CalcIntType) if is_numeric_type(other) => other.clone(),
        (a, b) if is_numeric_type(a) && is_numeric_type(b) => DataType::CalcIntType,
        _ => DataType::CalcIntType,
    }
}

fn kaitai_primitive_to_native(dt: &DataType) -> &'static str {
    match dt {
        DataType::Int1 { signed: false } => "u8",
        DataType::Int1 { signed: true } => "i8",
        DataType::IntMulti { signed: false, width: 2, .. } => "u16",
        DataType::IntMulti { signed: false, width: 4, .. } => "u32",
        DataType::IntMulti { signed: false, width: 8, .. } => "u64",
        DataType::IntMulti { signed: true, width: 2, .. } => "i16",
        DataType::IntMulti { signed: true, width: 4, .. } => "i32",
        DataType::IntMulti { signed: true, width: 8, .. } => "i64",
        DataType::Bits { .. } => "u64",
        DataType::Bits1 { .. } => "bool",
        DataType::CalcIntType => "i32",
        DataType::Float { width: 4, .. } => "f32",
        DataType::Float { width: 8, .. } | DataType::CalcFloatType => "f64",
        DataType::CalcBoolType => "bool",
        DataType::CalcStrType | DataType::Str { .. } => "String",
        _ => "i32",
    }
}

/// Translates a custom validation expression (such as `_ & 0x8000 == 0` or `_ == 0 or _ >= _sizeof`).
fn integer_suffix_for_type(dt: Option<&DataType>, fallback_native: &str) -> &'static str {
    if let Some(d) = dt {
        match kaitai_primitive_to_native(d) {
            "u8" => "u8",
            "u16" => "u16",
            "u32" => "u32",
            "u64" => "u64",
            "i8" => "i8",
            "i16" => "i16",
            "i32" => "i32",
            "i64" => "i64",
            "usize" => "usize",
            _ => "i32",
        }
    } else {
        match fallback_native {
            "u8" => "u8",
            "u16" => "u16",
            "u32" => "u32",
            "u64" => "u64",
            "i8" => "i8",
            "i16" => "i16",
            "i32" => "i32",
            "i64" => "i64",
            "usize" => "usize",
            _ => "i32",
        }
    }
}

/// Translates a custom validation expression (such as `_ & 0x8000 == 0` or `_ == 0 or _ >= _sizeof`).
#[must_use]
pub fn translate_validation_custom_expr(
    expr: &Expr,
    dt: &DataType,
    current_class: &ClassSpec,
    ctx: &TranslationContext<'_>,
) -> String {
    let dt_native = kaitai_primitive_to_native(dt);
    let dt_suffix = integer_suffix_for_type(Some(dt), dt_native);
    match expr {
        Expr::BinOp { left, op, right }
            if *op == Operator::BitAnd || *op == Operator::BitOr || *op == Operator::BitXor =>
        {
            let op_char = match op {
                Operator::BitAnd => '&',
                Operator::BitOr => '|',
                Operator::BitXor => '^',
                _ => '&',
            };
            let l = if let Expr::Name(n) = left.as_ref() {
                if n == "_" {
                    "_tmpa".to_string()
                } else {
                    translate_validation_custom_expr(left, dt, current_class, ctx)
                }
            } else {
                translate_validation_custom_expr(left, dt, current_class, ctx)
            };
            let r = match right.as_ref() {
                Expr::IntNum(n) => format!("{n}_{dt_suffix}"),
                Expr::UnaryOp { op: UnaryOp::Invert, operand } => match operand.as_ref() {
                    Expr::IntNum(n) => format!("!({n}_{dt_suffix})"),
                    _ => {
                        let inner = translate_validation_custom_expr(operand, dt, current_class, ctx);
                        format!("!({inner})")
                    }
                },
                _ => translate_validation_custom_expr(right, dt, current_class, ctx),
            };
            format!("({l} {op_char} {r})")
        }
        Expr::Compare { left, op, right } => {
            let op_str = match op {
                CmpOp::Eq => "==",
                CmpOp::NotEq => "!=",
                CmpOp::Lt => "<",
                CmpOp::LtE => "<=",
                CmpOp::Gt => ">",
                CmpOp::GtE => ">=",
            };
            if matches!(left.as_ref(), Expr::Str(_)) || matches!(right.as_ref(), Expr::Str(_)) {
                let l = translate_validation_custom_expr(left, dt, current_class, ctx);
                let r = translate_validation_custom_expr(right, dt, current_class, ctx);
                return format!("({l} {op_str} {r})");
            }
            if let Expr::Name(n) = left.as_ref() {
                if n == "_" {
                    if let Expr::IntNum(val) = right.as_ref() {
                        return format!("(_tmpa {op_str} {val}_{dt_suffix})");
                    }
                    if let Expr::Name(sn) = right.as_ref() {
                        if sn == "_sizeof" {
                            // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
                            let sz = calculate_class_seq_size_with_root(
                                current_class,
                                Some(ctx.root),
                            )
                            .unwrap_or(0);
                            return format!("(_tmpa {op_str} {sz}_{dt_suffix})");
                        }
                    }
                }
            }
            if let Expr::Name(n) = right.as_ref() {
                if n == "_" {
                    if let Expr::IntNum(val) = left.as_ref() {
                        return format!("({val}_{dt_suffix} {op_str} _tmpa)");
                    }
                }
            }
            let mut l = translate_validation_custom_expr(left, dt, current_class, ctx);
            let mut r = translate_validation_custom_expr(right, dt, current_class, ctx);
            if l == "_tmpa" && !needs_deref(dt) {
                l = "*_tmpa".to_string();
            }
            if r == "_tmpa" && !needs_deref(dt) {
                r = "*_tmpa".to_string();
            }
            let lt = if matches!(left.as_ref(), Expr::Name(n) if n == "_") {
                Some(dt.clone())
            } else if matches!(left.as_ref(), Expr::Subscript { .. })
                && matches!(dt, DataType::Bytes { .. } | DataType::CalcBytesType)
            {
                Some(DataType::Int1 { signed: false })
            } else if matches!(
                left.as_ref(),
                Expr::BinOp { op: Operator::BitAnd | Operator::BitOr | Operator::BitXor, .. }
            ) {
                Some(dt.clone())
            } else {
                detect_type_approx(left, ctx)
            };
            let rt = if matches!(right.as_ref(), Expr::Name(n) if n == "_") {
                Some(dt.clone())
            } else if matches!(right.as_ref(), Expr::Subscript { .. })
                && matches!(dt, DataType::Bytes { .. } | DataType::CalcBytesType)
            {
                Some(DataType::Int1 { signed: false })
            } else if matches!(
                right.as_ref(),
                Expr::BinOp { op: Operator::BitAnd | Operator::BitOr | Operator::BitXor, .. }
            ) {
                Some(dt.clone())
            } else {
                detect_type_approx(right, ctx)
            };
            if let Expr::IntNum(n) = right.as_ref() {
                let target_suffix = if is_usize_expr_str(&l) {
                    "usize"
                } else {
                    integer_suffix_for_type(lt.as_ref(), dt_suffix)
                };
                return format!("({l} {op_str} {n}_{target_suffix})");
            }
            if let Expr::IntNum(n) = left.as_ref() {
                let target_suffix = if is_usize_expr_str(&r) {
                    "usize"
                } else {
                    integer_suffix_for_type(rt.as_ref(), dt_suffix)
                };
                return format!("({n}_{target_suffix} {op_str} {r})");
            }
            match (&lt, &rt) {
                (Some(t1), Some(t2)) => {
                    let c1 = kaitai_primitive_to_native(t1);
                    let c2 = kaitai_primitive_to_native(t2);
                    if c1 == c2 {
                        format!("({l} {op_str} {r})")
                    } else if is_numeric_type(t1) && is_numeric_type(t2) {
                        format!("((to_i128({l})) {op_str} (to_i128({r})))")
                    } else {
                        format!("({l} {op_str} {r})")
                    }
                }
                _ => {
                    format!("({l} {op_str} {r})")
                }
            }
        }
        Expr::BoolOp { op, values } => {
            let op_str = match op {
                BoolOp::And => " && ",
                BoolOp::Or => " || ",
            };
            let rendered: Vec<String> = values
                .iter()
                .map(|v| translate_validation_custom_expr(v, dt, current_class, ctx))
                .collect();
            format!(" ({}) ", rendered.join(op_str))
        }
        Expr::UnaryOp { op: UnaryOp::Not, operand } => {
            let inner = translate_validation_custom_expr(operand, dt, current_class, ctx);
            format!("!({inner})")
        }
        Expr::Name(n) if n == "_" => "_tmpa".to_string(),
        Expr::Name(n) if n == "_sizeof" => {
            // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
            let sz = calculate_class_seq_size_with_root(current_class, Some(ctx.root)).unwrap_or(0);
            format!("{sz}_{dt_suffix}")
        }
        Expr::IntNum(n) => format!("{n}"),
        other => translate_expr(other, ctx),
    }
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