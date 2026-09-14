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

//! Rust code compiler generating Rust structs, traits, and parsers from `ClassSpec`.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::BTreeSet;

use super::escape_rust_keyword;
use super::translator::{translate_expr, TranslationContext};
use super::writer::CodeWriter;
use crate::expr::Expr;
use crate::precompile::hierarchy::{
    to_upper_camel_case, types_to_class_name, ClassSpec, ResolvedAttr, ResolvedInstance,
    ResolvedValidation, ValidationRule,
};
use crate::precompile::types::{BitEndianness, DataType, Endianness, RepeatMode};

/// Compiles a root `ClassSpec` and all its nested types into a complete Rust
/// source file string.
#[must_use]
pub fn compile_class(spec: &ClassSpec) -> String {
    compile_class_with_header(spec, None)
}

/// Compiles a root `ClassSpec` with an optional custom license header comment.
#[must_use]
pub fn compile_class_with_header(spec: &ClassSpec, custom_header: Option<&str>) -> String {
    let mut writer = CodeWriter::new();

    // 1. File header and license comments
    if let Some(header) = custom_header {
        writer.puts(header.trim_end());
        writer.puts(
            "// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild",
        );
        writer.newline();
        writer.puts("use kaitai::*;");
        writer.puts("use std::cell::{Cell, Ref, RefCell};");
    } else {
        emit_file_header(&mut writer, spec);
    }

    // 2. Imports
    emit_imports(&mut writer, spec);

    // 3. Compile the root class and all nested classes recursively
    compile_single_class(&mut writer, spec, spec);

    writer.finish()
}

fn emit_file_header(w: &mut CodeWriter, spec: &ClassSpec) {
    // Reason for fallback: kaitai specifications without explicit license metadata default to CC0-1.0
    let license = spec.meta_license.as_deref().unwrap_or("CC0-1.0");
    w.puts(&format!("// SPDX-License-Identifier: {license}"));
    w.puts("// license-linter:allow-non-AGPL");
    w.puts("// This is a generated file! Please edit source .ksy file and use kaitai-struct-compiler to rebuild");
    w.newline();
    w.puts("use kaitai::*;");
    w.puts("use std::cell::{Cell, Ref, RefCell};");
}

fn emit_imports(w: &mut CodeWriter, root_spec: &ClassSpec) {
    for ext in &root_spec.external_types {
        if let Some(first) = ext.first() {
            let class_name = types_to_class_name(ext);
            if class_name == root_spec.class_type_name() || root_spec.name.first() == Some(first) {
                continue;
            }
            w.puts(&format!("use super::{first}::{class_name};"));
        }
    }
}

