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

//! AST -> canonical `.sl3` / `.sr3` source formatter for StageL 3 and
//! StageR 3.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, Comment, Expr, ImportTarget, Module, RoutineDecl, StageDialect,
    StageLType, StructTypeRef, SymbolTarget, TestExpected, TopDecl,
    format_dc_shorthand,
};
use super::dc_chars_to_dcts_string;
use crate::DcChar;
use std::fmt::Write as _;

/// Renders a StageL 3 `Module` AST into canonical `.sl3` source text,
/// reconstructing comments, `: @dc <shorthand>` / `pub dc/<shorthand>`,
/// camelCase identifiers, and `test` blocks.
pub(super) fn format_module_to_sl3(module: &Module) -> Result<String> {
    ensure!(
        module.dialect == StageDialect::StageL3,
        "Expected StageL 3 module for format_module_to_sl3"
    );
    format_module_inner(module)
}

/// Renders a StageR 3 `Module` AST into canonical `.sr3` source text.
pub(super) fn format_module_to_sr3(module: &Module) -> Result<String> {
    ensure!(
        module.dialect == StageDialect::StageR3,
        "Expected StageR 3 module for format_module_to_sr3"
    );
    format_module_inner(module)
}

/// Formats a single top-level declaration (`TopDecl`) back into canonical
/// `.sl3` source text.
pub(super) fn format_top_decl_to_sl3(decl: &TopDecl) -> Result<String> {
    let mut out = String::new();
    write_top_decl(&mut out, decl)?;
    Ok(out)
}

/// Formats a single top-level StageR 3 declaration (`TopDecl`) back into
/// canonical `.sr3` source text.
pub(super) fn format_top_decl_to_sr3(decl: &TopDecl) -> Result<String> {
    format_top_decl_to_sl3(decl)
}