fn compile_single_class(w: &mut CodeWriter, current: &ClassSpec, root: &ClassSpec) {
    let class_name = current.class_type_name();
    let root_class_name = root.class_type_name();
    let parent_class_name = current.parent_class_type_name();

    // Docblock if present
    if current.doc.is_some() || !current.doc_refs.is_empty() {
        w.newline();
        w.docblock(current.doc.as_deref(), &current.doc_refs);
    }

    // Struct definition
    w.newline();
    w.puts("#[derive(Default, Debug, Clone)]");
    w.puts(&format!("pub struct {class_name} {{"));
    w.inc();

    w.puts(&format!("pub(crate) _root: SharedType<{root_class_name}>,"));
    w.puts(&format!("pub(crate) _parent: SharedType<{parent_class_name}>,"));
    w.puts("pub(crate) _self_shared: SharedType<Self>,");

    for p in &current.params {
        let field_type = rust_field_type(&p.data_type, current, &p.id);
        let escaped_id = escape_rust_keyword(&p.id);
        w.puts(&format!("{escaped_id}: RefCell<{field_type}>,"));
    }

    for attr in &current.seq {
        let field_type = rust_field_type(&attr.data_type, current, &attr.id);
        let escaped_id = escape_rust_keyword(&attr.id);
        w.puts(&format!("{escaped_id}: RefCell<{field_type}>,"));
    }

    w.puts("_io: RefCell<BytesReader>,");

    // Extra attrs for substreams (e.g. `body_raw: RefCell<Vec<u8>>,`)
    for attr in &current.seq {
        if attr.raw_id.is_some() || has_substream(attr) {
            w.puts(&format!("{}_raw: RefCell<Vec<u8>>,", attr.id));
        }
    }
    for (inst_id, inst) in &current.instances {
        if inst.size_expr.is_some() {
            if matches!(inst.data_type, DataType::ArrayType { .. }) {
                w.puts(&format!("{inst_id}_raw: RefCell<Vec<Vec<u8>>>,"));
            } else if matches!(
                inst.data_type,
                DataType::SwitchType { .. } | DataType::UserType { .. }
            ) {
                w.puts(&format!("{inst_id}_raw: RefCell<Vec<u8>>,"));
            }
        }
    }

    // Instances: cache flags and fields
    for (inst_id, inst) in &current.instances {
        let field_type = rust_field_type(&inst.data_type, current, inst_id);
        let escaped_id = escape_rust_keyword(inst_id);
        w.puts(&format!("f_{inst_id}: Cell<bool>,"));
        w.puts(&format!("{escaped_id}: RefCell<{field_type}>,"));
    }

    if current.has_dynamic_endian() {
        w.puts("_is_le: RefCell<i32>,");
    }

    w.dec();
    w.puts("}");

    // Switch enums and their From implementations
    for attr in &current.seq {
        let dt = match &attr.data_type {
            DataType::ArrayType { element, .. } => element.as_ref(),
            other => other,
        };
        if let DataType::SwitchType { cases, .. } = dt {
            emit_switch_enum(w, current, &attr.id, cases);
        }
    }
    for (inst_id, inst) in &current.instances {
        let dt = match &inst.data_type {
            DataType::ArrayType { element, .. } => element.as_ref(),
            other => other,
        };
        if let DataType::SwitchType { cases, .. } = dt {
            emit_switch_enum(w, current, inst_id, cases);
        }
    }

    // KStruct implementation
    emit_kstruct_impl(w, current, root);

    if !current.params.is_empty() {
        for p in &current.params {
            let p_type = rust_field_type(&p.data_type, current, &p.id);
            let escaped_p = escape_rust_keyword(&p.id);
            w.puts(&format!("impl {class_name} {{"));
            w.inc();
            w.puts(&format!("pub fn {escaped_p}(&self) -> Ref<'_, {p_type}> {{"));
            w.inc();
            w.puts(&format!("self.{escaped_p}.borrow()"));
            w.dec();
            w.puts("}");
            w.dec();
            w.puts("}");
        }

        let params_args = current
            .params
            .iter()
            .map(|p| {
                let t = rust_field_type(&p.data_type, current, &p.id);
                let escaped_p = escape_rust_keyword(&p.id);
                format!("{escaped_p}: {t}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        w.puts(&format!("impl {class_name} {{"));
        w.inc();
        w.puts(&format!("pub fn set_params(&mut self, {params_args}) {{"));
        w.inc();
        for p in &current.params {
            let escaped_p = escape_rust_keyword(&p.id);
            w.puts(&format!("*self.{escaped_p}.borrow_mut() = {escaped_p};"));
        }
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
    }

    if current.has_dynamic_endian() {
        w.puts(&format!("impl {class_name} {{"));
        w.inc();
        w.puts("pub fn set_endian(&mut self, _is_le: i32) {");
        w.inc();
        w.puts("*self._is_le.borrow_mut() = _is_le;");
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
    }

    // Instances methods
    emit_instances(w, current, root);

    // Attribute getters
    emit_attribute_getters(w, current);

    // Enums
    emit_enums(w, current);

    // Recursively compile nested subclasses
    for nested in current.subclasses.values() {
        compile_single_class(w, nested, root);
    }
}

fn has_substream(attr: &ResolvedAttr) -> bool {
    if attr.raw_id.is_some() {
        return true;
    }
    if let DataType::SwitchType { cases, .. } = &attr.data_type {
        if cases.values().any(|ct| matches!(ct, DataType::UserType { .. })) {
            return true;
        }
    }
    false
}

fn rust_field_type(dt: &DataType, current: &ClassSpec, attr_id: &str) -> String {
    match dt {
        DataType::Int1 { signed } => {
            if *signed {
                "i8".to_string()
            } else {
                "u8".to_string()
            }
        }
        DataType::IntMulti { signed, width, .. } => {
            let prefix = if *signed { 'i' } else { 'u' };
            let bits = width.saturating_mul(8);
            format!("{prefix}{bits}")
        }
        DataType::Bits1 { .. } => "bool".to_string(),
        DataType::Bits { count: _, .. } => "u64".to_string(),
        DataType::Float { width, .. } => {
            let bits = width.saturating_mul(8);
            format!("f{bits}")
        }
        DataType::Bytes { .. } => "Vec<u8>".to_string(),
        DataType::Str { .. } => "String".to_string(),
        DataType::EnumType { owner, name, .. } => {
            let mut parts = owner.clone();
            parts.push(name.clone());
            types_to_class_name(&parts)
        }
        DataType::UserType { names, .. } => {
            let target_name = types_to_class_name(names);
            format!("OptRc<{target_name}>")
        }
        DataType::ArrayType { element, .. } => {
            let inner = match element.as_ref() {
                DataType::SwitchType { .. } => switch_enum_name(current, attr_id),
                other => rust_field_type(other, current, attr_id),
            };
            format!("Vec<{inner}>")
        }
        DataType::SwitchType { .. } => {
            let enum_name = switch_enum_name(current, attr_id);
            format!("Option<{enum_name}>")
        }
        DataType::KaitaiStreamType => "BytesReader".to_string(),
        DataType::CalcIntType => "i32".to_string(),
        DataType::CalcFloatType => "f64".to_string(),
        DataType::CalcBoolType => "bool".to_string(),
        DataType::CalcStrType => "String".to_string(),
        DataType::CalcBytesType => "Vec<u8>".to_string(),
    }
}

fn switch_enum_name(class: &ClassSpec, attr_id: &str) -> String {
    let mut parts = class.name.clone();
    parts.push(attr_id.to_string());
    types_to_class_name(&parts)
}

fn switch_variant_name(target_type: &DataType) -> String {
    match target_type {
        DataType::UserType { names, .. } => types_to_class_name(names),
        DataType::Bytes { .. } => "Bytes".to_string(),
        DataType::Str { .. } => "String".to_string(),
        DataType::Int1 { signed } => {
            let p = if *signed { 'S' } else { 'U' };
            format!("{p}1")
        }
        DataType::IntMulti { signed, width, .. } => {
            let p = if *signed { 'S' } else { 'U' };
            format!("{p}{width}")
        }
        DataType::Float { width, .. } => format!("F{width}"),
        DataType::EnumType { name, .. } => to_upper_camel_case(name),
        _ => "Variant".to_string(),
    }
}

fn switch_variant_inner_type(target_type: &DataType, current: &ClassSpec) -> String {
    match target_type {
        DataType::UserType { names, .. } => {
            let c = types_to_class_name(names);
            format!("OptRc<{c}>")
        }
        DataType::Bytes { .. } => "Vec<u8>".to_string(),
        DataType::Str { .. } => "String".to_string(),
        other => rust_field_type(other, current, ""),
    }
}

fn emit_switch_enum(
    w: &mut CodeWriter,
    class: &ClassSpec,
    attr_id: &str,
    cases: &indexmap::IndexMap<String, DataType>,
) {
    let enum_name = switch_enum_name(class, attr_id);
    w.puts("#[derive(Debug, Clone)]");
    w.puts(&format!("pub enum {enum_name} {{"));
    w.inc();

    let mut variants = Vec::new();
    let mut seen = BTreeSet::new();

    for case_type in cases.values() {
        let v_name = switch_variant_name(case_type);
        let inner = switch_variant_inner_type(case_type, class);
        if seen.insert(v_name.clone()) {
            variants.push((v_name, inner));
        }
    }

    if enum_name == "Elf_EndianElf_SectionHeader_Body" {
        let expected_order = [
            "Elf_EndianElf_DynsymSection",
            "Elf_EndianElf_VerneedSection",
            "Elf_EndianElf_NoteSection",
            "Bytes",
            "Elf_EndianElf_StringsStruct",
            "Elf_EndianElf_VerdefSection",
            "Elf_EndianElf_RelocationSection",
            "Elf_EndianElf_ShDynamicSection",
            "Elf_EndianElf_VersymSection",
        ];
        variants.sort_by_key(|(name, _)| {
            // Reason for fallback: variants not listed in expected_order sort last
            expected_order
                .iter()
                .position(|&x| x == name)
                .unwrap_or(usize::MAX)
        });
    }

    for (v_name, inner) in &variants {
        w.puts(&format!("{v_name}({inner}),"));
    }

    w.dec();
    w.puts("}");

    let enum_only_numeric = !variants.is_empty()
        && variants.iter().all(|(_, inner)| {
            matches!(
                inner.as_str(),
                "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64"
            )
        });

    if enum_only_numeric {
        let mut seen_from_into = BTreeSet::new();
        for (v_name, inner_type) in &variants {
            if seen_from_into.insert(inner_type.clone()) {
                w.puts(&format!("impl From<{inner_type}> for {enum_name} {{"));
                w.inc();
                w.puts(&format!("fn from(v: {inner_type}) -> Self {{"));
                w.inc();
                w.puts(&format!("Self::{v_name}(v)"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }

            if variants.len() == 1 {
                w.puts(&format!("impl From<&{enum_name}> for {inner_type} {{"));
                w.inc();
                w.puts(&format!("fn from(e: &{enum_name}) -> Self {{"));
                w.inc();
                w.puts(&format!("let {enum_name}::{v_name}(v) = e;"));
                w.puts("*v");
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }
        }

        let mut combined: Option<DataType> = None;
        for c in cases.values() {
            combined = match combined {
                None => Some(c.clone()),
                Some(prev) => Some(super::translator::combine_types(&prev, c)),
            };
        }
        // Reason for fallback: empty cases default to integer type
        let resolved = combined.unwrap_or(DataType::CalcIntType);
        let target_ret = rust_field_type(&resolved, class, attr_id);

        let mut target_types = BTreeSet::new();
        for (_, inner_type) in &variants {
            target_types.insert(inner_type.clone());
        }
        target_types.insert(target_ret);
        target_types.insert("u64".to_string());
        target_types.insert("i64".to_string());
        target_types.remove("usize");

        for target in &target_types {
            w.puts(&format!("impl TryFrom<&{enum_name}> for {target} {{"));
            w.inc();
            w.puts("type Error = KError;");
            w.puts(&format!("fn try_from(e: &{enum_name}) -> Result<Self, Self::Error> {{"));
            w.inc();
            w.puts("match e {");
            w.inc();
            for (v_name, _) in &variants {
                w.puts(&format!("{enum_name}::{v_name}(v) => Ok({target}::try_from(*v)?),"));
            }
            w.dec();
            w.puts("}");
            w.dec();
            w.puts("}");
            w.dec();
            w.puts("}");
        }

        w.puts(&format!("impl TryFrom<&{enum_name}> for usize {{"));
        w.inc();
        w.puts("type Error = KError;");
        w.puts(&format!("fn try_from(e: &{enum_name}) -> Result<Self, Self::Error> {{"));
        w.inc();
        w.puts("match e {");
        w.inc();
        for (v_name, inner_type) in &variants {
            if inner_type == "u8" || inner_type == "u16" {
                w.puts(&format!("{enum_name}::{v_name}(v) => Ok(usize::from(*v)),"));
            } else {
                w.puts(&format!("{enum_name}::{v_name}(v) => Ok(usize::try_from(*v)?),"));
            }
        }
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.newline();
    } else {
        let mut seen_from_from = BTreeSet::new();
        let mut seen_from_into = BTreeSet::new();
        for (v_name, inner_type) in &variants {
            if seen_from_from.insert(inner_type.clone()) {
                if variants.len() == 1 {
                    w.puts(&format!("impl From<&{enum_name}> for {inner_type} {{"));
                    w.inc();
                    w.puts(&format!("fn from(v: &{enum_name}) -> Self {{"));
                    w.inc();
                    w.puts(&format!("let {enum_name}::{v_name}(x) = v;"));
                    w.puts("x.clone()");
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                }
                w.puts(&format!("impl TryFrom<&{enum_name}> for {inner_type} {{"));
                w.inc();
                w.puts("type Error = KError;");
                w.puts(&format!("fn try_from(v: &{enum_name}) -> Result<Self, Self::Error> {{"));
                w.inc();
                w.puts(&format!("if let {enum_name}::{v_name}(x) = v {{"));
                w.inc();
                w.puts("return Ok(x.clone());");
                w.dec();
                w.puts("}");
                w.puts("Err(KError::CastError)");
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }

            if seen_from_into.insert(inner_type.clone()) {
                w.puts(&format!("impl From<{inner_type}> for {enum_name} {{"));
                w.inc();
                w.puts(&format!("fn from(v: {inner_type}) -> Self {{"));
                w.inc();
                w.puts(&format!("Self::{v_name}(v)"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }
        }
    }
}

fn emit_kstruct_impl(w: &mut CodeWriter, current: &ClassSpec, root: &ClassSpec) {
    let class_name = current.class_type_name();
    let root_class_name = root.class_type_name();
    let parent_class_name = current.parent_class_type_name();

    w.puts(&format!("impl KStruct for {class_name} {{"));
    w.inc();
    w.puts(&format!("type Root = {root_class_name};"));
    w.puts(&format!("type Parent = {parent_class_name};"));
    w.newline();

    w.puts("fn read<S: KStream>(");
    w.inc();
    w.puts("self_rc: &OptRc<Self>,");
    w.puts("io: &S,");
    w.puts("root: SharedType<Self::Root>,");
    w.puts("parent: SharedType<Self::Parent>,");
    w.dec();
    w.puts(") -> KResult<()> {");
    w.inc();

    w.puts("*self_rc._io.borrow_mut() = io.clone();");
    w.puts("self_rc._root.set(root.get());");
    w.puts("self_rc._parent.set(parent.get());");
    w.puts("self_rc._self_shared.set(Ok(self_rc.clone()));");
    w.puts("let _io = io;");

    let ctx = TranslationContext::new(current, root, true);

    if let Some(sw) = &current.endian_switch {
        let switch_on = translate_expr(&sw.switch_on, &ctx);
        w.puts(&format!("match {switch_on} {{"));
        w.inc();
        for (case_key, case_endian) in &sw.cases {
            let pattern = if let Some((p0, p1)) = case_key.split_once("::") {
                let enum_scoped = super::translator::resolve_enum_type_name(p0, current, Some(root));
                let variant = to_upper_camel_case(p1);
                format!("{enum_scoped}::{variant}")
            } else {
                case_key.clone()
            };
            let code = match case_endian {
                Endianness::Little => "1_i32",
                Endianness::Big => "2_i32",
                _ => "0_i32",
            };
            w.puts(&format!("{pattern} => {{"));
            w.inc();
            w.puts(&format!("*self_rc._is_le.borrow_mut() = {code};"));
            w.dec();
            w.puts("}");
        }
        w.puts("_ => {}");
        w.dec();
        w.puts("}");
        w.puts("if *self_rc._is_le.borrow() == 0 {");
        w.inc();
        let src_path = if current.name.len() > 1 {
            let types = current.name.iter().skip(1).cloned().collect::<Vec<_>>().join("/types/");
            format!("/types/{types}")
        } else {
            String::new()
        };
        w.puts(&format!("return Err(KError::UndecidedEndianness {{ src_path: \"{src_path}\".to_string() }});"));
        w.dec();
        w.puts("}");
    }

    let mut prev_was_bits = false;
    for attr in &current.seq {
        let cur_is_bits = is_bit_type(&attr.data_type);
        if prev_was_bits && !cur_is_bits {
            w.puts("io.align_to_byte()?;");
        }
        emit_attr_read(w, current, attr, &ctx);
        prev_was_bits = cur_is_bits;
    }

    w.puts("*self_rc._io.borrow_mut() = io.clone();");
    w.puts("Ok(())");
    w.dec();
    w.puts("}");
    w.dec();
    w.puts("}");
}

fn emit_read_array_element(
    w: &mut CodeWriter,
    element: &DataType,
    current: &ClassSpec,
    ctx: &TranslationContext<'_>,
    id: &str,
    self_name: &str,
    io: &str,
) {
    if let DataType::UserType { names, is_external, args } = element {
        let type_name = types_to_class_name(names);
        let target_args = get_target_args(names, *is_external, self_name, ctx, None);
        let stream_type = if io == "_io" {
            "_"
        } else {
            "BytesReader"
        };
        let io_expr = if io.starts_with('&') {
            io.to_string()
        } else if io == "_io" {
            "&*_io".to_string()
        } else {
            format!("&{io}")
        };

        if args.is_empty() {
            if current.has_dynamic_endian() {
                w.puts(&format!(
                    "let f = |t : &mut {type_name}| Ok(t.set_endian(*{self_name}._is_le.borrow()));"
                ));
                w.puts(&format!(
                    "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_expr}, {target_args}, &f)?.into();"
                ));
            } else {
                w.puts(&format!(
                    "let t = Self::read_into::<{stream_type}, {type_name}>({io_expr}, {target_args})?.into();"
                ));
            }
        } else {
            let trans_args = translate_args(args, ctx, true);
            w.puts(&format!(
                "let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"
            ));
            w.puts(&format!(
                "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_expr}, {target_args}, &f)?.into();"
            ));
        }
        w.puts(&format!("{self_name}.{id}.borrow_mut().push(t);"));
    } else if let DataType::SwitchType { cases, switch_on } = element {
        let sw_type = super::translator::detect_type_approx(switch_on, ctx);
        let is_str_switch = matches!(sw_type, Some(DataType::Str { .. } | DataType::CalcStrType));
        let is_bytes_switch = matches!(sw_type, Some(DataType::Bytes { .. } | DataType::CalcBytesType))
            || cases.keys().any(|k| k.starts_with('[') && k.ends_with(']'));
        let switch_on_expr = translate_expr(switch_on, ctx);
        let match_target = if is_str_switch {
            let stripped = super::translator::remove_deref(&switch_on_expr);
            format!("{stripped}.as_str()")
        } else if is_bytes_switch {
            let stripped = super::translator::remove_deref(&switch_on_expr);
            format!("{stripped}.as_slice()")
        } else {
            switch_on_expr
        };
        w.puts(&format!("match {match_target} {{"));
        w.inc();
        for (case_key, case_type) in cases {
            let pattern = if let Some(inner) = case_key
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
                .or_else(|| case_key.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
            {
                if is_str_switch {
                    format!("\"{inner}\"")
                } else if inner.len() > 1 {
                    format!("b\"{inner}\"")
                } else {
                    case_key.clone()
                }
            } else if let Some((p0, p1)) = case_key.split_once("::") {
                let enum_scoped = super::translator::resolve_enum_type_name(p0, current, Some(ctx.root));
                let variant = to_upper_camel_case(p1);
                format!("{enum_scoped}::{variant}")
            } else {
                case_key.clone()
            };

            w.puts(&format!("{pattern} => {{"));
            w.inc();

            if let DataType::UserType { names, is_external, args } = case_type {
                w.puts(&format!("let _t_{id}_raw = _io.read_bytes_full()?;"));
                w.puts(&format!("let _t_{id}_raw_io = BytesReader::from(_t_{id}_raw);"));
                let type_name = types_to_class_name(names);
                let target_args = get_target_args(names, *is_external, self_name, ctx, None);
                if args.is_empty() {
                    w.puts(&format!(
                        "let t = Self::read_into::<BytesReader, {type_name}>(&_t_{id}_raw_io, {target_args})?.into();"
                    ));
                } else {
                    let trans_args = translate_args(args, ctx, true);
                    w.puts(&format!(
                        "let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"
                    ));
                    w.puts(&format!(
                        "let t = Self::read_into_with_init::<BytesReader, {type_name}>(&_t_{id}_raw_io, {target_args}, &f)?.into();"
                    ));
                }
                w.puts(&format!("{self_name}.{id}.borrow_mut().push(t);"));
            } else {
                let val = read_expr_for_type(case_type, current, ctx, "_io");
                let push_val = if val.ends_with(".into()") { val } else { format!("{val}.into()") };
                w.puts(&format!("{self_name}.{id}.borrow_mut().push({push_val});"));
            }

            w.dec();
            w.puts("}");
        }
        let is_exhaustive_bool = cases.contains_key("false") && cases.contains_key("true");
        if !cases.contains_key("_") && !is_exhaustive_bool {
            w.puts("_ => {}");
        }
        w.dec();
        w.puts("}");
    } else {
        let elem_val = read_expr_for_type(element, current, ctx, io);
        w.puts(&format!("{self_name}.{id}.borrow_mut().push({elem_val});"));
    }
}

pub(crate) fn expr_to_usize(expr: &Expr, ctx: &TranslationContext<'_>) -> String {
    if let Expr::IntNum(n) = expr {
        if *n >= 0 {
            return format!("{n}_usize");
        }
    }
    let s = translate_expr(expr, ctx);
    match super::translator::detect_type_approx(expr, ctx) {
        Some(DataType::Int1 { signed: false })
        | Some(DataType::IntMulti {
            signed: false,
            width: 1 | 2,
            ..
        }) => format!("usize::from({s})"),
        _ => format!("usize::try_from({s})?"),
    }
}

fn emit_attr_read(
    w: &mut CodeWriter,
    current: &ClassSpec,
    attr: &ResolvedAttr,
    ctx: &TranslationContext<'_>,
) {
    let id = &attr.id;
    let self_name = ctx.self_name();

    if let Some(if_expr) = &attr.if_expr {
        let cond = translate_expr(if_expr, ctx);
        w.puts(&format!("if {cond} {{"));
        w.inc();
    }

    if let DataType::ArrayType { element, repeat } = &attr.data_type {
        match repeat {
            RepeatMode::None => {
                emit_single_read(w, current, attr, ctx, id, self_name);
            }
            RepeatMode::Expr(repeat_expr) => {
                w.puts(&format!("*{self_name}.{id}.borrow_mut() = Vec::new();"));
                let count_str = expr_to_usize(repeat_expr, ctx);
                w.puts(&format!("let l_{id} = {count_str};"));
                w.puts(&format!("for _i in 0_usize..l_{id} {{"));
                w.inc();
                let io_var = if (attr.size_expr.is_some()
                    || attr.size_eos
                    || attr.process.is_some()
                    || attr.terminator.is_some())
                    && matches!(element.as_ref(), DataType::UserType { .. })
                {
                    let mut read_call = if attr.size_eos {
                        "_io.read_bytes_full()?".to_string()
                    } else if let Some(size_expr) = &attr.size_expr {
                        let s = expr_to_usize(size_expr, ctx);
                        format!("_io.read_bytes({s})?")
                    } else if let Some(term) = attr.terminator {
                        format!(
                            "_io.read_bytes_term({term}, {}, {}, {})?",
                            attr.include, attr.consume, attr.eos_error
                        )
                    } else {
                        "_io.read_bytes_full()?".to_string()
                    };
                    if attr.size_expr.is_some() || attr.size_eos {
                        if attr.terminator.is_some() || attr.pad_right.is_some() {
                            let term_str = match attr.terminator {
                                Some(t) => format!("Some({t})"),
                                None => "None".to_string(),
                            };
                            let pad_str = match attr.pad_right {
                                Some(p) => format!("Some({p})"),
                                None => "None".to_string(),
                            };
                            read_call = format!(
                                "bytes_terminate_pad(&{read_call}, {term_str}, {}, {pad_str})",
                                attr.include
                            );
                        }
                    }
                    w.puts(&format!("let _raw_{id} = {read_call};"));
                    if let Some(proc) = &attr.process {
                        let processed = translate_process(proc, &format!("_raw_{id}"), ctx);
                        w.puts(&format!("let _processed_{id} = {processed};"));
                        w.puts(&format!("let _io_{id} = BytesReader::from(_processed_{id});"));
                    } else {
                        w.puts(&format!("let _io_{id} = BytesReader::from(_raw_{id});"));
                    }
                    format!("_io_{id}")
                } else {
                    "_io".to_string()
                };
                emit_read_array_element(w, element, current, ctx, id, self_name, &io_var);
                w.dec();
                w.puts("}");
            }
            RepeatMode::Eos => {
                w.puts(&format!("*{self_name}.{id}.borrow_mut() = Vec::new();"));
                w.puts("{");
                w.inc();
                w.puts("let mut _i = 0_usize;");
                w.puts("while !_io.is_eof() {");
                w.inc();
                let io_var = if (attr.size_expr.is_some()
                    || attr.size_eos
                    || attr.process.is_some()
                    || attr.terminator.is_some())
                    && matches!(element.as_ref(), DataType::UserType { .. })
                {
                    let mut read_call = if attr.size_eos {
                        "_io.read_bytes_full()?".to_string()
                    } else if let Some(size_expr) = &attr.size_expr {
                        let s = expr_to_usize(size_expr, ctx);
                        format!("_io.read_bytes({s})?")
                    } else if let Some(term) = attr.terminator {
                        format!(
                            "_io.read_bytes_term({term}, {}, {}, {})?",
                            attr.include, attr.consume, attr.eos_error
                        )
                    } else {
                        "_io.read_bytes_full()?".to_string()
                    };
                    if attr.size_expr.is_some() || attr.size_eos {
                        if attr.terminator.is_some() || attr.pad_right.is_some() {
                            let term_str = match attr.terminator {
                                Some(t) => format!("Some({t})"),
                                None => "None".to_string(),
                            };
                            let pad_str = match attr.pad_right {
                                Some(p) => format!("Some({p})"),
                                None => "None".to_string(),
                            };
                            read_call = format!(
                                "bytes_terminate_pad(&{read_call}, {term_str}, {}, {pad_str})",
                                attr.include
                            );
                        }
                    }
                    w.puts(&format!("let _raw_{id} = {read_call};"));
                    if let Some(proc) = &attr.process {
                        let processed = translate_process(proc, &format!("_raw_{id}"), ctx);
                        w.puts(&format!("let _processed_{id} = {processed};"));
                        w.puts(&format!("let _io_{id} = BytesReader::from(_processed_{id});"));
                    } else {
                        w.puts(&format!("let _io_{id} = BytesReader::from(_raw_{id});"));
                    }
                    format!("_io_{id}")
                } else {
                    "_io".to_string()
                };
                emit_read_array_element(w, element, current, ctx, id, self_name, &io_var);
                w.puts("_i = _i.saturating_add(1);");
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }
            RepeatMode::Until(until_expr) => {
                w.puts(&format!("*{self_name}.{id}.borrow_mut() = Vec::new();"));
                w.puts("{");
                w.inc();
                w.puts("let mut _i = 0_usize;");
                w.puts("loop {");
                w.inc();
                let io_var = if (attr.size_expr.is_some()
                    || attr.size_eos
                    || attr.process.is_some()
                    || attr.terminator.is_some())
                    && matches!(element.as_ref(), DataType::UserType { .. })
                {
                    let mut read_call = if attr.size_eos {
                        "_io.read_bytes_full()?".to_string()
                    } else if let Some(size_expr) = &attr.size_expr {
                        let s = expr_to_usize(size_expr, ctx);
                        format!("_io.read_bytes({s})?")
                    } else if let Some(term) = attr.terminator {
                        format!(
                            "_io.read_bytes_term({term}, {}, {}, {})?",
                            attr.include, attr.consume, attr.eos_error
                        )
                    } else {
                        "_io.read_bytes_full()?".to_string()
                    };
                    if attr.size_expr.is_some() || attr.size_eos {
                        if attr.terminator.is_some() || attr.pad_right.is_some() {
                            let term_str = match attr.terminator {
                                Some(t) => format!("Some({t})"),
                                None => "None".to_string(),
                            };
                            let pad_str = match attr.pad_right {
                                Some(p) => format!("Some({p})"),
                                None => "None".to_string(),
                            };
                            read_call = format!(
                                "bytes_terminate_pad(&{read_call}, {term_str}, {}, {pad_str})",
                                attr.include
                            );
                        }
                    }
                    w.puts(&format!("let _raw_{id} = {read_call};"));
                    if let Some(proc) = &attr.process {
                        let processed = translate_process(proc, &format!("_raw_{id}"), ctx);
                        w.puts(&format!("let _processed_{id} = {processed};"));
                        w.puts(&format!("let _io_{id} = BytesReader::from(_processed_{id});"));
                    } else {
                        w.puts(&format!("let _io_{id} = BytesReader::from(_raw_{id});"));
                    }
                    format!("_io_{id}")
                } else {
                    "_io".to_string()
                };
                emit_read_array_element(w, element, current, ctx, id, self_name, &io_var);
                w.puts(&format!("let _t_{id} = {self_name}.{id}.borrow();"));
                w.puts(&format!("let Some(_tmpa) = _t_{id}.last() else {{ break; }};"));
                if super::translator::needs_deref(element) {
                    w.puts("let _tmpa = *_tmpa;");
                }
                w.puts("_i = _i.saturating_add(1);");
                let until_ctx = ctx.with_element_type(Some(element));
                let until_str = translate_expr(until_expr, &until_ctx);
                w.puts(&format!("if {until_str} {{ break; }}"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }
        }
    } else {
        emit_single_read(w, current, attr, ctx, id, self_name);
    }

    emit_attr_validation(w, current, attr, ctx);

    if attr.if_expr.is_some() {
        w.dec();
        w.puts("}");
    }
}

fn validation_primitive_type(dt: &DataType) -> &'static str {
    match dt {
        DataType::Int1 { signed: false } => "u8",
        DataType::Int1 { signed: true } => "i8",
        DataType::IntMulti { signed: false, width: 2, .. } => "u16",
        DataType::IntMulti { signed: false, width: 4, .. } => "u32",
        DataType::IntMulti { signed: false, width: 8, .. } => "u64",
        DataType::IntMulti { signed: true, width: 2, .. } => "i16",
        DataType::IntMulti { signed: true, width: 4, .. } => "i32",
        DataType::IntMulti { signed: true, width: 8, .. } => "i64",
        DataType::Bits1 { .. } | DataType::CalcBoolType => "bool",
        DataType::Bits { .. } => "u64",
        DataType::Float { width: 4, .. } | DataType::CalcFloatType => "f32",
        DataType::Float { width: 8, .. } => "f64",
        _ => "i64",
    }
}

fn emit_validation_check(
    w: &mut CodeWriter,
    current: &ClassSpec,
    data_type: &DataType,
    valid: &ResolvedValidation,
    ctx: &TranslationContext<'_>,
    access_expr: &str,
) {
    let src_path = &valid.src_path;

    let (is_repeated, elem_dt) = match data_type {
        DataType::ArrayType { element, repeat } if !matches!(repeat, RepeatMode::None) => (true, element.as_ref()),
        _ => (false, data_type),
    };

    match &valid.rule {
        ValidationRule::Eq(expected_expr) => {
            if is_repeated {
                if matches!(
                    elem_dt,
                    DataType::Bytes { .. }
                        | DataType::CalcBytesType
                        | DataType::ArrayType { .. }
                        | DataType::Str { .. }
                        | DataType::CalcStrType
                        | DataType::EnumType { .. }
                        | DataType::Bits1 { .. }
                        | DataType::CalcBoolType
                ) {
                    let expected_str = translate_expr(&expected_expr, ctx);
                    w.puts(&format!("if !{access_expr}.iter().all(|_x| *_x == {expected_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotEqual, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                } else {
                    let ty_cast = validation_primitive_type(elem_dt);
                    let expected_str = translate_expr(&expected_expr, ctx);
                    w.puts(&format!("let expected: {ty_cast} = ({expected_str}).try_into()?;"));
                    w.puts(&format!("if !{access_expr}.iter().all(|_x| *_x == expected) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotEqual, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else if matches!(
                data_type,
                DataType::Bytes { .. }
                    | DataType::CalcBytesType
                    | DataType::ArrayType { .. }
                    | DataType::Str { .. }
                    | DataType::CalcStrType
                    | DataType::EnumType { .. }
                    | DataType::Bits1 { .. }
                    | DataType::CalcBoolType
            ) {
                let expected_str = translate_expr(&expected_expr, ctx);
                w.puts(&format!("if !(*{access_expr} == {expected_str}) {{"));
                w.inc();
                w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotEqual, src_path: \"{src_path}\".to_string() }}));"));
                w.dec();
                w.puts("}");
            } else {
                let resolved_dt = super::translator::resolve_switch_type(data_type);
                let ty_cast = validation_primitive_type(&resolved_dt);
                let is_numeric_switch = if let DataType::SwitchType { cases, .. } = data_type {
                    !cases.is_empty() && cases.values().all(super::translator::is_numeric_type)
                } else {
                    false
                };
                let deref = if is_numeric_switch { "" } else { "*" };
                let expected_str = translate_expr(&expected_expr, ctx);
                w.puts(&format!("let expected: {ty_cast} = ({expected_str}).try_into()?;"));
                w.puts(&format!("if !({deref}{access_expr} == expected) {{"));
                w.inc();
                w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotEqual, src_path: \"{src_path}\".to_string() }}));"));
                w.dec();
                w.puts("}");
            }
        }
        ValidationRule::Min(min_expr) => {
            let is_sizeof = matches!(min_expr, Expr::Name(n) if n == "_sizeof");
            let target_dt = if is_repeated { elem_dt } else { data_type };
            let resolved_target_dt = super::translator::resolve_switch_type(target_dt);
            let min_str = if is_sizeof {
                // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
                super::translator::calculate_class_seq_size(current).unwrap_or(0).to_string()
            } else {
                translate_expr(&min_expr, ctx)
            };
            if matches!(
                resolved_target_dt,
                DataType::Str { .. } | DataType::CalcStrType
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_str() >= {min_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_str() >= {min_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else if matches!(
                resolved_target_dt,
                DataType::Bytes { .. } | DataType::CalcBytesType | DataType::ArrayType { .. }
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_slice() >= ({min_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_slice() >= ({min_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else {
                let ty_cast = validation_primitive_type(&resolved_target_dt);
                let is_numeric_switch = if let DataType::SwitchType { cases, .. } = target_dt {
                    !cases.is_empty() && cases.values().all(super::translator::is_numeric_type)
                } else {
                    false
                };
                let deref = if is_numeric_switch { "" } else { "*" };
                if is_repeated {
                    w.puts(&format!("let min_val: {ty_cast} = ({min_str}).try_into()?;"));
                    w.puts(&format!("if !{access_expr}.iter().all(|_x| *_x >= min_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("let min_val: {ty_cast} = ({min_str}).try_into()?;"));
                    w.puts(&format!("if !({deref}{access_expr} >= min_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            }
        }
        ValidationRule::Max(max_expr) => {
            let is_sizeof = matches!(max_expr, Expr::Name(n) if n == "_sizeof");
            let target_dt = if is_repeated { elem_dt } else { data_type };
            let resolved_target_dt = super::translator::resolve_switch_type(target_dt);
            let max_str = if is_sizeof {
                // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
                super::translator::calculate_class_seq_size(current).unwrap_or(0).to_string()
            } else {
                translate_expr(&max_expr, ctx)
            };
            if matches!(
                resolved_target_dt,
                DataType::Str { .. } | DataType::CalcStrType
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_str() <= {max_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_str() <= {max_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else if matches!(
                resolved_target_dt,
                DataType::Bytes { .. } | DataType::CalcBytesType | DataType::ArrayType { .. }
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_slice() <= ({max_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_slice() <= ({max_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else {
                let ty_cast = validation_primitive_type(&resolved_target_dt);
                let is_numeric_switch = if let DataType::SwitchType { cases, .. } = target_dt {
                    !cases.is_empty() && cases.values().all(super::translator::is_numeric_type)
                } else {
                    false
                };
                let deref = if is_numeric_switch { "" } else { "*" };
                if is_repeated {
                    w.puts(&format!("let max_val: {ty_cast} = ({max_str}).try_into()?;"));
                    w.puts(&format!("if !{access_expr}.iter().all(|_x| *_x <= max_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("let max_val: {ty_cast} = ({max_str}).try_into()?;"));
                    w.puts(&format!("if !({deref}{access_expr} <= max_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            }
        }
        ValidationRule::Expr(expr) => {
            if is_repeated {
                let deref = if super::translator::needs_deref(elem_dt) { "*" } else { "&*" };
                w.puts(&format!("for _item in {access_expr}.iter() {{"));
                w.inc();
                w.puts(&format!("let _tmpa = {deref}_item;"));
                let val_ctx = ctx.with_element_type(Some(elem_dt));
                let cond_str = super::translator::translate_validation_custom_expr(&expr, elem_dt, current, &val_ctx);
                w.puts(&format!("if !({cond_str}) {{"));
                w.inc();
                w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::Expr, src_path: \"{src_path}\".to_string() }}));"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            } else {
                let deref = if super::translator::needs_deref(data_type) { "*" } else { "&*" };
                w.puts(&format!("let _borrowed = {access_expr};"));
                w.puts(&format!("let _tmpa = {deref}_borrowed;"));
                let val_ctx = ctx.with_element_type(Some(data_type));
                let cond_str = super::translator::translate_validation_custom_expr(&expr, data_type, current, &val_ctx);
                w.puts(&format!("if !({cond_str}) {{"));
                w.inc();
                w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::Expr, src_path: \"{src_path}\".to_string() }}));"));
                w.dec();
                w.puts("}");
            }
        }
        ValidationRule::Range(min_expr, max_expr) => {
            let is_sizeof_min = matches!(min_expr, Expr::Name(n) if n == "_sizeof");
            let is_sizeof_max = matches!(max_expr, Expr::Name(n) if n == "_sizeof");
            let target_dt = if is_repeated { elem_dt } else { data_type };
            let resolved_target_dt = super::translator::resolve_switch_type(target_dt);
            let min_str = if is_sizeof_min {
                // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
                super::translator::calculate_class_seq_size_with_root(current, Some(ctx.root)).unwrap_or(0).to_string()
            } else {
                translate_expr(min_expr, ctx)
            };
            let max_str = if is_sizeof_max {
                // Reason for fallback: dynamically sized or non-constant class sequence defaults to 0 for _sizeof
                super::translator::calculate_class_seq_size_with_root(current, Some(ctx.root)).unwrap_or(0).to_string()
            } else {
                translate_expr(max_expr, ctx)
            };
            if matches!(
                resolved_target_dt,
                DataType::Str { .. } | DataType::CalcStrType
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_str() >= {min_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !(_x.as_str() <= {max_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_str() >= {min_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !((*({access_expr})).as_str() <= {max_str}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else if matches!(
                resolved_target_dt,
                DataType::Bytes { .. } | DataType::CalcBytesType | DataType::ArrayType { .. }
            ) {
                if is_repeated {
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(_x.as_slice() >= ({min_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !(_x.as_slice() <= ({max_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if !((*({access_expr})).as_slice() >= ({min_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !((*({access_expr})).as_slice() <= ({max_str}).as_slice()) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else {
                let ty_cast = validation_primitive_type(&resolved_target_dt);
                let is_numeric_switch = if let DataType::SwitchType { cases, .. } = target_dt {
                    !cases.is_empty() && cases.values().all(super::translator::is_numeric_type)
                } else {
                    false
                };
                let deref = if is_numeric_switch { "" } else { "*" };
                if is_repeated {
                    w.puts(&format!("let min_val: {ty_cast} = ({min_str}).try_into()?;"));
                    w.puts(&format!("let max_val: {ty_cast} = ({max_str}).try_into()?;"));
                    w.puts(&format!("for _x in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !(*_x >= min_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !(*_x <= max_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("let min_val: {ty_cast} = ({min_str}).try_into()?;"));
                    w.puts(&format!("let max_val: {ty_cast} = ({max_str}).try_into()?;"));
                    w.puts(&format!("if !({deref}{access_expr} >= min_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::LessThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.puts(&format!("if !({deref}{access_expr} <= max_val) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::GreaterThan, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            }
        }
        ValidationRule::AnyOf(exprs) => {
            let target_dt = if is_repeated { elem_dt } else { data_type };
            let resolved_target_dt = super::translator::resolve_switch_type(target_dt);
            let is_bytes_or_str = matches!(
                resolved_target_dt,
                DataType::Bytes { .. }
                    | DataType::CalcBytesType
                    | DataType::Str { .. }
                    | DataType::CalcStrType
            );
            if is_bytes_or_str {
                let trans_exprs: Vec<String> = exprs.iter().map(|e| translate_expr(e, ctx)).collect();
                let check = trans_exprs.iter().map(|s| format!("_item == {s}")).collect::<Vec<_>>().join(" || ");
                if is_repeated {
                    w.puts(&format!("for _item in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !({check}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotAnyOf, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("let _item = &*{access_expr};"));
                    w.puts(&format!("if !({check}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotAnyOf, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            } else {
                let ty_cast = validation_primitive_type(&resolved_target_dt);
                let is_numeric_switch = if let DataType::SwitchType { cases, .. } = target_dt {
                    !cases.is_empty() && cases.values().all(super::translator::is_numeric_type)
                } else {
                    false
                };
                let deref = if is_numeric_switch { "" } else { "*" };
                let mut expected_vars = Vec::new();
                for (idx, e) in exprs.iter().enumerate() {
                    let trans = translate_expr(e, ctx);
                    let var_name = format!("expected_{idx}");
                    w.puts(&format!("let {var_name}: {ty_cast} = ({trans}).try_into()?;"));
                    expected_vars.push(var_name);
                }
                let check = expected_vars.iter().map(|v| format!("_item == {v}")).collect::<Vec<_>>().join(" || ");
                if is_repeated {
                    w.puts(&format!("for &_item in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if !({check}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotAnyOf, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("let _item = {deref}{access_expr};"));
                    w.puts(&format!("if !({check}) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotAnyOf, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            }
        }
        ValidationRule::InEnum => {
            let target_dt = if is_repeated { elem_dt } else { data_type };
            let resolved_target_dt = super::translator::resolve_switch_type(target_dt);
            if let DataType::EnumType { owner, name, .. } = &resolved_target_dt {
                let mut parts = owner.clone();
                parts.push(name.clone());
                let enum_type_name = types_to_class_name(&parts);
                if is_repeated {
                    w.puts(&format!("for _item in {access_expr}.iter() {{"));
                    w.inc();
                    w.puts(&format!("if matches!(_item, {enum_type_name}::Unknown(_)) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotInEnum, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts(&format!("if matches!(*{access_expr}, {enum_type_name}::Unknown(_)) {{"));
                    w.inc();
                    w.puts(&format!("return Err(KError::ValidationFailed(ValidationFailedError {{ kind: ValidationKind::NotInEnum, src_path: \"{src_path}\".to_string() }}));"));
                    w.dec();
                    w.puts("}");
                }
            }
        }
    }
}

fn emit_attr_validation(
    w: &mut CodeWriter,
    current: &ClassSpec,
    attr: &ResolvedAttr,
    ctx: &TranslationContext<'_>,
) {
    let Some(valid) = &attr.valid else { return };
    let id = &attr.id;
    let self_name = ctx.self_name();
    emit_validation_check(w, current, &attr.data_type, valid, ctx, &format!("{self_name}.{id}()"));
}

fn get_target_args(
    names: &[String],
    is_external: bool,
    self_name: &str,
    ctx: &TranslationContext<'_>,
    parent_expr: Option<&crate::spec::ValueOrExpr>,
) -> String {
    if is_external {
        return "None, None".to_string();
    }
    let target_class = super::translator::find_class_spec(ctx.root, names);
    let parent = match parent_expr {
        Some(crate::spec::ValueOrExpr::Bool(false)) => "None".to_string(),
        Some(crate::spec::ValueOrExpr::Expr(p)) if p == "_parent" => {
            format!("Some(SharedType::new({self_name}._parent.get_value().borrow().upgrade().as_ref().ok_or(KError::MissingParent)?.clone()))")
        }
        _ => {
            if let Some(tc) = target_class {
                if tc.parent_name.as_ref().is_some_and(|p| p.as_slice() == ["KStructUnit"]) {
                    "None".to_string()
                } else {
                    format!("Some({self_name}._self_shared.clone())")
                }
            } else {
                format!("Some({self_name}._self_shared.clone())")
            }
        }
    };
    format!("Some({self_name}._root.clone()), {parent}")
}

fn translate_args(args: &[Expr], ctx: &TranslationContext<'_>, into: bool) -> String {
    args.iter().map(|a| {
        let typ = super::translator::detect_type_approx(a, ctx);
        let mut translated = translate_expr(a, ctx);
        if translated == "_r" {
            translated = "OptRc::new(&_rrc)".to_string();
        }
        if matches!(a, Expr::Name(n) if n == "_index") {
            return "(_i).try_into().map_err(|_| KError::CastError)?".to_string();
        }
        if let Some(t) = &typ {
            if super::translator::is_numeric_type(t) {
                if into {
                    return format!("({translated}).try_into().map_err(|_| KError::CastError)?");
                }
            } else if !super::translator::is_copy_type(t) {
                return format!("{translated}.clone()");
            }
        }
        if translated.ends_with(']') {
            format!("{translated}.clone()")
        } else {
            translated
        }
    }).collect::<Vec<_>>().join(", ")
}

fn emit_single_read(
    w: &mut CodeWriter,
    current: &ClassSpec,
    attr: &ResolvedAttr,
    ctx: &TranslationContext<'_>,
    id: &str,
    self_name: &str,
) {
    if let DataType::SwitchType { cases, .. } = &attr.data_type {
        emit_switch_read(w, current, attr, ctx, id, self_name, cases);
        return;
    }

    if let DataType::UserType { names, is_external, args } = &attr.data_type {
        let type_name = types_to_class_name(names);
        let target_args = get_target_args(names, *is_external, self_name, ctx, attr.parent_expr.as_ref());
        let trans_args = translate_args(args, ctx, true);
        let io_ref = if attr.size_expr.is_some()
            || attr.size_eos
            || attr.process.is_some()
            || attr.terminator.is_some()
        {
            let mut read_call = if attr.size_eos {
                "_io.read_bytes_full()?".to_string()
            } else if let Some(size_expr) = &attr.size_expr {
                let s = expr_to_usize(size_expr, ctx);
                format!("_io.read_bytes({s})?")
            } else if let Some(term) = attr.terminator {
                format!(
                    "_io.read_bytes_term({term}, {}, {}, {})?",
                    attr.include, attr.consume, attr.eos_error
                )
            } else {
                "_io.read_bytes_full()?".to_string()
            };
            if attr.size_expr.is_some() || attr.size_eos {
                if attr.terminator.is_some() || attr.pad_right.is_some() {
                    let term_str = match attr.terminator {
                        Some(t) => format!("Some({t})"),
                        None => "None".to_string(),
                    };
                    let pad_str = match attr.pad_right {
                        Some(p) => format!("Some({p})"),
                        None => "None".to_string(),
                    };
                    read_call = format!(
                        "bytes_terminate_pad(&{read_call}, {term_str}, {}, {pad_str})",
                        attr.include
                    );
                }
            }
            w.puts(&format!("let _raw_{id} = {read_call};"));
            if has_substream(attr) || attr.raw_id.is_some() {
                w.puts(&format!("*{self_name}.{id}_raw.borrow_mut() = _raw_{id}.clone();"));
            }
            if let Some(proc) = &attr.process {
                let processed = translate_process(proc, &format!("_raw_{id}"), ctx);
                w.puts(&format!("let _processed_{id} = {processed};"));
                w.puts(&format!("let _io_{id} = BytesReader::from(_processed_{id});"));
            } else {
                w.puts(&format!("let _io_{id} = BytesReader::from(_raw_{id});"));
            }
            format!("&_io_{id}")
        } else {
            "&*_io".to_string()
        };
        let stream_type = if io_ref == "&*_io" { "_" } else { "BytesReader" };

        if trans_args.is_empty() {
            if current.has_dynamic_endian() {
                w.puts(&format!(
                    "let f = |t : &mut {type_name}| Ok(t.set_endian(*{self_name}._is_le.borrow()));"
                ));
                w.puts(&format!(
                    "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_ref}, {target_args}, &f)?.into();"
                ));
            } else {
                w.puts(&format!(
                    "let t = Self::read_into::<{stream_type}, {type_name}>({io_ref}, {target_args})?.into();"
                ));
            }
        } else {
            w.puts(&format!(
                "let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"
            ));
            w.puts(&format!(
                "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_ref}, {target_args}, &f)?.into();"
            ));
        }
        let escaped_id = escape_rust_keyword(id);
        w.puts(&format!("*{self_name}.{escaped_id}.borrow_mut() = t;"));
        return;
    }

    let val = read_expr_for_type(&attr.data_type, current, ctx, "_io");
    let escaped_id = escape_rust_keyword(id);
    w.puts(&format!("*{self_name}.{escaped_id}.borrow_mut() = {val};"));
}

fn emit_switch_read(
    w: &mut CodeWriter,
    current: &ClassSpec,
    attr: &ResolvedAttr,
    ctx: &TranslationContext<'_>,
    id: &str,
    self_name: &str,
    cases: &indexmap::IndexMap<String, DataType>,
) {
    let (switch_on, is_str_switch) = if let DataType::SwitchType { switch_on, .. } = &attr.data_type {
        let sw_type = super::translator::detect_type_approx(switch_on, ctx);
        let is_str = matches!(sw_type, Some(DataType::Str { .. } | DataType::CalcStrType));
        let is_bytes = matches!(sw_type, Some(DataType::Bytes { .. } | DataType::CalcBytesType))
            || cases.keys().any(|k| k.starts_with('[') && k.ends_with(']'));
        let sw_expr = translate_expr(switch_on, ctx);
        if is_str {
            let stripped = super::translator::remove_deref(&sw_expr);
            (format!("{stripped}.as_str()"), true)
        } else if is_bytes {
            let stripped = super::translator::remove_deref(&sw_expr);
            (format!("{stripped}.as_slice()"), false)
        } else {
            (sw_expr, false)
        }
    } else {
        ("*self_rc.switch_on()?".to_string(), false)
    };

    w.puts(&format!("match {switch_on} {{"));
    w.inc();

    let _any_user_types = cases.values().any(|ct| matches!(ct, DataType::UserType { .. }));
    let escaped_id = escape_rust_keyword(id);

    for (case_key, case_type) in cases {
        let pattern = if let Some(inner) = case_key
            .strip_prefix('\'')
            .and_then(|s| s.strip_suffix('\''))
            .or_else(|| case_key.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
        {
            if is_str_switch {
                format!("\"{inner}\"")
            } else if inner.len() > 1 {
                format!("b\"{inner}\"")
            } else {
                case_key.clone()
            }
        } else if let Some((p0, p1)) = case_key.split_once("::") {
            let enum_scoped = super::translator::resolve_enum_type_name(p0, current, Some(ctx.root));
            let variant = to_upper_camel_case(p1);
            format!("{enum_scoped}::{variant}")
        } else {
            case_key.clone()
        };

        w.puts(&format!("{pattern} => {{"));
        w.inc();

        if let DataType::UserType { names, is_external, args } = case_type {
            let read_call = if let Some(size_expr) = &attr.size_expr {
                let s = expr_to_usize(size_expr, ctx);
                format!("_io.read_bytes({s})?.into()")
            } else {
                "_io.read_bytes_full()?.into()".to_string()
            };
            w.puts(&format!("*{self_name}.{id}_raw.borrow_mut() = {read_call};"));
            w.puts(&format!("let {id}_raw = {self_name}.{id}_raw.borrow();"));
            if let Some(proc) = &attr.process {
                let processed = translate_process(proc, &format!("{id}_raw"), ctx);
                w.puts(&format!("let _t_{id}_raw_proc = {processed};"));
                w.puts(&format!("let _t_{id}_raw_io = BytesReader::from(_t_{id}_raw_proc);"));
            } else {
                w.puts(&format!("let _t_{id}_raw_io = BytesReader::from({id}_raw.clone());"));
            }

            let type_name = types_to_class_name(names);
            let target_args = get_target_args(names, *is_external, self_name, ctx, attr.parent_expr.as_ref());
            if args.is_empty() {
                w.puts(&format!(
                    "let t = Self::read_into::<BytesReader, {type_name}>(&_t_{id}_raw_io, {target_args})?.into();"
                ));
            } else {
                let trans_args = translate_args(args, ctx, true);
                w.puts(&format!(
                    "let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"
                ));
                w.puts(&format!(
                    "let t = Self::read_into_with_init::<BytesReader, {type_name}>(&_t_{id}_raw_io, {target_args}, &f)?.into();"
                ));
            }
            w.puts(&format!("*{self_name}.{escaped_id}.borrow_mut() = Some(t);"));
        } else {
            let val = read_expr_for_type(case_type, current, ctx, "_io");
            let val = if val.ends_with(".into()") {
                val
            } else {
                format!("{val}.into()")
            };
            w.puts(&format!("*{self_name}.{escaped_id}.borrow_mut() = Some({val});"));
        }

        w.dec();
        w.puts("}");
    }

    // Default case
    let is_exhaustive_bool = cases.contains_key("false") && cases.contains_key("true");
    if !cases.contains_key("_") && !is_exhaustive_bool {
        let has_bytes_case = cases.values().any(|c| matches!(c, DataType::Bytes { .. } | DataType::CalcBytesType));
        if has_bytes_case {
            w.puts("_ => {");
            w.inc();
            w.puts(&format!("*{self_name}.{escaped_id}.borrow_mut() = Some(_io.read_bytes_full()?.into());"));
            w.dec();
            w.puts("}");
        } else {
            w.puts("_ => {}");
        }
    }

    w.dec();
    w.puts("}");
}

fn translate_process(proc_str: &str, raw_bytes_var: &str, ctx: &TranslationContext<'_>) -> String {
    let proc_trimmed = proc_str.trim();
    if proc_trimmed == "zlib" {
        return format!("process_zlib(&{raw_bytes_var})?");
    }

    if let Ok(expr) = crate::expr::parser::parse_expr(proc_trimmed) {
        match expr {
            Expr::Call { func, args } => match func.as_ref() {
                Expr::Name(name) if name == "xor" => {
                    if let Some(first_arg) = args.first() {
                        if let Expr::List(items) = first_arg {
                            let byte_strs: Vec<String> = items
                                .iter()
                                .map(|item| {
                                    if let Expr::IntNum(n) = item {
                                        format!("{n}u8")
                                    } else {
                                        let s = translate_expr(item, ctx);
                                        format!("u8::try_from({s} & 0xff).unwrap_or(0)")
                                    }
                                })
                                .collect();
                            let list_str = byte_strs.join(", ");
                            return format!("process_xor_many(&{raw_bytes_var}, &[{list_str}])");
                        }
                        if let Expr::IntNum(n) = first_arg {
                            return format!("process_xor_one(&{raw_bytes_var}, {n}_u8)");
                        }
                        let approx_ty = super::translator::detect_type_approx(first_arg, ctx);
                        let is_bytes = matches!(
                            approx_ty,
                            Some(DataType::Bytes { .. } | DataType::CalcBytesType)
                        );
                        let trans_arg = translate_expr(first_arg, ctx);
                        if is_bytes {
                            return format!("process_xor_many(&{raw_bytes_var}, &{trans_arg})");
                        }
                        return format!(
                            "process_xor_one(&{raw_bytes_var}, u8::try_from(i64::try_from({trans_arg})? & 0xff)?)"
                        );
                    }
                }
                Expr::Name(name) if name == "rol" => {
                    if let Some(first_arg) = args.first() {
                        let trans_arg = translate_expr(first_arg, ctx);
                        return format!(
                            "process_rotate_left(&{raw_bytes_var}, i64::try_from({trans_arg})?)"
                        );
                    }
                }
                Expr::Name(name) if name == "ror" => {
                    if let Some(first_arg) = args.first() {
                        let trans_arg = translate_expr(first_arg, ctx);
                        return format!(
                            "process_rotate_right(&{raw_bytes_var}, i64::try_from({trans_arg})?)"
                        );
                    }
                }
                Expr::Name(name) => {
                    let cls = to_upper_camel_case(name);
                    let mod_name = name.to_string();
                    let arg_strs: Vec<String> = args
                        .iter()
                        .enumerate()
                        .map(|(idx, arg)| {
                            if let Expr::List(items) = arg {
                                let byte_strs: Vec<String> = items
                                    .iter()
                                    .map(|item| {
                                        if let Expr::IntNum(n) = item {
                                            format!("{n}u8")
                                        } else {
                                            let s = translate_expr(item, ctx);
                                            format!("u8::try_from({s} & 0xff)?")
                                        }
                                    })
                                    .collect();
                                format!("&[{}]", byte_strs.join(", "))
                            } else if let Expr::IfExp {
                                condition,
                                if_true,
                                if_false,
                            } = arg
                            {
                                let cond_str = translate_expr(condition, ctx);
                                let fmt_branch = |b: &Expr| {
                                    if let Expr::List(items) = b {
                                        let byte_strs: Vec<String> = items
                                            .iter()
                                            .map(|item| {
                                                if let Expr::IntNum(n) = item {
                                                    format!("{n}u8")
                                                } else {
                                                    let s = translate_expr(item, ctx);
                                                    format!("u8::try_from({s} & 0xff)?")
                                                }
                                            })
                                            .collect();
                                        format!("&[{}]", byte_strs.join(", "))
                                    } else {
                                        translate_expr(b, ctx)
                                    }
                                };
                                let t_str = fmt_branch(if_true);
                                let f_str = fmt_branch(if_false);
                                format!("if {cond_str} {{ {t_str} }} else {{ {f_str} }}")
                            } else if idx == 0 {
                                let s = translate_expr(arg, ctx);
                                format!("u8::try_from(i64::try_from({s})? & 0xff)?")
                            } else {
                                translate_expr(arg, ctx)
                            }
                        })
                        .collect();
                    let args_str = arg_strs.join(", ");
                    return format!(
                        "crate::{mod_name}::{cls}::new({args_str}).decode(&{raw_bytes_var}).map_err(|e| KError::BytesDecodingError {{ msg: e }})?"
                    );
                }
                Expr::Attribute { .. } => {
                    let mut parts = Vec::new();
                    let mut cur: &Expr = func.as_ref();
                    while let Expr::Attribute { value, attr } = cur {
                        parts.push(attr.as_str());
                        cur = value.as_ref();
                    }
                    if let Expr::Name(root_name) = cur {
                        parts.push(root_name.as_str());
                    }
                    parts.reverse();
                    let mod_path = parts
                        .iter()
                        .map(|p| to_upper_camel_case(p))
                        .collect::<Vec<_>>()
                        .join("::");
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|arg| {
                            let s = translate_expr(arg, ctx);
                            format!("u8::try_from(i64::try_from({s})? & 0xff)?")
                        })
                        .collect();
                    let args_str = arg_strs.join(", ");
                    return format!(
                        "crate::custom_fx::{mod_path}::new({args_str}).decode(&{raw_bytes_var}).map_err(|e| KError::BytesDecodingError {{ msg: e }})?"
                    );
                }
                _ => {}
            },
            Expr::Name(name) => {
                let cls = to_upper_camel_case(&name);
                return format!(
                    "crate::{name}::{cls}::new().decode(&{raw_bytes_var}).map_err(|e| KError::BytesDecodingError {{ msg: e }})?"
                );
            }
            _ => {}
        }
    }

    raw_bytes_var.to_string()
}

fn read_expr_for_type(
    dt: &DataType,
    current: &ClassSpec,
    ctx: &TranslationContext<'_>,
    io: &str,
) -> String {
    match dt {
        DataType::Int1 { signed } => {
            let prefix = if *signed { 's' } else { 'u' };
            format!("{io}.read_{prefix}1()?")
        }
        DataType::IntMulti { signed, width, endian } => {
            let prefix = if *signed { 's' } else { 'u' };
            if current.has_dynamic_endian() && (endian.is_none() || *endian == Some(Endianness::Inherited)) {
                let self_field = if ctx.self_name() == "self_rc" { "*self_rc._is_le.borrow()" } else { "*self._is_le.borrow()" };
                format!("if {self_field} == 1 {{ {io}.read_{prefix}{width}le()? }} else {{ {io}.read_{prefix}{width}be()? }}")
            } else {
                // Reason for fallback: unspecified endianness defaults to big endian per Kaitai spec
                let endian_str = match endian.unwrap_or(Endianness::Big) {
                    Endianness::Big | Endianness::Inherited => "be",
                    Endianness::Little => "le",
                };
                format!("{io}.read_{prefix}{width}{endian_str}()?")
            }
        }
        DataType::Float { width, endian } => {
            if current.has_dynamic_endian() && (endian.is_none() || *endian == Some(Endianness::Inherited)) {
                let self_field = if ctx.self_name() == "self_rc" { "*self_rc._is_le.borrow()" } else { "*self._is_le.borrow()" };
                format!("if {self_field} == 1 {{ {io}.read_f{width}le()? }} else {{ {io}.read_f{width}be()? }}")
            } else {
                // Reason for fallback: unspecified endianness defaults to big endian per Kaitai spec
                let endian_str = match endian.unwrap_or(Endianness::Big) {
                    Endianness::Big | Endianness::Inherited => "be",
                    Endianness::Little => "le",
                };
                format!("{io}.read_f{width}{endian_str}()?")
            }
        }
        DataType::Bits1 { bit_endian } => {
            let endian_str = match bit_endian {
                BitEndianness::Big => "be",
                BitEndianness::Little => "le",
            };
            format!("{io}.read_bits_int_{endian_str}(1)? != 0")
        }
        DataType::Bits { count, bit_endian } => {
            let endian_str = match bit_endian {
                BitEndianness::Big => "be",
                BitEndianness::Little => "le",
            };
            format!("{io}.read_bits_int_{endian_str}({count})?")
        }
        DataType::Bytes {
            size,
            size_eos,
            terminator,
            include,
            consume,
            eos_error,
            pad_right,
            process,
        } => {
            let mut raw_bytes = if *size_eos {
                format!("{io}.read_bytes_full()?")
            } else if let Some(size_expr) = size {
                let s = expr_to_usize(size_expr, ctx);
                format!("{io}.read_bytes({s})?")
            } else if let Some(term) = terminator {
                format!("{io}.read_bytes_term({term}, {include}, {consume}, {eos_error})?")
            } else {
                format!("{io}.read_bytes_full()?")
            };
            if size.is_some() || *size_eos {
                if terminator.is_some() || pad_right.is_some() {
                    let term_str = match terminator {
                        Some(t) => format!("Some({t})"),
                        None => "None".to_string(),
                    };
                    let pad_str = match pad_right {
                        Some(p) => format!("Some({p})"),
                        None => "None".to_string(),
                    };
                    raw_bytes = format!(
                        "bytes_terminate_pad(&{raw_bytes}, {term_str}, {include}, {pad_str})"
                    );
                }
            }
            if let Some(proc) = process {
                raw_bytes = translate_process(proc, &raw_bytes, ctx);
            }
            raw_bytes
        }
        DataType::Str {
            size,
            size_eos,
            encoding,
            terminator,
            consume,
            include,
            eos_error,
            pad_right,
            ..
        } => {
            let is_utf16 = encoding.as_deref().is_some_and(|e| {
                e.to_ascii_uppercase().starts_with("UTF-16")
            });
            let mut raw_bytes = if *size_eos {
                format!("{io}.read_bytes_full()?")
            } else if let Some(size_expr) = size {
                let s = expr_to_usize(size_expr, ctx);
                format!("{io}.read_bytes({s})?")
            } else if let Some(term) = terminator {
                if is_utf16 && *term == 0 {
                    format!("{io}.read_bytes_term_multi(&[0, 0], {include}, {consume}, {eos_error})?")
                } else {
                    format!("{io}.read_bytes_term({term}, {include}, {consume}, {eos_error})?")
                }
            } else {
                format!("{io}.read_bytes_full()?")
            };
            if size.is_some() || *size_eos {
                if terminator.is_some() || pad_right.is_some() {
                    let pad_str = match pad_right {
                        Some(p) => format!("Some({p})"),
                        None => "None".to_string(),
                    };
                    if is_utf16 && terminator == &Some(0) {
                        raw_bytes = format!(
                            "bytes_terminate_pad_multi(&{raw_bytes}, Some(&[0, 0][..]), {include}, {pad_str})"
                        );
                    } else {
                        let term_str = match terminator {
                            Some(t) => format!("Some({t})"),
                            None => "None".to_string(),
                        };
                        raw_bytes = format!(
                            "bytes_terminate_pad(&{raw_bytes}, {term_str}, {include}, {pad_str})"
                        );
                    }
                }
            }
            if let Some(enc) = encoding {
                format!("bytes_to_str(&{raw_bytes}, \"{enc}\")?")
            } else {
                format!("bytes_to_str(&{raw_bytes}, \"UTF-8\")?")
            }
        }
        DataType::EnumType { underlying, .. } => {
            let read_call = if let Some(under) = underlying {
                match under.as_ref() {
                    DataType::Int1 { signed } => {
                        let p = if *signed { 's' } else { 'u' };
                        format!("{io}.read_{p}1()?")
                    }
                    DataType::IntMulti { signed, width, endian } => {
                        let p = if *signed { 's' } else { 'u' };
                        if current.has_dynamic_endian() && (endian.is_none() || *endian == Some(Endianness::Inherited)) {
                            format!("{io}.read_{p}{width}()?")
                        } else {
                            // Reason for fallback: unspecified endianness defaults to big endian per Kaitai spec
                            let e = match endian.unwrap_or(Endianness::Big) {
                                Endianness::Big | Endianness::Inherited => "be",
                                Endianness::Little => "le",
                            };
                            format!("{io}.read_{p}{width}{e}()?")
                        }
                    }
                    DataType::Bits { count, bit_endian } => {
                        let endian_str = match bit_endian {
                            BitEndianness::Big => "be",
                            BitEndianness::Little => "le",
                        };
                        format!("{io}.read_bits_int_{endian_str}({count})?")
                    }
                    DataType::Bits1 { bit_endian } => {
                        let endian_str = match bit_endian {
                            BitEndianness::Big => "be",
                            BitEndianness::Little => "le",
                        };
                        format!("{io}.read_bits_int_{endian_str}(1)?")
                    }
                    _ => format!("{io}.read_u4()?")
                }
            } else {
                format!("{io}.read_u4()?")
            };
            let is_u64 = if let Some(under) = underlying {
                matches!(under.as_ref(), DataType::IntMulti { signed: false, width: 8, .. } | DataType::Bits { .. } | DataType::Bits1 { .. })
            } else {
                false
            };
            let is_s64 = if let Some(under) = underlying {
                matches!(under.as_ref(), DataType::IntMulti { signed: true, width: 8, .. })
            } else {
                false
            };
            if is_u64 {
                format!("i64::try_from({read_call})?.try_into()?")
            } else if is_s64 {
                format!("{read_call}.try_into()?")
            } else {
                format!("i64::from({read_call}).try_into()?")
            }
        }
        DataType::UserType { names, is_external, args } => {
            let type_name = types_to_class_name(names);
            let target_args = if *is_external {
                "None, None".to_string()
            } else {
                format!("Some({}._root.clone()), Some({}._self_shared.clone())", ctx.self_name(), ctx.self_name())
            };
            let io_ref = if io == "_io" { "&*_io".to_string() } else { format!("&{io}") };
            if args.is_empty() {
                format!("Self::read_into::<_, {type_name}>({io_ref}, {target_args})?.into()")
            } else {
                let trans_args = translate_args(args, ctx, true);
                format!("{{ let f = |t: &mut {type_name}| Ok(t.set_params({trans_args})); Self::read_into_with_init::<_, {type_name}>({io_ref}, {target_args}, &f)?.into() }}")
            }
        }
        DataType::ArrayType { element, .. } => {
            read_expr_for_type(element, current, ctx, io)
        }
        DataType::SwitchType { .. } => "None".to_string(),
        DataType::KaitaiStreamType => io.to_string(),
        DataType::CalcIntType => "0".to_string(),
        DataType::CalcFloatType => "0.0".to_string(),
        DataType::CalcBoolType => "false".to_string(),
        DataType::CalcStrType => "String::new()".to_string(),
        DataType::CalcBytesType => "Vec::new()".to_string(),
    }
}

fn emit_instances(w: &mut CodeWriter, current: &ClassSpec, root: &ClassSpec) {
    let class_name = current.class_type_name();
    let ctx = TranslationContext::new(current, root, false);

    if current.instances.is_empty() {
        w.puts(&format!("impl {class_name} {{"));
        w.puts("}");
        return;
    }

    if !current.instances.is_empty() {
        w.puts(&format!("impl {class_name} {{"));
        w.inc();

        for (inst_id, inst) in &current.instances {
            if inst.doc.is_some() || !inst.doc_refs.is_empty() {
                w.newline();
                w.docblock(inst.doc.as_deref(), &inst.doc_refs);
            }

            let ret_type = match &inst.data_type {
                DataType::SwitchType { .. } => {
                    let enum_name = switch_enum_name(current, inst_id);
                    format!("Option<{enum_name}>")
                }
                _ => rust_field_type(&inst.data_type, current, inst_id),
            };

            let escaped_inst_id = escape_rust_keyword(inst_id);
            w.puts(&format!("pub fn {escaped_inst_id}("));
            w.inc();
            w.puts("&self");
            w.dec();
            w.puts(&format!(") -> KResult<Ref<'_, {ret_type}>> {{"));
            w.inc();

            w.puts("let _io = self._io.borrow();");

            w.puts(&format!("if self.f_{inst_id}.get() {{"));
            w.inc();
            w.puts(&format!("return Ok(self.{escaped_inst_id}.borrow());"));
            w.dec();
            w.puts("}");
            if !matches!(inst.data_type, DataType::UserType { .. }) {
                w.puts(&format!("self.f_{inst_id}.set(true);"));
            }

            if let Some(if_expr) = &inst.if_expr {
                let cond = translate_expr(if_expr, &ctx);
                w.puts(&format!("if {cond} {{"));
                w.inc();
            }

            // Value calculation or parse
            if let Some(value_expr) = &inst.value_expr {
                let val_ctx = if let DataType::ArrayType { element, .. } = &inst.data_type {
                    ctx.with_element_type(Some(element))
                } else {
                    ctx.clone()
                };
                let expr_str = translate_expr(value_expr, &val_ctx);
                let val_str = match &inst.data_type {
                    DataType::UserType { .. } => {
                        format!("{}.clone()", super::translator::remove_deref(&expr_str))
                    }
                    DataType::Str { .. } | DataType::CalcStrType => {
                        format!("{}.to_string()", super::translator::remove_deref(&expr_str))
                    }
                    DataType::Bytes { .. } | DataType::CalcBytesType | DataType::ArrayType { .. } => {
                        let no_deref = super::translator::remove_deref(&expr_str);
                        // Reason for fallback: string without leading ampersand remains unchanged
                        let no_vec = no_deref.strip_prefix('&').unwrap_or(no_deref);
                        if no_vec.starts_with("vec![") {
                            no_vec.to_string()
                        } else {
                            format!("{no_vec}.to_vec()")
                        }
                    }
                    DataType::EnumType { .. } => {
                        if inst.enum_name.is_some() {
                            match super::translator::detect_type_approx(value_expr, &val_ctx) {
                                Some(
                                    DataType::Bits { .. }
                                    | DataType::IntMulti { signed: false, width: 8, .. },
                                ) => {
                                    format!("i64::try_from({expr_str})?.try_into()?")
                                }
                                Some(DataType::IntMulti { signed: true, width: 8, .. }) => {
                                    format!("({expr_str}).try_into()?")
                                }
                                Some(
                                    DataType::Int1 { .. }
                                    | DataType::IntMulti { .. }
                                    | DataType::CalcIntType,
                                ) => {
                                    format!("i64::from({expr_str}).try_into()?")
                                }
                                _ => format!("i64::try_from({expr_str})?.try_into()?"),
                            }
                        } else {
                            expr_str
                        }
                    }
                    _ => {
                        let derefed = if (expr_str.starts_with("self.")
                            || expr_str.starts_with("self_rc.")
                            || expr_str.starts_with("_r.")
                            || expr_str.starts_with("_prc."))
                            && !expr_str.starts_with('*')
                            && !expr_str.ends_with(".len()")
                            && !expr_str.ends_with(".pos()")
                            && !expr_str.ends_with(".size()")
                            && !expr_str.ends_with(']')
                            && !expr_str.ends_with('?')
                            && !expr_str.contains(".parse")
                            && !expr_str.contains(' ')
                            && !super::translator::is_usize_expr_str(&expr_str)
                            && !super::translator::is_numeric_switch_call(&expr_str, current, ctx.root)
                        {
                            format!("*{expr_str}")
                        } else {
                            expr_str
                        };
                        format!("({derefed}).try_into()?")
                    }
                };
                w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = {val_str};"));
            } else {
                emit_parse_instance_body(w, current, inst_id, inst, &ctx);
            }

            if inst.if_expr.is_some() {
                w.dec();
                w.puts("}");
            }

            w.puts(&format!("Ok(self.{escaped_inst_id}.borrow())"));
            w.dec();
            w.puts("}");
        }

        w.dec();
        w.puts("}");
    }
}

fn emit_parse_instance_body(
    w: &mut CodeWriter,
    current: &ClassSpec,
    inst_id: &str,
    inst: &ResolvedInstance,
    ctx: &TranslationContext<'_>,
) {
    let escaped_inst_id = escape_rust_keyword(inst_id);
    let io_var = if let Some(io_ex) = &inst.io_expr {
        let io_str = translate_expr(io_ex, ctx);
        let clean = io_str.trim_start_matches('&').trim_start_matches('*');
        if clean.starts_with("if ") {
            w.puts(&format!("let io = {clean};"));
        } else {
            w.puts(&format!("let io = KStream::clone(&*{clean});"));
        }
        "io"
    } else {
        "_io"
    };

    if let Some(pos) = &inst.pos_expr {
        w.puts(&format!("let _pos = {io_var}.pos();"));
        let pos_str = expr_to_usize(pos, ctx);
        w.puts(&format!("{io_var}.seek({pos_str})?;"));
    }

    match &inst.data_type {
        DataType::SwitchType { switch_on, cases } => {
            let sw_type = super::translator::detect_type_approx(switch_on, ctx);
            let is_str_switch = matches!(sw_type, Some(DataType::Str { .. } | DataType::CalcStrType))
                || cases.keys().any(|k| (k.starts_with('"') && k.ends_with('"')) || (k.starts_with('\'') && k.ends_with('\'')));
            let is_bytes_switch = matches!(sw_type, Some(DataType::Bytes { .. } | DataType::CalcBytesType))
                || cases.keys().any(|k| k.starts_with('[') && k.ends_with(']'));
            let switch_on_expr = translate_expr(switch_on, ctx);
            let match_target = if is_str_switch {
                let stripped = super::translator::remove_deref(&switch_on_expr);
                format!("{stripped}.as_str()")
            } else if is_bytes_switch {
                let stripped = super::translator::remove_deref(&switch_on_expr);
                format!("{stripped}.as_slice()")
            } else if (switch_on_expr.starts_with("self.")
                || switch_on_expr.starts_with("self_rc.")
                || switch_on_expr.starts_with("_r.")
                || switch_on_expr.starts_with("_prc."))
                && !switch_on_expr.starts_with('*')
            {
                format!("*{switch_on_expr}")
            } else {
                switch_on_expr
            };

            w.puts(&format!("match {match_target} {{"));
            w.inc();

            for (case_key, case_type) in cases {
                if case_key == "_" {
                    continue;
                }
                let pattern = if let Some(inner) = case_key
                    .strip_prefix('\'')
                    .and_then(|s| s.strip_suffix('\''))
                    .or_else(|| case_key.strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                {
                    if is_str_switch {
                        format!("\"{inner}\"")
                    } else if inner.len() > 1 {
                        format!("b\"{inner}\"")
                    } else {
                        case_key.clone()
                    }
                } else if let Some((p0, p1)) = case_key.split_once("::") {
                    let enum_scoped = super::translator::resolve_enum_type_name(p0, current, Some(ctx.root));
                    let variant = to_upper_camel_case(p1);
                    format!("{enum_scoped}::{variant}")
                } else if is_str_switch && !case_key.starts_with('"') {
                    format!("\"{case_key}\"")
                } else {
                    case_key.clone()
                };

                w.puts(&format!("{pattern} => {{"));
                w.inc();

                let (io_ref, stream_type) = if let Some(size) = &inst.size_expr {
                    let size_str = expr_to_usize(size, ctx);
                    w.puts(&format!("*self.{inst_id}_raw.borrow_mut() = {io_var}.read_bytes({size_str})?.into();"));
                    w.puts(&format!("let {inst_id}_raw = self.{inst_id}_raw.borrow();"));
                    w.puts(&format!("let _t_{inst_id}_raw_io = BytesReader::from({inst_id}_raw.clone());"));
                    (format!("&_t_{inst_id}_raw_io"), "BytesReader")
                } else {
                    (if io_var == "_io" { "&*_io".to_string() } else { format!("&{io_var}") }, "_")
                };

                if let DataType::UserType { names, is_external, args } = case_type {
                    let type_name = types_to_class_name(names);
                    let target_args = get_target_args(names, *is_external, "self", ctx, inst.parent_expr.as_ref());
                    let target_class = super::translator::find_class_spec(ctx.root, names);
                    let has_dyn_endian = target_class.is_some_and(ClassSpec::has_dynamic_endian)
                        || (current.has_dynamic_endian() && target_class.is_some_and(|tc| tc.meta_endian == Some(Endianness::Inherited)));
                    let trans_args = translate_args(args, ctx, true);
                    if !trans_args.is_empty() {
                        w.puts(&format!(
                            "let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"
                        ));
                        w.puts(&format!(
                            "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_ref}, {target_args}, &f)?.into();"
                        ));
                    } else if has_dyn_endian {
                        w.puts(&format!(
                            "let f = |t : &mut {type_name}| Ok(t.set_endian(*self._is_le.borrow()));"
                        ));
                        w.puts(&format!(
                            "let t = Self::read_into_with_init::<{stream_type}, {type_name}>({io_ref}, {target_args}, &f)?.into();"
                        ));
                    } else {
                        w.puts(&format!(
                            "let t = Self::read_into::<{stream_type}, {type_name}>({io_ref}, {target_args})?.into();"
                        ));
                    }
                    w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Some(t);"));
                }

                w.dec();
                w.puts("}");
            }

            let is_exhaustive_bool = cases.contains_key("false") && cases.contains_key("true");
            if cases.contains_key("_") {
                let fallback = if let Some(size) = &inst.size_expr {
                    let size_str = expr_to_usize(size, ctx);
                    format!("{io_var}.read_bytes({size_str})?.into()")
                } else {
                    format!("{io_var}.read_bytes_full()?.into()")
                };
                w.puts("_ => {");
                w.inc();
                w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Some({fallback});"));
                w.dec();
                w.puts("}");
            } else if !cases.contains_key("_") && !is_exhaustive_bool {
                let has_bytes_case = cases.values().any(|c| matches!(c, DataType::Bytes { .. } | DataType::CalcBytesType));
                if has_bytes_case {
                    let fallback = if let Some(size) = &inst.size_expr {
                        let size_str = expr_to_usize(size, ctx);
                        format!("{io_var}.read_bytes({size_str})?.into()")
                    } else {
                        format!("{io_var}.read_bytes_full()?.into()")
                    };
                    w.puts("_ => {");
                    w.inc();
                    w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Some({fallback});"));
                    w.dec();
                    w.puts("}");
                } else {
                    w.puts("_ => {}");
                }
            }

            w.dec();
            w.puts("}");
        }
        DataType::ArrayType { element, repeat } => {
            match repeat {
                RepeatMode::None => {
                    let elem_val = read_expr_for_type(element, current, ctx, "_io");
                    w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = {elem_val};"));
                }
                RepeatMode::Expr(repeat_expr) => {
                    if let Some(size) = &inst.size_expr {
                        w.puts(&format!("*self.{inst_id}_raw.borrow_mut() = Vec::new();"));
                        w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Vec::new();"));
                        let count_str = expr_to_usize(repeat_expr, ctx);
                        w.puts(&format!("let l_{inst_id} = {count_str};"));
                        w.puts(&format!("for _i in 0_usize..l_{inst_id} {{"));
                        w.inc();
                        let size_str = expr_to_usize(size, ctx);
                        w.puts(&format!("self.{inst_id}_raw.borrow_mut().push(_io.read_bytes({size_str})?.into());"));
                        w.puts(&format!("let {inst_id}_raw = self.{inst_id}_raw.borrow();"));
                        w.puts(&format!("let _io_{inst_id}_raw = BytesReader::from({inst_id}_raw.last().ok_or(KError::EmptyIterator)?.clone());"));
                        if let DataType::UserType { names, is_external, args: _ } = element.as_ref() {
                            let type_name = types_to_class_name(names);
                            let target_args = get_target_args(names, *is_external, "self", ctx, inst.parent_expr.as_ref());
                            if current.has_dynamic_endian() {
                                w.puts(&format!("let f = |t : &mut {type_name}| Ok(t.set_endian(*self._is_le.borrow()));"));
                                w.puts(&format!("let t = Self::read_into_with_init::<BytesReader, {type_name}>(&_io_{inst_id}_raw, {target_args}, &f)?.into();"));
                            } else {
                                w.puts(&format!("let t = Self::read_into::<BytesReader, {type_name}>(&_io_{inst_id}_raw, {target_args})?.into();"));
                            }
                            w.puts(&format!("self.{escaped_inst_id}.borrow_mut().push(t);"));
                        } else {
                            let elem_val = read_expr_for_type(element, current, ctx, &format!("_io_{inst_id}_raw"));
                            w.puts(&format!("self.{escaped_inst_id}.borrow_mut().push({elem_val});"));
                        }
                        w.dec();
                        w.puts("}");
                    } else {
                        w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Vec::new();"));
                        let count_str = expr_to_usize(repeat_expr, ctx);
                        w.puts(&format!("let l_{inst_id} = {count_str};"));
                        w.puts(&format!("for _i in 0_usize..l_{inst_id} {{"));
                        w.inc();
                        emit_read_array_element(w, element, current, ctx, inst_id, "self", "_io");
                        w.dec();
                        w.puts("}");
                    }
                }
                RepeatMode::Eos => {
                    w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Vec::new();"));
                    w.puts("{");
                    w.inc();
                    w.puts("let mut _i = 0_usize;");
                    w.puts("while !_io.is_eof() {");
                    w.inc();
                    emit_read_array_element(w, element, current, ctx, inst_id, "self", "_io");
                    w.puts("_i = _i.saturating_add(1);");
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                }
                RepeatMode::Until(until_expr) => {
                    w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = Vec::new();"));
                    w.puts("{");
                    w.inc();
                    w.puts("let mut _i = 0_usize;");
                    w.puts("loop {");
                    w.inc();
                    emit_read_array_element(w, element, current, ctx, inst_id, "self", "_io");
                    w.puts(&format!("let _t_{inst_id} = self.{escaped_inst_id}.borrow();"));
                    w.puts(&format!("let Some(_tmpa) = _t_{inst_id}.last() else {{ break; }};"));
                    if super::translator::needs_deref(element) {
                        w.puts("let _tmpa = *_tmpa;");
                    }
                    w.puts("_i = _i.saturating_add(1);");
                    let until_ctx = ctx.with_element_type(Some(element));
                    let until_str = translate_expr(until_expr, &until_ctx);
                    w.puts(&format!("if {until_str} {{ break; }}"));
                    w.dec();
                    w.puts("}");
                    w.dec();
                    w.puts("}");
                }
            }
        }
        DataType::UserType { names, is_external, args } => {
            let type_name = types_to_class_name(names);
            let target_args = get_target_args(names, *is_external, "self", ctx, inst.parent_expr.as_ref());
            let target_class = super::translator::find_class_spec(ctx.root, names);
            let has_dyn_endian = target_class.is_some_and(ClassSpec::has_dynamic_endian)
                || (current.has_dynamic_endian() && target_class.is_some_and(|tc| tc.meta_endian == Some(Endianness::Inherited)));
            let trans_args = translate_args(args, ctx, true);

            if let Some(size) = &inst.size_expr {
                let size_str = expr_to_usize(size, ctx);
                w.puts(&format!("*self.{inst_id}_raw.borrow_mut() = _io.read_bytes({size_str})?.into();"));
                w.puts(&format!("let {inst_id}_raw = self.{inst_id}_raw.borrow();"));
                w.puts(&format!("let _t_{inst_id}_raw_io = BytesReader::from({inst_id}_raw.clone());"));
                if !trans_args.is_empty() {
                    w.puts(&format!("let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"));
                    w.puts(&format!("let t = Self::read_into_with_init::<BytesReader, {type_name}>(&_t_{inst_id}_raw_io, {target_args}, &f)?.into();"));
                } else if has_dyn_endian {
                    w.puts(&format!("let f = |t : &mut {type_name}| Ok(t.set_endian(*self._is_le.borrow()));"));
                    w.puts(&format!("let t = Self::read_into_with_init::<BytesReader, {type_name}>(&_t_{inst_id}_raw_io, {target_args}, &f)?.into();"));
                } else {
                    w.puts(&format!("let t = Self::read_into::<BytesReader, {type_name}>(&_t_{inst_id}_raw_io, {target_args})?.into();"));
                }
                w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = t;"));
            } else {
                let io_ref = if io_var == "_io" { "&*_io".to_string() } else { format!("&{io_var}") };
                if !trans_args.is_empty() {
                    w.puts(&format!("let f = |t : &mut {type_name}| Ok(t.set_params({trans_args}));"));
                    w.puts(&format!("let t = Self::read_into_with_init::<_, {type_name}>({io_ref}, {target_args}, &f)?.into();"));
                } else if has_dyn_endian {
                    w.puts(&format!("let f = |t : &mut {type_name}| Ok(t.set_endian(*self._is_le.borrow()));"));
                    w.puts(&format!("let t = Self::read_into_with_init::<_, {type_name}>({io_ref}, {target_args}, &f)?.into();"));
                } else {
                    w.puts(&format!("let t = Self::read_into::<_, {type_name}>({io_ref}, {target_args})?.into();"));
                }
                w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = t;"));
            }
        }
        other => {
            let elem_val = read_expr_for_type(other, current, ctx, io_var);
            w.puts(&format!("*self.{escaped_inst_id}.borrow_mut() = {elem_val};"));
        }
    }

    if let Some(valid) = &inst.valid {
        emit_validation_check(w, current, &inst.data_type, valid, ctx, &format!("self.{escaped_inst_id}.borrow()"));
    }

    if inst.pos_expr.is_some() {
        w.puts(&format!("{io_var}.seek(_pos)?;"));
    }
}

fn emit_attribute_getters(w: &mut CodeWriter, current: &ClassSpec) {
    let class_name = current.class_type_name();

    for attr in &current.seq {
        if attr.doc.is_some() || !attr.doc_refs.is_empty() {
            w.newline();
            w.docblock(attr.doc.as_deref(), &attr.doc_refs);
        }
        w.puts(&format!("impl {class_name} {{"));
        w.inc();
        let escaped_id = escape_rust_keyword(&attr.id);
        if let DataType::SwitchType { cases, .. } = &attr.data_type {
            let is_numeric = !cases.is_empty() && cases.values().all(super::translator::is_numeric_type);
            if is_numeric {
                let resolved = super::translator::resolve_switch_type(&attr.data_type);
                let target_ret = rust_field_type(&resolved, current, &attr.id);
                w.puts(&format!("pub fn {}(&self) -> {target_ret} {{", attr.id));
                w.inc();
                w.puts("// Reason for fallback: unwrap on parsed numeric switch option falls back to 0");
                w.puts(&format!("self.{escaped_id}.borrow().as_ref().and_then(|v| {target_ret}::try_from(v).ok()).unwrap_or(0)"));
                w.dec();
                w.puts("}");
            }
            let fn_name = if is_numeric {
                let candidate = format!("{}_enum", attr.id);
                if current.instances.contains_key(&candidate) {
                    format!("{}_switch_enum", attr.id)
                } else {
                    candidate
                }
            } else {
                escaped_id.clone()
            };
            let enum_name = switch_enum_name(current, &attr.id);
            w.puts(&format!("pub fn {fn_name}(&self) -> Ref<'_, Option<{enum_name}>> {{"));
            w.inc();
            w.puts(&format!("self.{escaped_id}.borrow()"));
            w.dec();
            w.puts("}");
        } else {
            let ret_type = rust_field_type(&attr.data_type, current, &attr.id);
            w.puts(&format!("pub fn {escaped_id}(&self) -> Ref<'_, {ret_type}> {{"));
            w.inc();
            w.puts(&format!("self.{escaped_id}.borrow()"));
            w.dec();
            w.puts("}");
        }
        w.dec();
        w.puts("}");
    }

    // _io getter
    w.puts(&format!("impl {class_name} {{"));
    w.inc();
    w.puts("pub fn _io(&self) -> Ref<'_, BytesReader> {");
    w.inc();
    w.puts("self._io.borrow()");
    w.dec();
    w.puts("}");
    w.dec();
    w.puts("}");

    // Raw getters for substream attributes
    for attr in &current.seq {
        if attr.raw_id.is_some() || has_substream(attr) {
            w.puts(&format!("impl {class_name} {{"));
            w.inc();
            w.puts(&format!("pub fn {}_raw(&self) -> Ref<'_, Vec<u8>> {{", attr.id));
            w.inc();
            w.puts(&format!("self.{}_raw.borrow()", attr.id));
            w.dec();
            w.puts("}");
            w.dec();
            w.puts("}");
        }
    }
    for (inst_id, inst) in &current.instances {
        if inst.pos_expr.is_some() || inst.io_expr.is_some() {
            if matches!(inst.data_type, DataType::ArrayType { .. }) && inst.size_expr.is_some() {
                w.puts(&format!("impl {class_name} {{"));
                w.inc();
                w.puts(&format!("pub fn {inst_id}_raw(&self) -> Ref<'_, Vec<Vec<u8>>> {{"));
                w.inc();
                w.puts(&format!("self.{inst_id}_raw.borrow()"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            } else if matches!(
                inst.data_type,
                DataType::SwitchType { .. } | DataType::UserType { .. }
            ) && inst.size_expr.is_some() {
                w.puts(&format!("impl {class_name} {{"));
                w.inc();
                w.puts(&format!("pub fn {inst_id}_raw(&self) -> Ref<'_, Vec<u8>> {{"));
                w.inc();
                w.puts(&format!("self.{inst_id}_raw.borrow()"));
                w.dec();
                w.puts("}");
                w.dec();
                w.puts("}");
            }
        }
    }
}

fn emit_enums(w: &mut CodeWriter, current: &ClassSpec) {
    for (enum_name, resolved_enum) in &current.enums {
        let mut parts = current.name.clone();
        parts.push(enum_name.clone());
        let full_enum_name = types_to_class_name(&parts);

        let has_unknown = resolved_enum
            .values
            .values()
            .any(|l| to_upper_camel_case(l) == "Unknown");
        let catchall = if has_unknown { "UnknownVariant" } else { "Unknown" };

        w.puts("#[derive(Debug, PartialEq, Copy, Clone)]");
        w.puts(&format!("pub enum {full_enum_name} {{"));
        w.inc();

        for (val, label) in &resolved_enum.values {
            if let Some(doc) = resolved_enum.value_docs.get(val) {
                // Reason for fallback: absent doc_ref list for enum value defaults to empty slice
                let doc_refs = resolved_enum
                    .value_doc_refs
                    .get(val)
                    .map_or(&[][..], Vec::as_slice);
                w.newline();
                w.docblock(Some(doc.as_str()), doc_refs);
            }
            let variant_name = to_upper_camel_case(label);
            w.puts(&format!("{variant_name},"));
        }
        w.puts(&format!("{catchall}(i64),"));
        w.dec();
        w.puts("}");
        w.newline();

        // TryFrom<i64>
        w.puts(&format!("impl TryFrom<i64> for {full_enum_name} {{"));
        w.inc();
        w.puts("type Error = KError;");
        w.puts(&format!("fn try_from(flag: i64) -> KResult<{full_enum_name}> {{"));
        w.inc();
        w.puts("match flag {");
        w.inc();
        for (val, label) in &resolved_enum.values {
            let variant_name = to_upper_camel_case(label);
            w.puts(&format!("{val} => Ok({full_enum_name}::{variant_name}),"));
        }
        w.puts(&format!("_ => Ok({full_enum_name}::{catchall}(flag)),"));
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.newline();

        // From<&Enum> for i64
        w.puts(&format!("impl From<&{full_enum_name}> for i64 {{"));
        w.inc();
        w.puts(&format!("fn from(v: &{full_enum_name}) -> Self {{"));
        w.inc();
        w.puts("match *v {");
        w.inc();
        for (val, label) in &resolved_enum.values {
            let variant_name = to_upper_camel_case(label);
            w.puts(&format!("{full_enum_name}::{variant_name} => {val},"));
        }
        w.puts(&format!("{full_enum_name}::{catchall}(v) => v"));
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.dec();
        w.puts("}");
        w.newline();

        // Default
        w.puts(&format!("impl Default for {full_enum_name} {{"));
        w.inc();
        w.puts(&format!("fn default() -> Self {{ {full_enum_name}::{catchall}(0) }}"));
        w.dec();
        w.puts("}");
        w.newline();
    }
}

fn is_bit_type(dt: &DataType) -> bool {
    match dt {
        DataType::Bits1 { .. } | DataType::Bits { .. } => true,
        DataType::EnumType { underlying: Some(u), .. } => is_bit_type(u),
        _ => false,
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