fn format_module_inner(module: &Module) -> Result<String> {
    let mut out = String::new();
    for c in &module.leading_comments {
        write_comment_line(&mut out, 0, c)?;
    }
    if let Some(ns_segs) = &module.namespace {
        writeln!(out, "ns {}.", ns_segs.join("/"))?;
    }
    for u in &module.uses {
        for c in &u.comments_before {
            write_comment_line(&mut out, 0, c)?;
        }
        write!(out, "use {}:", u.path.join("/"))?;
        for item in &u.items {
            match &item.target {
                ImportTarget::Name(n) => write!(out, " {n}")?,
                ImportTarget::Dc(dc) => {
                    write!(out, " dc/{}", format_dc_shorthand(*dc))?;
                }
            }
            if let Some(alias) = &item.alias {
                write!(out, " as {alias}")?;
            }
        }
        writeln!(out, " ::.")?;
    }
    if !module.items.is_empty()
        && (!module.leading_comments.is_empty()
            || module.namespace.is_some()
            || !module.uses.is_empty())
    {
        writeln!(out)?;
    }

    for (idx, item) in module.items.iter().enumerate() {
        write_top_decl(&mut out, item)?;
        if idx.saturating_add(1) < module.items.len() {
            writeln!(out)?;
        }
    }
    Ok(out)
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn write_comment_line(
    out: &mut String,
    indent: usize,
    comment: &Comment,
) -> Result<()> {
    write_indent(out, indent);
    if comment.text.is_empty() {
        writeln!(out, ":")?;
    } else {
        writeln!(out, ": {}", comment.text)?;
    }
    Ok(())
}

fn write_dc_directive_if_needed(
    out: &mut String,
    indent: usize,
    dc_id: Option<DcChar>,
    pub_dc_syntax: bool,
) -> Result<()> {
    if let Some(dc) = dc_id
        && !pub_dc_syntax
    {
        write_indent(out, indent);
        writeln!(out, ": @dc {}", format_dc_shorthand(dc))?;
    }
    Ok(())
}

fn write_decl_header_prefix(
    out: &mut String,
    indent: usize,
    comments_before: &[Comment],
    is_pub: bool,
    pub_dc_syntax: bool,
    dc_id: Option<DcChar>,
    private_kw: Option<&str>,
) -> Result<()> {
    for c in comments_before {
        write_comment_line(out, indent, c)?;
    }
    write_dc_directive_if_needed(out, indent, dc_id, pub_dc_syntax)?;
    write_indent(out, indent);
    if let Some(kw) = private_kw {
        write!(out, "{kw} ")?;
    } else if is_pub {
        write!(out, "pub ")?;
        if pub_dc_syntax && let Some(dc) = dc_id {
            write!(out, "dc/{} ", format_dc_shorthand(dc))?;
        }
    }
    Ok(())
}

fn write_symbol_target(out: &mut String, target: &SymbolTarget) -> Result<()> {
    match target {
        SymbolTarget::Ident(n) => out.push_str(n),
        SymbolTarget::Dc(dc) => {
            write!(out, "dc/{}", format_dc_shorthand(*dc))?;
        }
    }
    Ok(())
}

fn write_top_decl(out: &mut String, item: &TopDecl) -> Result<()> {
    match item {
        TopDecl::Struct(s) => {
            write_decl_header_prefix(
                out,
                0,
                &s.comments_before,
                s.is_pub,
                s.pub_dc_syntax,
                s.dc_id,
                None,
            )?;
            write!(out, "struct {}", s.name)?;
            if s.inline_body {
                write!(out, " ::")?;
                for fty in &s.fields {
                    write!(out, " {fty}")?;
                }
                writeln!(out, ".")?;
            } else {
                writeln!(out)?;
                for fty in &s.fields {
                    writeln!(out, "    {fty}.")?;
                }
            }
        }
        TopDecl::Value(v) => {
            let private_kw = if v.is_pub { None } else { Some("const") };
            write_decl_header_prefix(
                out,
                0,
                &v.comments_before,
                v.is_pub,
                v.pub_dc_syntax,
                v.dc_id,
                private_kw,
            )?;
            write!(out, "{}/{} ", v.ty, v.name)?;
            write_expr(out, &v.value, 0)?;
            writeln!(out, ".")?;
        }
        TopDecl::Routine(r) => {
            write_routine(out, r, 0, false)?;
        }
        TopDecl::Comment(c) => {
            write_comment_line(out, 0, c)?;
        }
        TopDecl::BrandType(b) => {
            write_decl_header_prefix(
                out,
                0,
                &b.comments_before,
                b.is_pub,
                b.pub_dc_syntax,
                b.dc_id,
                None,
            )?;
            let kw = if b.is_opaque { "type" } else { "brand" };
            writeln!(out, "{kw} {}/{}.", b.inner_ty, b.name)?;
        }
        TopDecl::NamedStruct(s) => {
            write_decl_header_prefix(
                out,
                0,
                &s.comments_before,
                s.is_pub,
                s.pub_dc_syntax,
                s.dc_id,
                None,
            )?;
            write!(out, "struct {}", s.name)?;
            if s.inline_body {
                write!(out, " ::")?;
                for f in &s.fields {
                    write!(out, " {}/{}", f.ty, f.name)?;
                }
                writeln!(out, ".")?;
            } else {
                writeln!(out)?;
                for f in &s.fields {
                    for c in &f.comments_before {
                        write_comment_line(out, 4, c)?;
                    }
                    writeln!(out, "    {}/{}.", f.ty, f.name)?;
                }
            }
        }
        TopDecl::Enum(e) => {
            write_decl_header_prefix(
                out,
                0,
                &e.comments_before,
                e.is_pub,
                e.pub_dc_syntax,
                e.dc_id,
                None,
            )?;
            write!(out, "enum {}", e.name)?;
            if e.inline_body || e.variants.is_empty() {
                write!(out, " ::")?;
                for v in &e.variants {
                    match &v.payload {
                        Some(ty) => write!(out, " {ty}/{}", v.name)?,
                        None => write!(out, " {}", v.name)?,
                    }
                }
                writeln!(out, ".")?;
            } else {
                writeln!(out)?;
                for v in &e.variants {
                    for c in &v.comments_before {
                        write_comment_line(out, 4, c)?;
                    }
                    match &v.payload {
                        Some(ty) => writeln!(out, "    {ty}/{}.", v.name)?,
                        None => writeln!(out, "    {}.", v.name)?,
                    }
                }
            }
        }
    }
    Ok(())
}

fn write_block_items(
    out: &mut String,
    stmts: &[BlockItem],
    inner_indent: usize,
) -> Result<()> {
    for item in stmts {
        match item {
            BlockItem::Comment(c) => {
                write_comment_line(out, inner_indent, c)?;
            }
            BlockItem::Expr(Expr::RoutineDecl(nested)) => {
                write_routine(out, nested, inner_indent, true)?;
            }
            BlockItem::Expr(e) => {
                write_indent(out, inner_indent);
                write_expr(out, e, inner_indent)?;
                writeln!(out, ".")?;
            }
        }
    }
    Ok(())
}

fn write_routine(
    out: &mut String,
    r: &RoutineDecl,
    indent: usize,
    is_nested: bool,
) -> Result<()> {
    let private_kw = if is_nested { Some("const") } else { None };
    write_decl_header_prefix(
        out,
        indent,
        &r.comments_before,
        r.is_pub,
        r.pub_dc_syntax,
        r.dc_id,
        private_kw,
    )?;
    write!(out, "v/{}/{}", r.return_type, r.name)?;
    for p in &r.params {
        write!(out, " {}/{}", p.ty, p.name)?;
    }
    if r.tests.is_empty() && r.inline_body {
        write!(out, " :: ")?;
        write_expr(out, &r.body, indent)?;
        writeln!(out, ".")?;
        return Ok(());
    }
    writeln!(out)?;
    let body_indent = indent.saturating_add(4);
    for tb in &r.tests {
        write_indent(out, body_indent);
        if let Some(label) = &tb.label {
            writeln!(out, "test {label}")?;
        } else {
            writeln!(out, "test")?;
        }
        let case_indent = body_indent.saturating_add(4);
        for tc in &tb.cases {
            write_indent(out, case_indent);
            write!(out, "in")?;
            for inp in &tc.inputs {
                out.push(' ');
                write_expr(out, inp, case_indent)?;
            }
            writeln!(out)?;
            match &tc.expected {
                TestExpected::Expr(e) => {
                    write_indent(out, case_indent);
                    write!(out, "out ")?;
                    write_expr(out, e, case_indent)?;
                    writeln!(out)?;
                }
                TestExpected::Fault(_) => {
                    write_indent(out, case_indent);
                    writeln!(out, "out fault")?;
                }
                TestExpected::Void(_) => {
                    write_indent(out, case_indent);
                    writeln!(out, "out")?;
                }
            }
        }
    }

    match &r.body {
        Expr::Block { stmts, .. } if !stmts.is_empty() => {
            write_block_items(out, stmts, body_indent)?;
        }
        other => {
            write_indent(out, body_indent);
            write_expr(out, other, body_indent)?;
            writeln!(out, ".")?;
        }
    }
    Ok(())
}

fn write_slash_escaped_ascii_or_char(
    out: &mut String,
    ch: char,
    hex_above_7e: bool,
) -> Result<()> {
    match ch {
        '"' => out.push_str("/\""),
        '/' => out.push_str("//"),
        '\0' => out.push_str("/0"),
        '\x07' => out.push_str("/a"),
        '\x08' => out.push_str("/b"),
        '\t' => out.push_str("/t"),
        '\n' => out.push_str("/n"),
        '\x0B' => out.push_str("/v"),
        '\x0C' => out.push_str("/f"),
        '\r' => out.push_str("/r"),
        c if u32::from(c) < 0x20
            || c == '\x7F'
            || (hex_above_7e && u32::from(c) > 0x7E) =>
        {
            write!(out, "/x{:02x}", u32::from(c))?;
        }
        c => out.push(c),
    }
    Ok(())
}

fn write_byte_literal(out: &mut String, bytes: &[u8]) -> Result<()> {
    out.push_str("b\"");
    for &b in bytes {
        write_slash_escaped_ascii_or_char(out, char::from(b), true)?;
    }
    out.push('"');
    Ok(())
}

fn write_dcts_literal(out: &mut String, value: &[DcChar]) -> Result<()> {
    let dcts = dc_chars_to_dcts_string(value);
    out.push('"');
    for ch in dcts.chars() {
        write_slash_escaped_ascii_or_char(out, ch, false)?;
    }
    out.push('"');
    Ok(())
}

fn write_prefix_op_call(
    out: &mut String,
    keyword: &str,
    args: &[&Expr],
    trailing_ty: Option<&StageLType>,
    indent: usize,
) -> Result<()> {
    out.push_str(keyword);
    for arg in args {
        out.push(' ');
        write_expr(out, arg, indent)?;
    }
    if let Some(ty) = trailing_ty {
        write!(out, " {ty}")?;
    }
    Ok(())
}

fn write_propagate_expr(
    out: &mut String,
    expr: &Expr,
    indent: usize,
) -> Result<()> {
    let mut depth = 0usize;
    let mut inner = expr;
    while let Expr::Propagate { expr: next, .. } = inner {
        depth = depth.saturating_add(1);
        inner = next;
    }
    let qmarks: String = std::iter::repeat_n('?', depth).collect();
    match inner {
        Expr::OptionSome {
            inner_ty, value, ..
        } => {
            write!(out, "option/{inner_ty}:some{qmarks} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultOk { ok_ty, value, .. } => {
            write!(out, "result/{ok_ty}:ok{qmarks} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultErr { ok_ty, error, .. } => {
            write!(out, "result/{ok_ty}:err{qmarks} ")?;
            write_expr(out, error, indent)?;
        }
        Expr::ValueRef { .. } => {
            write_expr(out, inner, indent)?;
            out.push_str(&qmarks);
        }
        _ => {
            let mut rendered = String::new();
            write_expr(&mut rendered, inner, indent)?;
            if let Some(split_idx) = rendered.find([' ', '\n']) {
                let (head, tail) = rendered.split_at(split_idx);
                out.push_str(head);
                out.push_str(&qmarks);
                out.push_str(tail);
            } else {
                out.push_str(&rendered);
                out.push_str(&qmarks);
            }
        }
    }
    Ok(())
}

fn write_expr(out: &mut String, expr: &Expr, indent: usize) -> Result<()> {
    match expr {
        Expr::Bool { value, .. } => {
            out.push_str(if *value { "true" } else { "false" });
        }
        Expr::Int { value, .. } => out.push_str(&value.display_literal()),
        Expr::Bytes { value, .. } => write_byte_literal(out, value)?,
        Expr::DctsString { value, .. } => write_dcts_literal(out, value)?,
        Expr::DcLit { value, .. } => {
            write!(out, "dc:{}", format_dc_shorthand(*value))?;
        }
        Expr::List {
            elem_ty, elements, ..
        } => {
            write!(out, "l/{elem_ty}:")?;
            for el in elements {
                out.push(' ');
                write_expr(out, el, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::TupleNew { elements, .. } => {
            out.push_str("tupNew:");
            for el in elements {
                out.push(' ');
                write_expr(out, el, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::StructNew { target, fields, .. } => {
            out.push_str("structNew ");
            write_symbol_target(out, target)?;
            out.push(':');
            for f in fields {
                out.push(' ');
                write_expr(out, f, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::NamedStructNew { target, fields, .. } => {
            out.push_str("structNew ");
            write_symbol_target(out, target)?;
            out.push(':');
            for (fname, fval, _) in fields {
                write!(out, " {fname} ")?;
                write_expr(out, fval, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::EnumVariant {
            enum_ty,
            variant,
            payload,
            ..
        } => {
            match enum_ty {
                StructTypeRef::Ident(n)
                | StructTypeRef::Canonical { name: n, .. } => {
                    write!(out, "{n}:{variant}")?;
                }
                StructTypeRef::Dc(dc) => {
                    write!(out, "dc/{}:{variant}", format_dc_shorthand(*dc))?;
                }
            }
            if let Some(p) = payload {
                out.push(' ');
                write_expr(out, p, indent)?;
            }
        }
        Expr::OptionNone { inner_ty, .. } => {
            write!(out, "option/{inner_ty}:none")?;
        }
        Expr::OptionSome {
            inner_ty, value, ..
        } => {
            write!(out, "option/{inner_ty}:some ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultOk { ok_ty, value, .. } => {
            write!(out, "result/{ok_ty}:ok ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultErr { ok_ty, error, .. } => {
            write!(out, "result/{ok_ty}:err ")?;
            write_expr(out, error, indent)?;
        }
        Expr::ErrorTag { tag, .. } => {
            write!(out, "error:{}", tag.keyword())?;
        }
        Expr::Field { base, index, .. } => {
            out.push_str("field ");
            write_expr(out, base, indent)?;
            write!(out, " u32:{index}")?;
        }
        Expr::NamedField { base, field, .. } => {
            out.push_str("field ");
            write_expr(out, base, indent)?;
            write!(out, " {field}")?;
        }
        Expr::With {
            base, index, value, ..
        } => {
            out.push_str("with ");
            write_expr(out, base, indent)?;
            write!(out, " u32:{index} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::NamedWith {
            base, field, value, ..
        } => {
            out.push_str("with ");
            write_expr(out, base, indent)?;
            write!(out, " {field} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::BufNew {
            elem_ty, capacity, ..
        } => {
            write!(out, "bufNew {elem_ty} ")?;
            write_expr(out, capacity, indent)?;
        }
        Expr::ValueRef { ty, target, .. } => {
            if let Some(t) = ty {
                write!(out, "{t}/")?;
            } else {
                out.push('/');
            }
            write_symbol_target(out, target)?;
        }
        Expr::Let { ty, name, init, .. } => {
            write!(out, "let {ty}/{name}")?;
            if let Some(e) = init {
                out.push(' ');
                write_expr(out, e, indent)?;
            }
        }
        Expr::Const {
            ty, name, value, ..
        } => {
            write!(out, "const {ty}/{name} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::Set { ty, name, value, .. } => {
            match ty {
                Some(t) => write!(out, "set {t}/{name} ")?,
                None => write!(out, "set /{name} ")?,
            }
            write_expr(out, value, indent)?;
        }
        Expr::RoutineDecl(r) => write_routine(out, r, indent, true)?,
        Expr::Call { target, args, .. } => {
            write_symbol_target(out, target)?;
            for arg in args {
                out.push(' ');
                write_expr(out, arg, indent)?;
            }
        }
        Expr::Block { stmts, .. } => {
            let has_multiline = stmts.iter().any(|it| {
                matches!(
                    it,
                    BlockItem::Comment(_) | BlockItem::Expr(Expr::RoutineDecl(_))
                )
            }) || stmts.len() > 1;
            if stmts.is_empty() {
                out.push_str("nop");
            } else if !has_multiline {
                if let Some(BlockItem::Expr(e)) = stmts.first() {
                    out.push_str("do ");
                    write_expr(out, e, indent)?;
                    out.push_str(" ::");
                }
            } else {
                writeln!(out, "do")?;
                let inner_indent = indent.saturating_add(4);
                write_block_items(out, stmts, inner_indent)?;
                write_indent(out, indent);
                out.push_str("::");
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            write_prefix_op_call(
                out,
                "if",
                &[cond, then_branch, else_branch],
                None,
                indent,
            )?;
        }
        Expr::While { cond, body, .. } => {
            write_prefix_op_call(out, "while", &[cond, body], None, indent)?;
        }
        Expr::ForEach {
            elem_ty,
            elem_name,
            list,
            body,
            ..
        } => {
            write!(out, "foreach {elem_ty}/{elem_name} ")?;
            write_expr(out, list, indent)?;
            out.push(' ');
            write_expr(out, body, indent)?;
        }
        Expr::Break { .. } => out.push_str("break"),
        Expr::Continue { .. } => out.push_str("continue"),
        Expr::Return { value, .. } => {
            out.push_str("return");
            if let Some(e) = value {
                out.push(' ');
                write_expr(out, e, indent)?;
            }
        }
        Expr::Fault { .. } => out.push_str("fault"),
        Expr::Nop { .. } => out.push_str("nop"),
        Expr::Unary { op, operand, .. } => {
            write_prefix_op_call(out, op.keyword(), &[operand], None, indent)?;
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            write_prefix_op_call(
                out,
                op.keyword(),
                &[left, right],
                None,
                indent,
            )?;
        }
        Expr::Ternary {
            op,
            first,
            second,
            third,
            ..
        } => {
            write_prefix_op_call(
                out,
                op.keyword(),
                &[first, second, third],
                None,
                indent,
            )?;
        }
        Expr::Conv {
            expr, target_ty, ..
        } => {
            write_prefix_op_call(out, "conv", &[expr], Some(target_ty), indent)?;
        }
        Expr::Bitcast {
            expr, target_ty, ..
        } => {
            write_prefix_op_call(
                out,
                "bitcast",
                &[expr],
                Some(target_ty),
                indent,
            )?;
        }
        Expr::WrapNominal {
            expr, target_ty, ..
        } => {
            write_prefix_op_call(out, "wrap", &[expr], Some(target_ty), indent)?;
        }
        Expr::UnwrapNominal { expr, .. } => {
            write_prefix_op_call(out, "unwrap", &[expr], None, indent)?;
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            out.push_str("match ");
            write_expr(out, scrutinee, indent)?;
            if arms.is_empty() && fallback.is_none() {
                out.push_str(" ::");
            } else {
                out.push('\n');
                let arm_indent = indent.saturating_add(4);
                for arm in arms {
                    for c in &arm.comments_before {
                        write_comment_line(out, arm_indent, c)?;
                    }
                    write_indent(out, arm_indent);
                    write!(out, "case {}", arm.variant)?;
                    if let Some((ty, bname, _)) = &arm.binding {
                        write!(out, " {ty}/{bname}")?;
                    }
                    out.push(' ');
                    write_expr(out, &arm.body, arm_indent)?;
                    writeln!(out, ".")?;
                }
                if let Some(fb) = fallback {
                    write_indent(out, arm_indent);
                    out.push_str("else ");
                    write_expr(out, fb, arm_indent)?;
                    writeln!(out, ".")?;
                }
                write_indent(out, indent);
                out.push_str("::");
            }
        }
        Expr::Propagate { .. } => {
            write_propagate_expr(out, expr, indent)?;
        }
    }
    Ok(())
}
